# MALD Architecture

MALD (Markdown Archive Linux Distribution) is designed as a cohesive system that integrates multiple technologies into a unified knowledge management and development environment.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MALD User Interface                      │
├─────────────────────────────────────────────────────────────┤
│  CLI (mald)  │  tmux Sessions  │  Neovim Editor             │
├─────────────────────────────────────────────────────────────┤
│                 Knowledge Management Layer                   │
├─────────────────────────────────────────────────────────────┤
│  PKM Engine  │  Markdown Parser │  Link Graph  │  Search    │
├─────────────────────────────────────────────────────────────┤
│                    AI/RAG Layer                             │
├─────────────────────────────────────────────────────────────┤
│   Ollama     │   llama.cpp     │  Embeddings  │  Vector DB  │
├─────────────────────────────────────────────────────────────┤
│                   System Layer                              │
├─────────────────────────────────────────────────────────────┤
│  s6 Init     │   btrfs        │    LUKS      │   Snapshots  │
├─────────────────────────────────────────────────────────────┤
│                   Arch Linux Base                           │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. MALD CLI (`mald`)
- **Purpose**: Central orchestration tool for all MALD functionality
- **Language**: Python 3.8+
- **Key Features**:
  - Environment initialization
  - Knowledge base management
  - Session orchestration
  - AI model management
  - ISO building
  - Configuration management

### 2. Knowledge Management (PKM)
- **Markdown-First**: All knowledge stored as markdown files
- **Linking System**: Obsidian-style `[[wikilinks]]` for document relationships
- **Tagging**: Hashtag-based categorization (`#tag`)
- **Search**: Full-text search across knowledge bases
- **Graph**: Visual representation of document relationships

### 3. Editor Integration
- **Neovim**: Primary editor with LSP support
- **tmux**: Session management and window multiplexing
- **Executable Panes**: Jupyter-style code execution within markdown
- **Live Preview**: Real-time markdown rendering

### 4. Local AI (Privacy-First)
- **Ollama**: Local LLM serving and management
- **llama.cpp**: Efficient CPU inference
- **RAG System**: Retrieval Augmented Generation from knowledge bases
- **Embeddings**: Vector search for semantic similarity
- **No External Calls**: All AI processing happens locally

### 5. System Foundation
- **Arch Linux**: Rolling release, minimal base
- **s6 Init**: Fast, reliable process supervision
- **btrfs**: Copy-on-write filesystem with snapshots
- **LUKS**: Full-disk encryption
- **Minimal**: Only essential packages included

## Data Flow

### Knowledge Creation
```
User Input → Neovim → Markdown File → PKM Engine → Index Update
                                                  ↓
                                            AI Embedding → Vector Store
```

### Knowledge Retrieval
```
Query → Search Engine → Results + Vector Search → Ranked Results
                     ↓
              RAG System → LLM Context → AI Response
```

### System Operations
```
mald CLI → System Commands → s6 Services → Hardware/Filesystem
                          ↓
                    btrfs Snapshots → Backup/Recovery
```

## File System Layout

```
/
├── etc/
│   ├── mald/              # System-wide MALD configuration
│   ├── s6-linux-init/     # s6 init configuration
│   └── tmux/              # tmux system configuration
├── usr/
│   ├── local/bin/mald     # MALD CLI executable
│   └── share/mald/        # MALD system files
└── home/user/
    └── .mald/             # User MALD environment
        ├── kb/            # Knowledge bases
        ├── config/        # User configuration
        ├── ai/            # AI models and indices
        ├── sessions/      # Session data
        └── cache/         # Temporary files
```

## Security Model

### Encryption
- **LUKS**: Full-disk encryption at rest
- **Memory**: Secure memory handling for AI models
- **Network**: No external network calls for AI (privacy-first)

### Isolation
- **User Spaces**: Each user has isolated MALD environment
- **Process**: s6 supervision for service isolation
- **Filesystem**: btrfs subvolumes for isolation

### Backup & Recovery
- **Snapshots**: Automatic btrfs snapshots before major operations
- **Versioning**: Git-style versioning for knowledge bases
- **Export**: Portable knowledge base export/import

## Performance Considerations

### AI Performance
- **Model Size**: Preference for smaller, efficient models (1B-7B parameters)
- **CPU Optimization**: llama.cpp optimizations for CPU inference
- **Memory Management**: Efficient model loading/unloading
- **Caching**: Aggressive caching of embeddings and results

### Filesystem Performance
- **btrfs**: Optimized for SSD with compression
- **Snapshots**: Incremental, space-efficient snapshots
- **Indexing**: Fast file indexing with inotify
- **Compression**: Transparent compression for knowledge bases

### Boot Performance
- **s6**: Parallel service startup
- **Minimal**: Reduced service set for fast boot
- **No Systemd**: Lighter init system
- **Preload**: Eager loading of frequently used components

## Extensibility

### Plugin System
- **Python Plugins**: Extend CLI functionality
- **Neovim Plugins**: Editor enhancements
- **AI Models**: Support for multiple model formats
- **Export Formats**: Multiple output formats (PDF, HTML, etc.)

### Integration Points
- **APIs**: Local REST API for external integration
- **Hooks**: Pre/post hooks for operations
- **Scripting**: Shell script integration
- **WSL**: Windows Subsystem for Linux support

## Development Workflow

### Local Development
```bash
# Setup development environment
git clone https://github.com/NAME0x0/MALD
cd MALD
python -m pip install -e .

# Initialize MALD
mald init

# Start development session
mald session --tmux
```

### ISO Building
```bash
# Build bootable ISO (requires Arch Linux)
mald iso build

# Test in VM
qemu-system-x86_64 -m 2G -cdrom mald-0.1.0-x86_64.iso
```

### Contributing
- **Code**: Python for CLI, Lua for Neovim configuration
- **Documentation**: Markdown in knowledge base format
- **Testing**: pytest for Python components
- **Standards**: Black formatting, type hints, comprehensive docs