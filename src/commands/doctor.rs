use anyhow::Result;
use std::time::Duration;

use crate::config::ConfigManager;
use crate::fs::mald_home;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorStatus {
    Ok,
    Warning,
    Failure,
}

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub name: String,
    pub status: DoctorStatus,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub issues: u32,
    pub warnings: u32,
}

impl DoctorReport {
    pub fn render_plain(&self) -> String {
        let mut lines = vec!["MALD Doctor".to_string(), String::new()];

        for check in &self.checks {
            let prefix = match check.status {
                DoctorStatus::Ok => "[ok]",
                DoctorStatus::Warning => "[warn]",
                DoctorStatus::Failure => "[FAIL]",
            };
            lines.push(format!("  {prefix} {} - {}", check.name, check.message));
        }

        lines.push(String::new());
        lines.push("─────────────────────────".into());

        if self.issues == 0 && self.warnings == 0 {
            lines.push("All checks passed.".into());
        } else {
            if self.issues > 0 {
                lines.push(format!("{} issue(s) found.", self.issues));
            }
            if self.warnings > 0 {
                lines.push(format!("{} warning(s).", self.warnings));
            }
        }

        lines.join("\n")
    }
}

/// Self-diagnostic command. Checks everything a user needs for MALD to work.
pub async fn run() -> Result<()> {
    let report = collect_report().await?;
    println!("{}", report.render_plain());
    Ok(())
}

pub async fn collect_report() -> Result<DoctorReport> {
    let home = mald_home();
    let mut report = DoctorReport::default();

    // 1. MALD home exists
    push_check(
        &mut report,
        "MALD home directory",
        home.exists(),
        home.display().to_string(),
        "Run `mald init`".into(),
    );

    // 2. Config
    let config_path = home.join("config").join("config.json");
    let config_ok = config_path.exists();
    push_check(
        &mut report,
        "Config file",
        config_ok,
        config_path.display().to_string(),
        "Run `mald init`".into(),
    );

    let config = if config_ok {
        ConfigManager::load(&config_path).ok()
    } else {
        None
    };

    // 3. Spaces
    let kb_dir = home.join("kb");
    let kb_count = if kb_dir.exists() {
        std::fs::read_dir(&kb_dir)
            .map(|dir| {
                dir.filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    push_check(
        &mut report,
        "Spaces",
        kb_count > 0,
        format!("{kb_count} found"),
        "Run `mald kb create <name>`".into(),
    );

    // 3b. MALD command availability
    push_warning(
        &mut report,
        "MALD shell command",
        crate::commands::setup::mald_on_path(),
        if crate::commands::setup::mald_on_path() {
            "available in PATH".into()
        } else {
            "not available in new shells yet".into()
        },
        if cfg!(windows) {
            "Run `mald setup path` to add MALD to PATH for Command Prompt and PowerShell".into()
        } else {
            "Add the MALD binary directory to PATH, or install with `cargo install --path . --features gui`".into()
        },
    );

    // 4. FTS index
    let meta_path = home.join("index").join("metadata.db");
    push_check(
        &mut report,
        "FTS search index",
        meta_path.exists(),
        meta_path.display().to_string(),
        "Run `mald init` to build index".into(),
    );

    // 5. Vector index
    let hnsw_path = home.join("index").join("hnsw.bin");
    push_warning(
        &mut report,
        "Vector index (HNSW)",
        hnsw_path.exists(),
        if hnsw_path.exists() {
            hnsw_path.display().to_string()
        } else {
            "Not built".into()
        },
        "Run `mald ai index <kb>` (requires Ollama)".into(),
    );

    // 6. Templates
    let template_dir = home.join("templates");
    let template_count = if template_dir.exists() {
        crate::fs::find_files(&template_dir, "md")
            .map(|files| files.len())
            .unwrap_or(0)
    } else {
        0
    };
    push_warning(
        &mut report,
        "Templates",
        template_count > 0,
        format!("{template_count} found"),
        "Run `mald template init`".into(),
    );

    // 7. Editor
    let editor = config
        .as_ref()
        .map(|cfg| cfg.typed().editor.clone())
        .unwrap_or_else(|| "nvim".into());
    push_check(
        &mut report,
        &format!("Editor ({editor})"),
        which_exists(&editor),
        "found in PATH".into(),
        format!("Install {editor}, or run `mald setup editor` to choose a detected editor"),
    );

    // 8. Ollama
    let ollama_running = check_ollama().await;
    push_warning(
        &mut report,
        "Ollama",
        ollama_running,
        if ollama_running {
            "running".into()
        } else {
            "not running".into()
        },
        "Install Ollama and run `ollama serve` for AI features".into(),
    );

    // 9. Models (only if Ollama running)
    if ollama_running {
        if let Some(cfg) = &config {
            let client = crate::ai::ollama::OllamaClient::from_config(cfg);
            let models = client.list_models().await.unwrap_or_default();
            push_warning(
                &mut report,
                "AI models",
                !models.is_empty(),
                format!("{} installed", models.len()),
                "Run `mald ai pull llama3.2`".into(),
            );
        }
    }

    // 10. Git (for sync)
    push_warning(
        &mut report,
        "Git (for sync)",
        which_exists("git"),
        "found in PATH".into(),
        "Install git for `mald sync` support".into(),
    );

    // 11. Broken links
    if kb_count > 0 {
        let mut broken_total = 0usize;
        if let Ok(entries) = std::fs::read_dir(&kb_dir) {
            for entry in entries.filter_map(|item| item.ok()) {
                if entry.path().is_dir() {
                    if let Ok(docs) = crate::parser::graph::parse_knowledge_base(&entry.path()) {
                        broken_total += crate::commands::review::find_broken_links(&docs).len();
                    }
                }
            }
        }
        push_warning(
            &mut report,
            "Broken wikilinks",
            broken_total == 0,
            if broken_total == 0 {
                "none".into()
            } else {
                format!("{broken_total} broken")
            },
            "Run `mald graph broken-links` to see details".into(),
        );
    }

    // 12. Plugins
    let plugin_dir = home.join("plugins");
    let plugin_count = if plugin_dir.exists() {
        std::fs::read_dir(&plugin_dir)
            .map(|dir| {
                dir.filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_file())
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    push_warning(
        &mut report,
        "Plugins",
        true,
        format!("{plugin_count} installed"),
        String::new(),
    );

    // 13. Log files
    let log_dir = home.join("logs");
    let log_file = log_dir.join("daemon.log");
    let log_size = if log_file.exists() {
        std::fs::metadata(&log_file)
            .map(|metadata| metadata.len())
            .unwrap_or(0)
    } else {
        0
    };
    push_warning(
        &mut report,
        "Daemon log",
        log_dir.exists(),
        if log_file.exists() {
            format!("{:.1} KB", log_size as f64 / 1024.0)
        } else {
            "no log yet".into()
        },
        "Logs created when daemon runs".into(),
    );

    // 14. Daemon runtime health (if running)
    if let Some(health) = crate::commands::daemon::query_health().await {
        let uptime = format_uptime(health.uptime_secs);
        push_check(
            &mut report,
            "Daemon status",
            health.healthy,
            format!("v{}, up {uptime}", health.version),
            "Daemon reports unhealthy state".into(),
        );

        let failures = health.index_status.index_failures;
        push_warning(
            &mut report,
            "Vector index failures",
            failures == 0,
            if failures == 0 {
                "none".into()
            } else {
                format!("{failures} since startup")
            },
            "Check Ollama connection or run `mald ai index`".into(),
        );

        if let Some(count) = health.index_status.document_count {
            push_warning(
                &mut report,
                "Indexed documents",
                true,
                format!("{count} in FTS"),
                String::new(),
            );
        }
    } else if crate::commands::daemon::is_running() {
        push_warning(
            &mut report,
            "Daemon status",
            false,
            "running but not responding to IPC".into(),
            "Try `mald daemon stop && mald daemon start`".into(),
        );
    }

    // 15. Chat history dir
    let chat_dir = home.join("sessions").join("chat");
    push_warning(
        &mut report,
        "Chat history directory",
        chat_dir.exists(),
        chat_dir.display().to_string(),
        "Run `mald init` to create".into(),
    );

    Ok(report)
}

fn push_check(report: &mut DoctorReport, name: &str, ok: bool, detail: String, fix: String) {
    let status = if ok {
        DoctorStatus::Ok
    } else {
        report.issues += 1;
        DoctorStatus::Failure
    };
    let message = if ok { detail } else { fix };

    report.checks.push(DoctorCheck {
        name: name.into(),
        status,
        message,
    });
}

fn push_warning(report: &mut DoctorReport, name: &str, ok: bool, detail: String, fix: String) {
    let status = if ok {
        DoctorStatus::Ok
    } else {
        report.warnings += 1;
        DoctorStatus::Warning
    };
    let message = if ok { detail } else { fix };

    report.checks.push(DoctorCheck {
        name: name.into(),
        status,
        message,
    });
}

/// Check if a command exists in PATH. Exposed for use by other modules.
pub fn which_exists_pub(cmd: &str) -> bool {
    which_exists(cmd)
}

fn which_exists(cmd: &str) -> bool {
    crate::commands::launch::command_exists(cmd)
}

async fn check_ollama() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_default();
    client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .is_ok()
}

fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}
