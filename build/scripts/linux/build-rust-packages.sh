#!/usr/bin/env bash
# Linux packages (DEB/RPM) for the Rust restart via nfpm.
# Usage: build-rust-packages.sh <publish-dir> <version> <is_pre> [out-dir]
#   publish-dir: unpacked release tree (e.g. dist/gregmodmanager-1.6.1-linux-x64)
#   version:     VERSION file content (e.g. 1.6.1)
#   is_pre:      true/false (prerelease suffix, informational only)
#   out-dir:     package destination (default: <publish-dir>/../packages)
set -euo pipefail

PUBLISH_DIR="${1:?publish dir required}"
VERSION="${2:?version required}"
IS_PRE="${3:-false}"
OUT_DIR="${4:-$(dirname "$PUBLISH_DIR")/packages}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

[ -d "$PUBLISH_DIR" ] || { echo "publish dir missing: $PUBLISH_DIR" >&2; exit 1; }
# Absolute: nfpm resolves src relative to the config file, not $PWD.
PUBLISH_DIR="$(cd "$PUBLISH_DIR" && pwd)"
OUT_DIR="$(mkdir -p "$OUT_DIR" && cd "$OUT_DIR" && pwd)"
command -v nfpm >/dev/null 2>&1 || { echo "nfpm not installed" >&2; exit 1; }
[ -x "$PUBLISH_DIR/gregmodmanager" ] || chmod +x "$PUBLISH_DIR/gregmodmanager" || true
NFP_CONFIG="$OUT_DIR/nfpm.yaml"
CLEANUP_SCRIPT="$REPO_ROOT/build/scripts/linux/gregmodmanager-cleanup.sh"

cat > "$NFP_CONFIG" <<EOF
name: gregmodmanager
arch: amd64
platform: linux
version: ${VERSION}
section: utils
priority: optional
maintainer: teamGreg <noreply@gregframework.eu>
description: gregModmanager desktop client (Rust + Slint).
vendor: gregFramework
homepage: https://git.datacentermods.com/teamGreg/gregModmanager
license: Proprietary
scripts:
  preinstall: ${CLEANUP_SCRIPT}
  postinstall: ${CLEANUP_SCRIPT}
contents:
  - src: ${PUBLISH_DIR}/
    dst: /opt/gregmodmanager/
  - src: ${REPO_ROOT}/build/scripts/linux/gregmodmanager.desktop
    dst: /usr/share/applications/gregmodmanager.desktop
    file_info:
      mode: 0644
  - src: ${REPO_ROOT}/build/scripts/linux/gregmodmanager
    dst: /usr/bin/gregmodmanager
    file_info:
      mode: 0755
EOF

(
  cd "$OUT_DIR"
  nfpm package --packager deb --config nfpm.yaml
  nfpm package --packager rpm --config nfpm.yaml
)
ls -la "$OUT_DIR"/gregmodmanager-*
