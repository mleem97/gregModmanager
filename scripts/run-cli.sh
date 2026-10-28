#!/usr/bin/env bash
# gregModmanager — run the headless CLI from source (publish/status/list).
# Usage: scripts/run-cli.sh [args...]
#   e.g. scripts/run-cli.sh publish --path <dir> --changelog <text>
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

exec cargo run -p greg-cli -- "$@"
