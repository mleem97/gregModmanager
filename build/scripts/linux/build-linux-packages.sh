#!/usr/bin/env bash
set -euo pipefail

SOURCE_DIR="${1:-$HOME/GregTools-Releases}"
OUTPUT_DIR="${2:-$HOME/GregTools-Releases/packages}"
FORMATS="${3:-deb,rpm,apk,archlinux}"
NFP_IMAGE="ghcr.io/goreleaser/nfpm:latest"

if [[ ! -d "$SOURCE_DIR" ]]; then
  echo "[linux-pack] Source directory not found: $SOURCE_DIR" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

IFS=',' read -r -a PACKAGERS <<< "$FORMATS"

run_nfpm() {
  local packager="$1"
  local config="$2"
  local target="$3"

  if command -v docker >/dev/null 2>&1; then
    local config_in_container
    local target_in_container
    config_in_container="${config/$TMP_ROOT/\/work}"
    target_in_container="${target/$OUTPUT_DIR/\/out}"

    docker run --rm \
      -u "$(id -u):$(id -g)" \
      -v "$TMP_ROOT:/work" \
      -v "$OUTPUT_DIR:/out" \
      -w /work \
      "$NFP_IMAGE" \
      package --packager "$packager" --config "$config_in_container" --target "$target_in_container"
    return
  fi

  if command -v nfpm >/dev/null 2>&1; then
    nfpm package --packager "$packager" --config "$config" --target "$target"
    return
  fi

  echo "[linux-pack] Neither docker nor nfpm found; cannot build $packager" >&2
  exit 1
}

build_for_bundle() {
  local bundle_path="$1"
  local bundle_name
  bundle_name="$(basename "$bundle_path")"

  if [[ ! "$bundle_name" =~ ^Linux-([A-Za-z0-9._-]+)-v([0-9A-Za-z._-]+)-signed\.zip$ ]]; then
    echo "[linux-pack] Skipping unsupported bundle name: $bundle_name"
    return
  fi

  local distro="${BASH_REMATCH[1]}"
  local version="${BASH_REMATCH[2]}"
  local distro_lc
  distro_lc="$(echo "$distro" | tr '[:upper:]' '[:lower:]')"

  local sha_path="$bundle_path.sha256"
  local sig_path="$bundle_path.sig"
  local cer_path="$bundle_path.sig.cer"

  for required in "$sha_path" "$sig_path" "$cer_path"; do
    if [[ ! -f "$required" ]]; then
      echo "[linux-pack] Missing companion artifact: $required" >&2
      exit 1
    fi
  done

  local work_dir="$TMP_ROOT/$distro_lc"
  local payload_dir="$work_dir/payload"
  local install_dir="$payload_dir/opt/gregtools-modmanager/$distro_lc"
  local bin_dir="$payload_dir/usr/local/bin"
  local desktop_dir="$payload_dir/usr/share/applications"

  mkdir -p "$install_dir" "$bin_dir" "$desktop_dir"

  cp "$bundle_path" "$install_dir/"
  cp "$sha_path" "$install_dir/"
  cp "$sig_path" "$install_dir/"
  cp "$cer_path" "$install_dir/"

  local verify_script="$bin_dir/gregtools-${distro_lc}-bundle-verify"
  cat > "$verify_script" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

  BASE_DIR="/opt/gregtools-modmanager"
TARGET_DIR="${1:-}" 

if [[ -z "$TARGET_DIR" ]]; then
  echo "Usage: $(basename "$0") <distro-folder>" >&2
  exit 1
fi

BUNDLE_DIR="$BASE_DIR/$TARGET_DIR"
ZIP_FILE="$(find "$BUNDLE_DIR" -maxdepth 1 -type f -name 'Linux-*-signed.zip' | head -n 1)"

if [[ -z "$ZIP_FILE" ]]; then
  echo "No Linux signed zip found in: $BUNDLE_DIR" >&2
  exit 1
fi

SHA_FILE="$ZIP_FILE.sha256"
SIG_FILE="$ZIP_FILE.sig"
CER_FILE="$ZIP_FILE.sig.cer"

sha256sum -c "$SHA_FILE"

PUBKEY_FILE="$(mktemp)"
trap 'rm -f "$PUBKEY_FILE"' EXIT
openssl x509 -in "$CER_FILE" -pubkey -noout > "$PUBKEY_FILE"
openssl dgst -sha256 -verify "$PUBKEY_FILE" -signature "$SIG_FILE" "$ZIP_FILE"

echo "Bundle verification successful: $ZIP_FILE"
SCRIPT
  chmod +x "$verify_script"

  local launcher_script="$bin_dir/gregtools-modmanager-${distro_lc}"
  cat > "$launcher_script" <<SCRIPT
#!/usr/bin/env bash
set -euo pipefail

xdg-open "/opt/gregtools-modmanager/${distro_lc}"
SCRIPT
  chmod +x "$launcher_script"

  local app_desktop_file="$desktop_dir/gregtools-modmanager-${distro_lc}.desktop"
  cat > "$app_desktop_file" <<DESKTOP
[Desktop Entry]
Type=Application
Version=1.0
Name=gregModmanager (${distro})
GenericName=gregModmanager
Comment=Open installed GregTools Linux bundle (${distro})
Exec=/usr/local/bin/gregtools-modmanager-${distro_lc}
Terminal=false
Categories=Utility;
Keywords=GregTools;Modmanager;Bundle;
DESKTOP

  local desktop_file="$desktop_dir/gregtools-${distro_lc}-bundle-verify.desktop"
  cat > "$desktop_file" <<DESKTOP
[Desktop Entry]
Type=Application
Version=1.0
Name=GregTools ${distro} Bundle Verify
Comment=Verify signed GregTools Linux bundle (${distro})
Exec=/usr/local/bin/gregtools-${distro_lc}-bundle-verify ${distro_lc}
Terminal=true
Categories=Utility;
DESKTOP

  local nfpm_config="$work_dir/nfpm.yaml"
  cat > "$nfpm_config" <<EOF
name: gregtools-modmanager-${distro_lc}
arch: amd64
platform: linux
version: ${version}
section: utils
priority: optional
maintainer: GregTools Team <noreply@example.com>
description: gregModmanager signed Linux release bundle for ${distro}.
vendor: GregTools
license: Proprietary
homepage: https://github.com/mleem97/gregModmanager
depends:
  - bash
  - coreutils
  - findutils
  - openssl
  - ca-certificates
contents:
  - src: ./${distro_lc}/payload/opt/gregtools-modmanager/${distro_lc}/
    dst: /opt/gregtools-modmanager/${distro_lc}/
  - src: ./${distro_lc}/payload/usr/local/bin/gregtools-${distro_lc}-bundle-verify
    dst: /usr/local/bin/gregtools-${distro_lc}-bundle-verify
    file_info:
      mode: 0755
  - src: ./${distro_lc}/payload/usr/local/bin/gregtools-modmanager-${distro_lc}
    dst: /usr/local/bin/gregtools-modmanager-${distro_lc}
    file_info:
      mode: 0755
  - src: ./${distro_lc}/payload/usr/share/applications/gregtools-modmanager-${distro_lc}.desktop
    dst: /usr/share/applications/gregtools-modmanager-${distro_lc}.desktop
    file_info:
      mode: 0644
  - src: ./${distro_lc}/payload/usr/share/applications/gregtools-${distro_lc}-bundle-verify.desktop
    dst: /usr/share/applications/gregtools-${distro_lc}-bundle-verify.desktop
    file_info:
      mode: 0644
overrides:
  deb:
    recommends:
      - desktop-file-utils
EOF

  for packager in "${PACKAGERS[@]}"; do
    packager="$(echo "$packager" | xargs)"
    [[ -z "$packager" ]] && continue

    local target_file
    case "$packager" in
      deb)
        target_file="$OUTPUT_DIR/gregtools-modmanager-${distro_lc}_${version}_amd64.deb"
        ;;
      rpm)
        target_file="$OUTPUT_DIR/gregtools-modmanager-${distro_lc}-${version}-1.x86_64.rpm"
        ;;
      apk)
        target_file="$OUTPUT_DIR/gregtools-modmanager-${distro_lc}-${version}-r1.apk"
        ;;
      archlinux)
        target_file="$OUTPUT_DIR/gregtools-modmanager-${distro_lc}-${version}-1-x86_64.pkg.tar.zst"
        ;;
      *)
        echo "[linux-pack] Unsupported packager '$packager' - skipping"
        continue
        ;;
    esac

    rm -f "$target_file"
    echo "[linux-pack] Building $packager for $bundle_name -> $target_file"
    run_nfpm "$packager" "$nfpm_config" "$target_file"
  done
}

shopt -s nullglob
bundles=("$SOURCE_DIR"/Linux-*-v*-signed.zip)
if [[ ${#bundles[@]} -eq 0 ]]; then
  echo "[linux-pack] No Linux signed bundles found in $SOURCE_DIR" >&2
  exit 1
fi

for bundle in "${bundles[@]}"; do
  build_for_bundle "$bundle"
done

echo "[linux-pack] Done. Packages written to: $OUTPUT_DIR"
ls -lah "$OUTPUT_DIR"

