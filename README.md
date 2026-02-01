# MALD

Plain markdown notes + local AI search + zero cloud dependencies.
One binary, no Electron, no sync service. Everything stays on your machine.

**M**arkdown **A**rchive & **L**ocalized **D**aemon — a terminal-first PKM tool with local AI (Ollama), semantic search, cited RAG chat, and a background daemon that keeps everything indexed.

## Install

```bash
# From crates.io
cargo install mald

# From source
git clone https://github.com/NAME0x0/MALD && cd MALD
cargo install --path .

# With local GGUF model support
cargo install --path . --features gguf
```

Prebuilt binaries for Linux, macOS, and Windows are available on the [Releases](https://github.com/NAME0x0/MALD/releases) page.

## Quickstart

```bash
mald init                          # Create ~/.mald/ structure
mald new "Meeting Notes"           # Create a note (opens $EDITOR)
mald q "quick thought"             # Capture without opening editor
mald search "meeting"              # Full-text search across all KBs
mald                               # Dashboard: recent notes, tasks, stats
```

That's it. No accounts, no config, no internet required.

## Core Commands

```
mald                        Dashboard (recent notes, tasks, quick actions)
mald new "Title"            Create a note (opens editor)
mald today                  Open/create today's daily note
mald q "text"               Quick capture to daily note
mald search "query"         Full-text search (all KBs)
mald search                 Interactive TUI search
mald edit "partial name"    Fuzzy-find and open a note
mald tasks                  Aggregate open tasks from checkboxes
mald tags                   List all tags with counts
mald review                 Weekly review: stale notes, orphans, broken links
mald import ~/ObsidianVault Import markdown from any folder
mald rename old "New Name"  Rename + update all [[backlinks]]
mald serve                  Local web server at http://127.0.0.1:3131
```

## JSON Output (for scripts and integrations)

```bash
mald search "query" --json    # Pipe to jq, fzf, editor plugins
mald tasks --json             # Structured task data
mald tags --json              # Tag counts with note lists
mald kb list --json           # KB metadata
```

## AI Features (optional, requires Ollama)

AI is entirely optional. MALD works fully without it — search, notes, tasks, sync all work with zero AI setup. When you're ready:

```bash
ollama pull llama3.2              # Install a model
ollama pull nomic-embed-text      # For semantic search
mald ai chat "what did I write about X?"  # Cited RAG chat
mald ai summarize note1 note2     # Summarize notes
mald ai quiz note1 --count 10     # Generate quiz questions
mald ai briefing --days 7         # Activity briefing
```

Every AI response includes `[1]`, `[2]` citations with file paths and line numbers. Chat sessions persist across invocations.

## Capture API (mobile/automation)

`mald serve` exposes a capture endpoint for Shortcuts, Tasker, or curl:

```bash
curl -X POST http://127.0.0.1:3131/api/capture \
  -H "Content-Type: application/json" \
  -d '{"text": "Idea from my phone", "tag": "inbox"}'
```

## Knowledge Graph

```bash
mald links note-name       # Outgoing links
mald backlinks note-name   # Who links to this note?
mald orphans               # Notes with no incoming links
mald graph broken-links    # Dead wikilinks
mald graph view            # Mermaid diagram (paste into GitHub/Obsidian)
```

## Sync (git-based)

```bash
mald sync init     # Initialize git in ~/.mald/
mald sync          # Commit + pull + push
mald sync log      # Version history
mald sync undo     # Revert last change
```

## Import

```bash
mald import ~/ObsidianVault              # Preserves folder structure
mald import ~/notes --kb research        # Into specific KB
mald import ~/notes --flatten            # Flatten into root
```

Copies files (never moves). Imports assets from `attachments/`, `images/`, `assets/` directories. Indexes everything automatically.

## Architecture

```
~/.mald/
├── kb/           Plain markdown files (your data)
├── config/       JSON config (dot-notation access)
├── index/        HNSW vector index + SQLite FTS5
├── templates/    Note templates
├── plugins/      Custom executable plugins
├── trash/        Deleted files (recoverable)
└── logs/         Daemon logs
```

## Configuration

```bash
mald config set editor code            # VS Code
mald config set ai.default_model llama3.2
mald config set hooks.on_save "..."    # Run on every file save
mald doctor                            # Self-diagnostics
```

## Extended Help

```bash
mald help-topic ai         # AI setup, troubleshooting
mald help-topic search     # Search backends, date filtering
mald help-topic sync       # Git sync workflow
mald help-topic templates  # Template variables
mald help-topic daemon     # Background indexer
mald help-topic plugins    # Plugin system
mald help-topic graph      # Knowledge graph
mald help-topic tasks      # Task aggregation
```

## License

MIT
