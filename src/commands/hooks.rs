use std::path::Path;

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Execute a hook if configured.
/// Hook config keys: hooks.on_create, hooks.on_save, hooks.on_daily, hooks.on_sync
pub fn run_hook(event: &str, note_path: Option<&Path>) {
    let config_path = mald_home().join("config").join("config.json");
    let config = match ConfigManager::load(&config_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let key = format!("hooks.{}", event);
    let cmd = match config.get_string(&key) {
        Some(c) if !c.is_empty() => c,
        _ => return,
    };

    let mut command = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", &cmd]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", &cmd]);
        c
    };

    // Pass note path as environment variable
    if let Some(p) = note_path {
        command.env("MALD_NOTE", p.to_string_lossy().to_string());
    }
    command.env("MALD_EVENT", event);
    command.env("MALD_HOME", mald_home().to_string_lossy().to_string());

    // Run async, don't block
    match command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(_) => tracing::debug!("Hook '{}' triggered: {}", event, cmd),
        Err(e) => tracing::warn!("Hook '{}' failed: {}", event, e),
    }
}
