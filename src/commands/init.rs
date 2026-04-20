use anyhow::Result;

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

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

    let config = ConfigManager::load(&config_path)?;
    let default_kb = config.typed().default_kb;
    let kb_root = home.join("kb");
    let existing_kbs = crate::config::manager::list_kb_names(&kb_root);
    let seed_kb = existing_kbs.first().cloned().unwrap_or(default_kb);

    ensure_directory(&kb_root.join(&seed_kb))?;
    let seed_kb_path = kb_root.join(&seed_kb);
    let existing_notes = crate::fs::find_files(&seed_kb_path, "md")?;
    if existing_notes.is_empty() {
        crate::commands::starter::seed_starter_space(&seed_kb_path, &seed_kb)?;
    }

    // Create default templates
    crate::commands::templates::init_defaults()?;

    // Build FTS index immediately
    let kb_dir = home.join("kb");
    let count = crate::daemon::indexer::fts_index_kb(&kb_dir)?;

    println!("MALD initialized at {}", home.display());
    println!("  Indexed {count} files");
    println!("\nNext steps:");
    println!("  mald                — open the desktop app");
    println!("  mald gui            — force the desktop app explicitly");
    println!("  mald tui            — open the terminal UI");
    println!("  mald launch         — pick a space and open MALD there");
    println!("  mald today          — open today's daily note in your editor");
    println!("  mald new \"Title\"    — create a new note");
    println!("  mald kb list        — inspect available spaces");
    println!("  mald search         — interactive fuzzy search");
    println!("  mald q \"thought\"    — quick capture to daily note");
    println!("  mald setup          — guided setup (editor, AI, etc.)");
    println!("  mald setup editor   — pick VS Code, Neovim, or another editor");
    println!("  mald setup path     — make the `mald` command work in new terminals");
    println!("  mald ai chat \"...\" — chat with your active space (needs Ollama)");
    println!("  mald help-topic ai  — learn about a feature in depth");

    // Suggest setup if editor isn't in PATH
    let editor = config.typed().editor.clone();
    let editor_missing = !crate::commands::doctor::which_exists_pub(&editor);
    if editor_missing {
        println!(
            "\nTip: '{editor}' is not launchable right now. Run `mald setup editor` to pick a detected editor."
        );
    }

    if cfg!(windows) && !crate::commands::setup::mald_on_path() {
        println!(
            "Tip: `mald` is not on PATH for new terminals yet. Run `mald setup path` once to fix that."
        );
    }

    Ok(())
}
