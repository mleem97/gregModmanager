#!/usr/bin/env bash
# Publish one verified, stable GitHub release from the exact CI commit.
set -euo pipefail

version=${1:?usage: publish-github-release.sh VERSION ARTIFACT_DIR}
artifact_dir=${2:?usage: publish-github-release.sh VERSION ARTIFACT_DIR}
[[ $# == 2 && "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || { echo 'Expected a stable numeric version and artifact directory' >&2; exit 1; }
: "${GH_TOKEN:?GH_TOKEN required}" "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY required}" "${GITHUB_SHA:?GITHUB_SHA required}"
[[ "$GITHUB_REPOSITORY" =~ ^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$ && "$GITHUB_SHA" =~ ^[0-9a-f]{40}$ ]] || { echo 'Invalid GitHub repository or commit' >&2; exit 1; }
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
artifact_dir=$(cd "$artifact_dir" && pwd)
tag="v$version"
packages=(
  "gregmodmanager-$version-win-x64-setup.exe"
  "gregmodmanager-$version-win-x64.msi"
  "gregmodmanager-$version-win-x64.zip"
  "gregmodmanager-$version-linux-x64.tar.gz"
  "gregmodmanager-$version-linux-x64.deb"
  "gregmodmanager-$version-linux-x64.rpm"
  "gregmodmanager-$version-linux-x64.pkg.tar.zst"
)

python3 - "$root" "$version" "$artifact_dir" "${packages[@]}" <<'PY'
import hashlib
from pathlib import Path
import re
import sys
import tomllib

root, version, directory = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
with (root / 'Cargo.toml').open('rb') as cargo_file:
    cargo_version = tomllib.load(cargo_file)['workspace']['package']['version']
if (root / 'VERSION').read_text().strip() != version or cargo_version != version:
    sys.exit('VERSION and Cargo workspace version must match the requested release')
for name in sys.argv[4:]:
    path = directory / name
    checksum = directory / (name + '.sha256')
    if not path.is_file() or not checksum.is_file():
        sys.exit(f'Missing release asset or checksum: {name}')
    content = checksum.read_text()
    if not re.fullmatch(r'[0-9a-fA-F]{64}  ' + re.escape(name) + r'\n?', content):
        sys.exit(f'Invalid checksum file: {checksum.name}')
    with path.open('rb') as artifact:
        digest = hashlib.file_digest(artifact, 'sha256').hexdigest()
    if digest != content[:64].lower():
        sys.exit(f'Checksum mismatch: {name}')
PY

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT
api="https://api.github.com/repos/$GITHUB_REPOSITORY"
get_api() {
  curl --silent --show-error --location --connect-timeout 15 --max-time 60 --output "$tmp_dir/response.json" \
    --write-out '%{http_code}' --header "Authorization: Bearer $GH_TOKEN" \
    --header 'Accept: application/vnd.github+json' "$1"
}

find_release() {
  # The REST tag endpoint only returns published releases; list also finds drafts.
  gh api --paginate "repos/$GITHUB_REPOSITORY/releases?per_page=100" \
    --jq ".[] | select(.tag_name == \"$tag\")" > "$tmp_dir/release.json"
}
find_release
if [ -s "$tmp_dir/release.json" ]; then
    python3 - "$tmp_dir/release.json" "$tag" <<'PY'
import json, sys
with open(sys.argv[1]) as response:
    release = json.load(response)
if release.get('tag_name') != sys.argv[2] or release.get('draft') is not False or release.get('prerelease') is not False:
    sys.exit('Existing release is not a published stable release; refusing to overwrite')
PY
    echo "Stable release $tag already exists; leaving it unchanged."
    exit 0
fi
verify_latest() {
  status=$(get_api "$api/releases/latest")
  case "$status" in
  200)
    python3 - "$tmp_dir/response.json" "$version" <<'PY'
import json, re, sys
with open(sys.argv[1]) as response:
    release = json.load(response)
tag = release.get('tag_name', '')
match = re.fullmatch(r'v?(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)', tag)
if release.get('draft') is not False or release.get('prerelease') is not False or not match:
    sys.exit('Cannot safely compare the latest stable release version')
if tuple(map(int, sys.argv[2].split('.'))) <= tuple(map(int, match.groups())):
    sys.exit('Requested version is not newer than the latest stable release')
PY
    ;;
  404) ;;
    *) echo "Cannot check latest stable release (HTTP $status)" >&2; exit 1 ;;
  esac
}
verify_latest
verify_main() {
  status=$(get_api "$api/git/ref/heads/main")
  [ "$status" = 200 ] || { echo "Cannot verify main (HTTP $status)" >&2; exit 1; }
  python3 - "$tmp_dir/response.json" "$GITHUB_SHA" <<'PY'
import json, sys
with open(sys.argv[1]) as response:
    obj = json.load(response).get('object', {})
if obj.get('type') != 'commit' or obj.get('sha') != sys.argv[2]:
    sys.exit('main no longer points to this commit; refusing publication')
PY
}
verify_main
status=$(get_api "$api/git/ref/tags/$tag")
case "$status" in
  404) ;;
  200) echo "Tag $tag already exists without a stable release; refusing to overwrite" >&2; exit 1 ;;
  *) echo "Cannot check existing tag (HTTP $status)" >&2; exit 1 ;;
esac

assets=()
for name in "${packages[@]}"; do
  assets+=("$artifact_dir/$name" "$artifact_dir/$name.sha256")
done
gh api --method POST "repos/$GITHUB_REPOSITORY/git/refs" \
  -f "ref=refs/tags/$tag" -f "sha=$GITHUB_SHA" > /dev/null
status=$(get_api "$api/git/ref/tags/$tag")
[ "$status" = 200 ] || { echo "Cannot verify new tag (HTTP $status)" >&2; exit 1; }
verify_tag() {
python3 - "$tmp_dir/response.json" "$GITHUB_SHA" <<'PY'
import json, sys
with open(sys.argv[1]) as response:
    obj = json.load(response).get('object', {})
if obj.get('type') != 'commit' or obj.get('sha') != sys.argv[2]:
    sys.exit('Release tag does not point to the expected commit; leaving release unpublished')
PY
}
verify_tag
gh release create "$tag" "${assets[@]}" --repo "$GITHUB_REPOSITORY" \
  --verify-tag --title "$tag" --draft --generate-notes \
  --notes 'Alpine APK is not included because it is not yet supported. Windows installers are unsigned.'

status=$(get_api "$api/git/ref/tags/$tag")
[ "$status" = 200 ] || { echo "Cannot verify release tag (HTTP $status)" >&2; exit 1; }
verify_tag
for attempt in {1..12}; do
  find_release
  [ -s "$tmp_dir/release.json" ] || { echo 'Cannot find the draft release' >&2; exit 1; }
  if python3 - "$tmp_dir/release.json" "$artifact_dir" "$tag" "${packages[@]}" <<'PY'
import hashlib, json, sys
from pathlib import Path
with open(sys.argv[1]) as response:
    release = json.load(response)
directory, tag = Path(sys.argv[2]), sys.argv[3]
if release.get('tag_name') != tag or release.get('draft') is not True or release.get('prerelease') is not False:
    sys.exit('Release is not the expected stable draft')
expected = {name for package in sys.argv[4:] for name in (package, package + '.sha256')}
assets = release.get('assets', [])
if len(assets) != len(expected) or {asset.get('name') for asset in assets} != expected:
    sys.exit('Draft release has missing or unexpected assets')
pending_digest = False
for asset in assets:
    path = directory / asset['name']
    with path.open('rb') as artifact:
        digest = hashlib.file_digest(artifact, 'sha256').hexdigest()
    if asset.get('size') != path.stat().st_size or asset.get('state') != 'uploaded':
        sys.exit(f"Draft asset failed verification: {asset['name']}")
    if asset.get('digest') is None:
        pending_digest = True
    elif asset['digest'] != 'sha256:' + digest:
        sys.exit(f"Draft asset digest mismatch: {asset['name']}")
if pending_digest:
    sys.exit(3)
PY
  then
    break
  else
    result=$?
    if [ "$result" -ne 3 ] || [ "$attempt" -eq 12 ]; then
      echo 'Draft asset verification failed; release remains unpublished' >&2
      exit 1
    fi
    sleep 5
  fi
done
verify_main
verify_latest
status=$(get_api "$api/git/ref/tags/$tag")
[ "$status" = 200 ] || { echo "Cannot recheck release tag (HTTP $status)" >&2; exit 1; }
verify_tag
gh release edit "$tag" --repo "$GITHUB_REPOSITORY" --draft=false --prerelease=false --latest
