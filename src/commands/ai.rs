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

async fn ensure_ollama(client: &OllamaClient) -> Result<()> {
    if !client.is_running().await {
        bail!(
            "Cannot connect to Ollama.\n\n\
             Ollama is required for AI features. To fix this:\n\
             1. Install Ollama: https://ollama.com\n\
             2. Start it: ollama serve\n\
             3. Pull a model: ollama pull llama3.2\n\n\
             Run `mald doctor` to check your setup."
        );
    }
    Ok(())
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
        bail!("Knowledge base '{}' not found", kb_name);
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
        bail!("Knowledge base '{}' not found", kb_name);
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
        .unwrap_or_else(|| "llama3.2".into());

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
