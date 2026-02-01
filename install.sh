#!/bin/sh
# MALD installer — downloads the latest release from GitHub and installs to ~/.local/bin
set -e

REPO="NAME0x0/MALD"
INSTALL_DIR="${MALD_INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS_NAME="linux" ;;
        Darwin) OS_NAME="macos" ;;
        *)
            echo "Unsupported OS: $OS"
            echo "Download manually from https://github.com/$REPO/releases"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_NAME="x64" ;;
        aarch64|arm64)   ARCH_NAME="arm64" ;;
        *)
            echo "Unsupported architecture: $ARCH"
            echo "Download manually from https://github.com/$REPO/releases"
            exit 1
            ;;
    esac

    # arm64 only available for macOS currently
    if [ "$OS_NAME" = "linux" ] && [ "$ARCH_NAME" = "arm64" ]; then
        echo "Linux ARM64 binary not yet available."
        echo "Build from source: cargo install mald"
        exit 1
    fi

    BINARY="mald-${OS_NAME}-${ARCH_NAME}"
}

# Get latest release tag from GitHub API
get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')
    else
        echo "Error: curl or wget required"
        exit 1
    fi

    if [ -z "$VERSION" ]; then
        echo "Error: could not determine latest version"
        exit 1
    fi
}

download() {
    URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY"
    CHECKSUM_URL="https://github.com/$REPO/releases/download/$VERSION/SHA256SUMS"

    echo "Downloading mald $VERSION for $OS_NAME/$ARCH_NAME..."

    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$URL" -o "$TMPDIR/mald"
        curl -fsSL "$CHECKSUM_URL" -o "$TMPDIR/SHA256SUMS" 2>/dev/null || true
    else
        wget -q "$URL" -O "$TMPDIR/mald"
        wget -q "$CHECKSUM_URL" -O "$TMPDIR/SHA256SUMS" 2>/dev/null || true
    fi

    # Verify checksum if available
    if [ -f "$TMPDIR/SHA256SUMS" ] && command -v sha256sum >/dev/null 2>&1; then
        EXPECTED=$(grep "$BINARY" "$TMPDIR/SHA256SUMS" | awk '{print $1}')
        ACTUAL=$(sha256sum "$TMPDIR/mald" | awk '{print $1}')
        if [ -n "$EXPECTED" ] && [ "$EXPECTED" != "$ACTUAL" ]; then
            echo "Checksum verification FAILED"
            echo "  Expected: $EXPECTED"
            echo "  Got:      $ACTUAL"
            exit 1
        fi
        echo "Checksum verified."
    fi

    # Install
    mkdir -p "$INSTALL_DIR"
    mv "$TMPDIR/mald" "$INSTALL_DIR/mald"
    chmod +x "$INSTALL_DIR/mald"

    echo "Installed to $INSTALL_DIR/mald"

    # Check PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            echo ""
            echo "Add to your PATH:"
            echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
            echo ""
            echo "Add this line to ~/.bashrc or ~/.zshrc to make it permanent."
            ;;
    esac

    echo ""
    echo "Run 'mald' to get started."
}

detect_platform
get_latest_version
download
