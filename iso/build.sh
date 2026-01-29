#!/bin/bash
# MALD ISO Build Script
# Builds a bootable Arch Linux-based ISO with MALD components
# Works on Ubuntu CI runners — no Arch host required

set -euo pipefail

MALD_VERSION="${MALD_VERSION:-0.1.0}"
BUILD_DIR="${MALD_OUTPUT_DIR:-$(pwd)/output}"
ISO_DIR="${MALD_ISO_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"

echo "MALD ISO Builder v${MALD_VERSION}"

# Build shared rootfs
export MALD_VERSION BUILD_DIR MALD_OUTPUT_DIR="${BUILD_DIR}" MALD_ISO_DIR="${ISO_DIR}"
bash "${ISO_DIR}/build-rootfs.sh"

# Assemble ISO
bash "${ISO_DIR}/build-iso.sh"

# Cleanup rootfs
rm -rf "${BUILD_DIR}/rootfs" "${BUILD_DIR}/bootstrap" "${BUILD_DIR}/iso-staging"

echo "MALD ISO build completed successfully!"
echo "Output: ${BUILD_DIR}/mald-${MALD_VERSION}-x86_64.iso"
