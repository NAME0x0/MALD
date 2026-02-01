use anyhow::Result;

use crate::fs::mald_home;

/// Run a plugin from ~/.mald/plugins/. Any executable file becomes `mald x-<name>`.
pub async fn run(name: &str, args: &[String]) -> Result<()> {
    let plugin_dir = mald_home().join("plugins");
    if !plugin_dir.exists() {
        anyhow::bail!(
            "Plugin '{name}' not found. No plugins directory exists.\n\
             Create plugins in ~/.mald/plugins/ — any executable becomes `mald x-<name>`."
        );
    }

    // Find the plugin executable
    let plugin = find_plugin(&plugin_dir, name);

    match plugin {
        Some(path) => {
            let mut cmd = std::process::Command::new(&path);
            cmd.args(args);
            cmd.env("MALD_HOME", mald_home().to_string_lossy().to_string());
            let status = cmd.status()?;
            if !status.success() {
                anyhow::bail!("Plugin '{}' exited with code {:?}", name, status.code());
            }
            Ok(())
        }
        None => {
            let available = list_plugins(&plugin_dir);
            if available.is_empty() {
                anyhow::bail!(
                    "Plugin '{}' not found. No plugins installed in {}",
                    name,
                    plugin_dir.display()
                );
            } else {
                anyhow::bail!(
                    "Plugin '{}' not found. Available plugins:\n{}",
                    name,
                    available
                        .iter()
                        .map(|n| format!("  mald x-{n}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
        }
    }
}

/// List all installed plugins.
pub async fn list() -> Result<()> {
    let plugin_dir = mald_home().join("plugins");
    if !plugin_dir.exists() {
        println!("No plugins directory. Create ~/.mald/plugins/ to add plugins.");
        println!("\nAny executable file in that directory becomes `mald x-<name>`.");
        return Ok(());
    }

    let plugins = list_plugins(&plugin_dir);
    if plugins.is_empty() {
        println!("No plugins installed in {}", plugin_dir.display());
    } else {
        println!("Installed plugins:\n");
        for name in &plugins {
            println!("  mald x-{name}");
        }
    }
    Ok(())
}

fn find_plugin(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if stem == name {
                return Some(path);
            }
        }
    }
    None
}

fn list_plugins(dir: &std::path::Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    names.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    names.sort();
    names
}
