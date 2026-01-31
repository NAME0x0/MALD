use anyhow::Result;
use std::io::{self, Write};

use crate::config::ConfigManager;
use crate::fs::{ensure_directory, mald_home};

pub async fn run() -> Result<()> {
    println!("MALD Setup\n");

    // 1. Init directory structure
    let home = mald_home();
    let dirs = [
        "kb/personal",
        "config",
        "sessions",
        "cache",
        "ai/models",
        "index",
    ];
    for dir in &dirs {
        ensure_directory(&home.join(dir))?;
    }

    let config_path = home.join("config").join("config.json");
    let mut config = ConfigManager::load(&config_path)?;

    // 2. Editor
    let editor = prompt("Editor", &[("nvim", "Neovim"), ("vim", "Vim"), ("code", "VS Code"), ("nano", "Nano")])?;
    config.set("editor", serde_json::Value::String(editor))?;

    // 3. Check Ollama
    println!("\nChecking Ollama...");
    let client = crate::ai::ollama::OllamaClient::from_config(&config);
    if client.is_running().await {
        println!("  Ollama is running.");

        let models = client.list_models().await.unwrap_or_default();
        if models.is_empty() {
            println!("  No models installed.");
            let pull = prompt_yn("Pull llama3.2 (recommended starter model)?")?;
            if pull {
                println!("  Pulling llama3.2... (this may take a while)");
                if let Err(e) = client.pull_model("llama3.2").await {
                    println!("  Failed to pull: {}. You can pull later with `mald ai pull llama3.2`", e);
                } else {
                    println!("  Done.");
                }
            }

            let pull_embed = prompt_yn("Pull nomic-embed-text (needed for semantic search)?")?;
            if pull_embed {
                println!("  Pulling nomic-embed-text...");
                if let Err(e) = client.pull_model("nomic-embed-text").await {
                    println!("  Failed: {}. Pull later with `mald ai pull nomic-embed-text`", e);
                } else {
                    println!("  Done.");
                }
            }
        } else {
            println!("  Models installed: {}", models.join(", "));
        }
        config.set("ai.backend", serde_json::Value::String("ollama".into()))?;
    } else {
        println!("  Ollama is not running.");
        println!("  Install from https://ollama.com and run `ollama serve`");
        println!("  MALD will work without AI (text search, notes, graph features).");
    }

    // 4. Default KB
    let kb_name = prompt_string("Default knowledge base name", "personal")?;
    let kb_path = home.join("kb").join(&kb_name);
    ensure_directory(&kb_path)?;
    config.set("default_kb", serde_json::Value::String(kb_name.clone()))?;

    let index_path = kb_path.join("index.md");
    if !index_path.exists() {
        std::fs::write(
            &index_path,
            format!(
                "---\ntitle: {}\ncreated: {}\ntags: []\n---\n\n# {}\n\nWelcome to your knowledge base.\n\n- [[inbox]] — capture thoughts\n- [[projects]] — active work\n",
                kb_name,
                chrono::Local::now().format("%Y-%m-%d"),
                kb_name,
            ),
        )?;
    }

    // 5. Index into FTS
    println!("\nIndexing knowledge base...");
    let count = crate::daemon::indexer::fts_index_kb(&kb_path)?;
    println!("  Indexed {} files.", count);

    println!("\nSetup complete.\n");
    println!("Quick start:");
    println!("  mald today          — open today's daily note");
    println!("  mald new \"Title\"    — create a new note");
    println!("  mald search         — interactive fuzzy search");
    println!("  mald links <note>   — see outgoing links");
    println!("  mald backlinks <n>  — see who links to a note");
    println!("  mald orphans        — find unlinked notes");

    Ok(())
}

fn prompt(label: &str, options: &[(&str, &str)]) -> Result<String> {
    println!("\n{}:", label);
    for (i, (val, desc)) in options.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, desc, val);
    }
    print!("Choice [1]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(options[0].0.to_string());
    }

    if let Ok(n) = input.parse::<usize>() {
        if n >= 1 && n <= options.len() {
            return Ok(options[n - 1].0.to_string());
        }
    }

    // Allow typing the value directly
    Ok(input.to_string())
}

fn prompt_string(label: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", label, default);
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

fn prompt_yn(label: &str) -> Result<bool> {
    print!("{} [Y/n]: ", label);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(input.is_empty() || input == "y" || input == "yes")
}
