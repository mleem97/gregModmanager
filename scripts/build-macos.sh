#!/usr/bin/env bash
# gregModmanager — release build + portable package for macOS.
# Requires a macOS host (Apple SDK); macOS targets cannot be cross-built on Linux.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OS="$(uname -s)"
if [ "$OS" != "Darwin" ]; then
  echo "error: macOS release build requires a macOS host (Apple SDK)." >&2
  echo "macOS targets cannot be cross-built on Linux; run this script on macOS." >&2
  exit 1
fi

exec cargo xtask dist
