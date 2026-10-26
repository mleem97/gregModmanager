#!/usr/bin/env bash
set -u

# Remove only known legacy gregModmanager paths. The canonical installation
# (/opt/gregmodmanager, /usr/bin/gregmodmanager and gregmodmanager.desktop)
# is owned by the package and remains intact.
for process_name in GregModmanager gregmodmanager; do
  if command -v pkill >/dev/null 2>&1; then
    pkill -x "$process_name" 2>/dev/null || true
  fi
done

for legacy_path in \
  /opt/gregModmanager \
  /opt/gregtools-modmanager \
  /opt/gregmodmanager-local-test \
  /usr/bin/gregModmanager \
  /usr/local/bin/gregModmanager \
  /usr/local/bin/gregmodmanager-local-test \
  /usr/share/applications/gregModmanager.desktop \
  /usr/local/share/applications/gregModmanager.desktop \
  /usr/share/applications/gregmodmanager-local-test.desktop \
  /usr/local/share/applications/gregmodmanager-local-test.desktop \
  /usr/share/applications/gregtools-modmanager.desktop \
  /usr/local/share/applications/gregtools-modmanager.desktop; do
  if [ -e "$legacy_path" ] || [ -L "$legacy_path" ]; then
    rm -rf -- "$legacy_path" 2>/dev/null || true
  fi
done

remove_legacy_user_links() {
  local app_dir="$1"
  [ -d "$app_dir" ] || return 0

  for link in "$app_dir"/gregModmanager*.desktop "$app_dir"/gregtools-modmanager*.desktop; do
    [ -e "$link" ] || [ -L "$link" ] || continue
    case "$(basename "$link")" in
      # The canonical desktop entry ships in /usr/share/applications via the
      # DEB. A stale user copy shadows it (XDG precedence) and keeps ancient
      # Exec targets (e.g. ~/GregModmanager-Test) in the start menu, so a
      # user copy that does NOT launch the packaged binary is removed.
      gregmodmanager.desktop)
        if grep -q 'Exec=/usr/bin/gregmodmanager' "$link" 2>/dev/null; then
          continue
        fi
        rm -f -- "$link" 2>/dev/null || true
        continue
        ;;
    esac
    rm -f -- "$link" 2>/dev/null || true
  done
}

remove_legacy_user_links "/root/.local/share/applications"
for home_dir in /home/*; do
  remove_legacy_user_links "$home_dir/.local/share/applications"
done

for desktop_dir in /usr/share/applications /usr/local/share/applications; do
  if command -v update-desktop-database >/dev/null 2>&1 && [ -d "$desktop_dir" ]; then
    update-desktop-database "$desktop_dir" >/dev/null 2>&1 || true
  fi
done
