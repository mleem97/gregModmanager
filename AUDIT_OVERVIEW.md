# Audit Overview — Universal Report Assistant Integration Basis

> Owner: gregModmanager maintainers. Evidence: source-tree audit (2026-10-28).
> Status: **Audit only.** No implementation of the Universal Report Assistant
> happened during this pass. Phase 2 (implementation) requires explicit approval.

## 1. Purpose and Scope

This audit investigates the existing repository to build a reliable technical
and design foundation for a future **Universal Report Assistant** (URA).

The URA shall let users create structured reports with minimal effort for:

- Bug reports, crash reports, feature requests, improvement suggestions,
- Compatibility problems, performance problems, UI/UX problems,
- Documentation errors, installation/configuration problems, other reports.

The assistant shall turn a simple user description into a structured report,
collect relevant additional information, process files safely, and prepare (or
later: create) a correctly formatted GitHub issue.

**Explicitly out of scope for this audit:** implementation. The URA was not
built, wired into the UI, or shipped in this pass.

## 2. Method and Boundaries

- Method: source-code-first audit (static inspection), supplemented by the
  maintainer's own documentation.
- Inspection mode: read-only. No builds, tests, or code generation were run as
  part of this audit; no network requests, probes, or live-system tests were
  performed.
- Evidence: current working-tree sources under `crates/`, `crates/greg-app/ui/`,
  `docs/`, root-level product documentation, CI/build configuration.
- Limits: no visual verification (no screenshots/rendered output) was possible in
  this environment; design findings are code-based and marked as such where
  visual confirmation is required.

## 3. Repository Snapshot

| Area | State | Evidence |
|---|---|---|
| Rust workspace | Members: `greg-core`, `greg-platform`, `greg-steam`, `greg-modstore`, `greg-loader`, `greg-app`, `greg-cli`, `xtask` | `Cargo.toml` (1–56) |
| Version | `1.7.5` | `Cargo.toml` `[workspace.package]` (15) |
| UI framework | Slint `1.18.1` + `lucide-slint 1.49.1`, custom frameless chrome | `Cargo.toml` (31–34), `crates/greg-app/ui/` |
| C# implementation | Frozen under `./.reference/modmanager-old/`, not extended | AGENTS.md |
| Domain logic | Pure Rust in `greg-core` (no UI/Steam/network deps) | `crates/greg-core/Cargo.toml` |
| Infrastructure | `greg-platform` (FS/log/telemetry/workspace ports) | `crates/greg-platform/src/` |
| Steam backend | `greg-steam` (trait + WorkshopService + poller + rate limit) | `crates/greg-steam/src/` |
| Modstore | `greg-modstore` (REST + publish/install clients) | `crates/greg-modstore/src/` |
| Game integration | `greg-loader` (launch/install/health/sync) | `crates/greg-loader/src/` |
| Desktop UI | `greg-app` (binary-only: `main.rs`, `state.rs`, `worker.rs`, `actions.rs`, `shots.rs`, `ui/*.slint`) | `crates/greg-app/` |
| Headless CLI | `greg-cli` (publish/status/list) | `crates/greg-cli/src/main.rs` |

## 4. Architecture at a Glance

- Layered desktop client: `greg-app` (composition root + presentation) →
  domain crates (`greg-core`, `greg-steam`, `greg-modstore`, `greg-loader`) →
  infrastructure (`greg-platform`). Rules are documented in `AGENTS.md` and
  `RUST_GRAPH.md`; `greg-core` must not depend on UI crates.
- UI state: `Arc<Mutex<AppState>>` behind a 150 ms UI-thread timer
  (`actions::tick`), background results via `UiEvent` mpsc + job drains.
- Background execution: `JobHandle` (std thread + `catch_unwind` → failed
  outcome), tokio runtime for async Modstore/auth, Steam poller thread,
  background Steam probe thread.
- Error surfacing: typed errors (`CoreError`, `PlatformError`) → job outcomes →
  UI status/log; auth failures via `UiEvent::AuthSessionFailed`.
- Persistence: JSON preferences (`preferences.json`, 0600), JSON workspace
  metadata, SHA-256 publish hash (`compute_publish_hash`).

Full detail: `AUDIT_ARCHITECTURE.md`.

## 5. Security Posture (summary)

- Threat model (`SECURITY.md`): desktop mod manager that installs foreign
  content and talks to Steam/Modstore APIs with user credentials; executes no
  downloaded code itself.
- Mapped controls (OWASP SAMM v2): governance, design, implementation, defect
  management, verification, operations, incident management.
- Known limits: OAuth access token persisted in preferences JSON (0600, not keyring
  yet); unsafe FFI usage in Steam backend (vendor `steamworks-sys`, deadline-polled
  completions); `cargo audit` advisory gate; license compliance manual.
- Secrets must never enter logs or diagnostic payloads.

During this audit, changed source paths under review were:

- `crates/greg-app/src/actions.rs` (upload freeze fix: lock handling around
  `save_metadata` / `append_log` / editor checks / publish spawn),
- `crates/greg-steam/src/service.rs` (`find_own_by_title` query scope).

These changes were source-inspected as context; a dedicated security assessment
of the diff is part of the final security review before any commit.

## 6. Key Cross-Cutting Findings Relevant to the URA

1. **Stale internal documentation:** `docs/codebase/{STRUCTURE,STACK,CONVENTIONS,
   TESTING,ARCHITECTURE,INTEGRATIONS}.md` still describe the frozen C# /
   Avalonia implementation. Rust equivalents drifted without updating them.
2. **Design source drift:** `DESIGN.md` is written against Avalonia conventions
   (`AppStrings.resx`, `IDialogService`, `MessageBox`, `ThemeVariant`,
   `.axaml` styles, `ReflectionBindingExtension`) and does not match the
   Slint implementation (global `Palette`, inline dialogs, std widgets,
   hardcoded strings). See `AUDIT_DESIGN_SYSTEM.md`.
3. **Dormant report infrastructure:** crash capture (`FileLog::mark_crash`,
   `CrashReport`), telemetry push (`push_loki`), and parts of the bug-report flow
   exist but are not wired into user-facing flows yet.
4. **Background execution seams are ready:** `JobHandle`, `UiEvent`, log buffer,
   support bundle and upload checks provide reusable building blocks.

## 7. Deliverables of This Audit

| Artifact | Content |
|---|---|
| `AUDIT_OVERVIEW.md` | This document — scope, method, snapshot, security posture |
| `AUDIT_FEATURE_INVENTORY.md` | Complete feature catalog and report-reuse map |
| `AUDIT_DESIGN_SYSTEM.md` | Design/token/a11y/localization audit |
| `AUDIT_ARCHITECTURE.md` | Architecture, threading, error/log/telemetry/persistence flows |
| `AUDIT_REPORT_ASSISTANT_PLAN.md` | Integration plan and implementation phases |
| `AUDIT_OPEN_QUESTIONS.md` | Open questions and unresolved decisions |

Final report (audit phase 14): see the closing summary of this audit session.

## 8. Immediate Audit Conclusion

The repository already contains the majority of the technical building blocks a
Universal Report Assistant needs (logging, crash marker, support bundle, upload
checks, publish flow, telemetry types, problems inbox, bug-report deep links).
The main integration work lies in: connecting the dormant pieces, defining report
context schemas, redaction/privacy handling, and UI workflow design — not in
rebuilding infrastructure.

## 9. Open Items Before Implementation

- Visual/design verification status of the Slint UI (screenshots pending).
- Resolution state of the outstanding upload performance work.
- Approval for Phase 2 (implementation of the URA).
