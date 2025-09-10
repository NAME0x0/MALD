# Changelog

All notable changes to MALD will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-01

### Added
- Initial implementation of MALD (Markdown Archive Linux Distribution)
- Core CLI tool (`mald`) with subcommands:
  - `init` - Initialize MALD environment
  - `kb` - Knowledge base management (create, list, open)
  - `session` - Interactive session management with tmux integration
  - `iso` - ISO building system for Arch-based bootable distribution
  - `ai` - AI model setup and management (Ollama integration)
  - `config` - Configuration management with hierarchical settings
- Personal Knowledge Management (PKM) system:
  - Obsidian-style wikilinks (`[[document]]`)
  - Hashtag tagging (`#tag`)
  - Markdown document parsing and analysis
  - Link graph generation and orphan detection
  - Full-text search across knowledge bases
- System configurations:
  - tmux configuration for terminal multiplexing
  - Neovim configuration for markdown editing
  - s6 init system setup (planned)
  - btrfs snapshots management (planned)
  - LUKS encryption setup (planned)
- AI Integration foundation:
  - Local AI model setup with Ollama
  - RAG (Retrieval Augmented Generation) preparation
  - Privacy-first local inference
- Build system:
  - Arch Linux-based ISO building
  - Python package with setuptools
  - Installation scripts for multiple platforms
- Documentation:
  - Comprehensive README with feature overview
  - Architecture documentation
  - Installation guide for multiple environments
  - PKM workflow guide
  - Development guide for contributors
- Testing infrastructure:
  - Unit tests for core functionality
  - Configuration testing
  - Markdown parsing tests
  - CLI command testing

### Technical Details
- Python 3.8+ compatibility
- JSON-based configuration system
- Modular command architecture
- Utility modules for filesystem, config, and markdown operations
- Cross-platform support (Linux primary, WSL secondary)
- Development tools (Makefile, formatting, linting)

### Planned Features
- Jupyter-style executable code blocks in markdown
- Enhanced AI integration with vector databases
- Complete ISO building with all MALD components
- WSL integration workflows
- Advanced PKM features (backlinking, graph visualization)
- Security enhancements (LUKS, secure deletion)
- Snapshot and backup automation

[0.1.0]: https://github.com/NAME0x0/MALD/releases/tag/v0.1.0