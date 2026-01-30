#!/bin/bash
# MALD Shared Rootfs Builder
# Downloads Arch Linux bootstrap tarball and builds a rootfs on Ubuntu (no Arch host needed)

set -euo pipefail

MALD_VERSION="${MALD_VERSION:-0.1.0}"
BUILD_DIR="${MALD_OUTPUT_DIR:-$(pwd)/output}"
ROOTFS_DIR="${BUILD_DIR}/rootfs"
BOOTSTRAP_DIR="${BUILD_DIR}/bootstrap"
ISO_DIR="${MALD_ISO_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
PROJECT_ROOT="${ISO_DIR}/.."

ARCH_MIRROR="https://geo.mirror.pkgbuild.com"
BOOTSTRAP_URL="${ARCH_MIRROR}/iso/latest/archlinux-bootstrap-x86_64.tar.zst"

echo "MALD Rootfs Builder v${MALD_VERSION}"
echo "Build directory: ${BUILD_DIR}"

# Download and extract bootstrap tarball
download_bootstrap() {
    echo "Downloading Arch Linux bootstrap tarball..."
    mkdir -p "${BUILD_DIR}"

    if [[ ! -f "${BUILD_DIR}/archlinux-bootstrap-x86_64.tar.zst" ]]; then
        curl -L -o "${BUILD_DIR}/archlinux-bootstrap-x86_64.tar.zst" "${BOOTSTRAP_URL}"
    fi

    echo "Extracting bootstrap..."
    rm -rf "${BOOTSTRAP_DIR}"
    mkdir -p "${BOOTSTRAP_DIR}"
    tar --zstd -xf "${BUILD_DIR}/archlinux-bootstrap-x86_64.tar.zst" \
        -C "${BOOTSTRAP_DIR}" --strip-components=1
}

# Mount filesystems into bootstrap
mount_bootstrap() {
    echo "Mounting filesystems into bootstrap..."
    mount --bind /dev "${BOOTSTRAP_DIR}/dev"
    mount --bind /proc "${BOOTSTRAP_DIR}/proc"
    mount --bind /sys "${BOOTSTRAP_DIR}/sys"
    mount -t tmpfs tmpfs "${BOOTSTRAP_DIR}/tmp"

    # Resolve DNS inside bootstrap
    cp /etc/resolv.conf "${BOOTSTRAP_DIR}/etc/resolv.conf"
}

# Unmount bootstrap filesystems
umount_bootstrap() {
    echo "Unmounting bootstrap filesystems..."
    umount -l "${BOOTSTRAP_DIR}/tmp" 2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/sys" 2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/proc" 2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/dev" 2>/dev/null || true
}

# Configure pacman inside bootstrap
configure_pacman() {
    echo "Configuring pacman mirror..."
    echo "Server = ${ARCH_MIRROR}/\$repo/os/\$arch" \
        > "${BOOTSTRAP_DIR}/etc/pacman.d/mirrorlist"

    # Disable CheckSpace — statvfs() fails inside chroot/bind-mount environments
    # (causes "could not determine cachedir mount point" errors)
    sed -i 's/^CheckSpace/#CheckSpace/' "${BOOTSTRAP_DIR}/etc/pacman.conf"

    # Initialize keyring
    chroot "${BOOTSTRAP_DIR}" pacman-key --init
    chroot "${BOOTSTRAP_DIR}" pacman-key --populate archlinux
}

# Install packages into a fresh rootfs using the bootstrap's pacman
install_packages() {
    echo "Installing packages into rootfs..."
    rm -rf "${ROOTFS_DIR}"
    mkdir -p "${ROOTFS_DIR}/var/lib/pacman"

    # Bind-mount rootfs into bootstrap so pacman can access it at /rootfs
    mkdir -p "${BOOTSTRAP_DIR}/rootfs"
    mount --bind "${ROOTFS_DIR}" "${BOOTSTRAP_DIR}/rootfs"

    # Mount dev/proc/sys into rootfs inside bootstrap so pacman post-install
    # hooks (mkinitcpio, systemd-tmpfiles, etc.) can run properly.
    mkdir -p "${BOOTSTRAP_DIR}/rootfs/dev" \
             "${BOOTSTRAP_DIR}/rootfs/proc" \
             "${BOOTSTRAP_DIR}/rootfs/sys" \
             "${BOOTSTRAP_DIR}/rootfs/tmp"
    mount --bind /dev  "${BOOTSTRAP_DIR}/rootfs/dev"
    mount --bind /proc "${BOOTSTRAP_DIR}/rootfs/proc"
    mount --bind /sys  "${BOOTSTRAP_DIR}/rootfs/sys"
    mount -t tmpfs tmpfs "${BOOTSTRAP_DIR}/rootfs/tmp"

    # Core packages; kernel/firmware skipped when MALD_NO_KERNEL=1 (e.g. WSL, CI)
    local kernel_pkgs=""
    if [[ "${MALD_NO_KERNEL:-0}" != "1" ]]; then
        kernel_pkgs="linux linux-firmware"
    fi

    chroot "${BOOTSTRAP_DIR}" pacman -r /rootfs -Sy --noconfirm \
        base ${kernel_pkgs} \
        neovim tmux fzf ripgrep fd \
        btrfs-progs \
        python python-pip python-yaml \
        git curl wget openssh \
        base-devel nodejs npm

    # Unmount rootfs virtual filesystems (configure_rootfs will re-mount on ROOTFS_DIR)
    umount -l "${BOOTSTRAP_DIR}/rootfs/tmp"  2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/rootfs/sys"  2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/rootfs/proc" 2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/rootfs/dev"  2>/dev/null || true
}

# Configure the rootfs via chroot
configure_rootfs() {
    echo "Configuring rootfs..."

    # Copy resolv.conf
    cp /etc/resolv.conf "${ROOTFS_DIR}/etc/resolv.conf"

    # Mount essentials into rootfs for chroot
    mount --bind /dev "${ROOTFS_DIR}/dev"
    mount --bind /proc "${ROOTFS_DIR}/proc"
    mount --bind /sys "${ROOTFS_DIR}/sys"

    # Locale and timezone
    chroot "${ROOTFS_DIR}" bash -c '
        echo "en_US.UTF-8 UTF-8" > /etc/locale.gen
        locale-gen
        echo "LANG=en_US.UTF-8" > /etc/locale.conf
        ln -sf /usr/share/zoneinfo/UTC /etc/localtime
    '

    # Regenerate initramfs if missing (pacman post-install hooks may fail in -r mode)
    if [[ -f "${ROOTFS_DIR}/boot/vmlinuz-linux" ]] && \
       [[ ! -f "${ROOTFS_DIR}/boot/initramfs-linux.img" ]]; then
        echo "Regenerating initramfs..."
        chroot "${ROOTFS_DIR}" mkinitcpio -P
    fi

    # Create mald user
    chroot "${ROOTFS_DIR}" useradd -m -G wheel -s /bin/bash mald
    chroot "${ROOTFS_DIR}" bash -c 'echo "mald:mald" | chpasswd'
    chroot "${ROOTFS_DIR}" bash -c 'echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/wheel'

    # Install MALD
    mkdir -p "${ROOTFS_DIR}/opt/mald"
    cp -r "${PROJECT_ROOT}/mald" "${ROOTFS_DIR}/opt/mald/"
    cp "${PROJECT_ROOT}/pyproject.toml" "${ROOTFS_DIR}/opt/mald/"
    [[ -f "${PROJECT_ROOT}/setup.py" ]] && cp "${PROJECT_ROOT}/setup.py" "${ROOTFS_DIR}/opt/mald/"
    chroot "${ROOTFS_DIR}" pip install --break-system-packages /opt/mald/

    # Welcome script
    cat > "${ROOTFS_DIR}/usr/local/bin/mald-welcome" << 'WELCOME_EOF'
#!/bin/bash
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
WELCOME_EOF
    chmod +x "${ROOTFS_DIR}/usr/local/bin/mald-welcome"

    # Profile setup
    cat > "${ROOTFS_DIR}/etc/profile.d/mald.sh" << 'PROFILE_EOF'
# MALD environment setup
export MALD_SYSTEM=1
export PATH="/usr/local/bin:$PATH"

# Run welcome on first login
if [[ -z "$MALD_WELCOMED" && "$-" == *i* ]]; then
    export MALD_WELCOMED=1
    mald-welcome
fi
PROFILE_EOF

    # Unmount rootfs mounts
    umount -l "${ROOTFS_DIR}/sys" 2>/dev/null || true
    umount -l "${ROOTFS_DIR}/proc" 2>/dev/null || true
    umount -l "${ROOTFS_DIR}/dev" 2>/dev/null || true
}

# Cleanup bootstrap
cleanup_bootstrap() {
    umount -l "${BOOTSTRAP_DIR}/rootfs/tmp"  2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/rootfs/sys"  2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/rootfs/proc" 2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/rootfs/dev"  2>/dev/null || true
    umount -l "${BOOTSTRAP_DIR}/rootfs"      2>/dev/null || true
    umount_bootstrap
}

# Main
main() {
    echo "Starting MALD rootfs build..."

    trap cleanup_bootstrap EXIT

    download_bootstrap
    mount_bootstrap
    configure_pacman
    install_packages

    # Free disk: remove bootstrap package cache and downloaded tarball
    rm -rf "${BOOTSTRAP_DIR}/var/cache/pacman/pkg"/*
    rm -f "${BUILD_DIR}/archlinux-bootstrap-x86_64.tar.zst"

    configure_rootfs

    echo "Rootfs ready at: ${ROOTFS_DIR}"
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
