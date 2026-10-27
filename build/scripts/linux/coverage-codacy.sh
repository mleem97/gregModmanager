#!/usr/bin/env bash
#
# coverage-codacy.sh — lokale Codacy Code-Coverage für das Rust-Workspace
#
# Generiert ein workspace-weites LCOV-Coverage-Report mit cargo-tarpaulin und
# lädt es über den Codacy Coverage Reporter an Codacy hoch.
#
# Dies spiegelt den CI-Job `coverage` (.forgejo/.gitea/workflows/rust.yml) wider,
# sodass lokale Uploads identisch mit CI-Uploads verlaufen.
#
# Voraussetzungen:
#   - cargo (Rust toolchain)
#   - CODACY_PROJECT_TOKEN (Codacy → Project settings → API token)
#   - Optional: CODACY_API_BASE_URL (bei Codacy Self-hosted)
#   - Optional: CODACY_REPORTER_VERSION (Codacy Coverage Reporter Version)
#
# Verwendung:
#   ./build/scripts/linux/coverage-codacy.sh                    # erzeugt + hochlädt (ein Report)
#   ./build/scripts/linux/coverage-codacy.sh --partial -r a.lcov   # Teil-Report hochladen
#   ./build/scripts/linux/coverage-codacy.sh --final                # final signal senden
#   ./build/scripts/linux/coverage-codacy.sh --check                # nur Voraussetzungen prüfen
#   ./build/scripts/linux/coverage-codacy.sh --install-only         # nur Reporter + tarpaulin installieren
#
# Mehrere Reports für dieselbe Sprache (erweitertes Szenario):
#   ./build/scripts/linux/coverage-codacy.sh --partial -l Rust -r report1.lcov
#   ./build/scripts/linux/coverage-codacy.sh --partial -l Rust -r report2.lcov
#   ./build/scripts/linux/coverage-codacy.sh --final
#
# Nicht unterstützte Sprache erzwingen:
#   ./build/scripts/linux/coverage-codacy.sh --force-language -l Kotlin -r report.lcov

set -euo pipefail

# --- Pfade ------------------------------------------------------------------

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

COVERAGE_DIR="$REPO_ROOT/target/codacy"
REPORTER_TMP="$HOME/.cache/codacy/codacy-coverage-reporter"
REPORTER_BIN="$REPORTER_TMP/codacy-coverage-reporter"

LANGUAGE="Rust"
FORCE_LANGUAGE=false
PARTIAL=false
FINAL=false
CHECK_ONLY=false
INSTALL_ONLY=false
REPORT_FILES=()

# --- Hilfsfunktionen --------------------------------------------------------

log() { printf '[coverage-codacy] %s\n' "$*"; }
err() { printf '[coverage-codacy] FEHLER: %s\n' "$*" >&2; }

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    err "Befehl '$1' nicht gefunden."
    return 1
  fi
}

# --- Argumente parsen -------------------------------------------------------

while [ "$#" -gt 0 ]; do
  case "$1" in
    -h|--help)
      sed -n '3,48p' "$0"
      exit 0
      ;;
    -l|--language)
      [ "$#" -ge 2 ] || { err "--language benötigt einen Wert"; exit 2; }
      LANGUAGE="$2"; shift 2 ;;
    --force-language)
      FORCE_LANGUAGE=true; shift ;;
    -r|--report)
      [ "$#" -ge 2 ] || { err "--report benötigt einen Pfad"; exit 2; }
      REPORT_FILES+=("$2"); shift 2 ;;
    --partial)
      PARTIAL=true; shift ;;
    --final)
      FINAL=true; shift ;;
    --check)
      CHECK_ONLY=true; shift ;;
    --install-only)
      INSTALL_ONLY=true; shift ;;
    --workspace-dir)
      [ "$#" -ge 2 ] || { err "--workspace-dir benötigt einen Pfad"; exit 2; }
      COVERAGE_DIR="$2"; shift 2 ;;
    --*)
      err "Unbekannte Option: $1"; exit 2 ;;
    *)
      err "Unbekanntes Argument: $1"; exit 2 ;;
  esac
done

# --- Voraussetzungen prüfen -------------------------------------------------

check_prereqs() {
  local ok=true
  need_cmd cargo || ok=false
  need_cmd git || ok=false
  if [ "$INSTALL_ONLY" = false ] && [ "$CHECK_ONLY" = false ]; then
    # cargo-tarpaulin wird installiert, falls fehlt; dennoch prüfen wir cargo.
    :
  fi
  if [ -z "${CODACY_PROJECT_TOKEN:-}" ]; then
    err "CODACY_PROJECT_TOKEN ist nicht gesetzt."
    err "  → Codacy → Project settings → API token → CODACY_PROJECT_TOKEN exportieren."
    ok=false
  fi
  if [ "$ok" = false ]; then
    exit 1
  fi
  log "Voraussetzungen OK (cargo, git, CODACY_PROJECT_TOKEN gesetzt)."
}

# --- cargo-tarpaulin installieren -------------------------------------------

ensure_tarpaulin() {
  if command -v cargo-tarpaulin >/dev/null 2>&1; then
    log "cargo-tarpaulin bereits installiert: $(command -v cargo-tarpaulin)"
    return 0
  fi
  log "cargo-tarpaulin nicht gefunden — installiere über cargo..."
  cargo install cargo-tarpaulin
}

# --- Codacy Coverage Reporter installieren ----------------------------------

ensure_reporter() {
  if [ -x "$REPORTER_BIN" ]; then
    log "Codacy Coverage Reporter bereits vorhanden: $REPORTER_BIN"
    return 0
  fi
  log "Lade Codacy Coverage Reporter herunter (get.sh) nach $REPORTER_TMP..."
  mkdir -p "$REPORTER_TMP"
  # Selbstkontainedes Bash-Skript von Codacy: lädt die passende Version herunter
  # und legt die Binärdatei ab. Wir rufen es mit 'download' auf, damit nur installiert wird.
  bash <(curl -Ls https://coverage.codacy.com/get.sh) download
  # Das get.sh-Skript legt die Binärdatei unter $HOME/.cache/codacy/<version>/codacy-cli-v2 ab?
  # Für den Coverage Reporter heisst das Artefakt codacy-coverage-reporter. Wir suchen es.
  if [ ! -x "$REPORTER_BIN" ]; then
    local found
    found="$(find "$REPORTER_TMP" -maxdepth 3 -type f -name 'codacy-coverage-reporter*' -executable 2>/dev/null | head -n 1 || true)"
    if [ -n "$found" ]; then
      REPORTER_BIN="$found"
      log "Reporter gefunden unter: $REPORTER_BIN"
    else
      err "Codacy Coverage Reporter konnte nicht installiert werden."
      exit 1
    fi
  fi
}

# --- LCOV Report generieren -------------------------------------------------

generate_report() {
  mkdir -p "$COVERAGE_DIR"
  log "Generiere workspace-weites LCOV-Coverage-Report in $COVERAGE_DIR..."
  cargo tarpaulin --workspace --out lcov --output-dir "$COVERAGE_DIR" --verbose
}

# --- Report-File finden -----------------------------------------------------

find_report() {
  if [ "${#REPORT_FILES[@]}" -gt 0 ]; then
    # Explizit übergebene Reports prüfen
    for r in "${REPORT_FILES[@]}"; do
      if [ ! -f "$r" ]; then
        err "Report-Datei nicht gefunden: $r"
        exit 1
      fi
    done
    return 0
  fi
  # Automatisch suchen (tarpaulin Standardausgabe)
  local candidate
  candidate="$(find "$COVERAGE_DIR" -maxdepth 1 -type f \( -name 'lcov.info' -o -name 'lcov.dat' -o -name '*.lcov' \) 2>/dev/null | head -n 1 || true)"
  if [ -z "$candidate" ]; then
    err "Kein LCOV-Coverage-Report in $COVERAGE_DIR gefunden."
    err "  → Zuerst Coverage erzeugen (ohne --check / --install-only) oder --report <datei> angeben."
    exit 1
  fi
  REPORT_FILES=("$candidate")
  log "Report-Datei erkannt: $candidate"
}

# --- Reporter-Argumente aufbauen --------------------------------------------

build_reporter_args() {
  local args=(report)
  if [ "$PARTIAL" = true ]; then args+=(--partial); fi
  if [ "$FINAL" = true ]; then args+=(final); return 0; fi
  args+=(-l "$LANGUAGE")
  if [ "$FORCE_LANGUAGE" = true ]; then args+=(--force-language); fi
  for r in "${REPORT_FILES[@]}"; do
    args+=(-r "$r")
  done
  # args in globale Variable für Aufruf
  REPORTER_ARGS="${args[*]}"
}

run_reporter() {
  log "Führe Codacy Coverage Reporter aus: $REPORTER_BIN $REPORTER_ARGS"
  # shellcheck disable=SC2086
  "$REPORTER_BIN" $REPORTER_ARGS
}

# --- Hauptlauf --------------------------------------------------------------

if [ "$CHECK_ONLY" = true ]; then
  check_prereqs
  exit 0
fi

if [ "$INSTALL_ONLY" = true ]; then
  check_prereqs
  ensure_tarpaulin
  ensure_reporter
  log "Installation abgeschlossen."
  exit 0
fi

check_prereqs
ensure_tarpaulin
ensure_reporter

if [ "$FINAL" = true ] && [ "${#REPORT_FILES[@]}" -eq 0 ]; then
  # reines --final ohne Reports: nur final signal senden
  build_reporter_args
  run_reporter
  exit 0
fi

generate_report
find_report
build_reporter_args
run_reporter

log "Fertig. Coverage an Codacy hochgeladen (Sprache: $LANGUAGE, Reports: ${REPORT_FILES[*]})."
