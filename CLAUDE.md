# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MALD (Markdown Archive & Localized Daemon) is a Rust-based terminal-first PKM (Personal Knowledge Management) tool with local AI integration (Ollama + optional GGUF). Daemon-based architecture with custom HNSW vector index. Privacy-first — no external API calls.

## Common Commands

```bash
cargo build              # Build debug binary
cargo build --release    # Build release binary
cargo test               # Run all tests (137 tests: 54 unit, 11+18 integration)
cargo fmt                # Format code
cargo clippy             # Lint
cargo run -- init        # Initialize ~/.mald/ structure
cargo run -- kb list     # List knowledge bases
cargo run -- doctor      # Self-diagnostics
cargo run -- --help      # Show CLI help

# Run a single test
cargo test test_extract_wikilinks

# Run integration tests only
cargo test --test cli_tests
cargo test --test feature_tests

# Build with GGUF support
cargo build --features gguf
```

## Architecture

**Layered design**: CLI → Commands → Daemon/AI/Index → Parser/Config/FS

```
src/
├── main.rs, lib.rs, cli.rs     — entry point and clap-derive CLI (~40 commands)
├── commands/                    — 30+ modules: init, kb, ai, search, edit, rename,
│                                  export, run, preview, capture, doctor, sync, tags,
│                                  tasks, review, import, info, open, serve, bench,
│                                  plugins, hooks, templates, dashboard, help_topics,
│                                  reindex, graph, session, setup, new, stamp, tui
├── daemon/                      — IPC server, file watcher, background indexer
├── ai/                          — Ollama HTTP client (streaming), GGUF backend,
│                                  embeddings, cited RAG chat, chat history
├── index/                       — HNSW vector index, mmap persistence,
│                                  SQLite FTS5 metadata, document chunker
├── parser/                      — Markdown parser, frontmatter, wikilinks, tags, graph
├── config/                      — JSON config with dot-notation, versioned migration
└── fs/                          — Safe file operations, trash instead of delete
```

**Data model**: Knowledge bases are directories of plain Markdown files under `~/.mald/kb/`. `MarkdownDocument` parses wikilinks (`[[target]]`), hashtags, YAML frontmatter, and code blocks via `pulldown-cmark` + regex. Graph functions operate over parsed KB collections.

**Config**: JSON at `~/.mald/config/config.json`, accessed via `ConfigManager` with dot-notation keys (e.g., `ai.default_model`). Versioned with auto-migration.

**Vector Index**: Custom HNSW implementation (M=16, ef_construction=200, cosine similarity) with binary mmap persistence and SQLite metadata for chunk tracking.

**Daemon**: Background process with file watcher (notify crate, 2s debounce) that auto-indexes changed markdown files. IPC via TCP (Windows) or Unix socket (Linux). Logs to `~/.mald/logs/daemon.log`.

**Sessions**: tmux-based with 3 windows (editor/terminal/AI), shell fallback on Windows.

## Code Quality

- **Formatter**: `cargo fmt` (rustfmt)
- **Linter**: `cargo clippy`
- **Tests**: Unit tests inline (`#[cfg(test)]`), integration tests in `tests/cli_tests.rs` and `tests/feature_tests.rs`
- Dead code warnings are expected for library functions not yet called from the binary entry point

## Key Dependencies

Core: `clap` (CLI), `clap_complete` (shell completions), `tokio` (async), `reqwest` (HTTP + streaming), `serde`/`serde_json`/`serde_yaml`, `rusqlite` (FTS5 metadata), `notify` (file watching), `memmap2` (mmap), `pulldown-cmark` (markdown), `regex`, `tracing`/`tracing-subscriber`, `chrono`, `ratatui`/`crossterm` (TUI), `indicatif` (progress bars), `futures-util` (streaming). Optional: `llama_cpp_rs` behind `gguf` feature flag.
