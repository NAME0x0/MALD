use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::config::ConfigManager;
use crate::fs::mald_home;

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
    let home = mald_home();
    let config_path = home.join("config").join("config.json");
    let _config = ConfigManager::load(&config_path)?;

    // Generate auth token
    let auth_token = generate_auth_token()?;
    tracing::info!("Daemon auth token written to ~/.mald/daemon.token");

    // Start file watcher
    let kb_dir = home.join("kb");
    let (watcher_tx, mut watcher_rx) = tokio::sync::mpsc::channel::<std::path::PathBuf>(100);
    let watcher_config_path = config_path.clone();

    tokio::spawn(async move {
        while let Some(path) = watcher_rx.recv().await {
            tracing::info!("File changed: {:?}", path);
            // Run on_save hook
            crate::commands::hooks::run_hook("on_save", Some(&path));
            // Update modified timestamp in frontmatter
            let _ = crate::commands::stamp::update_modified_timestamp(&path);
            // Always FTS index (no AI needed)
            let _ = crate::daemon::indexer::fts_index_file(&path);
            // Try vector index if Ollama available
            if let Ok(cfg) = ConfigManager::load(&watcher_config_path) {
                let _ = super::indexer::index_file(&path, &cfg).await;
            }
        }
    });

    if kb_dir.exists() {
        // FTS index all KBs on startup
        tracing::info!("Building FTS index...");
        let count = crate::daemon::indexer::fts_index_kb(&kb_dir)?;
        tracing::info!("FTS indexed {} files", count);

        super::watcher::start_watching(&kb_dir, watcher_tx)?;
    }

    tracing::info!("Daemon starting...");

    #[cfg(windows)]
    {
        start_tcp_server(auth_token).await
    }
    #[cfg(not(windows))]
    {
        start_unix_socket_server(auth_token).await
    }
}

#[cfg(windows)]
async fn start_tcp_server(auth_token: String) -> Result<()> {
    let port = {
        let config_path = mald_home().join("config").join("config.json");
        let config = ConfigManager::load(&config_path)?;
        config
            .get("daemon.port")
            .and_then(|v| v.as_u64())
            .unwrap_or(7433) as u16
    };

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    tracing::info!("Daemon listening on 127.0.0.1:{}", port);

    loop {
        let (stream, _) = listener.accept().await?;
        let token = auth_token.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &token).await {
                tracing::error!("Connection error: {}", e);
            }
        });
    }
}

#[cfg(not(windows))]
async fn start_unix_socket_server(auth_token: String) -> Result<()> {
    let sock_path = mald_home().join("daemon.sock");
    let _ = std::fs::remove_file(&sock_path);

    let listener = tokio::net::UnixListener::bind(&sock_path)?;
    tracing::info!("Daemon listening on {}", sock_path.display());

    loop {
        let (stream, _) = listener.accept().await?;
        let token = auth_token.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &token).await {
                tracing::error!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection<S>(stream: S, auth_token: &str) -> Result<()>
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

        let resp = handle_request(req).await;
        let mut out = serde_json::to_string(&resp)?;
        out.push('\n');
        writer.write_all(out.as_bytes()).await?;
    }

    Ok(())
}

async fn handle_request(req: Request) -> Response {
    match req.cmd.as_str() {
        "search" => {
            let query = req.query.unwrap_or_default();
            let k = req.k.unwrap_or(10);
            let index_dir = mald_home().join("index");
            let meta_path = index_dir.join("metadata.db");
            match crate::index::metadata::MetadataStore::open(&meta_path) {
                Ok(meta) => match meta.fts_search(&query, k) {
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
                },
                Err(e) => Response {
                    status: "error".into(),
                    data: serde_json::json!({"message": format!("DB error: {}", e)}),
                },
            }
        }
        "status" => Response {
            status: "ok".into(),
            data: serde_json::json!({"healthy": true}),
        },
        _ => Response {
            status: "error".into(),
            data: serde_json::json!({"message": format!("Unknown command: {}", req.cmd)}),
        },
    }
}
