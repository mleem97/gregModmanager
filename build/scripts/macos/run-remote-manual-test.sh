#!/usr/bin/env bash
# Builds and tests on a real macOS host, then leaves a launchable GUI build there.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
MACOS_HOST="${MACOS_TEST_HOST:-}"
REMOTE_DIR="${MACOS_TEST_DIR:-~/gregModmanager-manual-test}"
ARCH="${MACOS_TEST_ARCH:-arm64}"

usage() {
  cat <<'EOF'
Usage: run-remote-manual-test.sh --host user@mac-host [--arch arm64|x64] [--remote-dir path]

The macOS host must have SSH, rsync, Xcode command-line tools and the .NET SDK
required by global.json. After a successful run, connect to the Mac via Screen
Sharing or its local desktop and launch the printed executable manually.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host) MACOS_HOST="$2"; shift 2 ;;
    --arch) ARCH="$2"; shift 2 ;;
    --remote-dir) REMOTE_DIR="$2"; shift 2 ;;
    --help|-h) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "$MACOS_HOST" ]]; then
  echo "A macOS SSH host is required (--host or MACOS_TEST_HOST)." >&2
  exit 2
fi
if [[ "$ARCH" != "arm64" && "$ARCH" != "x64" ]]; then
  echo "--arch must be arm64 or x64." >&2
  exit 2
fi

if [[ ! "$REMOTE_DIR" =~ ^(~\/|/)[A-Za-z0-9_./-]*$ ]]; then
  echo "--remote-dir must be an absolute path or begin with ~/ (without spaces)." >&2
  exit 2
fi

echo "[macOS] Checking remote prerequisites on $MACOS_HOST ..."
ssh "$MACOS_HOST" "command -v dotnet >/dev/null && command -v rsync >/dev/null && mkdir -p $REMOTE_DIR"

echo "[macOS] Synchronizing source (no remote files are deleted) ..."
rsync -az \
  --exclude '.git/' --exclude '.vs/' --exclude '.idea/' --exclude 'bin/' --exclude 'obj/' \
  --exclude 'artifacts/' --exclude 'publish/' --exclude 'TestResults/' --exclude '.env' \
  "$REPO_ROOT/" "$MACOS_HOST:$REMOTE_DIR/"

echo "[macOS] Restoring, building, and testing on the real macOS host ..."
ssh "$MACOS_HOST" "set -euo pipefail; cd $REMOTE_DIR; dotnet restore GregModmanager.sln; dotnet build GregModmanager.sln --no-restore -c Release; dotnet test GregModmanager.sln --no-build -c Release --verbosity normal; dotnet publish src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj -c Release -r osx-$ARCH --self-contained true -o artifacts/manual-macos-osx-$ARCH"

echo
echo "[macOS] Manual GUI build is ready on $MACOS_HOST:"
echo "  $REMOTE_DIR/artifacts/manual-macos-osx-$ARCH/GregModmanager"
echo "Open that path in a terminal on the Mac's logged-in desktop, then test the GUI manually."
