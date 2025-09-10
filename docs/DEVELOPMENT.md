# MALD Development Guide

This guide covers developing and contributing to MALD.

## Development Setup

### Prerequisites
- Python 3.8+
- Git
- Linux environment (preferably Arch-based for ISO building)

### Quick Setup
```bash
# Clone repository
git clone https://github.com/NAME0x0/MALD.git
cd MALD

# Setup development environment
make dev

# Activate virtual environment
source venv/bin/activate

# Test installation
make quick-test
```

### Manual Setup
```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install in development mode
pip install -e .
pip install -e ".[dev]"

# Install optional AI dependencies
pip install -e ".[ai]"
```

## Project Structure

```
MALD/
├── mald/                   # Core Python package
│   ├── __init__.py        # Package initialization
│   ├── cli.py             # Main CLI interface
│   ├── commands/          # CLI command modules
│   │   ├── init.py        # Initialize environment
│   │   ├── kb.py          # Knowledge base management
│   │   ├── session.py     # Session management
│   │   ├── iso.py         # ISO building
│   │   ├── ai.py          # AI integration
│   │   └── config.py      # Configuration management
│   └── utils/             # Utility modules
│       ├── config_manager.py     # Configuration handling
│       ├── filesystem.py         # File operations
│       └── markdown_parser.py    # Markdown processing
├── bin/                   # Executable scripts
│   └── mald              # Main executable
├── scripts/               # Installation and utility scripts
│   └── install.sh        # Installation script
├── iso/                   # ISO building system
│   └── build.sh          # ISO build script
├── config/               # System configurations
├── docs/                 # Documentation
├── tests/                # Test suite
└── pyproject.toml        # Project configuration
```

## Core Components

### CLI System (`mald/cli.py`)
The main command-line interface built with argparse:
- Subcommand routing
- Argument parsing
- Error handling
- Logging setup

### Command Modules (`mald/commands/`)
Each command is implemented as a separate module:
- `init.py`: Environment initialization
- `kb.py`: Knowledge base operations
- `session.py`: Session management with tmux integration
- `iso.py`: ISO building and management
- `ai.py`: AI model setup and interaction
- `config.py`: Configuration management

### Utilities (`mald/utils/`)
Core utility modules:
- `config_manager.py`: JSON-based configuration system
- `filesystem.py`: Safe file operations and backup handling
- `markdown_parser.py`: Markdown parsing with PKM features

## Development Workflow

### Adding New Commands

1. Create command module in `mald/commands/`:
```python
# mald/commands/new_command.py
import logging

logger = logging.getLogger(__name__)

def handle(args):
    """Handle the new command"""
    logger.info("Executing new command...")
    # Implementation here
    return 0
```

2. Add to CLI parser in `mald/cli.py`:
```python
# Import the module
from .commands import new_command

# Add subparser
new_parser = subparsers.add_parser('new', help='New command')
new_parser.add_argument('--option', help='Example option')

# Add routing
elif args.command == 'new':
    return new_command.handle(args)
```

3. Update `mald/commands/__init__.py`:
```python
from . import init, kb, session, iso, ai, config, new_command

__all__ = ['init', 'kb', 'session', 'iso', 'ai', 'config', 'new_command']
```

### Adding Utility Functions

1. Add to appropriate utility module or create new one
2. Import in `mald/utils/__init__.py`
3. Use in command modules

### Testing

#### Unit Tests
```bash
# Run all tests
make test

# Run specific test file
python -m pytest tests/test_cli.py -v

# Run with coverage
python -m pytest tests/ --cov=mald --cov-report=html
```

#### Manual Testing
```bash
# Test CLI functionality
make quick-test

# Test initialization
make init-test

# Test specific commands
python -m mald kb create test
python -m mald config get ai.default_model
```

### Code Quality

#### Formatting
```bash
# Format code
make format

# Or manually
python -m black mald/ tests/
```

#### Linting
```bash
# Run linting
make lint

# Or manually
python -m flake8 mald/ tests/
python -m mypy mald/
```

## Architecture Decisions

### Configuration System
- JSON-based for simplicity and human readability
- Hierarchical structure with dot notation access
- Default configuration with user overrides
- Stored in `~/.mald/config/config.json`

### Knowledge Base Storage
- Plain markdown files for portability
- Directory-based organization
- Wikilink syntax for internal references
- Git-friendly for version control

### CLI Design
- Subcommand-based interface
- Consistent argument patterns
- Verbose logging option
- Return codes for scripting

### Error Handling
- Graceful error messages for users
- Detailed logging for debugging
- Return codes indicate success/failure
- Preserve user data on errors

## Contributing

### Code Style
- Follow PEP 8 guidelines
- Use Black for formatting
- Type hints where helpful
- Comprehensive docstrings

### Documentation
- Update relevant documentation
- Include examples in docstrings
- Add to PKM guide for user features
- Architecture documentation for system changes

### Testing
- Unit tests for new functionality
- Integration tests for CLI commands
- Manual testing for user workflows
- Performance testing for large knowledge bases

### Submitting Changes
1. Fork the repository
2. Create feature branch
3. Make changes with tests
4. Run quality checks
5. Submit pull request

## Debugging

### Common Issues

#### Import Errors
```bash
# Check Python path
python -c "import sys; print(sys.path)"

# Install in development mode
pip install -e .
```

#### Configuration Issues
```bash
# Check configuration
python -m mald config get mald_home

# Reinitialize
python -m mald init --force
```

#### Permission Errors
```bash
# Fix permissions
chmod +x bin/mald
chmod -R 755 ~/.mald
```

### Logging and Debugging
```bash
# Verbose mode
python -m mald --verbose <command>

# Debug environment variable
MALD_DEBUG=1 python -m mald <command>

# Check logs
tail -f ~/.mald/logs/mald.log
```

## Advanced Development

### ISO Building
Requires Arch Linux environment:
```bash
# Install dependencies
sudo pacman -S archiso

# Build ISO
make iso

# Test in VM
qemu-system-x86_64 -m 2G -cdrom iso/output/mald-*.iso
```

### AI Integration Development
```bash
# Install AI dependencies
pip install -e ".[ai]"

# Test AI functionality
python -m mald ai setup
python -m mald ai chat
```

### Plugin Development
MALD supports extending functionality through plugins:

```python
# Example plugin structure
class MALDPlugin:
    def __init__(self, mald_instance):
        self.mald = mald_instance
    
    def command_handler(self, args):
        """Handle plugin-specific commands"""
        pass
    
    def hook_pre_save(self, document):
        """Pre-save hook for documents"""
        pass
```

## Performance Optimization

### Profiling
```bash
# Profile CLI startup
python -m cProfile -s cumtime -m mald --help

# Profile specific operations
python -m cProfile -s cumtime -m mald kb list
```

### Memory Usage
```bash
# Monitor memory usage
/usr/bin/time -v python -m mald <command>
```

### Large Knowledge Bases
- Use knowledge base per project/domain
- Regular cleanup of orphaned files
- Efficient indexing strategies
- Lazy loading of documents

## Release Process

### Version Management
- Follow semantic versioning (MAJOR.MINOR.PATCH)
- Update version in `mald/__init__.py`
- Update version in `pyproject.toml`
- Tag releases in git

### Building Releases
```bash
# Clean build
make clean

# Build package
make build

# Upload to PyPI (maintainers only)
python -m twine upload dist/*
```

### Documentation Updates
- Update CHANGELOG.md
- Update documentation for new features
- Review and update installation instructions
- Update architecture documentation for significant changes