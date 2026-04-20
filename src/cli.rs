use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use crossterm::style::Stylize;

#[derive(Parser)]
#[command(
    name = "mald",
    about = "Markdown Archive & Localized Daemon — terminal-first PKM",
    after_help = "Run `mald` to open the desktop app.\n\
                  Run `mald gui` to force the desktop app.\n\
                  Run `mald tui` for the interactive terminal UI.\n\
                  Run `mald today` for your daily note.\n\
                  Run `mald <text>` to search and open a note.\n\n\
                  Shortcuts: q=capture, f=find, e=edit, s=search, n=new, t=today"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Launch the ratatui TUI instead of the GPU app
    #[arg(long)]
    pub tui: bool,

    /// Free-form text treated as a search query when no subcommand matches
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Guided onboarding and setup helpers
    #[command(
        after_help = "Examples:\n  mald setup\n  mald setup editor\n  mald setup editor code\n  mald setup path"
    )]
    Setup {
        #[command(subcommand)]
        action: Option<SetupAction>,
    },

    /// Open the desktop app explicitly
    #[command(
        alias = "app",
        visible_alias = "ui",
        visible_alias = "desktop",
        after_help = "Examples:\n  mald gui\n  mald ui\n  mald desktop"
    )]
    Gui,

    /// Pick a space and launch MALD into it
    #[command(
        visible_alias = "go",
        after_help = "Examples:\n  mald launch\n  mald launch work\n  mald launch client acme"
    )]
    Launch {
        #[arg(trailing_var_arg = true)]
        kb: Vec<String>,
    },

    /// Initialize MALD directory structure
    #[command(after_help = "Examples:\n  mald init")]
    Init,

    /// Create a new note
    #[command(
        alias = "n",
        after_help = "Examples:\n  mald new \"Meeting Notes\"\n  mald new \"API Design\" --kb work\n  mald new \"Standup\" --template meeting\n  mald new \"Incident\" --path projects/ops"
    )]
    New {
        title: String,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long)]
        template: Option<String>,
        #[arg(
            long,
            value_name = "DIR",
            help = "Create the note inside a subdirectory of the active space"
        )]
        path: Option<String>,
    },

    /// Open today's daily note
    #[command(
        alias = "t",
        after_help = "Examples:\n  mald today\n  mald today --kb work"
    )]
    Today {
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Quick capture to daily note (no quotes needed)
    #[command(
        alias = "q",
        after_help = "Examples:\n  mald q buy groceries\n  mald q --tag work fix the auth bug\n  mald q --kb study read chapter 5\n\nNote: flags (--tag, --kb) must come before the text."
    )]
    Capture {
        #[arg(trailing_var_arg = true, required = true)]
        text: Vec<String>,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Search and open in one step (uses fzf if available)
    #[command(
        alias = "f",
        after_help = "Examples:\n  mald find rust async\n  mald f meeting\n  mald find              # interactive (fzf or TUI)"
    )]
    Find {
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Fuzzy-find and open a note in your editor
    #[command(
        alias = "e",
        after_help = "Examples:\n  mald edit rust\n  mald e meeting\n  mald edit api --kb work"
    )]
    Edit {
        query: String,
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Rename a note and update all wikilink references
    #[command(
        after_help = "Examples:\n  mald rename old-name \"New Title\"\n  mald rename api-design \"API v2 Design\" --kb work"
    )]
    Rename {
        old_name: String,
        new_name: String,
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Open the active space directory in your editor
    #[command(after_help = "Examples:\n  mald open\n  mald open --kb work")]
    Open {
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Show detailed note metadata
    #[command(after_help = "Examples:\n  mald info rust-notes\n  mald info meeting --kb work")]
    Info {
        note: String,
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Space management
    #[command(visible_alias = "space", visible_alias = "spaces")]
    Kb {
        #[command(subcommand)]
        action: KbAction,
    },

    /// Search notes (all spaces, no args = interactive TUI)
    #[command(
        alias = "s",
        after_help = "Examples:\n  mald search \"rust async\"\n  mald search \"meeting\" --since 7d\n  mald search --json     # JSON output for scripts\n  mald search            # opens interactive TUI"
    )]
    Search {
        query: Option<String>,
        #[arg(short, long, default_value = "10")]
        k: usize,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        json: bool,
    },

    /// Show outgoing links from a note
    #[command(after_help = "Examples:\n  mald links rust-notes")]
    Links {
        note: String,
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Show notes that link to a given note
    #[command(after_help = "Examples:\n  mald backlinks rust-notes")]
    Backlinks {
        note: String,
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Find orphaned notes (no incoming links)
    Orphans {
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// List all tags or filter notes by tag
    #[command(
        after_help = "Examples:\n  mald tags           # list all tags\n  mald tags rust      # notes tagged #rust\n  mald tags --json    # JSON output"
    )]
    Tags {
        tag: Option<String>,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(long)]
        json: bool,
    },

    /// Aggregate open tasks from all notes
    #[command(
        after_help = "Examples:\n  mald tasks\n  mald tasks --kb work\n  mald tasks --all\n  mald tasks --json"
    )]
    Tasks {
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },

    /// Weekly/daily review: recent activity, stale notes, orphans, broken links
    #[command(
        after_help = "Examples:\n  mald review\n  mald review --days 30\n  mald review --kb work"
    )]
    Review {
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long, default_value = "7")]
        days: u64,
    },

    /// Execute code blocks from a note
    #[command(
        after_help = "Examples:\n  mald run script-note --list         # preview blocks without running\n  mald run script-note --allow-exec   # execute all blocks\n  mald run script-note -n 2 --allow-exec  # execute only block 2\n  mald run script-note --save --allow-exec  # execute and save output\n\nSAFETY: Code execution requires --allow-exec flag. Always review code before running."
    )]
    Run {
        note: String,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short = 'n', long)]
        block: Option<usize>,
        #[arg(short, long)]
        list: bool,
        #[arg(short, long)]
        save: bool,
        /// Allow code execution (required for safety)
        #[arg(long)]
        allow_exec: bool,
    },

    /// Render a note in the terminal with colors
    #[command(after_help = "Examples:\n  mald preview rust-notes")]
    Preview {
        note: String,
        #[arg(short, long)]
        kb: Option<String>,
    },

    /// Export notes to HTML or portable markdown
    #[command(
        after_help = "Examples:\n  mald export rust-notes\n  mald export --all --output-dir ~/export\n  mald export --all --format md"
    )]
    Export {
        note: Option<String>,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        output_dir: Option<String>,
        #[arg(long, default_value = "html")]
        format: String,
    },

    /// Import markdown from external folder (Obsidian, Logseq, etc.)
    #[command(
        after_help = "Examples:\n  mald import ~/ObsidianVault\n  mald import ~/notes --kb research\n  mald import ~/notes --flatten"
    )]
    Import {
        source: String,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long)]
        flatten: bool,
    },

    /// Serve a space as a local website (with capture API)
    #[command(
        after_help = "Examples:\n  mald serve\n  mald serve --port 8080\n  mald serve --kb work"
    )]
    Serve {
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long, default_value = "3131")]
        port: u16,
    },

    /// Template management
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Git-based sync and version history
    #[command(
        after_help = "Examples:\n  mald sync              # commit + push + pull\n  mald sync init         # initialize git\n  mald sync log          # show history\n  mald sync undo         # revert last change"
    )]
    Sync {
        #[command(subcommand)]
        action: Option<SyncAction>,
    },

    /// Knowledge graph analysis
    #[command(
        after_help = "Examples:\n  mald graph stats\n  mald graph broken-links\n  mald graph view         # output Mermaid diagram"
    )]
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },

    /// Session management (tmux/shell)
    #[command(hide = true)]
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// AI-powered analysis (requires Ollama)
    #[command(
        after_help = "Examples:\n  mald ai chat                    # interactive REPL\n  mald ai chat \"what is X?\"\n  mald ai summarize note1 note2\n  mald ai quiz note1 --count 10\n  mald ai briefing --days 3"
    )]
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },

    /// Configuration management
    #[command(
        after_help = "Examples:\n  mald config get editor\n  mald config set editor code\n  mald config set hooks.on_create \"echo created\""
    )]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Background daemon management
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Rebuild the search index from scratch
    #[command(after_help = "Examples:\n  mald reindex")]
    Reindex,

    /// Extended help on a topic (ai, search, sync, templates, daemon, plugins, graph, tasks)
    #[command(after_help = "Examples:\n  mald help-topic ai\n  mald help-topic search")]
    HelpTopic { topic: String },

    /// Run self-diagnostics
    #[command(after_help = "Examples:\n  mald doctor")]
    Doctor,

    /// Benchmark HNSW vector index performance
    #[command(
        hide = true,
        after_help = "Examples:\n  mald bench\n  mald bench --dim 768 --count 5000"
    )]
    Bench {
        #[arg(short, long, default_value = "384")]
        dim: usize,
        #[arg(short, long, default_value = "1000")]
        count: usize,
    },

    /// Generate shell completions
    #[command(
        hide = true,
        after_help = "Examples:\n  mald completions bash >> ~/.bashrc\n  mald completions zsh > ~/.zfunc/_mald\n  mald completions powershell >> $PROFILE"
    )]
    Completions { shell: Shell },

    /// Manage plugins
    #[command(
        hide = true,
        after_help = "Examples:\n  mald plugin list\n  mald plugin run my-script"
    )]
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },

    /// Run a plugin (shorthand: mald x-<name>)
    #[command(name = "x", hide = true)]
    RunPlugin {
        name: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Check for updates and self-update
    #[command(after_help = "Examples:\n  mald update")]
    Update,

    /// Show text dashboard (non-interactive)
    #[command(after_help = "Examples:\n  mald status")]
    Status,

    /// Open the interactive terminal UI
    #[command(
        name = "tui",
        alias = "hub",
        alias = "h",
        after_help = "Examples:\n  mald tui\n  mald tui --help"
    )]
    Hub,

    /// Scan and fix broken wikilinks
    #[command(
        name = "fix-links",
        after_help = "Examples:\n  mald fix-links          # show broken links\n  mald fix-links --fix    # auto-fix high-confidence matches"
    )]
    FixLinks {
        #[arg(short, long)]
        kb: Option<String>,
        /// Auto-fix high-confidence matches
        #[arg(long)]
        fix: bool,
    },
}

#[derive(Subcommand)]
pub enum KbAction {
    /// Create a new space
    #[command(after_help = "Examples:\n  mald kb create work\n  mald kb create client acme prod")]
    Create {
        #[arg(required = true, trailing_var_arg = true)]
        name: Vec<String>,
    },
    /// List all spaces
    List {
        #[arg(long)]
        json: bool,
    },
    /// Show the current default space
    Current,
    /// Set the default space
    #[command(
        after_help = "Examples:\n  mald kb use              # picker when interactive\n  mald kb use work\n  mald kb use client acme"
    )]
    Use {
        #[arg(trailing_var_arg = true)]
        name: Vec<String>,
    },
    /// Open a space in your editor
    #[command(
        after_help = "Examples:\n  mald kb open            # picker when interactive\n  mald kb open work\n  mald kb open client acme"
    )]
    Open {
        #[arg(trailing_var_arg = true)]
        name: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum SetupAction {
    /// Pick or auto-detect an editor without typing full paths
    #[command(
        after_help = "Examples:\n  mald setup editor\n  mald setup editor code\n  mald setup editor neovim"
    )]
    Editor {
        #[arg(trailing_var_arg = true)]
        editor: Vec<String>,
    },
    /// Install the current MALD binary into a stable location and add it to PATH
    #[command(after_help = "Examples:\n  mald setup path")]
    Path,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// List available templates
    List,
    /// Create a note from a template
    #[command(
        after_help = "Examples:\n  mald template use meeting \"Q4 Planning\"\n  mald template use project \"MALD Roadmap\" --path projects/mald"
    )]
    Use {
        template: String,
        title: String,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(
            long,
            value_name = "DIR",
            help = "Create the note inside a subdirectory of the active space"
        )]
        path: Option<String>,
    },
    /// Create a new template
    #[command(after_help = "Examples:\n  mald template create standup")]
    Create { name: String },
    /// Edit a template
    Edit { name: String },
    /// Delete a template
    Delete { name: String },
    /// Create default templates (meeting, project, reference, decision, review)
    Init,
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Initialize git in MALD home
    Init,
    /// Commit current changes
    Commit,
    /// Show version history
    Log {
        note: Option<String>,
        #[arg(short, long, default_value = "20")]
        count: usize,
    },
    /// Undo last change
    Undo,
}

#[derive(Subcommand)]
pub enum GraphAction {
    /// Show graph statistics
    Stats,
    /// Find broken wikilinks
    BrokenLinks,
    /// Output Mermaid diagram of link structure
    View,
}

#[derive(Subcommand)]
pub enum SessionAction {
    Start {
        #[arg(short, long)]
        kb: Option<String>,
    },
    List,
}

#[derive(Subcommand)]
pub enum AiAction {
    /// Chat with your current space (no args = interactive REPL with streaming)
    Chat {
        message: Option<String>,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(long)]
        new: bool,
    },
    /// Summarize one or more notes
    Summarize {
        notes: Vec<String>,
        #[arg(short, long)]
        kb: Option<String>,
    },
    /// Generate quiz questions from your notes
    Quiz {
        notes: Vec<String>,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long, default_value = "5")]
        count: usize,
    },
    /// Daily briefing from recent notes
    Briefing {
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long, default_value = "7")]
        days: u64,
    },
    /// Compare two or more notes
    Compare {
        notes: Vec<String>,
        #[arg(short, long)]
        kb: Option<String>,
    },
    /// Extract timeline from notes
    Timeline {
        notes: Vec<String>,
        #[arg(short, long)]
        kb: Option<String>,
    },
    /// Explain a note in simple terms
    Explain {
        note: String,
        #[arg(short, long)]
        kb: Option<String>,
    },
    /// Auto-install and configure Ollama for AI features
    #[command(after_help = "Examples:\n  mald ai setup")]
    Setup,
    /// List chat sessions
    History,
    /// List available Ollama models
    Models,
    /// Download a model
    Pull { model: String },
    /// Build vector embeddings for a space
    Index { kb: String },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Get { key: String },
    Set { key: String, value: String },
}

#[derive(Subcommand)]
pub enum DaemonAction {
    Start,
    Stop,
    Status,
    #[clap(hide = true)]
    _Run,
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// List installed plugins
    List,
    /// Run a plugin by name
    Run {
        name: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

pub fn print_parse_error_and_exit(err: clap::Error) -> ! {
    let kind = err.kind();
    if matches!(kind, ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
        err.exit();
    }
    let _ = err.print();

    for hint in parse_error_hints(&std::env::args().skip(1).collect::<Vec<_>>(), kind) {
        eprintln!("  {} {}", "hint:".cyan().bold(), hint.as_str().cyan());
    }
    eprintln!(
        "  {}",
        "Run `mald --help` for the full command list.".dark_grey()
    );

    std::process::exit(2);
}

fn parse_error_hints(args: &[String], kind: ErrorKind) -> Vec<String> {
    if args.is_empty() {
        return Vec::new();
    }

    let head = args[0].to_lowercase();
    let mut hints = match head.as_str() {
        "setup"
            if args
                .get(1)
                .is_some_and(|arg| arg.eq_ignore_ascii_case("ai")) =>
        {
            vec![
                "Did you mean `mald ai setup`?".into(),
                "Use `mald ai chat` after setup to chat over your notes.".into(),
            ]
        }
        "setup" if args.len() == 1 => vec![
            "Try `mald setup` for the guided wizard.".into(),
            "Try `mald setup editor` to pick VS Code, Neovim, or another editor.".into(),
            "Try `mald setup path` if `mald` does not work in Command Prompt or PowerShell.".into(),
        ],
        "editor" | "code" | "nvim" | "neovim" => vec![
            "Try `mald setup editor` to pick a detected editor.".into(),
            "You can also run `mald config get editor` to inspect the current one.".into(),
        ],
        "path" | "install" => vec![
            "Try `mald setup path` to add MALD to PATH from inside the app.".into(),
            "If you downloaded the standalone EXE, this is the easiest way to make `mald` work everywhere.".into(),
        ],
        "help" if args.get(1).is_some_and(|arg| !arg.starts_with('-')) => vec![
            format!("Did you mean `mald help-topic {}`?", args[1]),
            "Use `mald --help` for the full command list.".into(),
        ],
        "ui" | "app" | "desktop" | "window" => vec![
            "Did you mean `mald gui`?".into(),
            "Use `mald tui` if you want the terminal UI instead.".into(),
        ],
        "workspace" | "workspaces" | "vault" | "vaults" => vec![
            "Try `mald kb list` to see available spaces.".into(),
            "Try `mald launch` to pick one and open MALD there.".into(),
        ],
        "kb" | "space" | "spaces" if args.len() == 1 => vec![
            "Try `mald kb list` to see spaces.".into(),
            "Try `mald kb current` to inspect the active one.".into(),
            "Try `mald kb use <name>` to switch the default space.".into(),
        ],
        "ai" if args.len() == 1 => vec![
            "Try `mald ai chat` to chat over your current space.".into(),
            "Try `mald ai setup` to install and configure local AI.".into(),
        ],
        "search" if args.len() == 1 => vec![
            "Run `mald search \"term\"` for CLI results.".into(),
            "Use `mald tui` or `mald gui` for interactive search.".into(),
        ],
        _ => best_command_matches(&head)
            .into_iter()
            .map(|candidate| format!("Did you mean `mald {candidate}`?"))
            .collect(),
    };

    if matches!(
        kind,
        ErrorKind::InvalidSubcommand | ErrorKind::UnknownArgument
    ) && hints.is_empty()
    {
        hints.push("Try `mald gui`, `mald launch`, `mald tui`, or `mald kb list`.".into());
    }

    hints.truncate(3);
    hints
}

fn best_command_matches(input: &str) -> Vec<&'static str> {
    const COMMANDS: &[&str] = &[
        "gui",
        "launch",
        "tui",
        "setup",
        "setup editor",
        "setup path",
        "init",
        "today",
        "new",
        "capture",
        "find",
        "edit",
        "open",
        "kb list",
        "kb current",
        "kb use",
        "search",
        "doctor",
        "status",
        "ai chat",
        "ai setup",
    ];

    let mut scored: Vec<(&str, usize)> = COMMANDS
        .iter()
        .map(|candidate| (*candidate, suggestion_distance(input, candidate)))
        .filter(|(_, score)| *score <= 4)
        .collect();
    scored.sort_by_key(|(_, score)| *score);
    scored
        .into_iter()
        .map(|(candidate, _)| candidate)
        .take(3)
        .collect()
}

fn suggestion_distance(input: &str, candidate: &str) -> usize {
    let candidate = candidate.split_whitespace().next().unwrap_or(candidate);
    if candidate.starts_with(input) || input.starts_with(candidate) {
        return 0;
    }
    if candidate.contains(input) || input.contains(candidate) {
        return 1;
    }

    let a: Vec<char> = input.chars().collect();
    let b: Vec<char> = candidate.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b.len()]
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let free_args = cli.args;
    match cli.command {
        // No args: non-interactive falls back to dashboard; interactive GUI is handled in main.rs.
        None => {
            if !free_args.is_empty() {
                // Free-form text: smart search → open
                crate::commands::find::run(free_args, None).await
            } else if !crate::fs::mald_home().exists() {
                crate::commands::wizard::run().await
            } else {
                crate::commands::dashboard::run().await
            }
        }

        Some(Command::Gui) | Some(Command::Launch { .. }) => Ok(()),
        Some(Command::Hub) => crate::commands::tui::run_full_tui().await,
        Some(Command::Status) => crate::commands::dashboard::run().await,
        Some(Command::Setup { action }) => crate::commands::setup::run(action).await,
        Some(Command::Init) => crate::commands::init::run().await,
        Some(Command::New {
            title,
            kb,
            template,
            path,
        }) => {
            if let Some(tmpl) = template {
                crate::commands::templates::create_from_template(
                    &tmpl,
                    &title,
                    kb.as_deref(),
                    path.as_deref(),
                )
                .await
            } else {
                crate::commands::new::run(&title, kb.as_deref(), path.as_deref()).await
            }
        }
        Some(Command::Today { kb }) => crate::commands::new::today(kb.as_deref()).await,
        Some(Command::Capture { text, kb, tag }) => {
            let joined = text.join(" ");
            crate::commands::capture::run(&joined, kb.as_deref(), tag.as_deref()).await
        }
        Some(Command::Find { query, kb }) => crate::commands::find::run(query, kb.as_deref()).await,
        Some(Command::Edit { query, kb }) => {
            crate::commands::edit::run(&query, kb.as_deref()).await
        }
        Some(Command::Rename {
            old_name,
            new_name,
            kb,
        }) => crate::commands::rename::run(&old_name, &new_name, kb.as_deref()).await,
        Some(Command::Open { kb }) => crate::commands::open::run(kb.as_deref()).await,
        Some(Command::Info { note, kb }) => crate::commands::info::run(&note, kb.as_deref()).await,
        Some(Command::Kb { action }) => match action {
            KbAction::Create { name } => {
                let name = name.join(" ");
                crate::commands::kb::create(name.trim()).await
            }
            KbAction::List { json } => crate::commands::kb::list(json).await,
            KbAction::Current => crate::commands::kb::current().await,
            KbAction::Use { name } => {
                let name = name.join(" ");
                let requested = (!name.trim().is_empty()).then_some(name);
                crate::commands::kb::use_kb(requested.as_deref()).await
            }
            KbAction::Open { name } => {
                let name = name.join(" ");
                let requested = (!name.trim().is_empty()).then_some(name);
                crate::commands::kb::open(requested.as_deref()).await
            }
        },
        Some(Command::Search {
            query,
            k,
            since,
            json,
        }) => match query {
            Some(q) => crate::commands::search::run(&q, k, since.as_deref(), json).await,
            None => {
                if json {
                    return Err(crate::errors::bail_ctx(
                        "--json requires a query",
                        "Usage: `mald search \"query\" --json`",
                    ));
                }
                crate::commands::search::interactive()
            }
        },
        Some(Command::Links { note, kb }) => {
            crate::commands::graph::links(&note, kb.as_deref()).await
        }
        Some(Command::Backlinks { note, kb }) => {
            crate::commands::graph::backlinks(&note, kb.as_deref()).await
        }
        Some(Command::Orphans { kb }) => crate::commands::graph::orphans(kb.as_deref()).await,
        Some(Command::Tags { tag, kb, json }) => match tag {
            Some(t) => crate::commands::tags::filter(&t, kb.as_deref(), json).await,
            None => crate::commands::tags::list(kb.as_deref(), json).await,
        },
        Some(Command::Tasks { kb, all, json }) => {
            crate::commands::tasks::list(kb.as_deref(), all, json).await
        }
        Some(Command::Review { kb, days }) => {
            crate::commands::review::run(kb.as_deref(), days).await
        }
        Some(Command::Run {
            note,
            kb,
            block,
            list,
            save,
            allow_exec,
        }) => {
            if list {
                crate::commands::run::list_blocks(&note, kb.as_deref()).await
            } else {
                crate::commands::run::run(&note, kb.as_deref(), block, save, allow_exec).await
            }
        }
        Some(Command::Preview { note, kb }) => {
            crate::commands::preview::run(&note, kb.as_deref()).await
        }
        Some(Command::Export {
            note,
            kb,
            output,
            all,
            output_dir,
            format,
        }) => {
            if all {
                let dir = output_dir.as_deref().unwrap_or("./mald-export");
                crate::commands::export::export_all(kb.as_deref(), dir, &format).await
            } else if let Some(n) = note {
                crate::commands::export::run(&n, kb.as_deref(), output.as_deref()).await
            } else {
                Err(crate::errors::bail_ctx(
                    "Specify a note name, or use --all to export the entire space",
                    "Examples: `mald export my-note` or `mald export --all`",
                ))
            }
        }
        Some(Command::Import {
            source,
            kb,
            flatten,
        }) => crate::commands::import::run(&source, kb.as_deref(), flatten).await,
        Some(Command::Serve { kb, port }) => crate::commands::serve::run(kb.as_deref(), port).await,
        Some(Command::Template { action }) => match action {
            TemplateAction::List => crate::commands::templates::list().await,
            TemplateAction::Use {
                template,
                title,
                kb,
                path,
            } => {
                crate::commands::templates::create_from_template(
                    &template,
                    &title,
                    kb.as_deref(),
                    path.as_deref(),
                )
                .await
            }
            TemplateAction::Create { name } => crate::commands::templates::create(&name).await,
            TemplateAction::Edit { name } => crate::commands::templates::edit(&name).await,
            TemplateAction::Delete { name } => crate::commands::templates::delete(&name).await,
            TemplateAction::Init => {
                crate::commands::templates::init_defaults()?;
                println!("Default templates created in ~/.mald/templates/");
                Ok(())
            }
        },
        Some(Command::Sync { action }) => match action {
            None => crate::commands::sync::sync().await,
            Some(SyncAction::Init) => crate::commands::sync::init().await,
            Some(SyncAction::Commit) => crate::commands::sync::commit().await,
            Some(SyncAction::Log { note, count }) => {
                crate::commands::sync::log(note.as_deref(), count).await
            }
            Some(SyncAction::Undo) => crate::commands::sync::undo().await,
        },
        Some(Command::Graph { action }) => match action {
            GraphAction::Stats => crate::commands::graph::stats(None).await,
            GraphAction::BrokenLinks => crate::commands::graph::broken_links(None).await,
            GraphAction::View => crate::commands::graph::view(None).await,
        },
        Some(Command::Session { action }) => match action {
            SessionAction::Start { kb } => crate::commands::session::start(kb.as_deref()).await,
            SessionAction::List => crate::commands::session::list().await,
        },
        Some(Command::Ai { action }) => match action {
            AiAction::Chat { message, kb, new } => {
                crate::commands::ai::chat_cmd(message.as_deref(), kb.as_deref(), new).await
            }
            AiAction::Summarize { notes, kb } => {
                crate::commands::ai::summarize(&notes, kb.as_deref()).await
            }
            AiAction::Quiz { notes, kb, count } => {
                crate::commands::ai::quiz(&notes, kb.as_deref(), count).await
            }
            AiAction::Briefing { kb, days } => {
                crate::commands::ai::briefing(kb.as_deref(), days).await
            }
            AiAction::Compare { notes, kb } => {
                crate::commands::ai::compare(&notes, kb.as_deref()).await
            }
            AiAction::Timeline { notes, kb } => {
                crate::commands::ai::timeline(&notes, kb.as_deref()).await
            }
            AiAction::Explain { note, kb } => {
                crate::commands::ai::explain(&note, kb.as_deref()).await
            }
            AiAction::Setup => crate::commands::ai::setup_ai().await,
            AiAction::History => crate::commands::ai::chat_history().await,
            AiAction::Models => crate::commands::ai::models().await,
            AiAction::Pull { model } => crate::commands::ai::pull(&model).await,
            AiAction::Index { kb } => crate::commands::ai::index(&kb).await,
        },
        Some(Command::Config { action }) => match action {
            ConfigAction::Get { key } => crate::commands::config::get(&key).await,
            ConfigAction::Set { key, value } => crate::commands::config::set(&key, &value).await,
        },
        Some(Command::Daemon { action }) => match action {
            DaemonAction::Start => crate::commands::daemon::start().await,
            DaemonAction::Stop => crate::commands::daemon::stop().await,
            DaemonAction::Status => crate::commands::daemon::status().await,
            DaemonAction::_Run => crate::daemon::server::run().await,
        },
        Some(Command::Reindex) => crate::commands::reindex::run().await,
        Some(Command::HelpTopic { topic }) => crate::commands::help_topics::run(&topic).await,
        Some(Command::Doctor) => crate::commands::doctor::run().await,
        Some(Command::Bench { dim, count }) => crate::commands::bench::run(dim, count).await,
        Some(Command::Completions { shell }) => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "mald", &mut std::io::stdout());
            Ok(())
        }
        Some(Command::Plugin { action }) => match action {
            PluginAction::List => crate::commands::plugins::list().await,
            PluginAction::Run { name, args } => crate::commands::plugins::run(&name, &args).await,
        },
        Some(Command::RunPlugin { name, args }) => {
            crate::commands::plugins::run(&name, &args).await
        }
        Some(Command::Update) => crate::commands::update::run().await,
        Some(Command::FixLinks { kb, fix }) => {
            crate::commands::fix_links::run(kb.as_deref(), fix).await
        }
    }
}
