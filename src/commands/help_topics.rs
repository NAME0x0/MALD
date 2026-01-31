use anyhow::Result;

/// Show extended help for a topic, explaining concepts and workflows.
pub async fn run(topic: &str) -> Result<()> {
    let text = match topic.to_lowercase().as_str() {
        "ai" => AI_HELP,
        "search" => SEARCH_HELP,
        "sync" => SYNC_HELP,
        "templates" => TEMPLATE_HELP,
        "daemon" => DAEMON_HELP,
        "plugins" => PLUGIN_HELP,
        "graph" => GRAPH_HELP,
        "tasks" => TASK_HELP,
        _ => {
            println!("Unknown topic: '{}'\n", topic);
            println!("Available topics:");
            println!("  ai         — AI chat, RAG, summarize, quiz, models");
            println!("  search     — FTS, semantic search, date filtering");
            println!("  sync       — Git-based version history and remote sync");
            println!("  templates  — Note templates with variables");
            println!("  daemon     — Background indexer and file watcher");
            println!("  plugins    — Custom plugin system");
            println!("  graph      — Knowledge graph, backlinks, orphans");
            println!("  tasks      — Task aggregation from markdown checkboxes");
            return Ok(());
        }
    };
    println!("{}", text);
    Ok(())
}

const AI_HELP: &str = "\
AI Features (requires Ollama)

MALD uses Ollama for all AI. Nothing leaves your machine.

SETUP:
  1. Install Ollama: https://ollama.com
  2. Pull a model:  ollama pull llama3.2
  3. For search:    ollama pull nomic-embed-text
  4. Configure:     mald config set ai.default_model llama3.2

CHAT (with cited RAG):
  mald ai chat                 Interactive REPL
  mald ai chat \"question\"      One-shot question
  mald ai chat --kb work       Use specific KB as context
  mald ai chat --new           Start fresh conversation

  In REPL: /quit, /new, /history

  Responses include [1], [2] citations pointing to source files.

ANALYSIS:
  mald ai summarize note1 note2    Summarize notes
  mald ai quiz note1 --count 10    Generate quiz questions
  mald ai briefing --days 3        Recent activity briefing
  mald ai compare note1 note2      Compare notes
  mald ai timeline --kb work       Extract timeline
  mald ai explain complex-note     Simplify a note

MODELS:
  mald ai models                   List installed models
  mald ai pull <model>             Download a model
  mald ai index <kb>               Build vector embeddings

TROUBLESHOOTING:
  - 'Connection refused': Ollama isn't running. Start with `ollama serve`.
  - 'Model not found': Pull it with `ollama pull <model>`.
  - Slow responses: Use a smaller model or check `ollama ps`.
  - Run `mald doctor` to check AI setup.";

const SEARCH_HELP: &str = "\
Search

MALD has two search backends that work together:

FTS (Full-Text Search):
  - Always available, no AI needed
  - SQLite FTS5 with ranking
  - Searches title + content

Semantic (Vector Search):
  - Requires Ollama + embeddings
  - Finds conceptually related content
  - Build with: mald ai index <kb>

USAGE:
  mald search \"query\"             Search all KBs
  mald search \"query\" --since 7d  Last 7 days only
  mald search \"query\" --since 2025-01-01
  mald search                     Interactive TUI with preview pane
  mald search \"query\" -k 20       Return top 20 results

HOW IT WORKS:
  1. If vector index exists and Ollama is running, tries semantic search first
  2. Falls back to FTS5 (always works)
  3. Search covers ALL knowledge bases automatically

TROUBLESHOOTING:
  - No results: Run `mald reindex` to rebuild the index
  - Stale results: The daemon auto-indexes on file changes
  - Start daemon: `mald daemon start`";

const SYNC_HELP: &str = "\
Sync (Git-based version history)

MALD uses git to track changes and sync across machines.

SETUP:
  mald sync init                    Initialize git in ~/.mald/
  cd ~/.mald && git remote add origin <url>   Add remote

DAILY USE:
  mald sync                        Commit + pull + push
  mald sync commit                 Commit only (no push)
  mald sync log                    Show history
  mald sync log note-name          History for one note
  mald sync undo                   Revert last change

CONFLICT HANDLING:
  If a rebase conflict occurs during sync, MALD will:
  1. Abort the rebase
  2. Try a merge instead
  3. If that fails, print instructions for manual resolution

TIPS:
  - Sync works without a remote (local history only)
  - Use `mald config set hooks.on_save \"...\"` to auto-commit on save
  - The .gitignore excludes daemon files and cache";

const TEMPLATE_HELP: &str = "\
Templates

Templates are markdown files with variables, stored in ~/.mald/templates/.

VARIABLES:
  {{title}}     Note title
  {{date}}      YYYY-MM-DD
  {{time}}      HH:MM
  {{datetime}}  YYYY-MM-DD HH:MM
  {{kb}}        Knowledge base name
  {{year}}      YYYY
  {{month}}     MM
  {{day}}       DD
  {{weekday}}   Monday, Tuesday, etc.

USAGE:
  mald template list               List templates
  mald template init               Create default templates
  mald template create standup     Create a new template
  mald template edit meeting       Edit existing template
  mald template delete old-one     Delete a template
  mald new \"Title\" --template meeting   Use a template

DEFAULT TEMPLATES:
  meeting, project, reference, decision, review";

const DAEMON_HELP: &str = "\
Daemon (background indexer)

The daemon watches ~/.mald/kb/ for file changes and auto-indexes them.

USAGE:
  mald daemon start      Start in background
  mald daemon stop       Stop the daemon
  mald daemon status     Check if running

WHAT IT DOES:
  - Watches all KB directories for file changes
  - Re-indexes changed files into FTS (instant)
  - Re-indexes into vector index (if Ollama running)
  - Updates modified timestamp in frontmatter
  - Fires hooks (hooks.on_save)

IPC:
  - Windows: TCP on 127.0.0.1:7433
  - Linux/macOS: Unix socket ~/.mald/daemon.sock
  - Authenticated with token in ~/.mald/daemon.token

TROUBLESHOOTING:
  - Status shows 'running' but daemon is dead:
    Delete ~/.mald/daemon.pid and restart
  - Run `mald doctor` to check daemon health";

const PLUGIN_HELP: &str = "\
Plugins

Any executable in ~/.mald/plugins/ becomes a MALD command.

SETUP:
  mkdir -p ~/.mald/plugins

CREATE A PLUGIN:
  # Create a shell script
  echo '#!/bin/bash' > ~/.mald/plugins/hello
  echo 'echo Hello from MALD' >> ~/.mald/plugins/hello
  chmod +x ~/.mald/plugins/hello

  # Or a Python script, Rust binary, etc.

RUN:
  mald x-hello              Run the 'hello' plugin
  mald plugin list           List installed plugins
  mald plugin run hello      Alternative syntax

ENVIRONMENT VARIABLES:
  MALD_HOME    Path to ~/.mald/
  MALD_NOTE    Path to the current note (if applicable)
  MALD_EVENT   Event name (on_create, on_save, etc.)

HOOKS VS PLUGINS:
  - Hooks run automatically on events (on_create, on_save)
  - Plugins run manually via `mald x-<name>`";

const GRAPH_HELP: &str = "\
Knowledge Graph

MALD parses wikilinks [[target]] to build a link graph across your notes.

COMMANDS:
  mald links note-name       Outgoing links from a note
  mald backlinks note-name   Notes that link TO a note
  mald orphans               Notes with no incoming links
  mald graph stats           Note/link/tag counts
  mald graph broken-links    Wikilinks to non-existent notes
  mald graph view            Output Mermaid diagram

MERMAID OUTPUT:
  `mald graph view` produces a Mermaid diagram you can paste into:
  - GitHub markdown
  - Obsidian
  - mermaid.live
  Broken links are highlighted in red.

TIPS:
  - Use `mald rename` to rename notes (updates all backlinks)
  - `mald review` includes orphan and broken link detection
  - `mald doctor` counts broken links across all KBs";

const TASK_HELP: &str = "\
Tasks

MALD aggregates `- [ ]` checkboxes from your markdown notes.

USAGE:
  mald tasks              Open tasks from default KB
  mald tasks --kb work    From specific KB
  mald tasks --all        From all KBs

  mald review             Includes tasks in the weekly review
  mald                    Dashboard shows top 5 tasks

SYNTAX:
  - [ ] This is an open task
  - [x] This is a completed task

Tasks are plain markdown. Edit them in any editor.
MALD just reads and aggregates — it never modifies your task checkboxes.";
