use anyhow::Result;
use std::path::PathBuf;

use crate::fs::mald_home;

fn pid_file() -> PathBuf {
    mald_home().join("daemon.pid")
}

pub async fn start() -> Result<()> {
    if is_running() {
        println!("Daemon is already running.");
        return Ok(());
    }

    let exe = std::env::current_exe()?;
    let child = std::process::Command::new(exe)
        .arg("daemon")
        .arg("_run")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match child {
        Ok(child) => {
            std::fs::write(pid_file(), child.id().to_string())?;
            println!("Daemon started (PID: {})", child.id());
        }
        Err(e) => {
            // If we can't spawn a background process, run inline
            tracing::warn!("Could not spawn daemon process: {}", e);
            println!("Starting daemon in foreground...");
            crate::daemon::server::run().await?;
        }
    }
    Ok(())
}

pub async fn stop() -> Result<()> {
    let pid_path = pid_file();
    if !pid_path.exists() {
        println!("Daemon is not running.");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str.trim().parse()?;

    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output();
    }
    #[cfg(not(windows))]
    {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    let _ = std::fs::remove_file(&pid_path);
    println!("Daemon stopped.");
    Ok(())
}

pub async fn status() -> Result<()> {
    if is_running() {
        let pid = std::fs::read_to_string(pid_file())
            .unwrap_or_default()
            .trim()
            .to_string();
        println!("Daemon is running (PID: {pid})");
    } else {
        println!("Daemon is not running.");
    }
    Ok(())
}

/// Ensure the daemon is running. Starts it silently if not.
/// Called automatically from main before dispatching commands.
pub fn ensure_running() {
    let home = mald_home();
    if !home.exists() || !home.join("config").join("config.json").exists() {
        return;
    }
    if is_running() {
        return;
    }
    // Silently spawn daemon in background
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe)
            .arg("daemon")
            .arg("_run")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|child| {
                let _ = std::fs::write(pid_file(), child.id().to_string());
            });
    }
}

pub fn is_running() -> bool {
    let pid_path = pid_file();
    if !pid_path.exists() {
        return false;
    }
    let pid_str = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Corrupt PID file, clean up
            let _ = std::fs::remove_file(&pid_path);
            return false;
        }
    };

    let alive = process_alive(pid);

    // Clean up stale PID file if process is dead
    if !alive {
        let _ = std::fs::remove_file(&pid_path);
        // Also clean up token and socket
        let _ = std::fs::remove_file(mald_home().join("daemon.token"));
        let _ = std::fs::remove_file(mald_home().join("daemon.sock"));
    }

    alive
}

fn process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output();
        match output {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout);
                // tasklist returns "INFO: No tasks are running..." when PID doesn't exist
                !s.contains("INFO:") && s.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

/// Health metrics returned by daemon status query
#[derive(Debug, serde::Deserialize)]
pub struct DaemonHealth {
    pub healthy: bool,
    pub uptime_secs: u64,
    pub version: String,
    pub index_status: IndexStatus,
    pub kb_path: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct IndexStatus {
    pub fts_available: bool,
    pub vector_available: bool,
    pub document_count: Option<usize>,
    pub index_failures: u64,
}

/// Query daemon health via IPC. Returns None if daemon is not responding.
pub async fn query_health() -> Option<DaemonHealth> {
    if !is_running() {
        return None;
    }

    let token = crate::daemon::server::read_auth_token()?;
    let request = serde_json::json!({
        "token": token,
        "cmd": "status"
    });
    let request_str = format!("{}\n", serde_json::to_string(&request).ok()?);

    #[cfg(windows)]
    {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let port = {
            let config_path = mald_home().join("config").join("config.json");
            if let Ok(cfg) = crate::config::ConfigManager::load(&config_path) {
                cfg.typed().daemon.port
            } else {
                7433
            }
        };

        let stream = TcpStream::connect(format!("127.0.0.1:{port}")).await.ok()?;
        let (reader, mut writer) = tokio::io::split(stream);
        writer.write_all(request_str.as_bytes()).await.ok()?;

        let mut lines = BufReader::new(reader).lines();
        let line = lines.next_line().await.ok()??;
        parse_health_response(&line)
    }
    #[cfg(not(windows))]
    {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let sock_path = mald_home().join("daemon.sock");
        let stream = UnixStream::connect(&sock_path).await.ok()?;
        let (reader, mut writer) = tokio::io::split(stream);
        writer.write_all(request_str.as_bytes()).await.ok()?;

        let mut lines = BufReader::new(reader).lines();
        let line = lines.next_line().await.ok()??;
        parse_health_response(&line)
    }
}

fn parse_health_response(line: &str) -> Option<DaemonHealth> {
    #[derive(serde::Deserialize)]
    struct Response {
        status: String,
        data: serde_json::Value,
    }

    let resp: Response = serde_json::from_str(line).ok()?;
    if resp.status != "ok" {
        return None;
    }
    serde_json::from_value(resp.data).ok()
}
