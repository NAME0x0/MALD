#!/bin/bash
# MALD Installation Script

set -e

echo "MALD - Markdown Archive Linux Distribution"
echo "Installing MALD CLI..."

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

# Set installation directory
INSTALL_DIR="/usr/local/bin"
if [[ ! -w "$INSTALL_DIR" ]]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Download or copy MALD
if [[ -f "bin/mald" ]]; then
    # Local installation
    echo "Installing from local source..."
    cp bin/mald "$INSTALL_DIR/mald"
    chmod +x "$INSTALL_DIR/mald"
else
    # Remote installation
    echo "Downloading MALD..."
    curl -sSL "https://raw.githubusercontent.com/NAME0x0/MALD/main/bin/mald" -o "$INSTALL_DIR/mald"
    chmod +x "$INSTALL_DIR/mald"
fi

# Check Python dependencies
echo "Checking Python dependencies..."
python3 -c "import sys; assert sys.version_info >= (3, 8)" || {
    echo "Error: Python 3.8+ required"
    exit 1
}

# Install Python package if in development
if [[ -f "pyproject.toml" ]]; then
    echo "Installing Python package..."
    pip3 install -e . --user
fi

# Add to PATH if needed
if [[ "$INSTALL_DIR" == "$HOME/.local/bin" ]]; then
    echo "Adding $INSTALL_DIR to PATH..."
    
    # Add to shell profile
    SHELL_PROFILE=""
    if [[ -f "$HOME/.bashrc" ]]; then
        SHELL_PROFILE="$HOME/.bashrc"
    elif [[ -f "$HOME/.zshrc" ]]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [[ -f "$HOME/.profile" ]]; then
        SHELL_PROFILE="$HOME/.profile"
    fi
    
    if [[ -n "$SHELL_PROFILE" ]]; then
        if ! grep -q "$INSTALL_DIR" "$SHELL_PROFILE"; then
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_PROFILE"
            echo "Added $INSTALL_DIR to PATH in $SHELL_PROFILE"
            echo "Run 'source $SHELL_PROFILE' or restart your shell"
        fi
    fi
fi

echo "MALD installed successfully!"
echo "Run 'mald --help' to get started"
echo "Initialize with 'mald init'"