#!/bin/bash
# MALD ISO Assembler
# Takes the rootfs from build-rootfs.sh and creates a bootable ISO

set -euo pipefail

MALD_VERSION="${MALD_VERSION:-0.1.0}"
BUILD_DIR="${MALD_OUTPUT_DIR:-$(pwd)/output}"
ROOTFS_DIR="${BUILD_DIR}/rootfs"
ISO_STAGING="${BUILD_DIR}/iso-staging"
ISO_DIR="${MALD_ISO_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
OUTPUT_NAME="mald-${MALD_VERSION}-x86_64.iso"

echo "MALD ISO Assembler v${MALD_VERSION}"

# Check dependencies
for cmd in mksquashfs grub-mkrescue xorriso; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "Error: $cmd not found. Install: sudo apt install squashfs-tools grub-pc-bin grub-common xorriso mtools"
        exit 1
    fi
done

if [[ ! -d "${ROOTFS_DIR}" ]]; then
    echo "Error: rootfs not found at ${ROOTFS_DIR}"
    echo "Run build-rootfs.sh first"
    exit 1
fi

# Setup ISO staging area
echo "Setting up ISO staging area..."
rm -rf "${ISO_STAGING}"
mkdir -p "${ISO_STAGING}/boot/grub" "${ISO_STAGING}/live"

# Create squashfs
echo "Creating squashfs..."
mksquashfs "${ROOTFS_DIR}" "${ISO_STAGING}/live/filesystem.squashfs" \
    -comp xz -Xbcj x86 -b 1M -noappend

# Copy kernel and initramfs
echo "Copying kernel and initramfs..."
cp "${ROOTFS_DIR}/boot/vmlinuz-linux" "${ISO_STAGING}/boot/vmlinuz-linux"
cp "${ROOTFS_DIR}/boot/initramfs-linux.img" "${ISO_STAGING}/boot/initramfs-linux.img"

# Install GRUB config
cp "${ISO_DIR}/grub.cfg" "${ISO_STAGING}/boot/grub/grub.cfg"

# Build ISO
echo "Building ISO with grub-mkrescue..."
grub-mkrescue -o "${BUILD_DIR}/${OUTPUT_NAME}" "${ISO_STAGING}"

echo "ISO created: ${BUILD_DIR}/${OUTPUT_NAME}"
