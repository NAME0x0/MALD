use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, RwLock, Semaphore};

use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::index::hnsw::HnswIndex;
use crate::index::metadata::MetadataStore;

/// Maximum concurrent connections to prevent FD exhaustion.
const MAX_CONNECTIONS: usize = 100;

/// Daemon startup timestamp for uptime tracking
static DAEMON_START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Counter for vector index failures (for observability)
static INDEX_FAILURES: AtomicU64 = AtomicU64::new(0);

/// Increment the index failure counter (call on vector index errors)
pub fn record_index_failure() {
    INDEX_FAILURES.fetch_add(1, Ordering::Relaxed);
}

/// Get current index failure count
pub fn get_index_failure_count() -> u64 {
    INDEX_FAILURES.load(Ordering::Relaxed)
}

/// Shared daemon state holding in-memory index resources.
///
/// `MetadataStore` wraps a `rusqlite::Connection` which is `Send` but not `Sync`,
/// so we use `tokio::sync::Mutex` for it. `HnswIndex` is fully `Send + Sync`,
/// so we use `tokio::sync::RwLock` for concurrent read access during searches.
pub struct DaemonState {
    /// Shared SQLite metadata store (FTS + chunk tracking).
    pub meta: Mutex<MetadataStore>,
    /// In-memory HNSW vector index. `None` until first vector indexing.
    pub hnsw: RwLock<Option<HnswIndex>>,
    /// Shared config.
    pub config: RwLock<ConfigManager>,
    /// Path to the HNSW binary file on disk for periodic persistence.
    pub hnsw_path: PathBuf,
    /// Flag indicating the HNSW index has been modified since last save.
    hnsw_dirty: AtomicBool,
}

impl DaemonState {
    /// Mark the HNSW index as modified (needs persistence).
    pub fn mark_hnsw_dirty(&self) {
        self.hnsw_dirty.store(true, Ordering::Release);
    }

    /// Check and clear the dirty flag. Returns true if index was dirty.
    pub fn take_hnsw_dirty(&self) -> bool {
        self.hnsw_dirty.swap(false, Ordering::AcqRel)
    }
}

#[derive(Deserialize)]
struct Request {
    token: Option<String>,
    cmd: String,
    query: Option<String>,
    k: Option<usize>,
}

#[derive(Serialize)]
struct Response {
    status: String,
    data: serde_json::Value,
}

/// Health metrics for daemon status reporting
#[derive(Serialize)]
struct HealthMetrics {
    healthy: bool,
    uptime_secs: u64,
    version: &'static str,
    index_status: IndexStatus,
    kb_path: String,
}

#[derive(Serialize)]
struct IndexStatus {
    fts_available: bool,
    vector_available: bool,
    document_count: Option<usize>,
    /// Number of vector indexing failures since daemon start
    index_failures: u64,
}

/// Generate a random auth token and write it to ~/.mald/daemon.token
fn generate_auth_token() -> Result<String> {
    let mut rng = rand::thread_rng();
    let token: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect();

    let token_path = mald_home().join("daemon.token");
    std::fs::write(&token_path, &token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&token_path, perms)?;
    }
    Ok(token)
}

/// Read the auth token from disk.
pub fn read_auth_token() -> Option<String> {
    let token_path = mald_home().join("daemon.token");
    std::fs::read_to_string(token_path)
        .ok()
        .map(|s| s.trim().to_string())
}

pub async fn run() -> Result<()> {
    // Initialize daemon start time for uptime tracking
    DAEMON_START.get_or_init(Instant::now);

    let home = mald_home();
    let config_path = home.join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;

    // Generate auth token
    let auth_token = generate_auth_token()?;
    tracing::info!("Daemon auth token written to ~/.mald/daemon.token");

    // --- Initialize shared daemon state (Step 1 & 2) ---
    let index_dir = home.join("index");
    crate::fs::ensure_directory(&index_dir)?;
    let meta_path = index_dir.join("metadata.db");
    let hnsw_path = index_dir.join("hnsw.bin");

    // Open MetadataStore once at startup
    let meta = MetadataStore::open(&meta_path)?;

    // Load HNSW index once at startup (if it exists on disk)
    let hnsw = if hnsw_path.exists() {
        match HnswIndex::load(&hnsw_path) {
            Ok(idx) => {
                tracing::info!("Loaded HNSW index: {} vectors", idx.len());
                Some(idx)
            }
            Err(e) => {
                tracing::warn!("Failed to load HNSW index, starting fresh: {}", e);
                None
            }
        }
    } else {
        None
    };

    let state = Arc::new(DaemonState {
        meta: Mutex::new(meta),
        hnsw: RwLock::new(hnsw),
        config: RwLock::new(config),
        hnsw_path: hnsw_path.clone(),
        hnsw_dirty: AtomicBool::new(false),
    });

    // --- Periodic HNSW persistence (Step 4) ---
    let state_for_persist = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if state_for_persist.take_hnsw_dirty() {
                let hnsw_guard = state_for_persist.hnsw.read().await;
                if let Some(ref index) = *hnsw_guard {
                    match index.save(&state_for_persist.hnsw_path) {
                        Ok(()) => tracing::info!("Persisted HNSW index: {} vectors", index.len()),
                        Err(e) => tracing::error!("Failed to persist HNSW index: {}", e),
                    }
                }
            }
        }
    });

    // --- File watcher with shared state (Step 5) ---
    let kb_dir = home.join("kb");
    let (watcher_tx, mut watcher_rx) = tokio::sync::mpsc::channel::<std::path::PathBuf>(100);

    // Limit concurrent indexing tasks to prevent resource exhaustion
    const MAX_CONCURRENT_INDEX: usize = 4;
    let index_semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_INDEX));

    let state_for_watcher = Arc::clone(&state);
    tokio::spawn(async move {
        while let Some(path) = watcher_rx.recv().await {
            let state = Arc::clone(&state_for_watcher);
            let permit = Arc::clone(&index_semaphore);

            // Spawn parallel task for each file
            tokio::spawn(async move {
                let _permit = permit.acquire().await;

                tracing::info!("File changed: {:?}", path);

                // Run on_save hook (sync, runs in spawned task)
                crate::commands::hooks::run_hook("on_save", Some(&path));

                // Update modified timestamp in frontmatter
                if let Err(e) = crate::commands::stamp::update_modified_timestamp(&path) {
                    tracing::debug!("Failed to update timestamp for {:?}: {}", path, e);
                }

                // Vector+FTS index using shared state (no disk round-trips)
                if let Err(e) = super::indexer::index_file_shared(&path, &state).await {
                    tracing::debug!("Indexing failed for {:?}: {}", path, e);
                    record_index_failure();
                }
            });
        }
    });

    if kb_dir.exists() {
        // FTS index all KBs on startup using shared state
        tracing::info!("Building FTS index...");
        let count = super::indexer::fts_index_kb_shared(&kb_dir, &state).await?;
        tracing::info!("FTS indexed {} files", count);

        super::watcher::start_watching(&kb_dir, watcher_tx)?;
    }

    tracing::info!("Daemon starting...");

    #[cfg(windows)]
    {
        start_tcp_server(auth_token, state).await
    }
    #[cfg(not(windows))]
    {
        start_unix_socket_server(auth_token, state).await
    }
}

#[cfg(windows)]
async fn start_tcp_server(auth_token: String, state: Arc<DaemonState>) -> Result<()> {
    let port = {
        let config = state.config.read().await;
        config.typed().daemon.port
    };

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    tracing::info!("Daemon listening on 127.0.0.1:{}", port);

    // Connection limiter to prevent FD exhaustion
    let semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));

    loop {
        let (stream, _) = listener.accept().await?;
        let token = auth_token.clone();
        let permit = semaphore.clone().acquire_owned().await;
        let conn_state = Arc::clone(&state);

        tokio::spawn(async move {
            let _permit = permit; // Hold permit until handler completes
            let result = tokio::time::timeout(
                Duration::from_secs(300),
                handle_connection(stream, &token, &conn_state),
            )
            .await;
            match result {
                Ok(Err(e)) => tracing::error!("Connection error: {}", e),
                Err(_) => tracing::warn!("Connection timed out after 300s"),
                Ok(Ok(())) => {}
            }
        });
    }
}

#[cfg(not(windows))]
async fn start_unix_socket_server(auth_token: String, state: Arc<DaemonState>) -> Result<()> {
    let sock_path = mald_home().join("daemon.sock");
    let _ = std::fs::remove_file(&sock_path);

    let listener = tokio::net::UnixListener::bind(&sock_path)?;
    tracing::info!("Daemon listening on {}", sock_path.display());

    // Connection limiter to prevent FD exhaustion
    let semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));

    loop {
        let (stream, _) = listener.accept().await?;
        let token = auth_token.clone();
        let permit = semaphore.clone().acquire_owned().await;
        let conn_state = Arc::clone(&state);

        tokio::spawn(async move {
            let _permit = permit; // Hold permit until handler completes
            let result = tokio::time::timeout(
                Duration::from_secs(300),
                handle_connection(stream, &token, &conn_state),
            )
            .await;
            match result {
                Ok(Err(e)) => tracing::error!("Connection error: {}", e),
                Err(_) => tracing::warn!("Connection timed out after 300s"),
                Ok(Ok(())) => {}
            }
        });
    }
}

async fn handle_connection<S>(stream: S, auth_token: &str, state: &Arc<DaemonState>) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(stream);
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response {
                    status: "error".into(),
                    data: serde_json::json!({"message": format!("Invalid request: {}", e)}),
                };
                let mut out = serde_json::to_string(&resp)?;
                out.push('\n');
                writer.write_all(out.as_bytes()).await?;
                continue;
            }
        };

        // Auth check
        let token_valid = req.token.as_ref().map(|t| t == auth_token).unwrap_or(false);

        if !token_valid {
            let resp = Response {
                status: "error".into(),
                data: serde_json::json!({"message": "Invalid or missing auth token"}),
            };
            let mut out = serde_json::to_string(&resp)?;
            out.push('\n');
            writer.write_all(out.as_bytes()).await?;
            continue;
        }

        let resp = handle_request(req, state).await;
        let mut out = serde_json::to_string(&resp)?;
        out.push('\n');
        writer.write_all(out.as_bytes()).await?;
    }

    Ok(())
}

async fn handle_request(req: Request, state: &Arc<DaemonState>) -> Response {
    match req.cmd.as_str() {
        "search" => {
            let query = req.query.unwrap_or_default();
            let k = req.k.unwrap_or(10);

            // Use the shared MetadataStore instead of opening a new connection
            let meta = state.meta.lock().await;
            match meta.fts_search(&query, k) {
                Ok(results) => {
                    let items: Vec<serde_json::Value> = results
                        .iter()
                        .map(|r| {
                            serde_json::json!({
                                "path": r.path,
                                "title": r.title,
                                "snippet": r.snippet,
                            })
                        })
                        .collect();
                    Response {
                        status: "ok".into(),
                        data: serde_json::json!({"results": items}),
                    }
                }
                Err(e) => Response {
                    status: "error".into(),
                    data: serde_json::json!({"message": format!("Search failed: {}", e)}),
                },
            }
        }
        "status" => {
            let health = get_health_metrics_shared(state).await;
            Response {
                status: "ok".into(),
                data: serde_json::to_value(&health).unwrap_or(serde_json::json!({"healthy": true})),
            }
        }
        _ => Response {
            status: "error".into(),
            data: serde_json::json!({"message": format!("Unknown command: {}", req.cmd)}),
        },
    }
}

/// Collect daemon health metrics using shared state (no new DB connections).
async fn get_health_metrics_shared(state: &DaemonState) -> HealthMetrics {
    let home = mald_home();
    let kb_path = home.join("kb");

    // Check FTS availability and document count via shared MetadataStore
    let (fts_available, document_count) = {
        let meta = state.meta.lock().await;
        let count = meta.document_count().ok();
        (true, count)
    };

    // Check vector index availability via shared HNSW
    let vector_available = {
        let hnsw_guard = state.hnsw.read().await;
        hnsw_guard.is_some()
    };

    // Calculate uptime
    let uptime_secs = DAEMON_START
        .get()
        .map(|start| start.elapsed().as_secs())
        .unwrap_or(0);

    HealthMetrics {
        healthy: fts_available, // Healthy if at least FTS works
        uptime_secs,
        version: env!("CARGO_PKG_VERSION"),
        index_status: IndexStatus {
            fts_available,
            vector_available,
            document_count,
            index_failures: get_index_failure_count(),
        },
        kb_path: kb_path.display().to_string(),
    }
}
