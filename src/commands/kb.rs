use anyhow::{bail, Result};

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

pub async fn create(name: &str) -> Result<()> {
    let kb_path = mald_home().join("kb").join(name);
    if kb_path.exists() {
        bail!("Knowledge base '{name}' already exists");
    }
    ensure_directory(&kb_path)?;

    let index_content = format!(
        "# {name}\n\nKnowledge base created by MALD.\n\n## Notes\n\n- Start adding notes here\n"
    );
    std::fs::write(kb_path.join("index.md"), index_content)?;

    // Create templates directory
    let templates = kb_path.join("templates");
    ensure_directory(&templates)?;
    std::fs::write(
        templates.join("default.md"),
        "---\ntitle: \ntags: []\ncreated: \n---\n\n# \n\n",
    )?;

    println!("Created knowledge base: {name}");
    println!("  Path: {}", kb_path.display());
    Ok(())
}

pub async fn list(json: bool) -> Result<()> {
    let kb_dir = mald_home().join("kb");
    if !kb_dir.exists() {
        if json {
            println!("[]");
        } else {
            println!("No knowledge bases found. Run `mald init` first.");
        }
        return Ok(());
    }

    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let default_kb = config.typed().default_kb.clone();

    let mut kbs: Vec<serde_json::Value> = Vec::new();
    let mut count = 0;

    for entry in std::fs::read_dir(&kb_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let files = crate::fs::find_files(&entry.path(), "md")?;
            let is_default = name == default_kb;
            if json {
                kbs.push(serde_json::json!({
                    "name": name,
                    "notes": files.len(),
                    "path": entry.path().to_string_lossy(),
                    "default": is_default,
                }));
            } else {
                let marker = if is_default { " *" } else { "" };
                println!("  {} ({} files){}", name, files.len(), marker);
            }
            count += 1;
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&kbs)?);
    } else if count == 0 {
        println!("No knowledge bases found. Create one with `mald kb create <name>`.");
    }
    Ok(())
}

pub async fn open(name: &str) -> Result<()> {
    let kb_path = mald_home().join("kb").join(name);
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Knowledge base '{name}' not found"),
            "Run `mald kb list` to see available knowledge bases",
        ));
    }

    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let editor = config.typed().editor.clone();

    let status = std::process::Command::new(&editor)
        .arg(kb_path.to_str().unwrap())
        .status()?;

    if !status.success() {
        bail!("Editor '{editor}' exited with error");
    }
    Ok(())
}
