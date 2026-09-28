# Security

Threat model in one sentence: a desktop mod manager that downloads and
installs foreign content (Workshop items, Modstore releases, plugin DLLs),
talks to Steam/modstore APIs with user credentials, and executes no
downloaded code itself (MelonLoader plugins stay .NET and load in-game).

Report suspected vulnerabilities through the bug tracker
(`Report a Bug` in-app or `teamGreg/gregModmanager` issues) — do not open
a public issue with exploit details before a fix exists.

## Practice mapping (OWASP SAMM v2)

Control names follow standard catalogues (OWASP ASVS, NIST) so they resolve
in the [OpenCRE](https://opencre.org/) explorer.

| SAMM practice | What this repo does | Evidence |
| --- | --- | --- |
| Governance — Policy & Compliance | Agent guardrails forbid UI deps in `greg-core`, Conventional Commits, `CHANGELOG.md` discipline | `AGENTS.md`, `RUST_GRAPH.md` |
| Governance — Strategy & Metrics | Dual CI must stay byte-identical (`parity` job); Rust CI gates fmt/clippy/test/build | `.forgejo/`, `.gitea/` |
| Design — Threat Assessment | Protocol handler accepts only the `greg://` scheme; install intents fail closed without a verifier; OAuth binds `requestId` and validates the callback | `actions.rs` (`parse_protocol_url`, `forward_to_primary`), `intent.rs` (`RejectAll`) |
| Design — Secure Architecture | Secrets never in code (telemetry placeholders only); tokens live in an owner-only prefs file, never in logs; background jobs can't touch the UI directly (150 ms drain) | `prefs.rs` (0600), `worker.rs`, `state.rs` |
| Implementation — Secure Build | `cargo audit` gates CI (advisories fail, unmaintained warns); `clippy -- -D warnings`; pinned lockfile (`Cargo.lock`) | `rust.yml`, `Cargo.lock` |
| Implementation — Secure Deployment | Release assets hashed (SHA-256 sidecars); Windows Authenticode best-effort; Linux packages install to `/opt` with a desktop entry; no macOS asset is faked (no runner) | `build-and-release.yml`, `xtask dist` |
| Implementation — Defect Management | Upload readiness checks block bad publishes (title, preview, size, changelog); worker panics surface as failed outcomes, never silent hangs | `upload.rs`, `worker.rs` |
| Verification — Security Testing | Symlink-jail test for recursive copy, prefs-permission test, protocol-parser tests, gallery validation tests | `workspace.rs`, `prefs.rs`, `actions.rs`, `service.rs` tests |
| Verification — Requirements-driven Testing | Headless screenshot suite pins every page layout; 80+ unit tests across crates | `shots.rs`, `cargo test --workspace` |
| Operations — Environment Management | Single-instance guard + handoff file (0600) for protocol URLs; file-logged publish lifecycle for stuck-upload diagnosis | `instance.rs`, `actions.rs` |
| Operations — Incident Management | Stuck background jobs time out with user-visible errors; cancel flags on every loop | `backend.rs` (deadlines), `worker.rs` |

## Known limits (honest)

- The OAuth access token is stored in `preferences.json` (0600 on Unix),
  not in the OS keyring — acceptable for a single-user desktop app, flagged
  for a future `keyring` migration.
- Gallery Steam calls use `unsafe` FFI (`steamworks-sys`) because the safe
  `steamworks` 0.13.1 API has no wrappers. Reviewed rules: bindgen layouts
  only, no manual vtables, completion polled with deadlines, handles always
  released, worst case is a timeout error. See `backend.rs` (`upload_gallery`,
  `additional_preview_urls`).
- `cargo audit` tracks RustSec advisories; license compliance is manual
  (no `cargo-deny` gate yet).
- The frozen C# tree under `.reference/` is excluded from all of the above.
