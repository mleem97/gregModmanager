#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
PROJECT_PATH="$REPO_ROOT/src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj"
OUTPUT_ROOT="${1:-$REPO_ROOT/artifacts/avalonia-linux-local-test}"
PUBLISH_DIR="$OUTPUT_ROOT/publish"

mkdir -p "$PUBLISH_DIR"
dotnet publish "$PROJECT_PATH" -c Release -r linux-x64 --self-contained true -o "$PUBLISH_DIR"
python3 "$REPO_ROOT/build/scripts/linux/patch-steamworks-symbols.py" "$PUBLISH_DIR/GregModmanager"
rm -f "$PUBLISH_DIR/steam_api64.dll"

for steam_library in libsteam_api64.so libsteam_api.so; do
  if [ -f "$REPO_ROOT/src/GregModmanager.Core/$steam_library" ]; then
    cp "$REPO_ROOT/src/GregModmanager.Core/$steam_library" "$PUBLISH_DIR/$steam_library"
  fi
done

cat > "$OUTPUT_ROOT/run-local-test.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
export IS_LOCAL_TEST_BUILD=TRUE
export MODSTORE_WEB_URL=https://datacentermods.home
export MODSTORE_API_URL=https://api.datacentermods.home
exec "\$(dirname "\$(readlink -f "\$0")")/publish/GregModmanager" "\$@"
EOF
chmod +x "$OUTPUT_ROOT/run-local-test.sh"

tar -C "$PUBLISH_DIR" -czf "$OUTPUT_ROOT/gregModmanager-local-test-Linux.tar.gz" .
echo "Local test artifacts ready in: $OUTPUT_ROOT"
