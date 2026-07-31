#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$SCRIPT_DIR/src/GregModmanager.Core"
PUBLISH_DIR="$SCRIPT_DIR/publish"
ARTIFACTS_DIR="$SCRIPT_DIR/artifacts"
LINUX_DIR="$PUBLISH_DIR/linux-x64"
WINDOWS_DIR="$PUBLISH_DIR/win-x64"
TIMESTAMP="$(date -u +%Y%m%d%H%M%S)"

# --- Version ---
get_version() {
  local csproj="$SCRIPT_DIR/src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj"
  sed -n 's/.*<Version>\(.*\)<\/Version>.*/\1/p' "$csproj"
}
VERSION="$(get_version)"
IS_PRE=false
if echo "$VERSION" | grep -qiE '(pre|alpha|beta|rc)'; then IS_PRE=true; fi
PRE_SUFFIX=""; $IS_PRE && PRE_SUFFIX="-pre"

echo "=== Building gregModmanager v${VERSION}${PRE_SUFFIX} ==="

# --- Clean ---
rm -rf "$PUBLISH_DIR" "$ARTIFACTS_DIR"
mkdir -p "$PUBLISH_DIR" "$ARTIFACTS_DIR"

# --- Restore ---
echo ""
echo "=== Restore ==="
dotnet restore "$SCRIPT_DIR/GregModmanager.sln"

# --- Build Linux ---
echo ""
echo "=== Build Linux (linux-x64) ==="
dotnet publish "$SCRIPT_DIR/src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj" \
  -c Release -r linux-x64 \
  --self-contained true \
  /p:PublishTrimmed=true /p:TrimMode=full \
  /p:PublishSingleFile=true /p:DebugType=none /p:DebugSymbols=false \
  -o "$LINUX_DIR"

# Post-publish: patch Steam symbols, remove Windows DLL/crap, copy .so files
python3 "$SCRIPT_DIR/build/scripts/linux/patch-steamworks-symbols.py" "$LINUX_DIR/GregModmanager"
rm -f "$LINUX_DIR/steam_api64.dll" "$LINUX_DIR/libHarfBuzzSharp.pdb" "$LINUX_DIR/libSkiaSharp.pdb"
for lib in libsteam_api64.so libsteam_api.so; do
  if [ -f "$CORE_DIR/$lib" ]; then
    cp "$CORE_DIR/$lib" "$LINUX_DIR/$lib"
  fi
done

# --- Build Windows ---
echo ""
echo "=== Build Windows (win-x64) ==="
dotnet publish "$SCRIPT_DIR/src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj" \
  -c Release -r win-x64 \
  --self-contained true \
  /p:PublishTrimmed=true /p:TrimMode=full \
  /p:PublishSingleFile=true /p:DebugType=none /p:DebugSymbols=false \
  -o "$WINDOWS_DIR"

# Clean up platform-mismatched native libs and debug symbols
rm -f "$WINDOWS_DIR/libsteam_api64.so" "$WINDOWS_DIR/libsteam_api.so"
rm -f "$WINDOWS_DIR/libHarfBuzzSharp.pdb" "$WINDOWS_DIR/libSkiaSharp.pdb"
# Ensure Windows native DLL is present
if [ -f "$CORE_DIR/steam_api64.dll" ]; then
  cp "$CORE_DIR/steam_api64.dll" "$WINDOWS_DIR/steam_api64.dll"
fi

# --- Generate self-signed code signing certificate ---
echo ""
echo "=== Self-signed code signing certificate ==="
CERT_DIR="$ARTIFACTS_DIR/codesign"
mkdir -p "$CERT_DIR"
openssl req -x509 -newkey rsa:2048 -keyout "$CERT_DIR/codesign-key.pem" \
  -out "$CERT_DIR/codesign-cert.pem" -days 3650 -nodes \
  -subj "/CN=gregModmanager Build/OU=Build/O=Greg Modding Team/L=Berlin/C=DE" \
  -addext "keyUsage=digitalSignature" \
  -addext "extendedKeyUsage=codeSigning"
# Export as PFX for Windows signtool
openssl pkcs12 -export -in "$CERT_DIR/codesign-cert.pem" \
  -inkey "$CERT_DIR/codesign-key.pem" \
  -out "$CERT_DIR/codesign.pfx" -passout pass:
echo "  Certificate: $CERT_DIR/codesign-cert.pem"
echo "  PFX (empty password): $CERT_DIR/codesign.pfx"

# --- Package: Linux tar.gz ---
echo ""
echo "=== Package Linux tar.gz ==="
LINUX_TGZ="$ARTIFACTS_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.tar.gz"
tar -C "$LINUX_DIR" -czf "$LINUX_TGZ" .
echo "  $LINUX_TGZ ($(du -h "$LINUX_TGZ" | cut -f1))"

# --- Package: Linux ZIP ---
echo ""
echo "=== Package Linux ZIP ==="
LINUX_ZIP="$ARTIFACTS_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.zip"
(cd "$LINUX_DIR" && zip -r "$LINUX_ZIP" .)
echo "  $LINUX_ZIP ($(du -h "$LINUX_ZIP" | cut -f1))"

# --- Package: Linux DEB/RPM/Arch with nfpm ---
echo ""
echo "=== Package Linux DEB/RPM/Arch ==="
PKG_DIR="$ARTIFACTS_DIR/packages"
mkdir -p "$PKG_DIR"

CLEANUP_SCRIPT="$SCRIPT_DIR/build/scripts/linux/gregmodmanager-cleanup.sh"
NFP_CONFIG="$PKG_DIR/nfpm.yaml"
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
scripts:
  preinstall: ${CLEANUP_SCRIPT}
  postinstall: ${CLEANUP_SCRIPT}
contents:
  - src: ${LINUX_DIR}/
    dst: /opt/gregmodmanager/
  - src: ${SCRIPT_DIR}/build/scripts/linux/gregmodmanager.desktop
    dst: /usr/share/applications/gregmodmanager.desktop
    file_info:
      mode: 0644
  - src: ${SCRIPT_DIR}/build/scripts/linux/gregmodmanager
    dst: /usr/bin/gregmodmanager
    file_info:
      mode: 0755
EOF

nfpm package --packager deb --config "$NFP_CONFIG" \
  --target "$PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.deb"
echo "  DEB: $PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.deb"

nfpm package --packager rpm --config "$NFP_CONFIG" \
  --target "$PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.rpm"
echo "  RPM: $PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.rpm"

nfpm package --packager archlinux --config "$NFP_CONFIG" \
  --target "$PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.pkg.tar.zst"
echo "  Arch: $PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.pkg.tar.zst"

# --- Package: Windows portable ZIP ---
echo ""
echo "=== Package Windows ZIP ==="
WINDOWS_ZIP="$ARTIFACTS_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Windows.zip"
(cd "$WINDOWS_DIR" && zip -r "$WINDOWS_ZIP" .)
echo "  $WINDOWS_ZIP ($(du -h "$WINDOWS_ZIP" | cut -f1))"

# --- SHA256 hashes for all artifacts ---
echo ""
echo "=== SHA256 hashes ==="
SHA_FILE="$ARTIFACTS_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-sha256.txt"
>"$SHA_FILE"
for f in "$LINUX_TGZ" "$LINUX_ZIP" "$WINDOWS_ZIP" \
  "$PKG_DIR"/*.deb "$PKG_DIR"/*.rpm "$PKG_DIR"/*.pkg.tar.zst; do
  if [ -f "$f" ]; then
    hash=$(sha256sum "$f" | cut -d' ' -f1)
    echo "$hash  $(basename "$f")" >> "$SHA_FILE"
  fi
done
cat "$SHA_FILE"

# --- Detached OpenSSL signatures ---
echo ""
echo "=== Detached signatures ==="
# Create detached PKCS7 signature for each artifact using the self-signed cert
for f in "$LINUX_TGZ" "$LINUX_ZIP" "$WINDOWS_ZIP" \
  "$PKG_DIR"/*.deb "$PKG_DIR"/*.rpm "$PKG_DIR"/*.pkg.tar.zst; do
  if [ -f "$f" ]; then
    openssl smime -sign -in "$f" -out "${f}.sig" -signer "$CERT_DIR/codesign-cert.pem" \
      -inkey "$CERT_DIR/codesign-key.pem" -binary -outform DER 2>/dev/null || true
    cp "$CERT_DIR/codesign-cert.pem" "${f}.sig.cer"
  fi
done
echo "  .sig and .sig.cer files created alongside each artifact."

# --- (Optional) APT repo ---
echo ""
echo "=== APT repository ==="
APT_DIR="$ARTIFACTS_DIR/apt-repo"
mkdir -p "$APT_DIR/pool/main" "$APT_DIR/dists/stable/main/binary-amd64"
cp "$PKG_DIR"/*.deb "$APT_DIR/pool/main/" 2>/dev/null || true
cd "$APT_DIR"
dpkg-scanpackages pool/main 2>/dev/null | gzip -9c > dists/stable/main/binary-amd64/Packages.gz || true
cd "$SCRIPT_DIR"
echo "  APT repo: $APT_DIR"

# --- Summary ---
echo ""
echo "============================================"
echo "  Build complete: gregModmanager v${VERSION}${PRE_SUFFIX}"
echo "============================================"
echo ""
echo "Artifacts:"
echo "  Linux tar.gz:  $LINUX_TGZ"
echo "  Linux ZIP:     $LINUX_ZIP"
echo "  DEB package:   $PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.deb"
echo "  RPM package:   $PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.rpm"
echo "  Arch package:  $PKG_DIR/gregModmanager-${VERSION}${PRE_SUFFIX}-Linux.pkg.tar.zst"
echo "  Windows ZIP:   $WINDOWS_ZIP"
echo "  SHA256:        $SHA_FILE"
echo "  Code signing:  $CERT_DIR/codesign.pfx"
echo "  APT repo:      $APT_DIR"
echo ""
echo "To sign Windows binaries with Authenticode (requires Windows + signtool.exe):"
echo "  signtool sign /fd SHA256 /f $CERT_DIR/codesign.pfx /tr http://timestamp.digicert.com"
echo ""
echo "Note: Detached signatures (.sig + .sig.cer) generated for all artifacts."
echo "============================================"
