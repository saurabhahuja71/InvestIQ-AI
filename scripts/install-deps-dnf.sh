#!/usr/bin/env bash
# Install InvestIQ AI host dependencies via dnf (RHEL / Oracle Linux / Fedora).
set -euo pipefail

if ! command -v dnf >/dev/null 2>&1; then
  echo "dnf not found. This script is for Fedora/RHEL/Oracle Linux." >&2
  exit 1
fi

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "Re-running with sudo..."
  exec sudo "$0" "$@"
fi

echo "==> Updating metadata"
dnf -y update

echo "==> Installing Podman, build tools, and CLIs"
dnf -y install \
  podman \
  podman-compose \
  git \
  curl \
  ca-certificates \
  gcc \
  gcc-c++ \
  make \
  openssl-devel \
  pkgconf-pkg-config \
  postgresql \
  redis \
  which \
  unzip \
  xz

echo "==> Optional: podman-docker (docker CLI → Podman)"
dnf -y install podman-docker || true

echo "==> Optional: Linux desktop / Flutter toolchain helpers"
dnf -y install \
  clang \
  cmake \
  ninja-build \
  gtk3-devel \
  mesa-libGL-devel \
  java-17-openjdk-devel || true

echo ""
echo "Installed. Verify:"
echo "  podman --version"
echo "  podman compose version   # or: podman-compose --version"
echo ""
echo "Rust (if missing): curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "Flutter: install SDK then export PATH=\"\$HOME/development/flutter/bin:\$PATH\""
echo "Docs: docs/09-podman-dnf-setup.md"
