mod ai;
mod cli;
mod commands;
mod config;
mod daemon;
mod fs;
mod index;
mod parser;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    }

    let cli = cli::Cli::parse();
    cli::run(cli).await
}
