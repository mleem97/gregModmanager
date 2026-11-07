# Testing Patterns

> Owner: gregModmanager maintainers. Evidence: workspace test layout, CI workflow.
> Update after test tooling, test scope, or validation command changes.

## Stack and Commands

```bash
cargo test --workspace
cargo test -p greg-core -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
```

Use the stable Rust toolchain from rustup.

## Layout

Tests live next to the code (`#[cfg(test)] mod tests`) and in crate-level
`tests/` integration dirs where present; files use `snake_case`.

## Scope

| Scope | Covered | Notes |
| --- | --- | --- |
| Unit | yes | models, logic, parsers |
| Integration | partial | filesystem/service boundaries; no live Steam/Web service |
| E2E | no in desktop repo | UI, native Steam and real game flows remain manual |

## Gaps

No enforced coverage threshold. Missing tests cover native Steam, auth/network,
UI picker, package installation and transactional deployment.

## Evidence

- `crates/greg-core/src/` (test modules)
- `.forgejo/workflows/rust.yml`
