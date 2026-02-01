use anyhow::{bail, Result};

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Open the KB directory in the configured editor (or file manager).
pub async fn run(kb: Option<&str>) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        bail!("Knowledge base '{kb_name}' not found. Create it with: mald kb create {kb_name}");
    }

    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());

    // Some editors (VS Code, Sublime) can open directories
    // For terminal editors, open the index.md or first file
    let target = if editor.contains("code") || editor.contains("subl") || editor.contains("zed") {
        kb_path.clone()
    } else {
        // Terminal editor — open index.md or first file found
        let index = kb_path.join("index.md");
        if index.exists() {
            index
        } else {
            let files = crate::fs::find_files(&kb_path, "md")?;
            files.into_iter().next().unwrap_or(kb_path.clone())
        }
    };

    std::process::Command::new(&editor)
        .arg(target.to_str().unwrap())
        .status()?;

    Ok(())
}
