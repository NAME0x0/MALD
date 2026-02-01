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
    let editor = choose_editor()?;
    config.set("editor", serde_json::Value::String(editor.clone()))?;
    println!(
        "  {} Editor set to {}\n",
        "->".green(),
        editor.as_str().bold()
    );

    // 2. Choose KB name
    let kb_name = prompt_string("  Knowledge base name", "personal")?;
    let kb_path = home.join("kb").join(&kb_name);
    ensure_directory(&kb_path)?;
    config.set("default_kb", serde_json::Value::String(kb_name.clone()))?;
    println!(
        "  {} Created KB: {}\n",
        "->".green(),
        kb_name.as_str().bold()
    );

    // 3. Create default templates
    crate::commands::templates::init_defaults()?;
    println!("  {} Default templates installed\n", "->".green());

    // 4. Create sample note
    create_sample_note(&kb_path, &kb_name)?;
    println!("  {} Sample note created\n", "->".green());

    // 5. Build FTS index
    print!("  Indexing knowledge base...");
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
            "mald setup ai".cyan()
        );
    }

    println!();
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
    println!("  Let's set up your knowledge base.\n");
}

fn print_next_steps(kb_name: &str) {
    println!("  {}", "Setup complete!".green().bold());
    println!();
    println!("  {}", "Next steps:".bold());
    println!(
        "    {}          Open today's daily note",
        "mald today".cyan()
    );
    println!("    {}  Create a new note", "mald new \"Title\"".cyan());
    println!("    {}   Quick capture a thought", "mald q <text>".cyan());
    println!("    {}        Interactive search", "mald search".cyan());
    println!(
        "    {}   Browse knowledge base",
        "mald open".cyan().to_string().as_str()
    );
    println!("    {}   Set up local AI", "mald setup ai".cyan());
    println!("    {}        Check your setup", "mald doctor".cyan());
    println!();
    println!("  Run {} for all commands.", "mald --help".cyan());
    let _ = kb_name; // used in banner context
}

/// Detect installed editors and present a choice.
fn choose_editor() -> Result<String> {
    let editors = [
        ("nvim", "Neovim"),
        ("vim", "Vim"),
        ("code", "VS Code"),
        ("nano", "Nano"),
        ("hx", "Helix"),
        ("emacs", "Emacs"),
    ];

    let mut available: Vec<(&str, &str)> = Vec::new();
    for (cmd, name) in &editors {
        if crate::commands::doctor::which_exists_pub(cmd) {
            available.push((cmd, name));
        }
    }

    if available.is_empty() {
        available = editors.to_vec();
    }

    println!("  {}:", "Choose your editor".bold());
    for (i, (cmd, name)) in available.iter().enumerate() {
        let detected = if crate::commands::doctor::which_exists_pub(cmd) {
            " (detected)".green().to_string()
        } else {
            String::new()
        };
        println!("    {}. {} ({}){}", i + 1, name, cmd, detected);
    }

    print!("  Choice [1]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(available[0].0.to_string());
    }

    if let Ok(n) = input.parse::<usize>() {
        if n >= 1 && n <= available.len() {
            return Ok(available[n - 1].0.to_string());
        }
    }

    // Allow typing the editor name directly
    Ok(input.to_string())
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

fn create_sample_note(kb_path: &std::path::Path, kb_name: &str) -> Result<()> {
    let index_path = kb_path.join("index.md");
    if !index_path.exists() {
        let date = chrono::Local::now().format("%Y-%m-%d");
        std::fs::write(
            &index_path,
            format!(
                "---\ntitle: {kb_name}\ncreated: {date}\ntags: [mald, guide]\n---\n\n\
                # {kb_name}\n\n\
                Welcome to your knowledge base!\n\n\
                ## Getting Started\n\n\
                - [[inbox]] — capture thoughts quickly\n\
                - [[projects]] — track active work\n\n\
                ## Quick Tips\n\n\
                - Use `mald q \"thought\"` to capture ideas fast\n\
                - Use `mald today` to open your daily note\n\
                - Use `mald search` for interactive fuzzy search\n\
                - Use `[[wikilinks]]` to connect your notes\n\n\
                ## Tasks\n\n\
                - [ ] Create your first note with `mald new \"My Note\"`\n\
                - [ ] Try quick capture: `mald q buy milk`\n\
                - [ ] Explore search: `mald search`\n"
            ),
        )?;
    }
    Ok(())
}
