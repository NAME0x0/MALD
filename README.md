<p align="center">
  <img src="assets/favicon.svg" width="96" alt="MALD">
</p>

<h1 align="center">MALD</h1>

<p align="center"><em>Terminal-first personal knowledge OS. Local files, local AI, sovereign defaults.</em></p>

<p align="center">
  <a href="https://github.com/NAME0x0/MALD/releases/latest"><img alt="Release" src="https://img.shields.io/github/v/release/NAME0x0/MALD?style=flat-square&color=8FAF8F"></a>
  <a href="https://crates.io/crates/mald"><img alt="Crates.io" src="https://img.shields.io/crates/v/mald.svg?style=flat-square&color=8FAF8F"></a>
  <a href="https://github.com/NAME0x0/MALD/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/NAME0x0/MALD/ci.yml?branch=main&style=flat-square"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-8FAF8F?style=flat-square"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-555?style=flat-square">
</p>

---

MALD is a local-first knowledge workspace that keeps every note as plain markdown on disk. It ships a desktop app, terminal UI, and CLI sharing one engine: HNSW vector search + SQLite FTS5, a file-watcher daemon, and optional Ollama-backed RAG with cited answers.

## Features

- **Local files** — plain `.md` on your disk, no proprietary formats, no cloud
- **Three surfaces** — desktop GUI (Iced), terminal UI (ratatui), CLI (~40 commands)
- **Multiple spaces** — separate notes for personal / work / research in one workspace
- **Fast search** — custom HNSW vector index + FTS5 keyword fallback
- **Cited AI** — Ollama-backed chat with inline `[1] [2]` citations to your own notes
- **Graph + backlinks** — wikilinks parsed live, force-directed graph, orphan detection
- **Privacy by default** — no telemetry, no external calls, no account required

## Install

**Windows** — Scoop:
```powershell
scoop bucket add mald https://github.com/NAME0x0/scoop-mald
scoop install mald
```

Or one-shot installer:
```powershell
powershell -c "irm https://raw.githubusercontent.com/NAME0x0/MALD/main/install.ps1 | iex"
```

**macOS / Linux**:
```bash
curl -fsSL https://raw.githubusercontent.com/NAME0x0/MALD/main/install.sh | sh
```

**From source** (requires Rust 1.78+):
```bash
cargo install mald
```

## Quickstart

```bash
mald init           # create ~/.mald workspace
mald new            # create your first note
mald                # launch the desktop app
```

For local AI:
```bash
ollama pull gemma4:e4b   # or qwen3.5:9b — see Settings panel for picks
mald ask "what did I write about HNSW?"
```

## Configuration

MALD stores everything inside `MALD_HOME` (default `~/.mald`). Override it for project-scoped workspaces:

```bash
export MALD_HOME="$PWD/.mald"
mald init
```

Config lives in `$MALD_HOME/config/config.json`. Edit via `mald config set <key> <value>` or the GUI Settings panel.

## Documentation

| Topic | File |
| --- | --- |
| User journey & flow diagram | [USER_JOURNEY.md](USER_JOURNEY.md) |
| Design system tokens | [DESIGN_SYSTEM.md](DESIGN_SYSTEM.md) |
| Frontend guidelines | [FRONTEND_GUIDELINES.md](FRONTEND_GUIDELINES.md) |
| Tech stack details | [TECH_STACK.md](TECH_STACK.md) |
| Product spec (PRD) | [PRD.md](PRD.md) |
| Architecture & internals | [CLAUDE.md](CLAUDE.md) |

For the full CLI reference: `mald --help` and `mald <subcommand> --help`.

## Stack

Rust · Iced 0.14 (GUI) · ratatui (TUI) · clap (CLI) · tokio · rusqlite (FTS5) · custom HNSW · pulldown-cmark · notify · reqwest · Ollama (optional)

## License

MIT — see [LICENSE](LICENSE).
