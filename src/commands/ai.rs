use anyhow::{bail, Result};

use crate::ai::chat;
use crate::ai::history::{self, ChatSession};
use crate::ai::ollama::OllamaClient;
use crate::config::ConfigManager;
use crate::fs::mald_home;

fn load_config() -> Result<(ConfigManager, OllamaClient)> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let client = OllamaClient::from_config(&config);
    Ok((config, client))
}

/// Default chat model — Google Gemma 3N for fast inference and low compute.
pub const DEFAULT_CHAT_MODEL: &str = "gemma3n:e4b";
/// Fallback if gemma3n isn't available yet on the user's Ollama version.
pub const FALLBACK_CHAT_MODEL: &str = "gemma3:4b";
/// Default embedding model.
pub const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";

/// Ensure Ollama is running. If not installed, offer lazy auto-setup.
/// This is the key to "just works" AI — users never need to manually install.
async fn ensure_ollama(client: &OllamaClient) -> Result<()> {
    if client.is_running().await {
        return Ok(());
    }

    // Only attempt auto-setup in interactive terminals
    use std::io::IsTerminal;
    if !std::io::stderr().is_terminal() {
        return Err(crate::errors::bail_ctx_topic(
            "Cannot connect to Ollama",
            "Run `mald setup ai` to install Ollama and pull models",
            "ai",
        ));
    }

    use crossterm::style::Stylize;
    eprintln!("{}", "Ollama not running. Starting automatic setup...".yellow());

    // Try starting ollama serve first (maybe installed but not running)
    let _ = std::process::Command::new("ollama")
        .arg("serve")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    // Wait briefly for it to come up
    for _ in 0..5 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if client.is_running().await {
            eprintln!("{}", "Ollama is now running.".green());

            // Auto-pull default model if needed
            auto_pull_default_model(client).await;
            return Ok(());
        }
    }

    // Not installed — run full setup
    eprintln!("Ollama not found. Running `mald ai setup`...\n");
    setup_ai().await?;

    // Check again after setup
    if client.is_running().await {
        return Ok(());
    }

    Err(crate::errors::bail_ctx_topic(
        "Cannot connect to Ollama after setup attempt",
        "Run `mald ai setup` manually, or install from https://ollama.com",
        "ai",
    ))
}

/// Auto-pull the default chat model if no models are installed.
async fn auto_pull_default_model(client: &OllamaClient) {
    use crossterm::style::Stylize;

    let models = client.list_models().await.unwrap_or_default();
    if !models.is_empty() {
        return; // User already has models
    }

    eprintln!(
        "{}",
        "No AI models installed. Pulling default model...".yellow()
    );

    // Try gemma3n first (fastest inference), fall back to gemma3:4b
    let model = if client.pull_model_stream(DEFAULT_CHAT_MODEL).await.is_ok() {
        DEFAULT_CHAT_MODEL
    } else {
        eprintln!("gemma3n not available, trying {}...", FALLBACK_CHAT_MODEL);
        if client
            .pull_model_stream(FALLBACK_CHAT_MODEL)
            .await
            .is_ok()
        {
            FALLBACK_CHAT_MODEL
        } else {
            eprintln!(
                "{}",
                "Could not pull default model. Pull manually: mald ai pull gemma3n:e4b"
                    .yellow()
            );
            return;
        }
    };
    eprintln!("{} {} ready", "->".green(), model);

    // Also pull embedding model
    let _ = client.pull_model_stream(DEFAULT_EMBED_MODEL).await;

    // Update config
    let config_path = mald_home().join("config").join("config.json");
    if let Ok(mut config) = ConfigManager::load(&config_path) {
        let _ = config.set(
            "ai.default_model",
            serde_json::Value::String(model.to_string()),
        );
    }
}

/// Load note contents by names. Returns (title, content) pairs.
fn load_notes(notes: &[String], kb: Option<&str>) -> Result<Vec<(String, String)>> {
    let mut results = Vec::new();
    for note in notes {
        let path = super::run::resolve_note_pub(note, kb)?;
        let content = std::fs::read_to_string(&path)?;
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| note.clone());
        results.push((title, content));
    }
    Ok(results)
}

/// Load all notes from a KB (for digest/briefing).
fn load_all_notes(kb: Option<&str>) -> Result<Vec<(String, String)>> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Knowledge base '{}' not found", kb_name),
            format!("Create it with: `mald kb create {}`", kb_name),
        ));
    }

    let files = crate::fs::find_files(&kb_path, "md")?;
    let mut results = Vec::new();
    for f in files {
        let content = std::fs::read_to_string(&f)?;
        let title = f
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        results.push((title, content));
    }
    Ok(results)
}

/// Load recently modified notes (last N days).
fn load_recent_notes(kb: Option<&str>, days: u64) -> Result<Vec<(String, String)>> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let kb_name = kb
        .map(String::from)
        .or_else(|| config.get_string("default_kb"))
        .unwrap_or_else(|| "personal".into());

    let kb_path = mald_home().join("kb").join(&kb_name);
    if !kb_path.exists() {
        return Err(crate::errors::bail_ctx(
            format!("Knowledge base '{}' not found", kb_name),
            format!("Create it with: `mald kb create {}`", kb_name),
        ));
    }

    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 86400);
    let files = crate::fs::find_files(&kb_path, "md")?;
    let mut results = Vec::new();
    for f in &files {
        if let Ok(meta) = f.metadata() {
            if let Ok(modified) = meta.modified() {
                if modified >= cutoff {
                    let content = std::fs::read_to_string(f)?;
                    let title = f
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    results.push((title, content));
                }
            }
        }
    }
    Ok(results)
}

/// Chat with cited RAG and conversation history.
/// If message is None, enters interactive REPL mode.
pub async fn chat_cmd(message: Option<&str>, kb: Option<&str>, new_session: bool) -> Result<()> {
    let (config, client) = load_config()?;
    ensure_ollama(&client).await?;

    let kb_name = kb.unwrap_or("personal");

    let mut session = if new_session {
        ChatSession::new(kb_name)
    } else {
        history::latest_session(kb_name).unwrap_or_else(|| ChatSession::new(kb_name))
    };

    match message {
        Some(msg) => {
            // Single-shot mode with streaming
            do_chat_turn(&client, &config, msg, kb_name, &mut session).await?;
        }
        None => {
            // Interactive REPL
            println!("MALD AI Chat (kb: {}) — type /quit to exit\n", kb_name);
            loop {
                use std::io::Write;
                print!("you> ");
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }
                if input == "/quit" || input == "/exit" || input == "/q" {
                    break;
                }
                if input == "/new" {
                    session = ChatSession::new(kb_name);
                    println!("(new conversation started)\n");
                    continue;
                }
                if input == "/history" {
                    for msg in &session.messages {
                        let prefix = if msg.role == "user" { "you" } else { "ai" };
                        let preview: String = msg.content.chars().take(80).collect();
                        println!("  {}: {}", prefix, preview);
                    }
                    println!();
                    continue;
                }

                print!("\nai> ");
                std::io::stdout().flush()?;
                do_chat_turn(&client, &config, input, kb_name, &mut session).await?;
                println!();
            }
        }
    }

    Ok(())
}

async fn do_chat_turn(
    client: &OllamaClient,
    config: &ConfigManager,
    message: &str,
    kb_name: &str,
    session: &mut ChatSession,
) -> Result<()> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| DEFAULT_CHAT_MODEL.into());

    // Retrieve sources
    let sources = chat::retrieve_sources(client, config, message, 5).await?;

    // Build messages with context
    let context = if sources.is_empty() {
        String::new()
    } else {
        let mut ctx = String::from("Sources:\n\n");
        for (i, src) in sources.iter().enumerate() {
            let loc = if src.start_line > 0 {
                format!("{}:L{}-{}", src.path, src.start_line, src.end_line)
            } else {
                src.path.clone()
            };
            ctx.push_str(&format!("[{}] {}\n{}\n\n", i + 1, loc, src.content));
        }
        ctx
    };

    let system_prompt = if context.is_empty() {
        format!(
            "You are a helpful assistant for the knowledge base '{}'. Answer directly and concisely.",
            kb_name
        )
    } else {
        format!(
            "You are a helpful assistant for the knowledge base '{}'. \
             Answer using ONLY the following sources. Cite sources using [1], [2], etc.\n\n{}",
            kb_name, context
        )
    };

    let mut messages = vec![crate::ai::ollama::ChatMessage {
        role: "system".into(),
        content: system_prompt,
    }];

    // Include conversation history (last 10 messages)
    let history_start = session.messages.len().saturating_sub(10);
    for msg in &session.messages[history_start..] {
        messages.push(msg.clone());
    }
    messages.push(crate::ai::ollama::ChatMessage {
        role: "user".into(),
        content: message.to_string(),
    });

    // Stream the response
    let response = client.chat_streaming(&model, &messages).await?;

    // Print citations
    if !sources.is_empty() {
        println!("{}", chat::format_citations(&sources));
    }

    // Save conversation
    session.add("user", message);
    session.add("assistant", &response);
    session.save()?;

    Ok(())
}

/// Summarize notes.
pub async fn summarize(notes: &[String], kb: Option<&str>) -> Result<()> {
    let (config, client) = load_config()?;
    ensure_ollama(&client).await?;

    let contents = load_notes(notes, kb)?;
    if contents.is_empty() {
        bail!("No notes found");
    }

    let result = chat::summarize(&client, &config, &contents).await?;
    println!("{}", result);
    Ok(())
}

/// Generate quiz from notes.
pub async fn quiz(notes: &[String], kb: Option<&str>, count: usize) -> Result<()> {
    let (config, client) = load_config()?;
    ensure_ollama(&client).await?;

    let contents = if notes.is_empty() {
        load_all_notes(kb)?
    } else {
        load_notes(notes, kb)?
    };

    if contents.is_empty() {
        bail!("No notes found");
    }

    let result = chat::quiz(&client, &config, &contents, count).await?;
    println!("{}", result);
    Ok(())
}

/// Generate daily briefing from recent notes.
pub async fn briefing(kb: Option<&str>, days: u64) -> Result<()> {
    let (config, client) = load_config()?;
    ensure_ollama(&client).await?;

    let contents = load_recent_notes(kb, days)?;
    if contents.is_empty() {
        println!("No notes modified in the last {} days.", days);
        return Ok(());
    }

    println!("Analyzing {} recent notes...\n", contents.len());
    let result = chat::briefing(&client, &config, &contents).await?;
    println!("{}", result);
    Ok(())
}

/// Compare notes.
pub async fn compare(notes: &[String], kb: Option<&str>) -> Result<()> {
    let (config, client) = load_config()?;
    ensure_ollama(&client).await?;

    let contents = load_notes(notes, kb)?;
    if contents.len() < 2 {
        bail!("Need at least 2 notes to compare");
    }

    let result = chat::compare(&client, &config, &contents).await?;
    println!("{}", result);
    Ok(())
}

/// Extract timeline from notes.
pub async fn timeline(notes: &[String], kb: Option<&str>) -> Result<()> {
    let (config, client) = load_config()?;
    ensure_ollama(&client).await?;

    let contents = if notes.is_empty() {
        load_all_notes(kb)?
    } else {
        load_notes(notes, kb)?
    };

    if contents.is_empty() {
        bail!("No notes found");
    }

    let result = chat::timeline(&client, &config, &contents).await?;
    println!("{}", result);
    Ok(())
}

/// Explain a note in simple terms.
pub async fn explain(note: &str, kb: Option<&str>) -> Result<()> {
    let (config, client) = load_config()?;
    ensure_ollama(&client).await?;

    let path = super::run::resolve_note_pub(note, kb)?;
    let content = std::fs::read_to_string(&path)?;
    let title = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| note.to_string());

    let result = chat::explain(&client, &config, &title, &content).await?;
    println!("{}", result);
    Ok(())
}

/// List chat sessions.
pub async fn chat_history() -> Result<()> {
    let sessions = history::list_sessions();
    if sessions.is_empty() {
        println!("No chat sessions.");
    } else {
        for s in sessions {
            println!(
                "  {} | kb: {} | {} messages | {}",
                s.id,
                s.kb,
                s.messages.len(),
                s.created
            );
        }
    }
    Ok(())
}

pub async fn models() -> Result<()> {
    let (_config, client) = load_config()?;
    if !client.is_running().await {
        println!("Ollama is not running. Start it with `ollama serve`.");
        return Ok(());
    }

    let models = client.list_models().await?;
    if models.is_empty() {
        println!("No models found. Pull one with `mald ai pull <model>`.");
    } else {
        for model in models {
            println!("  {}", model);
        }
    }
    Ok(())
}

pub async fn pull(model: &str) -> Result<()> {
    let (_config, client) = load_config()?;
    ensure_ollama(&client).await?;
    println!("Pulling model: {}", model);
    client.pull_model(model).await?;
    println!("Done.");
    Ok(())
}

/// Auto-install and configure Ollama, pull default models.
pub async fn setup_ai() -> Result<()> {
    use crossterm::style::Stylize;

    println!("{}\n", "MALD AI Setup".bold());

    // 1. Check if ollama binary exists
    let has_ollama = crate::commands::doctor::which_exists_pub("ollama");

    if !has_ollama {
        println!("  Ollama not found in PATH. Installing...\n");

        let install_ok = install_ollama().await;
        if !install_ok {
            println!();
            println!("  {} Could not install Ollama automatically.", "warning:".yellow().bold());
            println!("  Install manually:");
            println!("    Linux/macOS: curl -fsSL https://ollama.com/install.sh | sh");
            println!("    Windows:     Download from https://ollama.com/download");
            println!();
            println!("  After installing, run {} again.", "mald setup ai".cyan());
            return Ok(());
        }
        println!("  {} Ollama installed\n", "->".green());
    } else {
        println!("  {} Ollama found in PATH\n", "->".green());
    }

    // 2. Wait for Ollama to be running
    print!("  Waiting for Ollama to start...");
    std::io::Write::flush(&mut std::io::stdout())?;

    let client = OllamaClient::new("http://localhost:11434");
    let mut running = false;
    for _ in 0..30 {
        if client.is_running().await {
            running = true;
            break;
        }
        // Try starting ollama serve in background
        if !running {
            let _ = std::process::Command::new("ollama")
                .arg("serve")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    if !running {
        println!(" {}", "timeout".red());
        println!();
        println!("  Ollama did not start within 60 seconds.");
        println!("  Start it manually: ollama serve");
        println!("  Then run {} again.", "mald setup ai".cyan());
        return Ok(());
    }
    println!(" {}", "running".green());

    // 3. Pull default models with progress
    println!();
    println!("  Pulling models (this may take a few minutes on first run)...\n");

    // Try gemma3n first (Google Gemma 3 Nano — fastest inference, lowest compute)
    // Fall back to gemma3:4b if gemma3n isn't available in this Ollama version
    println!("  Pulling {} (chat model — fast inference, low compute)...", DEFAULT_CHAT_MODEL);
    let chat_model = if client.pull_model_stream(DEFAULT_CHAT_MODEL).await.is_ok() {
        println!("  {} {} ready", "->".green(), DEFAULT_CHAT_MODEL);
        DEFAULT_CHAT_MODEL
    } else {
        println!("  {} not available, trying {}...", DEFAULT_CHAT_MODEL, FALLBACK_CHAT_MODEL);
        if client.pull_model_stream(FALLBACK_CHAT_MODEL).await.is_ok() {
            println!("  {} {} ready", "->".green(), FALLBACK_CHAT_MODEL);
            FALLBACK_CHAT_MODEL
        } else {
            println!("  {} Could not pull chat model", "warning:".yellow());
            println!("  You can pull manually: mald ai pull {}", DEFAULT_CHAT_MODEL);
            DEFAULT_CHAT_MODEL
        }
    };

    println!();
    println!("  Pulling {} (embedding model)...", DEFAULT_EMBED_MODEL);
    if let Err(e) = client.pull_model_stream(DEFAULT_EMBED_MODEL).await {
        println!(
            "  {} Failed to pull {}: {}",
            "warning:".yellow(),
            DEFAULT_EMBED_MODEL,
            e
        );
        println!("  You can pull it later: mald ai pull {}", DEFAULT_EMBED_MODEL);
    } else {
        println!("  {} {} ready", "->".green(), DEFAULT_EMBED_MODEL);
    }

    // 4. Update config
    let config_path = mald_home().join("config").join("config.json");
    if let Ok(mut config) = ConfigManager::load(&config_path) {
        let _ = config.set("ai.backend", serde_json::Value::String("ollama".into()));
        let _ = config.set(
            "ai.default_model",
            serde_json::Value::String(chat_model.to_string()),
        );
    }

    println!();
    println!("  {}", "AI setup complete!".green().bold());
    println!();
    println!("  Try it out:");
    println!("    {}             Interactive AI chat", "mald ai chat".cyan());
    println!(
        "    {}  Build semantic search index",
        "mald ai index <kb>".cyan()
    );
    println!("    {}           List installed models", "mald ai models".cyan());
    println!();
    println!("  Model: {} (Google Gemma 3 Nano — optimized for on-device speed)", chat_model);

    Ok(())
}

async fn install_ollama() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Download OllamaSetup.exe and run silently
        println!("  Downloading Ollama installer...");
        let url = "https://ollama.com/download/OllamaSetup.exe";
        if let Ok(resp) = reqwest::get(url).await {
            if let Ok(bytes) = resp.bytes().await {
                let tmp = std::env::temp_dir().join("OllamaSetup.exe");
                if std::fs::write(&tmp, &bytes).is_ok() {
                    println!("  Running installer (this may take a moment)...");
                    let status = std::process::Command::new(&tmp)
                        .arg("/SILENT")
                        .status();
                    let _ = std::fs::remove_file(&tmp);
                    return status.map(|s| s.success()).unwrap_or(false);
                }
            }
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Use the official install script
        println!("  Running: curl -fsSL https://ollama.com/install.sh | sh");
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg("curl -fsSL https://ollama.com/install.sh | sh")
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }
}

pub async fn index(kb_name: &str) -> Result<()> {
    let kb_path = mald_home().join("kb").join(kb_name);
    if !kb_path.exists() {
        bail!("Knowledge base '{}' not found", kb_name);
    }

    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let client = OllamaClient::from_config(&config);
    ensure_ollama(&client).await?;

    println!("Indexing knowledge base: {}", kb_name);
    crate::daemon::indexer::full_index(&kb_path, &config).await?;
    println!("Indexing complete.");
    Ok(())
}
