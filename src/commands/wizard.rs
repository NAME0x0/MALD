use anyhow::Result;
use crossterm::style::Stylize;
use std::io::{self, Write};

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

/// Interactive first-run wizard. Launches when MALD_HOME doesn't exist yet.
pub async fn run() -> Result<()> {
    print_banner();

    let home = mald_home();

    // Create directory structure
    println!("  Setting up {}...\n", home.display());
    let dirs = [
        "kb",
        "config",
        "sessions",
        "sessions/chat",
        "cache",
        "ai/models",
        "index",
        "templates",
        "plugins",
        "logs",
        "trash",
    ];
    for dir in &dirs {
        ensure_directory(&home.join(dir))?;
    }

    let config_path = home.join("config").join("config.json");
    let mut config = ConfigManager::load(&config_path)?;

    // 1. Detect and choose editor
    let editor = crate::commands::setup::select_editor(None, true)?;
    config.set("editor", serde_json::Value::String(editor.clone()))?;
    println!(
        "  {} Editor set to {}\n",
        "->".green(),
        editor.as_str().bold()
    );

    // 2. Choose space name
    let kb_name = prompt_string("  Space name", "personal")?;
    let kb_path = home.join("kb").join(&kb_name);
    ensure_directory(&kb_path)?;
    config.set("default_kb", serde_json::Value::String(kb_name.clone()))?;
    println!(
        "  {} Created space: {}\n",
        "->".green(),
        kb_name.as_str().bold()
    );

    // 3. Create default templates
    crate::commands::templates::init_defaults()?;
    println!("  {} Default templates installed\n", "->".green());

    // 4. Create starter notes
    crate::commands::starter::seed_starter_space(&kb_path, &kb_name)?;
    println!("  {} Starter notes created\n", "->".green());

    // 5. Build FTS index
    print!("  Indexing space...");
    io::stdout().flush()?;
    let count = crate::daemon::indexer::fts_index_kb(&kb_path)?;
    println!(" {} {} files indexed\n", "->".green(), count);

    // 6. Check Ollama (non-blocking)
    print!("  Checking for Ollama...");
    io::stdout().flush()?;
    let ollama_running = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .is_ok();
    if ollama_running {
        println!(" {} Ollama is running", "->".green());
        config.set("ai.backend", serde_json::Value::String("ollama".into()))?;
    } else {
        println!(
            " {} not detected. Run {} later for AI features (auto-installs Ollama + Gemma 3N).",
            "->".yellow(),
            "mald ai setup".cyan()
        );
    }

    println!();
    crate::commands::setup::maybe_offer_path_setup()?;
    print_next_steps(&kb_name);

    Ok(())
}

fn print_banner() {
    println!();
    println!(
        "  {}",
        "╔══════════════════════════════════════╗".dark_magenta()
    );
    println!(
        "  {}",
        "║   MALD — Welcome to your PKM        ║".dark_magenta()
    );
    println!(
        "  {}",
        "║   Markdown Archive & Localized Daemon║".dark_magenta()
    );
    println!(
        "  {}",
        "╚══════════════════════════════════════╝".dark_magenta()
    );
    println!();
    println!("  Let's set up your space.\n");
}

fn print_next_steps(kb_name: &str) {
    println!("  {}", "Setup complete!".green().bold());
    println!();
    println!("  {}", "Next steps:".bold());
    println!("    {}            Open the desktop app", "mald".cyan());
    println!(
        "    {}      Pick a space and launch MALD there",
        "mald launch".cyan()
    );
    println!("    {}        Open the terminal UI", "mald tui".cyan());
    println!(
        "    {}          Open today's daily note in your editor",
        "mald today".cyan()
    );
    println!("    {}  Create a new note", "mald new \"Title\"".cyan());
    println!("    {}   Quick capture a thought", "mald q <text>".cyan());
    println!("    {}        Interactive search", "mald search".cyan());
    println!("    {}       Inspect spaces", "mald kb list".cyan());
    println!("    {}  Pick a detected editor", "mald setup editor".cyan());
    println!(
        "    {}   Browse the active space",
        "mald open".cyan().to_string().as_str()
    );
    if cfg!(windows) {
        println!(
            "    {}     Make `mald` work in new terminals",
            "mald setup path".cyan()
        );
    }
    println!("    {}     Set up local AI", "mald ai setup".cyan());
    println!("    {}        Check your setup", "mald doctor".cyan());
    println!();
    println!("  Run {} for all commands.", "mald --help".cyan());
    let _ = kb_name; // used in banner context
}

fn prompt_string(label: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", label, default.bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}
