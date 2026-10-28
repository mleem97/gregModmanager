# Technology Stack

> Owner: gregModmanager maintainers. Evidence: workspace `Cargo.toml`, crate
> manifests, `scripts/`, `.forgejo/workflows/`.
> Update after a dependency, target framework, SDK, or build-tool change.

## Runtime Summary

| Area | Value | Evidence |
| --- | --- | --- |
| Language | Rust (latest stable) | `Cargo.toml` (edition 2021) |
| UI | Slint 1.18.1 + `lucide-slint` 1.49.1 | `crates/greg-app/Cargo.toml`, `crates/greg-app/ui/` |
| Build | Cargo workspace + `xtask` | `Cargo.toml`, `xtask/` |
| Tests | `cargo test --workspace` | `scripts/check.sh` |

## Production Dependencies

| Dependency | Version | Role | Evidence |
| --- | ---: | --- | --- |
| `slint` / `slint-build` | 1.18.1 | Cross-platform desktop UI | `crates/greg-app/Cargo.toml` |
| `lucide-slint` | 1.49.1 | UI icons via `IconDisplay`/`IconSet` | `crates/greg-app/build.rs` |
| `steamworks` / `steamworks-sys` | 0.13.1 / 0.13.0 | Steam Workshop integration | `crates/greg-steam/Cargo.toml` |
| `tokio`, `reqwest` (rustls) | — | Async runtime / Modstore REST + OAuth | `crates/greg-modstore/Cargo.toml` |
| `serde` / `serde_json` | — | Models, metadata, manifests | `crates/greg-core/Cargo.toml` |

## Development Toolchain

- rustfmt + clippy via the Rust toolchain; CI runs `cargo fmt --all -- --check`
  and `cargo clippy --workspace --all-targets -- -D warnings`.
- `cargo audit` for RustSec advisories; CI gate in `.forgejo/workflows/rust.yml`.

## Commands

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p greg-app
cargo run -p greg-cli -- publish --path <dir> --changelog <text>
cargo xtask check
cargo xtask dist
```

## Configuration

`crates/greg-platform/src/prefs.rs` (JSON preferences, 0600) and
`crates/greg-platform/src/workspace.rs` (workspace metadata) provide local
settings. Native Steam libraries and `steam_appid.txt` are staged beside the
binaries by `cargo xtask dist`.

### Environment variables

| Variable | Purpose | Secret? |
| --- | --- | --- |
| `GREG_WORKSPACE` | override automatic workspace root | no |
| `GREG_MODSTORE_TOKEN` / `--token` | Modstore API token | secret |
| `GREG_MODSTORE_API_URL` | override Modstore API base URL | no, but use trusted HTTPS endpoints |
| `TELEMETRY_URL` | telemetry endpoint | telemetry credentials are secret; URLs are not |
| `IS_LOCAL_TEST_BUILD`, `MODSTORE_WEB_URL`, `MODSTORE_API_URL` | local development endpoints | no |

Never commit endpoint credentials, session tokens, certificate passwords, or
private keys.

## Evidence

- `Cargo.toml` (workspace)
- `crates/greg-app/Cargo.toml`
- `crates/greg-core/Cargo.toml`
- `.forgejo/workflows/rust.yml`
