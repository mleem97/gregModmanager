#!/usr/bin/env bash
# gregModmanager — dev/debug build of the whole Rust workspace (all crates).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

exec cargo build --workspace
