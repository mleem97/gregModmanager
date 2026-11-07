#!/usr/bin/env bash
# Package the x86-64 Linux dist tree. Usage: <publish-dir> <version> <is-pre> [out-dir]
set -euo pipefail

publish_dir="${1:?publish dir required}"
version="${2:?version required}"
out_dir="${4:-$(dirname "$publish_dir")/packages}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$ ]] || { echo "invalid version: $version" >&2; exit 1; }
for file in gregmodmanager gregcli libsteam_api.so steam_appid.txt; do
  [ -f "$publish_dir/$file" ] || { echo "missing $publish_dir/$file" >&2; exit 1; }
done
[ -x "$publish_dir/gregmodmanager" ] || { echo "application binary is not executable" >&2; exit 1; }
command -v nfpm >/dev/null 2>&1 || { echo "nfpm not installed" >&2; exit 1; }
publish_dir="$(cd "$publish_dir" && pwd)"
out_dir="$(mkdir -p "$out_dir" && cd "$out_dir" && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

# Quote filesystem paths as YAML single-quoted scalars.
yaml_path() { local path="$1"; printf "'%s'" "${path//\'/\'\'}"; }

for format in deb rpm apk archlinux; do
  case "$format" in
    deb) kind=deb; extension=deb; depends='[libc6, libfontconfig1, libxkbcommon0, libxkbcommon-x11-0, libgl1, liblzma5, libbz2-1.0]' ;;
    rpm) kind=rpm; extension=rpm; depends='[glibc, fontconfig, libxkbcommon, libglvnd-glx, xz-libs, bzip2-libs]' ;;
    apk) kind=apk; extension=apk; depends='[gcompat, fontconfig, libxkbcommon, mesa-gl, xz-libs, bzip2-libs]' ;;
    archlinux) kind=arch; extension=pkg.tar.zst; depends='[glibc, fontconfig, libxkbcommon, libglvnd, xz, bzip2]' ;;
  esac
  printf '%s\n' "$kind" > "$tmp_dir/install-kind"
  cat > "$tmp_dir/nfpm.yaml" <<EOF
name: gregmodmanager
arch: amd64
platform: linux
version: ${version}
section: utils
priority: optional
maintainer: teamGreg <noreply@gregframework.eu>
description: gregModmanager desktop client
vendor: gregFramework
homepage: https://github.com/mleem97/gregModmanager
license: Apache-2.0
depends: ${depends}
contents:
  - src: $(yaml_path "$publish_dir/")
    dst: /opt/gregmodmanager/
  - src: $(yaml_path "$tmp_dir/install-kind")
    dst: /opt/gregmodmanager/.greg-install-kind
    file_info:
      mode: 0644
  - src: $(yaml_path "$repo_root/build/scripts/linux/gregmodmanager.desktop")
    dst: /usr/share/applications/gregmodmanager.desktop
    file_info:
      mode: 0644
  - src: $(yaml_path "$repo_root/build/scripts/linux/gregmodmanager")
    dst: /usr/bin/gregmodmanager
    file_info:
      mode: 0755
EOF
  target="$out_dir/gregmodmanager-${version}-linux-x64.${extension}"
  nfpm package --packager "$format" --config "$tmp_dir/nfpm.yaml" --target "$target"
  (cd "$out_dir" && sha256sum "$(basename "$target")" > "$(basename "$target").sha256")
done
