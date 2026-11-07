# Audit Architecture — Current Structure Relevant to the Universal Report Assistant

> Owner: gregModmanager maintainers. Evidence: source-tree audit (2026-10-28).
> Status: **Audit only** — no implementation.

## 1. Architectural Style

- Layered desktop client with constructor-injected services (Rust flavor of DI:
  composition root in `crates/greg-app/src/main.rs` `wire()`).
- Event-driven Workshop/session updates; UI refresh via 150 ms timer tick.
- Primary constraints: Steam initialization order, platform native libraries,
  local game-file safety.

## 2. Layering and Boundaries

| Layer | Owns | Must not own | Evidence |
|---|---|---|---|
| Presentation | `greg-app`: `main.rs`, `state.rs`, `worker.rs`, `actions.rs`, `shots.rs`, `ui/*.slint` | domain rules | `crates/greg-app/src/main.rs` |
| Domain | `greg-core` (pure logic), plus feature services `greg-steam`, `greg-modstore`, `greg-loader` | UI | crate manifests |
| Infrastructure | `greg-platform` (FS/log/telemetry/workspace/paths/protocol/process) | UI | `crates/greg-platform/src/` |
| Headless | `greg-cli` | UI | `crates/greg-cli/src/main.rs` |

Dependency rule respected: no domain crate depends on UI crates
(`greg-core` has no slint/tokio/reqwest/steam deps).

## 3. UI Threading and State Model

- State container: `Arc<Mutex<AppState>>` (`crates/greg-app/src/state.rs`).
- UI handle access through `AppState::ui(&self) -> MainWindow` (upgrade from
  `slint::Weak`).
- Timer: `slint::Timer` 150 ms → `actions::tick` drains events/jobs/timers.
- Events: `UiEvent::{BackendReady, Probe, AuthSessionEstablished, AuthSessionFailed}`
  over `std::sync::mpsc`, drained in `tick`.
- Mutations from UI callbacks: actions lock state, mutate, push via
  `state.ui().set_*` setters; background results via same setters from drains.
- Known pattern (post upload-freeze fix): never hold the `state` mutex while
  calling functions that lock it again (`append_log` locks state itself).

## 4. Background Execution Model

| Mechanism | Used for | Notes |
|---|---|---|
| `JobHandle::spawn` + `JobSink` | Editor publish, sync hashing, thumbnails | std thread, `catch_unwind` → failed outcome ("Internal error, please report: …"), cancel flag, Drop cancels+joins |
| `JobHandle::drain` / `is_finished` | `tick`, `drain_publish`, `drain_sync`, `drain_thumb` | |
| Tokio runtime (multi-thread) | Async Modstore/auth flows | runtime handle spawned tasks send `UiEvent` back |
| `rt.block_on` | Some Modstore calls | from action contexts |
| Steam poller thread | Workshop subscription sync | |
| Background Steam probe thread | Steam availability/connect | `state.rs` `spawn_probe` |

### Blocking-on-UI-thread audit (relevant to URA design)

- `run_editor_checks` historically ran synchronously on UI thread (hashing all
  content bytes, converting all READMEs to BBCode) — removed from publish drain
  after upload-freeze fix; checks remain available via `editor_changed`.
- `local_refresh` holds state lock and runs `greg_loader::local_content::scan`
  (fs) directly — invoked from UI callbacks and `tick`.
- `probe_gregapi` uses `reqwest::blocking` — only called from background thread.

## 5. Error Handling, Logging, Telemetry, Persistence

- Errors: `CoreError` (Io/Json/Version/Changelog/NotFound/Other), `PlatformError`
  (Io/Json/Project/Network/Other, `From<CoreError>`); UI via job outcomes and
  `UiEvent`s.
- Logging: `tracing` (env-filter, default info) + `FileLog` (file + console,
  best-effort writes, session markers, `mark_crash`); in-memory `LogBuffer`
  (capacity 500).
- Telemetry: opt-in Loki push skeleton; secrets via env
  (`GREG_LOKI_URL/USER/PASS/TENANT`, local override `GREG_LOCAL_BUILD` →
  `localhost:3100/loki`); gate env `GREG_TELEMETRY_ENABLED` (default off);
  payloads: startup, sync collection; **sender not wired yet**.
- Persistence: JSON preferences (0600 unix), workspace metadata JSON, SHA-256
  publish hash.

## 6. Report-Assistant Integration Seams (as found)

1. Bug-report deep link actions (`bug_open_manager`, `bug_open_org`,
   `bug_close`, `percent_encode_bug`) — reusable URL-building pattern.
2. `CrashReport` + `FileLog::mark_crash` + session markers — capture backend.
3. `repro::create_bundle` + `settings_make_bundle` — diagnostics bundle.
4. `upload::check` / `is_ready_to_upload` — state/context checks.
5. `WorkshopService::publish*` + rate limiter — failure context.
6. Docs resolution (`resolve_steam_description`, `resolve_steam_changelog`) —
   version/change context.
7. Problems inbox (`refresh_problems`, `problems_act`) — user-visible problem feed.
8. Telemetry skeletons, log buffer/file log.
9. Modstore publish/install, game launch/health.

## 7. Architecture Risks / Observations

- `Arc<Mutex<AppState>>` on UI thread: correct lock discipline required for any
  new code touching logs/state.
- Worker + tokio + threads: no single unified background task scheduler;
  coordination via timer tick and job handles.
- Error taxonomy differs from likely future report-error categories
  (crash vs. user error vs. network vs. auth vs. rate limit).

## 8. Audit Conclusion

Architecture supports incremental URA integration through existing seams; no
fundamental architectural change is required before Phase 2.

## 9. Out of Scope for This Audit

- Implementation of the URA.
- Live Steam/Modstore/GitLab/GitHub probes.
- Third-party library security assessment beyond repository usage.

## 10. Open Architecture Questions

- Where report context collection should live (greg-core domain vs. greg-app).
- Whether report creation should be synchronous (UI) or job-based.
- Consent storage model for telemetry/privacy notice.
