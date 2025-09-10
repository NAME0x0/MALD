#!/bin/bash
# MALD ISO Build Script
# Builds a bootable Arch Linux-based ISO with MALD components

set -e

# Configuration
MALD_VERSION="0.1.0"
ARCH_ISO_URL="https://archlinux.org/iso/latest/"
BUILD_DIR="${MALD_OUTPUT_DIR:-$(pwd)/output}"
WORK_DIR="${BUILD_DIR}/work"
ISO_DIR="${MALD_ISO_DIR:-$(pwd)}"

echo "MALD ISO Builder v${MALD_VERSION}"
echo "Build directory: ${BUILD_DIR}"
echo "Work directory: ${WORK_DIR}"

# Create build directories
mkdir -p "${BUILD_DIR}" "${WORK_DIR}"

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    for cmd in archiso mkarchiso pacstrap; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo "Missing dependencies: ${missing_deps[*]}"
        echo "On Arch Linux: sudo pacman -S archiso"
        echo "This script requires Arch Linux or an Arch-based system"
        exit 1
    fi
}

# Download base Arch ISO (for reference)
download_arch_iso() {
    echo "Checking for Arch Linux ISO..."
    
    if [[ ! -f "${BUILD_DIR}/archlinux.iso" ]]; then
        echo "Downloading Arch Linux ISO for reference..."
        # This is a placeholder - in practice, we'd build from archiso profiles
        echo "Using archiso profiles instead of downloading full ISO"
    fi
}

# Setup archiso profile
setup_archiso_profile() {
    echo "Setting up archiso profile..."
    
    local profile_dir="${WORK_DIR}/mald-profile"
    
    # Copy base archiso profile
    if [[ -d "/usr/share/archiso/configs/releng" ]]; then
        cp -r /usr/share/archiso/configs/releng "${profile_dir}"
    else
        echo "Error: archiso profiles not found"
        echo "Install archiso: sudo pacman -S archiso"
        exit 1
    fi
    
    # Customize for MALD
    customize_profile "${profile_dir}"
}

# Customize archiso profile for MALD
customize_profile() {
    local profile_dir="$1"
    
    echo "Customizing profile for MALD..."
    
    # Add MALD packages to packages.x86_64
    cat >> "${profile_dir}/packages.x86_64" << 'EOF'

# MALD Core Components
s6
s6-rc
s6-linux-init

# Editor and Terminal
neovim
tmux
fzf
ripgrep
fd

# Filesystem
btrfs-progs
snapper

# Security
cryptsetup

# Python and AI tools
python
python-pip
git

# Network tools
curl
wget
openssh

# Development
base-devel
nodejs
npm

EOF

    # Create MALD-specific airootfs customizations
    local airootfs="${profile_dir}/airootfs"
    
    # Install MALD CLI
    mkdir -p "${airootfs}/usr/local/bin"
    cp "${ISO_DIR}/../bin/mald" "${airootfs}/usr/local/bin/"
    chmod +x "${airootfs}/usr/local/bin/mald"
    
    # Install MALD Python package
    mkdir -p "${airootfs}/opt/mald"
    cp -r "${ISO_DIR}/../mald" "${airootfs}/opt/mald/"
    
    # Create s6 service definitions
    setup_s6_services "${airootfs}"
    
    # Setup default configurations
    setup_default_configs "${airootfs}"
    
    # Create MALD-specific scripts
    create_mald_scripts "${airootfs}"
}

# Setup s6 init services
setup_s6_services() {
    local airootfs="$1"
    local s6_dir="${airootfs}/etc/s6-linux-init"
    
    mkdir -p "${s6_dir}/skel/rc.d"
    
    # Basic s6 setup script
    cat > "${s6_dir}/skel/rc.d/mald-init" << 'EOF'
#!/bin/bash
# MALD initialization script

# Initialize MALD environment for root
mald init --force

# Setup default knowledge base
if [[ ! -d "/root/.mald/kb/system" ]]; then
    mald kb create system
fi

echo "MALD system initialized"
EOF
    
    chmod +x "${s6_dir}/skel/rc.d/mald-init"
}

# Setup default configurations
setup_default_configs() {
    local airootfs="$1"
    
    # Copy MALD configurations
    if [[ -d "${ISO_DIR}/../config" ]]; then
        mkdir -p "${airootfs}/etc/mald"
        cp -r "${ISO_DIR}/../config"/* "${airootfs}/etc/mald/"
    fi
    
    # Default tmux configuration
    mkdir -p "${airootfs}/etc/skel"
    cat > "${airootfs}/etc/skel/.tmux.conf" << 'EOF'
# Load MALD tmux configuration
if-shell "test -f /etc/mald/tmux.conf" "source-file /etc/mald/tmux.conf"
EOF

    # Default neovim configuration
    mkdir -p "${airootfs}/etc/skel/.config/nvim"
    cat > "${airootfs}/etc/skel/.config/nvim/init.lua" << 'EOF'
-- Load MALD neovim configuration
local mald_config = "/etc/mald/nvim/init.lua"
if vim.fn.filereadable(mald_config) == 1 then
    dofile(mald_config)
end
EOF
}

# Create MALD-specific scripts
create_mald_scripts() {
    local airootfs="$1"
    
    # MALD welcome script
    cat > "${airootfs}/usr/local/bin/mald-welcome" << 'EOF'
#!/bin/bash
# MALD Welcome Script

clear
echo "=========================================="
echo "  Welcome to MALD"
echo "  Markdown Archive Linux Distribution"
echo "=========================================="
echo
echo "Quick Start:"
echo "  mald init        - Initialize MALD"
echo "  mald session     - Start session"
echo "  mald kb create   - Create knowledge base"
echo "  mald ai setup    - Setup AI models"
echo
echo "Documentation: /usr/share/doc/mald/"
echo "Configuration: ~/.mald/"
echo
echo "Press Enter to continue..."
read
EOF
    
    chmod +x "${airootfs}/usr/local/bin/mald-welcome"
    
    # Auto-start script for terminal
    cat > "${airootfs}/etc/profile.d/mald.sh" << 'EOF'
# MALD environment setup
export MALD_SYSTEM=1
export PATH="/usr/local/bin:$PATH"

# Run welcome on first login
if [[ -z "$MALD_WELCOMED" && "$-" == *i* ]]; then
    export MALD_WELCOMED=1
    mald-welcome
fi
EOF
}

# Build the ISO
build_iso() {
    echo "Building MALD ISO..."
    
    local profile_dir="${WORK_DIR}/mald-profile"
    local output_name="mald-${MALD_VERSION}-x86_64.iso"
    
    cd "${WORK_DIR}"
    
    # Build with mkarchiso
    sudo mkarchiso -v -w "${WORK_DIR}/tmp" -o "${BUILD_DIR}" "${profile_dir}"
    
    # Rename output
    if [[ -f "${BUILD_DIR}/archlinux-"*.iso ]]; then
        mv "${BUILD_DIR}/archlinux-"*.iso "${BUILD_DIR}/${output_name}"
        echo "MALD ISO created: ${BUILD_DIR}/${output_name}"
    else
        echo "Error: ISO build failed"
        exit 1
    fi
}

# Main build process
main() {
    echo "Starting MALD ISO build process..."
    
    check_dependencies
    download_arch_iso
    setup_archiso_profile
    build_iso
    
    echo "MALD ISO build completed successfully!"
    echo "Output: ${BUILD_DIR}/mald-${MALD_VERSION}-x86_64.iso"
}

# Check if running as script
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi