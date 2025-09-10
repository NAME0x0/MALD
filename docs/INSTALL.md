# MALD Installation Guide

This guide covers installing MALD in various environments, from development setup to full ISO deployment.

## Quick Install (Development)

### Prerequisites
- Linux system (preferably Arch-based)
- Python 3.8+
- Git

### One-Line Install
```bash
curl -sSL https://raw.githubusercontent.com/NAME0x0/MALD/main/scripts/install.sh | bash
```

### Manual Install
```bash
# Clone repository
git clone https://github.com/NAME0x0/MALD.git
cd MALD

# Install CLI
./scripts/install.sh

# Initialize MALD environment
mald init
```

## Full System Install (ISO)

### Build ISO (Arch Linux Required)
```bash
# Install archiso tools
sudo pacman -S archiso

# Clone MALD
git clone https://github.com/NAME0x0/MALD.git
cd MALD

# Build ISO
mald iso build

# Flash to USB (replace /dev/sdX with your USB device)
sudo dd if=iso/output/mald-0.1.0-x86_64.iso of=/dev/sdX bs=4M status=progress
```

### Boot from ISO
1. Boot from USB/DVD
2. Follow installation prompts
3. Default credentials: `mald/mald`
4. Run `mald init` after first boot

## Development Setup

### Python Development
```bash
# Clone repository
git clone https://github.com/NAME0x0/MALD.git
cd MALD

# Create virtual environment
python -m venv venv
source venv/bin/activate

# Install in development mode
pip install -e .
pip install -e ".[dev]"

# Run tests
pytest

# Format code
black mald/
```

### Docker Development
```bash
# Build development container
docker build -t mald-dev -f docker/Dockerfile.dev .

# Run development environment
docker run -it --rm -v $(pwd):/workspace mald-dev
```

## WSL Installation

### Prerequisites
- Windows 10/11 with WSL2
- Ubuntu/Debian WSL distribution

### Install in WSL
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Python and dependencies
sudo apt install python3 python3-pip git curl tmux neovim -y

# Install MALD
curl -sSL https://raw.githubusercontent.com/NAME0x0/MALD/main/scripts/install.sh | bash

# Initialize
mald init
```

### WSL Integration
MALD provides special WSL integration features:
- Windows filesystem access
- Cross-platform knowledge bases
- AI model sharing between Windows and WSL

## Component Installation

### AI Components
```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Setup AI models
mald ai setup

# Test AI functionality
mald ai chat
```

### Optional Dependencies
```bash
# Enhanced search (ripgrep, fd)
sudo pacman -S ripgrep fd  # Arch
sudo apt install ripgrep fd-find  # Debian/Ubuntu

# PDF export
pip install weasyprint

# Advanced markdown processing
pip install markdown-extensions

# Vector database for RAG
pip install chromadb sentence-transformers
```

## Configuration

### Environment Variables
```bash
# MALD home directory (default: ~/.mald)
export MALD_HOME="$HOME/.mald"

# Default knowledge base
export MALD_DEFAULT_KB="default"

# AI model preferences
export MALD_AI_MODEL="llama3.2:1b"
```

### System Integration
```bash
# Add to shell profile
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc

# Enable tmux by default
echo 'alias mald-session="mald session --tmux"' >> ~/.bashrc

# Auto-completion (bash)
mald --completion bash >> ~/.bash_completion
```

## Verification

### Test Installation
```bash
# Check MALD CLI
mald --version

# Test initialization
mald init --force

# Create test knowledge base
mald kb create test-kb

# List knowledge bases
mald kb list

# Test session
mald session
```

### Performance Test
```bash
# Test AI functionality (if installed)
echo "Test query" | mald ai chat

# Test search
mald search "test" --kb test-kb

# Test backup
mald backup create test-backup
```

## Troubleshooting

### Common Issues

#### Python Version
```bash
# Check Python version
python3 --version  # Should be 3.8+

# Install specific Python version (Arch)
sudo pacman -S python39

# Install specific Python version (Ubuntu)
sudo apt install python3.9
```

#### Missing Dependencies
```bash
# Install build tools (Debian/Ubuntu)
sudo apt install build-essential python3-dev

# Install build tools (Arch)
sudo pacman -S base-devel
```

#### Permission Issues
```bash
# Fix PATH for local installation
export PATH="$HOME/.local/bin:$PATH"

# Fix file permissions
chmod +x ~/.local/bin/mald

# Fix MALD home permissions
chmod -R 755 ~/.mald
```

#### tmux Issues
```bash
# Install tmux
sudo pacman -S tmux  # Arch
sudo apt install tmux  # Debian/Ubuntu

# Fix tmux configuration
mald config set terminal.tmux_enabled true
```

### Getting Help
- Check logs: `~/.mald/logs/mald.log`
- Verbose mode: `mald --verbose <command>`
- Debug mode: `MALD_DEBUG=1 mald <command>`
- Report issues: https://github.com/NAME0x0/MALD/issues

## Uninstallation

### Remove MALD CLI
```bash
# Remove binary
rm ~/.local/bin/mald
# or
sudo rm /usr/local/bin/mald

# Remove Python package
pip uninstall mald
```

### Remove MALD Data
```bash
# Backup first (optional)
tar -czf mald-backup.tar.gz ~/.mald

# Remove MALD directory
rm -rf ~/.mald
```

### Clean System
```bash
# Remove from shell profile
sed -i '/mald/d' ~/.bashrc

# Remove completion
sed -i '/mald --completion/d' ~/.bash_completion
```

## Next Steps

After installation:
1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for system overview
2. Follow [PKM.md](PKM.md) for knowledge management workflows  
3. Check [AI.md](AI.md) for AI integration setup
4. See [CONFIG.md](CONFIG.md) for customization options