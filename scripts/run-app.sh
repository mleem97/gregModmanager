#!/usr/bin/env bash
# gregModmanager — build the desktop app from source (debug) and start it.
#   1. `cargo xtask setup` copies the Steam native library next to the dev binaries.
#   2. `cargo run -p greg-app` debug-builds and launches the Slint desktop UI.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

cargo xtask setup
exec cargo run -p greg-app
