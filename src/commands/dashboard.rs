use anyhow::Result;
use chrono::Local;

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Default zero-arg experience: show a dashboard with recent activity,
/// quick stats, and actionable suggestions.
pub async fn run() -> Result<()> {
    let home = mald_home();

    if !home.exists() {
        println!("Welcome to MALD — Markdown Archive & Localized Daemon\n");
        println!("Get started:");
        println!("  mald init              # Create ~/.mald/ structure");
        println!("  mald kb create work    # Create a knowledge base");
        println!("  mald new \"First Note\"  # Create your first note");
        return Ok(());
    }

    let config_path = home.join("config").join("config.json");
    let config = match ConfigManager::load(&config_path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // No config yet, just show the welcome
    };
    let default_kb = config
        .get_string("default_kb")
        .unwrap_or_else(|| "personal".into());

    println!("MALD Dashboard — {}\n", Local::now().format("%A, %B %d, %Y"));

    // KB stats
    let kb_dir = home.join("kb");
    let mut total_notes = 0usize;
    let mut kb_names = Vec::new();
    if kb_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&kb_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let count = crate::fs::find_files(&entry.path(), "md")
                        .map(|f| f.len())
                        .unwrap_or(0);
                    kb_names.push((name, count));
                    total_notes += count;
                }
            }
        }
    }

    if kb_names.is_empty() {
        println!("  No knowledge bases. Run `mald kb create <name>`.\n");
        return Ok(());
    }

    println!("Knowledge bases:");
    for (name, count) in &kb_names {
        let marker = if *name == default_kb { " *" } else { "" };
        println!("  {:20} {:>4} notes{}", name, count, marker);
    }
    println!("  {:20} {:>4} total\n", "", total_notes);

    // Recent notes (last 7 days)
    let mut recent: Vec<(String, std::time::SystemTime)> = Vec::new();
    if let Ok(files) = crate::fs::find_files(&kb_dir, "md") {
        for f in &files {
            if let Ok(meta) = f.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() < 7 * 24 * 3600 {
                            let name = f
                                .strip_prefix(&kb_dir)
                                .unwrap_or(f)
                                .to_string_lossy()
                                .to_string();
                            recent.push((name, modified));
                        }
                    }
                }
            }
        }
    }
    recent.sort_by(|a, b| b.1.cmp(&a.1));
    recent.truncate(5);

    if !recent.is_empty() {
        println!("Recent activity:");
        for (name, _) in &recent {
            println!("  {}", name.replace('\\', "/"));
        }
        println!();
    }

    // Open tasks across all notes
    let tasks = collect_open_tasks(&kb_dir);
    if !tasks.is_empty() {
        println!("Open tasks ({}):", tasks.len());
        for (note, task) in tasks.iter().take(5) {
            let short_note = std::path::Path::new(note)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            println!("  [ ] {} ({})", task, short_note);
        }
        if tasks.len() > 5 {
            println!("  ... and {} more. Run `mald tasks` to see all.", tasks.len() - 5);
        }
        println!();
    }

    // Contextual suggestions — only show relevant ones
    let mut has_suggestions = false;

    if !home.join(".git").exists() && total_notes > 3 {
        if !has_suggestions {
            println!("Suggestions:");
            has_suggestions = true;
        }
        println!("  mald sync init        # Enable version history");
    }

    // Only suggest AI after user has real content
    if total_notes > 5 {
        let hnsw_path = home.join("index").join("hnsw.bin");
        if !hnsw_path.exists() {
            let ollama_running = reqwest::Client::new()
                .get("http://localhost:11434/api/tags")
                .send()
                .await
                .is_ok();
            if ollama_running {
                if !has_suggestions {
                    println!("Suggestions:");
                    has_suggestions = true;
                }
                println!("  mald ai index {}  # Build semantic search", default_kb);
            }
        }
    }

    if has_suggestions {
        println!();
    }

    // Always show quick actions
    println!("Quick actions:");
    println!("  mald q \"thought\"      # Quick capture");
    println!("  mald today            # Open daily note");
    println!("  mald search           # Interactive search");

    Ok(())
}

/// Collect all unchecked `- [ ]` tasks across the KB directory.
pub fn collect_open_tasks(kb_dir: &std::path::Path) -> Vec<(String, String)> {
    let mut tasks = Vec::new();
    if let Ok(files) = crate::fs::find_files(kb_dir, "md") {
        for f in &files {
            if let Ok(content) = std::fs::read_to_string(f) {
                let note_name = f
                    .strip_prefix(kb_dir)
                    .unwrap_or(f)
                    .to_string_lossy()
                    .to_string();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("- [ ] ") {
                        let task_text = trimmed[6..].trim().to_string();
                        if !task_text.is_empty() {
                            tasks.push((note_name.clone(), task_text));
                        }
                    }
                }
            }
        }
    }
    tasks
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_collect_open_tasks() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("note.md"),
            "# Tasks\n\n- [ ] Buy milk\n- [x] Done\n- [ ] Fix bug\n",
        )
        .unwrap();

        let tasks = collect_open_tasks(dir.path());
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].1, "Buy milk");
        assert_eq!(tasks[1].1, "Fix bug");
    }

    #[test]
    fn test_collect_open_tasks_empty() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("note.md"), "# No tasks here\n").unwrap();

        let tasks = collect_open_tasks(dir.path());
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_collect_open_tasks_ignores_empty_checkboxes() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("note.md"), "- [ ] \n- [ ] Real task\n").unwrap();

        let tasks = collect_open_tasks(dir.path());
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].1, "Real task");
    }
}
