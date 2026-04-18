use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

#[derive(Parser)]
#[command(
    name = "mald",
    about = "Markdown Archive & Localized Daemon — terminal-first PKM",
    after_help = "Run `mald` with no args to open today's daily note.\n\
                  Run `mald hub` for the interactive TUI.\n\
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
    /// Interactive setup wizard
    #[command(hide = true, after_help = "Examples:\n  mald setup")]
    Setup,

    /// Initialize MALD directory structure
    #[command(after_help = "Examples:\n  mald init")]
    Init,

    /// Create a new note
    #[command(
        alias = "n",
        after_help = "Examples:\n  mald new \"Meeting Notes\"\n  mald new \"API Design\" --kb work\n  mald new \"Standup\" --template meeting"
    )]
    New {
        title: String,
        #[arg(short, long)]
        kb: Option<String>,
        #[arg(short, long)]
        template: Option<String>,
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

    /// Open the KB directory in your editor
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

    /// Knowledge base management
    Kb {
        #[command(subcommand)]
        action: KbAction,
    },

    /// Search notes (all KBs, no args = interactive TUI)
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

    /// Serve KB as local website (with capture API)
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

    /// Open the interactive TUI hub
    #[command(alias = "h", after_help = "Examples:\n  mald hub")]
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
    /// Create a new knowledge base
    #[command(after_help = "Examples:\n  mald kb create work")]
    Create { name: String },
    /// List all knowledge bases
    List {
        #[arg(long)]
        json: bool,
    },
    /// Open a knowledge base in editor
    Open { name: String },
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// List available templates
    List,
    /// Create a note from a template
    #[command(after_help = "Examples:\n  mald template use meeting \"Q4 Planning\"")]
    Use {
        template: String,
        title: String,
        #[arg(short, long)]
        kb: Option<String>,
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
    /// Chat with your KB (no args = interactive REPL with streaming)
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
    /// Build vector embeddings for a KB
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

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let free_args = cli.args;
    match cli.command {
        // No args: open today's note (muscle memory — fastest path)
        None => {
            if !free_args.is_empty() {
                // Free-form text: smart search → open
                crate::commands::find::run(free_args, None).await
            } else if !crate::fs::mald_home().exists() {
                crate::commands::wizard::run().await
            } else if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                crate::commands::new::today(None).await
            } else {
                // Non-interactive (piped/test): show dashboard
                crate::commands::dashboard::run().await
            }
        }

        Some(Command::Hub) => crate::commands::tui::run_full_tui().await,
        Some(Command::Status) => crate::commands::dashboard::run().await,
        Some(Command::Setup) => crate::commands::setup::run().await,
        Some(Command::Init) => crate::commands::init::run().await,
        Some(Command::New {
            title,
            kb,
            template,
        }) => {
            if let Some(tmpl) = template {
                crate::commands::templates::create_from_template(&tmpl, &title, kb.as_deref()).await
            } else {
                crate::commands::new::run(&title, kb.as_deref()).await
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
            KbAction::Create { name } => crate::commands::kb::create(&name).await,
            KbAction::List { json } => crate::commands::kb::list(json).await,
            KbAction::Open { name } => crate::commands::kb::open(&name).await,
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
                    "Specify a note name, or use --all to export entire KB",
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
            } => {
                crate::commands::templates::create_from_template(&template, &title, kb.as_deref())
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
