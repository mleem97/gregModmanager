#!/usr/bin/env bash
# gregModmanager — cross-build the Windows x64 portable release package.
# Intended to run from Linux (Ubuntu/Debian) or macOS with mingw.
# Requirements:
#   rustup target add x86_64-pc-windows-gnu
#   sudo apt-get install -y gcc-mingw-w64-x86-64   # Debian/Ubuntu
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

rustup target add x86_64-pc-windows-gnu

if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
  echo "error: mingw cross compiler 'x86_64-w64-mingw32-gcc' not found." >&2
  echo "install it via: sudo apt-get install -y gcc-mingw-w64-x86-64" >&2
  exit 1
fi

exec cargo xtask dist --target x86_64-pc-windows-gnu
