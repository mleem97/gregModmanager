#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
PROJECT_PATH="$REPO_ROOT/src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj"
OUTPUT_ROOT="${1:-$REPO_ROOT/artifacts/avalonia-linux}"
VERSION="${2:-1.1.0}"
IS_PRE="${3:-false}"
RID="linux-x64"

PRE_SUFFIX=""
if [ "$IS_PRE" = "true" ] || [ "$IS_PRE" = "1" ]; then
  PRE_SUFFIX="-pre"
fi

PUBLISH_DIR="$OUTPUT_ROOT/publish"
PKG_DIR="$OUTPUT_ROOT/packages"
mkdir -p "$PUBLISH_DIR" "$PKG_DIR"

dotnet publish "$PROJECT_PATH" -c Release -r "$RID" --self-contained true -o "$PUBLISH_DIR"

tar -C "$PUBLISH_DIR" -czf "$PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.tar.gz" .

NFP_CONFIG="$OUTPUT_ROOT/nfpm.yaml"
cat > "$NFP_CONFIG" <<EOF
name: gregmodmanager
arch: amd64
platform: linux
version: ${VERSION}
section: utils
priority: optional
maintainer: teamGreg <noreply@gregframework.eu>
description: gregModmanager desktop client built with Avalonia UI.
vendor: gregFramework
homepage: https://github.com/mleem97/gregModmanager
license: Proprietary
depends:
  - libicu
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

build_nfpm() {
  local packager="$1"
  local target="$2"

  if command -v nfpm >/dev/null 2>&1; then
    nfpm package --packager "$packager" --config "$NFP_CONFIG" --target "$target"
    return
  fi

  echo "nfpm not found in PATH. Install with: echo 'deb [trusted=yes] https://repo.goreleaser.com/apt/ /' | sudo tee /etc/apt/sources.list.d/goreleaser.list && sudo apt update && sudo apt install nfpm" >&2
  exit 1
}

build_nfpm deb "$PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.deb"
build_nfpm rpm "$PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.rpm"
build_nfpm archlinux "$PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.pkg.tar.zst"

echo "Artifacts ready in: $OUTPUT_ROOT"
