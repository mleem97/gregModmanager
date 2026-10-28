#!/usr/bin/env bash
# gregModmanager — release build + portable package for Linux (host platform).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
  echo "error: Linux release build must run on a Linux host." >&2
  exit 1
fi

exec cargo xtask dist
