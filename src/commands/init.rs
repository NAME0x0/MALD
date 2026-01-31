use anyhow::Result;

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

const DEFAULT_KB_INDEX: &str = r#"---
title: Personal
created: 2024-01-01
tags: []
---

# Personal Knowledge Base

Welcome to your MALD knowledge base.

## Getting Started

- Create new notes with `mald new "Title"`
- Open today's daily note with `mald today`
- Use [[wikilinks]] to connect ideas
- Tag notes with #hashtags
- Use `mald search` for interactive fuzzy search

## Quick Links

- [[inbox]] — capture fleeting thoughts
- [[projects]] — active project notes
"#;

pub async fn run() -> Result<()> {
    let home = mald_home();

    let dirs = [
        "kb/personal",
        "config",
        "sessions",
        "sessions/chat",
        "cache",
        "ai/models",
        "index",
        "templates",
        "plugins",
    ];
    for dir in &dirs {
        ensure_directory(&home.join(dir))?;
    }

    let config_path = home.join("config").join("config.json");
    if !config_path.exists() {
        let config = ConfigManager::load(&config_path)?;
        config.save()?;
    }

    let index_path = home.join("kb").join("personal").join("index.md");
    if !index_path.exists() {
        std::fs::write(&index_path, DEFAULT_KB_INDEX)?;
    }

    // Create default templates
    crate::commands::templates::init_defaults()?;

    // Build FTS index immediately
    let kb_dir = home.join("kb");
    let count = crate::daemon::indexer::fts_index_kb(&kb_dir)?;

    println!("MALD initialized at {}", home.display());
    println!("  Indexed {} files", count);
    println!("\nNext steps:");
    println!("  mald today          — open today's daily note");
    println!("  mald new \"Title\"    — create a new note");
    println!("  mald search         — interactive fuzzy search");
    println!("  mald q \"thought\"    — quick capture to daily note");
    println!("  mald setup          — guided setup (editor, AI, etc.)");
    println!("  mald ai chat \"...\" — chat with your KB (needs Ollama)");
    println!("  mald help <topic>   — learn about a feature (ai, search, sync, ...)");

    // Suggest setup if editor isn't in PATH
    let config = ConfigManager::load(&config_path)?;
    let editor = config.get_string("editor").unwrap_or_else(|| "nvim".into());
    let editor_missing = !crate::commands::doctor::which_exists_pub(&editor);
    if editor_missing {
        println!("\nTip: '{}' not found in PATH. Run `mald setup` to configure your editor.", editor);
    }

    Ok(())
}
