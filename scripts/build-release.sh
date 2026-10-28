#!/usr/bin/env bash
# gregModmanager — release build + portable package for the host platform.
# Flags are passed through to `cargo xtask dist`:
#   scripts/build-release.sh
#   scripts/build-release.sh --target x86_64-pc-windows-gnu
#   scripts/build-release.sh --out <dir>
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

exec cargo xtask dist "$@"
