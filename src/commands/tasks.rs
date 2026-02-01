use anyhow::Result;

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Aggregate all open tasks (`- [ ]`) across all notes.
pub async fn list(kb: Option<&str>, all_kbs: bool, json: bool) -> Result<()> {
    let home = mald_home();
    let kb_dir = home.join("kb");

    if !kb_dir.exists() {
        if json {
            println!("[]");
        } else {
            println!("No knowledge bases found. Run `mald init` first.");
        }
        return Ok(());
    }

    let dirs: Vec<std::path::PathBuf> = if all_kbs {
        std::fs::read_dir(&kb_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect()
    } else {
        let config_path = home.join("config").join("config.json");
        let config = ConfigManager::load(&config_path)?;
        let kb_name = kb
            .map(String::from)
            .or_else(|| config.get_string("default_kb"))
            .unwrap_or_else(|| "personal".into());
        let p = kb_dir.join(&kb_name);
        if !p.exists() {
            if json {
                println!("[]");
            } else {
                println!("Knowledge base '{}' not found.", kb_name);
            }
            return Ok(());
        }
        vec![p]
    };

    let mut total = 0usize;
    let mut done = 0usize;
    let mut all_tasks: Vec<serde_json::Value> = Vec::new();

    for dir in &dirs {
        let kb_name = dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let files = crate::fs::find_files(dir, "md")?;
        let mut kb_tasks: Vec<(String, String)> = Vec::new();

        for f in &files {
            if let Ok(content) = std::fs::read_to_string(f) {
                let note = f
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                        let task = rest.trim().to_string();
                        if !task.is_empty() {
                            kb_tasks.push((note.clone(), task.clone()));
                            if json {
                                all_tasks.push(serde_json::json!({
                                    "task": task,
                                    "note": note,
                                    "kb": kb_name,
                                    "done": false,
                                }));
                            }
                            total += 1;
                        }
                    } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
                        done += 1;
                    }
                }
            }
        }

        if !json && !kb_tasks.is_empty() {
            if dirs.len() > 1 {
                println!("[{}]", kb_name);
            }
            for (note, task) in &kb_tasks {
                println!("  [ ] {} ({})", task, note);
            }
            if dirs.len() > 1 {
                println!();
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&all_tasks)?);
    } else if total == 0 {
        println!("No open tasks.");
    } else {
        println!("\n{} open, {} done", total, done);
    }

    Ok(())
}
