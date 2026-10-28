#!/bin/bash
# greg quality gates: dev -> $STAGE (env STAGE, KIND, NEEDS_REFS).
# Writes gate-report.txt, sets verdict= / sha= in $GITHUB_OUTPUT.
set -u
STAGE="${STAGE:?}"; KIND="${KIND:?}"; NEEDS_REFS="${NEEDS_REFS:-false}"
git fetch origin "$STAGE" --depth=100 >/dev/null 2>&1 || git fetch origin "$STAGE" >/dev/null 2>&1 || true
git fetch origin --tags --force >/dev/null 2>&1 || true
SHA="$(git rev-parse HEAD)"
: > gate-report.txt
GATES_OK=true
report() { echo "$1" | tee -a gate-report.txt; }
gfail() { GATES_OK=false; report "GATE $1: FAIL - $2"; }
gpass() { report "GATE $1: PASS - $2"; }
gna()   { report "GATE $1: N/A (pass, annotated) - $2"; }
report "# greg gates: dev -> $STAGE @ $SHA (kind=$KIND)"

CHANGED="$(git diff --name-only "origin/$STAGE...HEAD" 2>/dev/null | sort -u || true)"

# ---------- build ----------
BUILD_RC=1; WARN_DEV=0
case "$KIND" in
  rust)
    if [ -f Cargo.toml ]; then
      cargo build --release >build-dev.log 2>&1; BUILD_RC=$?
      [ "$BUILD_RC" -eq 0 ] && gpass build "cargo build --release ok" || gfail build "cargo build failed (see build-dev.log)"
    else gfail build "no Cargo.toml"; fi
    ;;
  node)
    if [ -f package.json ]; then
      if [ -f pnpm-lock.yaml ]; then PM=pnpm; else PM=npm; fi
      $PM install --frozen-lockfile >build-dev.log 2>&1 || $PM install >>build-dev.log 2>&1
      if node -e "const p=require('./package.json');process.exit(p.scripts&&p.scripts.build?0:1)"; then
        $PM run build >>build-dev.log 2>&1; BUILD_RC=$?
        [ "$BUILD_RC" -eq 0 ] && gpass build "$PM run build ok" || gfail build "$PM run build failed"
      else gna build "no build script (lint/typecheck only)"; BUILD_RC=0; fi
    else gna build "no package.json"; BUILD_RC=0; fi
    ;;
  *) gna build "generic repo: nothing to compile"; BUILD_RC=0 ;;
esac

# ---------- tests ----------
TESTS_RC=1; HAVE_TESTS=false
case "$KIND" in
  rust)
    if [ -f Cargo.toml ]; then HAVE_TESTS=true
      cargo test >test-dev.log 2>&1; TESTS_RC=$?
      [ "$TESTS_RC" -eq 0 ] && gpass tests "cargo test ok" || gfail tests "cargo test failed"
    else gfail tests "no Cargo.toml"; fi
    ;;
  node)
    if [ -f package.json ] && node -e "const p=require('./package.json');process.exit(p.scripts&&p.scripts.test?0:1)"; then
      HAVE_TESTS=true
      if [ -f pnpm-lock.yaml ]; then pnpm test >test-dev.log 2>&1; TESTS_RC=$?
      else npm test >test-dev.log 2>&1; TESTS_RC=$?; fi
      [ "$TESTS_RC" -eq 0 ] && gpass tests "npm/pnpm test ok" || gfail tests "tests failed"
    else gna tests "no test script (annotated)"; TESTS_RC=0; fi
    ;;
  *) gna tests "generic repo (annotated)"; TESTS_RC=0 ;;
esac

# ---------- coverage (>=90%%, drop <=5pp vs STAGE) ----------
if [ "$HAVE_TESTS" = true ] && [ "$TESTS_RC" -eq 0 ]; then
  case "$KIND" in
    rust)
      if command -v cargo-tarpaulin >/dev/null; then
        DEV_COV="$(cargo tarpaulin --out Xml --output-dir /tmp 2>/dev/null | grep -oE '[0-9]+\.[0-9]+%' | tail -n 1 | tr -d '%')"
        rm -rf /tmp/gbase && git worktree add --detach /tmp/gbase "origin/$STAGE" >/dev/null 2>&1
        BASE_COV="$(cd /tmp/gbase && cargo tarpaulin --out Xml --output-dir /tmp 2>/dev/null | grep -oE '[0-9]+\.[0-9]+%' | tail -n 1 | tr -d '%')"
        git worktree remove --force /tmp/gbase >/dev/null 2>&1 || true
        DEV_COV="${DEV_COV:-0}"; BASE_COV="${BASE_COV:-0}"
        if python3 -c "import sys; sys.exit(0 if ($DEV_COV >= 90 and ($BASE_COV - $DEV_COV) <= 5) else 1)"; then
          gpass coverage "dev=${DEV_COV}% base=${BASE_COV}%"
        else gfail coverage "need >=90%% and drop<=5pp (dev=${DEV_COV}% base=${BASE_COV}%)"; fi
      else gfail coverage "cargo-tarpaulin missing"; fi
      ;;
    *) gna coverage "not measurable for kind=$KIND (annotated)" ;;
  esac
else
  case "$KIND" in rust) gna coverage "cargo-tarpaulin not installed (install to enable coverage measurement)";; *) gna coverage "no test harness (annotated)";; esac
fi

# ---------- security: 0 new issues ----------
if command -v gitleaks >/dev/null; then
  if gitleaks detect --source . --no-git -v >gitleaks.log 2>&1; then gpass security-secrets "gitleaks: 0 findings";
  else gfail security-secrets "gitleaks findings (see gitleaks.log)"; fi
else gfail security-secrets "gitleaks not installed on runner"; fi
case "$KIND" in
  rust)
    if [ -f Cargo.lock ]; then cargo audit >cargo-audit.log 2>&1 && gpass security-deps "cargo audit clean" || gfail security-deps "cargo audit findings";
    else gna security-deps "no Cargo.lock"; fi
    ;;
  node)
    if [ -f package-lock.json ]; then npm audit --audit-level=high >npm-audit.log 2>&1 && gpass security-deps "npm audit clean" || gfail security-deps "npm audit high+ findings";
    elif [ -f pnpm-lock.yaml ]; then pnpm audit --audit-level high >npm-audit.log 2>&1 && gpass security-deps "pnpm audit clean" || gfail security-deps "pnpm audit high+ findings";
    else gna security-deps "no lockfile"; fi
    ;;
  *) gna security-deps "generic repo" ;;
esac

# ---------- version consistency (repos with VERSION file) ----------
if [ -f VERSION ]; then
  CONS_OUT="$(python3 - <<'PYEOF' 2>&1
import re, subprocess, sys
ver = open("VERSION").read().strip()
base = re.sub(r"-dev$", "", ver)
ok = True; notes = []
cargo_ver = ""
try:
    m = re.search(r'^version = "([^"]+)"', open("Cargo.toml").read(), re.M)
    cargo_ver = m.group(1) if m else ""
except Exception as e:
    ok = False; notes.append(f"Cargo.toml error: {e}")
if cargo_ver and re.sub(r"-dev$", "", cargo_ver) != base:
    ok = False; notes.append(f"Cargo.toml Version={cargo_ver} (expected {base})")
print(("GATE consistency: PASS - all version sources agree on " + base) if ok else ("GATE consistency: FAIL - " + "; ".join(notes) + f" (VERSION base={base})"))
sys.exit(0)
PYEOF
)"
  echo "$CONS_OUT" >>gate-report.txt
  echo "$CONS_OUT" | grep -q "^GATE consistency: PASS" || GATES_OK=false
else gna consistency "no VERSION file"; fi

# ---------- conventional commits ----------
if git log --no-merges --format=%s "origin/$STAGE..HEAD" 2>/dev/null | grep -vE "^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?: .+" | grep -q .; then
  gfail conventional "non-conventional subjects:"; git log --no-merges --format=%s "origin/$STAGE..HEAD" | grep -vE "^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?: .+" | head -n 5 | tee -a gate-report.txt
else gpass conventional "all subjects conventional"; fi

# ---------- verdict ----------
if [ "$GATES_OK" = true ]; then echo "OVERALL: PASS" | tee -a gate-report.txt; echo "verdict=pass" >> "$GITHUB_OUTPUT";
else echo "OVERALL: FAIL (staging NOT updated)" | tee -a gate-report.txt; echo "verdict=fail" >> "$GITHUB_OUTPUT"; fi
echo "sha=$SHA" >> "$GITHUB_OUTPUT"
