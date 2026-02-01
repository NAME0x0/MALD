use anyhow::Result;

use crate::config::ConfigManager;
use crate::fs::mald_home;

/// Self-diagnostic command. Checks everything a user needs for MALD to work.
pub async fn run() -> Result<()> {
    let home = mald_home();
    let mut issues = 0u32;
    let mut warnings = 0u32;

    println!("MALD Doctor\n");

    // 1. MALD home exists
    check(
        "MALD home directory",
        home.exists(),
        &format!("{}", home.display()),
        "Run `mald init`",
        &mut issues,
    );

    // 2. Config
    let config_path = home.join("config").join("config.json");
    let config_ok = config_path.exists();
    check(
        "Config file",
        config_ok,
        &config_path.to_string_lossy(),
        "Run `mald init`",
        &mut issues,
    );

    let config = if config_ok {
        ConfigManager::load(&config_path).ok()
    } else {
        None
    };

    // 3. Knowledge bases
    let kb_dir = home.join("kb");
    let kb_count = if kb_dir.exists() {
        std::fs::read_dir(&kb_dir)
            .map(|d| d.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count())
            .unwrap_or(0)
    } else {
        0
    };
    check(
        "Knowledge bases",
        kb_count > 0,
        &format!("{} found", kb_count),
        "Run `mald kb create <name>`",
        &mut issues,
    );

    // 4. FTS index
    let meta_path = home.join("index").join("metadata.db");
    check(
        "FTS search index",
        meta_path.exists(),
        &meta_path.to_string_lossy(),
        "Run `mald init` to build index",
        &mut issues,
    );

    // 5. Vector index
    let hnsw_path = home.join("index").join("hnsw.bin");
    let hnsw_detail = if hnsw_path.exists() {
        hnsw_path.to_string_lossy().to_string()
    } else {
        "Not built".to_string()
    };
    check_warn(
        "Vector index (HNSW)",
        hnsw_path.exists(),
        &hnsw_detail,
        "Run `mald ai index <kb>` (requires Ollama)",
        &mut warnings,
    );

    // 6. Templates
    let template_dir = home.join("templates");
    let template_count = if template_dir.exists() {
        crate::fs::find_files(&template_dir, "md")
            .map(|f| f.len())
            .unwrap_or(0)
    } else {
        0
    };
    check_warn(
        "Templates",
        template_count > 0,
        &format!("{} found", template_count),
        "Run `mald template init`",
        &mut warnings,
    );

    // 7. Editor
    let editor = config
        .as_ref()
        .and_then(|c| c.get_string("editor"))
        .unwrap_or_else(|| "nvim".into());
    let editor_exists = which_exists(&editor);
    check(
        &format!("Editor ({})", editor),
        editor_exists,
        "found in PATH",
        &format!("Install {} or run `mald config set editor <editor>`", editor),
        &mut issues,
    );

    // 8. Ollama
    let ollama_running = check_ollama().await;
    check_warn(
        "Ollama",
        ollama_running,
        if ollama_running {
            "running"
        } else {
            "not running"
        },
        "Install Ollama and run `ollama serve` for AI features",
        &mut warnings,
    );

    // 9. Models (only if Ollama running)
    if ollama_running {
        if let Some(cfg) = &config {
            let client = crate::ai::ollama::OllamaClient::from_config(cfg);
            let models = client.list_models().await.unwrap_or_default();
            check_warn(
                "AI models",
                !models.is_empty(),
                &format!("{} installed", models.len()),
                "Run `mald ai pull llama3.2`",
                &mut warnings,
            );
        }
    }

    // 10. Git (for sync)
    let git_exists = which_exists("git");
    check_warn(
        "Git (for sync)",
        git_exists,
        "found in PATH",
        "Install git for `mald sync` support",
        &mut warnings,
    );

    // 11. Broken links
    if kb_count > 0 {
        let kb_dir = home.join("kb");
        let mut broken_total = 0usize;
        if let Ok(entries) = std::fs::read_dir(&kb_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().is_dir() {
                    if let Ok(docs) = crate::parser::graph::parse_knowledge_base(&entry.path()) {
                        broken_total += crate::commands::review::find_broken_links(&docs).len();
                    }
                }
            }
        }
        check_warn(
            "Broken wikilinks",
            broken_total == 0,
            if broken_total == 0 {
                "none".to_string()
            } else {
                format!("{} broken", broken_total)
            }
            .as_str(),
            "Run `mald graph broken-links` to see details",
            &mut warnings,
        );
    }

    // 12. Plugins
    let plugin_dir = home.join("plugins");
    let plugin_count = if plugin_dir.exists() {
        std::fs::read_dir(&plugin_dir)
            .map(|d| d.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).count())
            .unwrap_or(0)
    } else {
        0
    };
    check_warn(
        "Plugins",
        true, // always ok, just informational
        &format!("{} installed", plugin_count),
        "",
        &mut warnings,
    );

    // 13. Log files
    let log_dir = home.join("logs");
    let log_file = log_dir.join("daemon.log");
    let log_size = if log_file.exists() {
        std::fs::metadata(&log_file)
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };
    check_warn(
        "Daemon log",
        log_dir.exists(),
        &if log_file.exists() {
            format!("{:.1} KB", log_size as f64 / 1024.0)
        } else {
            "no log yet".to_string()
        },
        "Logs created when daemon runs",
        &mut warnings,
    );

    // 14. Chat history dir
    let chat_dir = home.join("sessions").join("chat");
    check_warn(
        "Chat history directory",
        chat_dir.exists(),
        &chat_dir.to_string_lossy(),
        "Run `mald init` to create",
        &mut warnings,
    );

    // Summary
    use crossterm::style::Stylize;
    println!("\n─────────────────────────");
    if issues == 0 && warnings == 0 {
        println!("{}", "All checks passed.".green().bold());
    } else {
        if issues > 0 {
            println!("{}", format!("{} issue(s) found.", issues).red().bold());
        }
        if warnings > 0 {
            println!("{}", format!("{} warning(s).", warnings).yellow());
        }
    }

    Ok(())
}

fn check(name: &str, ok: bool, detail: &str, fix: &str, issues: &mut u32) {
    use crossterm::style::Stylize;
    if ok {
        println!("  {} {} — {}", "[ok]".green().bold(), name, detail);
    } else {
        println!("  {} {} — {}", "[FAIL]".red().bold(), name, fix.yellow());
        *issues += 1;
    }
}

fn check_warn(name: &str, ok: bool, detail: &str, fix: &str, warnings: &mut u32) {
    use crossterm::style::Stylize;
    if ok {
        println!("  {} {} — {}", "[ok]".green().bold(), name, detail);
    } else {
        println!("  {} {} — {}", "[warn]".yellow().bold(), name, fix.dark_grey());
        *warnings += 1;
    }
}

/// Check if a command exists in PATH. Exposed for use by other modules.
pub fn which_exists_pub(cmd: &str) -> bool {
    which_exists(cmd)
}

fn which_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("where")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

async fn check_ollama() -> bool {
    reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .is_ok()
}
