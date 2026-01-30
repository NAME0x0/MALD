#!/bin/bash
# MALD WSL Rootfs Build Script
# Builds a WSL-importable tarball: wsl --import MALD <dir> mald-*.tar.gz

set -euo pipefail

MALD_VERSION="${MALD_VERSION:-0.1.0}"
BUILD_DIR="${MALD_OUTPUT_DIR:-$(pwd)/output}"
ROOTFS_DIR="${BUILD_DIR}/rootfs"
ISO_DIR="${MALD_ISO_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
OUTPUT_NAME="mald-${MALD_VERSION}-x86_64.tar.gz"

echo "MALD WSL Builder v${MALD_VERSION}"
echo "Build directory: ${BUILD_DIR}"

# Build shared rootfs
echo "Building rootfs..."
export MALD_VERSION BUILD_DIR MALD_OUTPUT_DIR="${BUILD_DIR}" MALD_ISO_DIR="${ISO_DIR}"
export MALD_NO_KERNEL=1  # WSL uses the Windows kernel; skip linux + linux-firmware
bash "${ISO_DIR}/build-rootfs.sh"

# Apply WSL-specific configuration
echo "Applying WSL configuration..."
cp "${ISO_DIR}/wsl.conf" "${ROOTFS_DIR}/etc/wsl.conf"

# Mark as WSL build in profile
sed -i 's/^export MALD_SYSTEM=1$/export MALD_SYSTEM=1\nexport MALD_WSL=1/' \
    "${ROOTFS_DIR}/etc/profile.d/mald.sh"

# Append Windows drive note to welcome
sed -i '/^echo$/a echo "Windows drives mounted at /mnt/c, /mnt/d, etc."\necho' \
    "${ROOTFS_DIR}/usr/local/bin/mald-welcome"

# Create tarball
echo "Creating WSL tarball..."
tar czf "${BUILD_DIR}/${OUTPUT_NAME}" -C "${ROOTFS_DIR}" .

# Cleanup
rm -rf "${ROOTFS_DIR}"

echo "MALD WSL build completed successfully!"
echo "Output: ${BUILD_DIR}/${OUTPUT_NAME}"
echo
echo "Import with: wsl --import MALD <install-dir> ${BUILD_DIR}/${OUTPUT_NAME}"
