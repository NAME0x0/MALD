# MALD

**M**arkdown **A**rchive & **L**ocalized **D**aemon

Your notes, your AI, your machine. Nothing leaves localhost.

Terminal-first PKM with local AI (Ollama), semantic search (custom HNSW), cited RAG chat, and a background daemon that keeps everything indexed.

## Install

```bash
cargo install --path .
```

Or with local GGUF model support:

```bash
cargo install --path . --features gguf
```

## 60-Second Quickstart

```bash
mald init                          # Create ~/.mald/ structure
mald kb create work                # New knowledge base
mald new "Project Plan" --kb work  # Create a note
mald q "quick thought"             # Capture to today's daily note
mald search "project"              # Full-text search (all KBs)
mald ai chat --kb work             # Interactive RAG chat with citations
mald                               # Dashboard: recent notes, open tasks
```

## What It Does

| Feature | Command |
|---|---|
| **Dashboard** (zero-arg) | `mald` |
| Create/manage knowledge bases | `mald kb create/list/open` |
| Create notes from templates | `mald new "Title" --template meeting` |
| Quick capture to daily note | `mald q "thought" --tag idea` |
| Full-text search (all KBs) | `mald search "query"` |
| Date-filtered search | `mald search "query" --since 7d` |
| Interactive search TUI | `mald search` (no args) |
| Fuzzy-find and edit notes | `mald edit "partial name"` |
| Rename with backlink update | `mald rename old-name "New Title"` |
| AI chat with cited RAG | `mald ai chat "question"` |
| AI summarize/quiz/briefing | `mald ai summarize/quiz/briefing` |
| Compare two notes with AI | `mald ai compare note1 note2` |
| Timeline extraction | `mald ai timeline --kb work` |
| Execute code blocks in notes | `mald run note.md` |
| Save execution output back | `mald run note.md --save` |
| Render markdown in terminal | `mald preview note.md` |
| Export to HTML | `mald export note.md` |
| Export entire KB | `mald export --all --format html` |
| Serve KB as local website | `mald serve` (http://127.0.0.1:3131) |
| Note metadata & stats | `mald info note.md` |
| Aggregate open tasks | `mald tasks` / `mald tasks --all` |
| Weekly review | `mald review` |
| Tag browsing | `mald tags` / `mald tags rust` |
| Git-based sync | `mald sync` / `mald sync log` |
| Import from Obsidian/folders | `mald import /path/to/vault` |
| Graph: broken links | `mald graph broken-links` |
| Graph: Mermaid diagram | `mald graph view` |
| Graph: statistics | `mald graph stats` |
| Template management | `mald template create/list/edit/delete` |
| Plugin system | `mald plugin list` / `mald x-<name>` |
| Benchmark HNSW index | `mald bench` |
| Shell completions | `mald completions bash/zsh/fish` |
| Self-diagnostics | `mald doctor` |
| Background daemon | `mald daemon start/stop/status` |
| Configuration | `mald config get/set key value` |

## Dashboard

Running `mald` with no arguments shows a dashboard with:
- Knowledge base stats (note counts per KB)
- Recently modified notes (last 7 days)
- Open tasks aggregated from all notes
- Quick action suggestions

## AI Features (Ollama)

MALD uses Ollama for all AI — nothing leaves your machine. Install [Ollama](https://ollama.com), pull a model, and go:

```bash
ollama pull llama3.2
ollama pull nomic-embed-text    # For semantic search
mald config set ai.default_model llama3.2
```

**Cited RAG**: Every AI response includes source citations with file paths and line numbers. Chat sessions persist across invocations.

**Interactive REPL**: `mald ai chat` with no message enters conversational mode with `/quit`, `/new`, `/history` commands.

**NotebookLM-style features**: `summarize`, `quiz`, `briefing`, `compare`, `timeline`, `explain` — all grounded in your notes.

## Review & Tasks

```bash
mald review              # Weekly review: recent, stale, orphans, broken links, tasks
mald review --days 30    # Custom time range
mald tasks               # Open tasks from current KB
mald tasks --all         # Open tasks from all KBs
```

## Rename with Backlink Updates

```bash
mald rename old-name "New Title"    # Renames file + updates all [[old-name]] references
```

This is the #1 missing feature in most PKM tools. MALD finds and updates every wikilink across the entire KB.

## Local Web Server

```bash
mald serve                  # http://127.0.0.1:3131
mald serve --port 8080      # Custom port
mald serve --kb work        # Serve specific KB
```

Read-only. Renders markdown to HTML with wikilinks converted to clickable links. Access from any device on your local network.

## Plugins

Any executable in `~/.mald/plugins/` becomes a command:

```bash
# Create a plugin
echo '#!/bin/bash\necho "Hello from $MALD_HOME"' > ~/.mald/plugins/hello
chmod +x ~/.mald/plugins/hello

# Run it
mald x-hello
mald plugin list
```

Plugins receive `MALD_HOME`, `MALD_NOTE`, and `MALD_EVENT` environment variables.

## Architecture

```
~/.mald/
├── kb/           # Knowledge bases (plain markdown)
├── config/       # JSON config (dot-notation access)
├── index/        # HNSW vector index + SQLite FTS5
├── sessions/     # tmux configs + chat history
├── templates/    # Note templates (user-editable)
├── plugins/      # Custom executable plugins
├── trash/        # Deleted files (recoverable)
└── cache/        # AI model cache
```

```
src/
├── cli.rs           — clap-derive CLI with help examples on every command
├── commands/        — 30+ command modules
├── daemon/          — IPC server, file watcher, background indexer
├── ai/              — Ollama client (streaming), GGUF backend, cited RAG
├── index/           — Custom HNSW (M=16, cosine), mmap, SQLite FTS5
├── parser/          — Markdown, frontmatter, wikilinks, tags, graph
├── config/          — JSON config with parse-error fallback
└── fs/              — Safe file operations, trash instead of delete
```

## Benchmarks

```bash
mald bench                          # Default: 384-dim, 1000 vectors
mald bench --dim 768 --count 5000   # Custom parameters
```

Reports insert throughput, query latency, and persistence speed.

## Configuration

```bash
mald config set ai.default_model llama3.2
mald config set ai.embedding_model nomic-embed-text
mald config set editor "nvim"
mald config set hooks.on_create "echo 'New note: $MALD_NOTE'"
mald config set hooks.on_save "git -C $MALD_HOME add -A && git commit -m auto"
```

## Cross-Platform

- **Linux/macOS**: tmux sessions with editor/terminal/AI panes
- **Windows**: PowerShell sessions, named pipe IPC

## License

MIT — see [LICENSE](LICENSE).
