# Tech Stack

## Core
- **Language:** Rust (Stable)
- **Architecture:** Monolithic binary + Daemon mode (single `mald` executable)

## GUI
- **Framework:** `iced` 0.14 (Elm Architecture)
- **Animations:** `iced_anim` (animated widgets), velocity-aware panel transitions
- **Auxiliary widgets:** `iced_aw` (badges, cards, tabs)
- **Renderer:** wgpu (Vulkan/Metal/DX12)
- **Charts:** `plotters` + `plotters-iced`
- **Default font:** monospace (Phase 12 pivot)

## TUI
- **Framework:** `ratatui` 0.29 + `crossterm` (CLI fallback when GUI feature is disabled or `--tui` flag is set)

## Backend / Logic
- **CLI parsing:** `clap` derive + `clap_complete`
- **Async runtime:** `tokio` (multi-thread for the binary, current-thread for blocking helpers)
- **HTTP:** `reqwest` (streaming chat + Deep-mode web fetch)
- **Markdown:** `pulldown-cmark`
- **Syntax Highlighting:** `syntect` + `tree-sitter-highlight`
- **Vector Search:** Custom HNSW (M=16, ef_construction=200, cosine; mmap persistence)
- **Metadata store:** `rusqlite` with FTS5 (prepared-statement cache)
- **AI:** Ollama HTTP client (custom; streaming `/api/chat`, `/api/embeddings`); optional `llama_cpp_rs` behind `gguf` feature
- **File Watching:** `notify` (2s debounced)
- **Logging:** `tracing` + `tracing-subscriber`
- **Config:** Versioned JSON via custom `ConfigManager`

## Build Tooling
- **Manager:** Cargo with feature flags (`gui` default, `gguf` opt-in)
- **CI:** GitHub Actions matrix (linux/windows/macos × x64/arm64)
- **Audit:** `cargo-audit` (advisory-db gate; `cargo update -p <crate>` for transient CVEs)
- **Scripting:** PowerShell / Bash (install/uninstall scripts)
