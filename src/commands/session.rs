use anyhow::Result;

use crate::config::ConfigManager;
use crate::fs::mald_home;

pub async fn start(kb: Option<&str>) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;

    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        anyhow::bail!(
            "Knowledge base '{kb_name}' not found. Create it with `mald kb create {kb_name}`"
        );
    }

    let tmux_enabled = config
        .get("session.tmux_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if tmux_enabled && which_exists("tmux") {
        start_tmux_session(&kb_name, &kb_path, &config).await
    } else {
        start_shell_session(&kb_name, &kb_path, &config).await
    }
}

pub async fn list() -> Result<()> {
    if which_exists("tmux") {
        let output = std::process::Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let sessions = String::from_utf8_lossy(&out.stdout);
                let mald_sessions: Vec<&str> = sessions
                    .lines()
                    .filter(|s| s.starts_with("mald-"))
                    .collect();
                if mald_sessions.is_empty() {
                    println!("No active MALD sessions.");
                } else {
                    for s in mald_sessions {
                        println!("  {s}");
                    }
                }
            }
            _ => println!("No active MALD sessions."),
        }
    } else {
        println!("Session listing requires tmux. On Windows, sessions are shell-only.");
    }
    Ok(())
}

async fn start_tmux_session(
    kb_name: &str,
    kb_path: &std::path::Path,
    config: &ConfigManager,
) -> Result<()> {
    let session_name = format!("mald-{kb_name}");
    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    let home = mald_home();

    let status = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", &session_name, "-n", "editor"])
        .env("MALD_HOME", home.to_str().unwrap())
        .env("MALD_SESSION", &session_name)
        .env("MALD_KB", kb_path.to_str().unwrap())
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to create tmux session");
    }

    // Create windows
    let _ = std::process::Command::new("tmux")
        .args(["new-window", "-t", &session_name, "-n", "terminal"])
        .status();

    let _ = std::process::Command::new("tmux")
        .args(["new-window", "-t", &session_name, "-n", "ai"])
        .status();

    // Select editor window and launch editor
    let _ = std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{session_name}:editor"),
            &format!("{} {}", editor, kb_path.display()),
            "Enter",
        ])
        .status();

    // Attach
    std::process::Command::new("tmux")
        .args(["attach-session", "-t", &session_name])
        .status()?;

    Ok(())
}

async fn start_shell_session(
    kb_name: &str,
    kb_path: &std::path::Path,
    _config: &ConfigManager,
) -> Result<()> {
    let home = mald_home();

    println!("MALD Session: {kb_name}");
    println!("Knowledge Base: {}", kb_path.display());
    println!("Type 'exit' to end session.\n");

    let shell = if cfg!(windows) { "powershell" } else { "bash" };

    std::process::Command::new(shell)
        .env("MALD_HOME", home.to_str().unwrap())
        .env("MALD_SESSION", kb_name)
        .env("MALD_KB", kb_path.to_str().unwrap())
        .status()?;

    Ok(())
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
