#!/usr/bin/env bash
# gregModmanager — local quality gate.
# Runs the same checks as `cargo xtask check` across the whole Rust workspace
# (all crates): rustfmt check -> clippy (-D warnings) -> cargo test.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "== [check] fmt --all -- --check"
cargo fmt --all -- --check
echo "== [check] clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings
echo "== [check] test --workspace"
cargo test --workspace
echo "== [check] OK"
