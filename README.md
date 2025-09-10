# MALD
**M**arkdown **A**rchive **L**inux **D**istribution

A terminal-first, bootable Linux distro (Arch-based) combining Obsidian-style Markdown PKM, Jupyter-style executable panes, and NotebookLM-style local RAG. Privacy-first local AI (Ollama + llama.cpp), s6 init, LUKS encryption, btrfs snapshots, tmux+neovim editor UX, `mald` CLI, ISO → WSL workflow.

## Features

- **Terminal-First**: Optimized for keyboard-driven workflows
- **Markdown PKM**: Obsidian-style personal knowledge management
- **Executable Panes**: Jupyter-style interactive computing
- **Local RAG**: NotebookLM-style retrieval augmented generation
- **Privacy-First AI**: Local Ollama + llama.cpp integration
- **Secure by Default**: LUKS encryption, btrfs snapshots
- **Minimal Init**: s6 supervision suite
- **Editor UX**: tmux + neovim optimized configuration
- **Cross-Platform**: ISO → WSL workflow support

## Quick Start

```bash
# Install MALD CLI
curl -sSL https://raw.githubusercontent.com/NAME0x0/MALD/main/scripts/install.sh | bash

# Initialize MALD environment
mald init

# Create new knowledge base
mald kb create my-notes

# Start interactive session
mald session
```

## Components

- `mald/` - Core CLI tool and orchestration
- `iso/` - Arch-based bootable ISO build system  
- `config/` - System configurations (s6, tmux, neovim)
- `pkm/` - Personal Knowledge Management system
- `ai/` - Local AI integration (Ollama, llama.cpp)
- `security/` - LUKS and encryption utilities
- `snapshots/` - btrfs snapshot management
- `wsl/` - Windows Subsystem for Linux integration

## Architecture

MALD combines several open-source technologies into a cohesive, privacy-focused development environment:

- **Base**: Arch Linux minimal installation
- **Init**: s6 supervision suite for fast boot and service management
- **Storage**: btrfs with automatic snapshots
- **Security**: LUKS full-disk encryption
- **Terminal**: tmux session management
- **Editor**: Neovim with LSP and plugin ecosystem
- **Knowledge**: Markdown-based PKM with linking and search
- **AI**: Local LLM inference via Ollama and llama.cpp
- **Interactivity**: Jupyter-style executable code blocks

## Installation

See [INSTALL.md](docs/INSTALL.md) for detailed installation instructions.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design and components
- [Configuration](docs/CONFIG.md) - Customization and settings
- [PKM Guide](docs/PKM.md) - Personal Knowledge Management workflows
- [AI Integration](docs/AI.md) - Local AI setup and usage
- [Development](docs/DEVELOPMENT.md) - Contributing and development setup

## License

MIT License - see [LICENSE](LICENSE) for details.