#!/usr/bin/env bash
set -euo pipefail

# build-apt-repo.sh — Builds .deb package and creates a self-hosted APT repository structure.
#
# Usage:
#   ./build-apt-repo.sh [VERSION] [OUTPUT_DIR]
#
# The APT repo can be served via:
#   - GitHub Pages (push apt-repo/ to gh-pages branch)
#   - Gitea Pages
#   - Any static web server
#
# Users add the repo with:
#   echo "deb [signed-by=/usr/share/keyrings/gregmodmanager.gpg] https://<host>/ stable main" | sudo tee /etc/apt/sources.list.d/gregmodmanager.list

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
VERSION="${1:-$(grep -oP '<Version>\K[^<]+' "$REPO_ROOT/src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj")}"
OUTPUT_DIR="${2:-$REPO_ROOT/artifacts/apt-repo}"
PUBLISH_DIR="$REPO_ROOT/artifacts/publish/linux-x64"
PROJECT_PATH="$REPO_ROOT/src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj"

echo "========================================="
echo "  gregModmanager APT Repo Builder"
echo "  Version: $VERSION"
echo "========================================="

# Step 1: Publish if not already done
if [ ! -f "$PUBLISH_DIR/GregModmanager" ]; then
  echo "[apt-repo] Publishing linux-x64..."
  dotnet publish "$PROJECT_PATH" -c Release -r linux-x64 \
    --self-contained true \
    /p:PublishTrimmed=true /p:TrimMode=full \
    /p:PublishSingleFile=true \
    /p:DebugType=none /p:DebugSymbols=false \
    -o "$PUBLISH_DIR"
fi

# Step 2: Build .deb with nfpm
echo "[apt-repo] Building .deb package..."
DEB_DIR="$OUTPUT_DIR/pool/main"
mkdir -p "$DEB_DIR"

NFP_CONFIG="$(mktemp)"
cat > "$NFP_CONFIG" <<EOF
name: gregmodmanager
arch: amd64
platform: linux
version: ${VERSION}
section: utils
priority: optional
maintainer: gregFramework <noreply@gregframework.eu>
description: gregModmanager — Cross-platform mod manager for gregFramework.
  Features Steam Workshop integration, mod management, and plugin system.
vendor: gregFramework
homepage: https://github.com/mleem97/gregModManager
license: Proprietary
depends:
  - libicu74 | libicu72 | libicu71 | libicu70 | libicu66 | libicu63 | libicu60 | libicu55 | libicu52
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

nfpm package --packager deb --config "$NFP_CONFIG" --target "$DEB_DIR/"
rm -f "$NFP_CONFIG"

DEB_FILE="$(ls "$DEB_DIR"/*.deb | head -1)"
echo "[apt-repo] Built: $DEB_FILE"

# Step 3: Create APT repo structure
echo "[apt-repo] Creating APT repository structure..."
DIST_DIR="$OUTPUT_DIR/dists/stable/main/binary-amd64"
mkdir -p "$DIST_DIR"

# Generate Packages
dpkg-scanpackages --multiversion "$DEB_DIR" /dev/null > "$DIST_DIR/Packages"
gzip -9c "$DIST_DIR/Packages" > "$DIST_DIR/Packages.gz"

# Generate Release
RELEASE_FILE="$OUTPUT_DIR/dists/stable/Release"
cat > "$RELEASE_FILE" <<EOF
Origin: gregFramework
Label: gregModmanager
Suite: stable
Codename: stable
Architectures: amd64
Components: main
Description: gregModmanager APT Repository
Date: $(date -Ru)
$(cd "$OUTPUT_DIR/dists/stable" && apt-ftparchive release .)
EOF

# Add checksums to Release
(cd "$OUTPUT_DIR/dists/stable" && apt-ftparchive release . >> "$RELEASE_FILE" 2>/dev/null) || true

# Step 4: Sign with GPG if key is available
if [ -n "${GPG_KEY_ID:-}" ] || [ -n "${GPG_SIGNING_KEY:-}" ]; then
  echo "[apt-repo] Signing Release file..."
  GPG_OPTS=(-batch -yes -armor)
  [ -n "${GPG_KEY_ID:-}" ] && GPG_OPTS+=(--local-user "$GPG_KEY_ID")

  gpg "${GPG_OPTS[@]}" --detach-sign --output "$OUTPUT_DIR/dists/stable/Release.gpg" "$RELEASE_FILE"
  gpg "${GPG_OPTS[@]}" --clearsign --output "$OUTPUT_DIR/dists/stable/InRelease" "$RELEASE_FILE"

  # Export public key for users
  if [ -n "${GPG_KEY_ID:-}" ]; then
    gpg --armor --export "$GPG_KEY_ID" > "$OUTPUT_DIR/gregmodmanager.gpg"
  fi
  echo "[apt-repo] Signed."
else
  echo "[apt-repo] No GPG key configured. Skipping signing."
  echo "[apt-repo] Set GPG_KEY_ID or GPG_SIGNING_KEY to enable signing."
fi

# Step 5: Create install instructions
cat > "$OUTPUT_DIR/README.md" <<'README'
# gregModmanager APT Repository

## Add repository (unsigned)

```bash
echo "deb https://<your-host>/ stable main" | sudo tee /etc/apt/sources.list.d/gregmodmanager.list
sudo apt update
sudo apt install gregmodmanager
```

## Add repository (signed)

```bash
curl -fsSL https://<your-host>/gregmodmanager.gpg | sudo gpg --dearmor -o /usr/share/keyrings/gregmodmanager.gpg
echo "deb [signed-by=/usr/share/keyrings/gregmodmanager.gpg] https://<your-host>/ stable main" | sudo tee /etc/apt/sources.list.d/gregmodmanager.list
sudo apt update
sudo apt install gregmodmanager
```

## Host via GitHub Pages

1. Push the `apt-repo/` content to the `gh-pages` branch
2. Enable GitHub Pages in repo settings
3. Users can then add: `deb https://<user>.github.io/<repo>/ stable main`

## Host via Gitea Pages

1. Push to a repo with Pages enabled
2. Or serve via any static web server

## Manual install

```bash
sudo dpkg -i pool/main/gregmodmanager_*.deb
sudo apt-get install -f  # fix dependencies if needed
```
README

echo ""
echo "========================================="
echo "  APT Repository created!"
echo "========================================="
echo ""
echo "  Output: $OUTPUT_DIR"
echo ""
echo "  Structure:"
find "$OUTPUT_DIR" -type f | sort | sed 's|^|    |'
echo ""
echo "  Size: $(du -sh "$OUTPUT_DIR" | cut -f1)"
echo ""
