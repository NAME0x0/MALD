# MALD Build Environment
# Provides a reproducible Ubuntu-based environment for building the ISO and WSL tarball
# Usage:
#   docker build -t mald-builder .
#   docker run --privileged -v $(pwd)/output:/app/output mald-builder bash iso/build.sh
#   docker run --privileged -v $(pwd)/output:/app/output mald-builder bash iso/build-wsl.sh

FROM ubuntu:24.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

# Install all build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    curl \
    xorriso \
    grub-pc-bin \
    grub-common \
    mtools \
    squashfs-tools \
    zstd \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Install MALD Python package (for validation only)
RUN pip install --break-system-packages -e "."

# Default: print usage
CMD ["bash", "-c", "echo 'Usage: docker run --privileged -v $(pwd)/output:/app/output mald-builder bash iso/build.sh' && echo '       docker run --privileged -v $(pwd)/output:/app/output mald-builder bash iso/build-wsl.sh'"]
