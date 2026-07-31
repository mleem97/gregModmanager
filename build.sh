#!/usr/bin/env bash
# Compatibility entry point for Linux package builds.
#
# The release implementation lives in build/scripts/linux/. Keep this wrapper
# small so it cannot diverge into a second, unsupported packaging pipeline.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec bash "$script_dir/build/scripts/linux/build-avalonia-packages.sh" "$@"
