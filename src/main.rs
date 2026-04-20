#![allow(dead_code)]

mod ai;
mod cli;
mod commands;
mod config;
mod daemon;
mod errors;
mod fs;
#[cfg(feature = "gui")]
mod gui;
mod index;
mod parser;
mod web_assets;

use clap::Parser;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let startup_time = Instant::now();

    // Set up logging: stderr for interactive, file for daemon
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("mald=info".parse().unwrap());

    // Check if we're running as daemon (suppress stderr logging)
    let is_daemon = std::env::args().any(|a| a == "_run");

    if is_daemon {
        // Daemon: log to file
        let log_dir = fs::mald_home().join("logs");
        let _ = fs::ensure_directory(&log_dir);
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("daemon.log"))
            .unwrap_or_else(|_| {
                // Fallback to /dev/null equivalent
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(if cfg!(windows) { "NUL" } else { "/dev/null" })
                    .unwrap()
            });
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(std::sync::Mutex::new(log_file))
            .with_ansi(false)
            .init();
    } else {
        // Interactive: log to stderr (suppressed by default, enabled with RUST_LOG)
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    tracing::debug!(
        elapsed_ms = startup_time.elapsed().as_millis(),
        "logging initialized"
    );

    // Auto-start daemon if MALD is set up (silent, non-blocking)
    if !is_daemon {
        commands::daemon::ensure_running();
        tracing::debug!(
            elapsed_ms = startup_time.elapsed().as_millis(),
            "daemon check complete"
        );
    }

    let mut cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => cli::print_parse_error_and_exit(err),
    };
    tracing::debug!(
        elapsed_ms = startup_time.elapsed().as_millis(),
        "CLI parsed"
    );

    if let Some(cli::Command::Launch { kb }) = &cli.command {
        let query = if kb.is_empty() {
            None
        } else {
            Some(kb.join(" "))
        };
        let selected = commands::kb::resolve_launch_target(query.as_deref())?;
        let Some(selected) = selected else {
            return Ok(());
        };
        commands::kb::set_default_kb_sync(&selected)?;
        cli.command = Some(cli::Command::Gui);
    }

    let wants_gui = matches!(cli.command, Some(cli::Command::Gui))
        || (cli.command.is_none() && cli.args.is_empty() && !cli.tui && !is_daemon);

    if wants_gui {
        if !fs::mald_home().exists() {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(commands::wizard::run())?;
        }
        launch_gui(startup_time);
    }

    // Everything else needs tokio runtime
    let startup_ms = startup_time.elapsed().as_millis();
    tracing::debug!(startup_ms, "entering tokio runtime");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(cli, is_daemon, startup_time))
}

#[cfg(feature = "gui")]
fn launch_gui(startup_time: Instant) -> ! {
    let startup_ms = startup_time.elapsed().as_millis();
    tracing::info!(startup_ms, "launching GUI");
    if startup_ms > 500 {
        tracing::warn!(startup_ms, "startup exceeded 500ms target");
    }
    if let Err(err) = gui::run() {
        use crossterm::style::Stylize;
        eprintln!("{}: {:?}", "error".red().bold(), err);
        std::process::exit(1);
    }
    std::process::exit(0);
}

#[cfg(not(feature = "gui"))]
fn launch_gui(startup_time: Instant) -> ! {
    let startup_ms = startup_time.elapsed().as_millis();
    tracing::info!(startup_ms, "GUI unavailable, launching TUI instead");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("current-thread runtime");
    if let Err(err) = runtime.block_on(commands::tui::run_full_tui()) {
        use crossterm::style::Stylize;
        eprintln!("{}: {:?}", "error".red().bold(), err);
        std::process::exit(1);
    }
    std::process::exit(0);
}

async fn async_main(cli: cli::Cli, is_daemon: bool, startup_time: Instant) -> anyhow::Result<()> {
    // Routing: --tui flag → ratatui TUI
    if cli.tui && cli.command.is_none() && cli.args.is_empty() {
        let startup_ms = startup_time.elapsed().as_millis();
        tracing::info!(startup_ms, "launching TUI");
        if startup_ms > 500 {
            tracing::warn!(startup_ms, "startup exceeded 500ms target");
        }
        if let Err(err) = commands::tui::run_full_tui().await {
            use crossterm::style::Stylize;
            eprintln!("{}: {:?}", "error".red().bold(), err);
            std::process::exit(1);
        }
        return Ok(());
    }

    let _ = is_daemon; // Silence unused warning

    // Log startup time for CLI commands
    tracing::debug!(
        elapsed_ms = startup_time.elapsed().as_millis(),
        "executing command"
    );

    if let Err(err) = cli::run(cli).await {
        if let Some(ctx) = errors::extract_contextual(&err) {
            ctx.print();
        } else {
            use crossterm::style::Stylize;
            eprintln!("{}: {:?}", "error".red().bold(), err);
        }
        std::process::exit(1);
    }
    Ok(())
}
