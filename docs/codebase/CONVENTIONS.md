# Coding Conventions

> Owner: gregModmanager maintainers. Evidence: workspace lints, rustfmt, clippy
> configuration in CI, `scripts/check.sh`.
> Update after repository-wide conventions or quality gates change.

## Naming

| Item | Rule | Example |
| --- | --- | --- |
| Files | `snake_case.rs` module files | `workspace.rs`, `service.rs` |
| Modules | PascalCase folder/module names | `pub mod workspace;` |
| Items | `snake_case` functions/variants, `PascalCase` types | `fn resolve_workspace`, `struct Workspace` |
| Constants | `SCREAMING_SNAKE_CASE` | `const STEAM_APP_ID: u32 = 4170200;` |

## Formatting and Quality

- rustfmt defaults via `cargo fmt --all -- --check`.
- Clippy via `cargo clippy --workspace --all-targets -- -D warnings`.
- `cargo test --workspace` for all workspace tests.

## Errors and Logging

- Typed errors via `thiserror` in domain crates; `anyhow` at shell boundaries.
- Structured logs via `tracing` / `tracing-subscriber`.
- Diagnostic payloads must not contain tokens.

## Testing

- Rust unit + integration tests via `cargo test --workspace`.
- Temporary filesystem fixtures; cleanup in `Drop`.

## Evidence

- `scripts/check.sh`
- `crates/greg-core/src/error.rs`
- `crates/greg-platform/src/filelog.rs`
