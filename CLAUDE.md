# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MALD (Markdown Archive Linux Distribution) is a Python 3.8+ terminal-first development environment combining PKM (Personal Knowledge Management), local AI integration (Ollama/llama.cpp), and a bootable Arch Linux ISO. Privacy-first — no external API calls.

## Common Commands

```bash
make dev          # Create venv and install dev dependencies
make test         # Run pytest with coverage (pytest --cov=mald --cov-report=term-missing)
make format       # Run Black formatter
make lint         # Run flake8 + mypy
make build        # Build Python package
make iso          # Build Arch Linux ISO (requires archiso on Arch)
make quick-test   # Verify CLI works (python -m mald --help)

# Run a single test
pytest tests/test_markdown.py::TestMarkdownParser::test_extract_wikilinks

# Install in development mode
pip install -e ".[dev,ai]"
```

## Architecture

**Layered design**: CLI → Commands → Utils → System

- **`mald/cli.py`** — argparse-based subcommand router, entry point via `mald.cli:main`
- **`mald/commands/`** — one module per command: `init`, `kb`, `session`, `ai`, `iso`, `config`
- **`mald/utils/`** — shared utilities: `config_manager` (JSON with dot-notation access), `markdown_parser` (wikilinks, tags, code blocks, graph analysis), `filesystem` (safe file ops, secure delete)
- **`iso/build.sh`** — archiso-based build script using s6 init, btrfs, optional LUKS

**Data model**: Knowledge bases are directories of plain Markdown files stored under `~/.mald/kb/`. The `MarkdownDocument` class parses wikilinks (`[[target]]`), hashtags, YAML frontmatter, and code blocks. Graph functions (`generate_graph_data`, `find_orphaned_files`) operate over parsed KB collections.

**Config**: JSON at `~/.mald/config/config.json`, accessed via `ConfigManager` with dot-notation keys (e.g., `ai.default_model`).

**Sessions**: tmux-based with 3 windows (editor/terminal/AI), environment variables `MALD_HOME`, `MALD_SESSION`, `MALD_KB`.

## Code Quality

- **Formatter**: Black (88-char line length)
- **Linter**: flake8
- **Type checker**: mypy (strict mode, `disallow_untyped_defs = true`)
- **Tests**: pytest with fixtures in `tests/conftest.py` (`temp_mald_home`, `sample_kb_path`)

## Dependencies

Core: `pyyaml`, `click`. Optional AI extras: `ollama`, `llama-cpp-python`, `chromadb`, `sentence-transformers`. All configured in `pyproject.toml`.
