# Audit Feature Inventory — Existing Functionality for the Universal Report Assistant

> Owner: gregModmanager maintainers. Evidence: source-tree audit (2026-10-28).
> Status: **Audit only** — no implementation.

## 1. How Features Were Inventoried

Static source inspection of:

- `crates/greg-app/src/` (actions, state, worker, main, shots) and `crates/greg-app/ui/*.slint`
- `crates/greg-core/src/` (domain models, checks, docs, upload, log, prefs, limits, l10n, telemetry models)
- `crates/greg-platform/src/` (filelog, telemetry, repro, workspace, prefs, paths, instance, protocol, process)
- `crates/greg-steam/src/` (service, backend, poller, rate_limit, inference, vdf, models)
- `crates/greg-modstore/src/` (auth client, client, collections, intent, publish, upload client, install)
- `crates/greg-loader/src/` (game, launch, health, installers, channels, sync, packs, local content, native config)
- `crates/greg-cli/src/` (headless publish/status/list)
- Root documentation (`docs/`, `CHANGELOG.md`, `current_features.md`, `functions.md`, `ModStoreAndManager-Roadmap.md`, `SECURITY.md`, `CONTRIBUTING.md`)

## 2. Feature Catalog

### 2.1 Desktop UI — `greg-app`

| # | Feature | User Entry | Technical Entry | Status |
|---|---|---|---|---|
| 1 | Game launch / stop | Header buttons `start-game`, `stop-game` | `actions.rs` `start_game` (222), `stop_game` (345), `poll_game_state`, `finalize_game_state`, `recover_parked_mods`, `check_melon_loader_on_start` | Complete |
| 2 | Game launch error handling | Status/log surfaces | `actions.rs` 222–305, `launch.rs` GameSession deadline (30 s) | Complete |
| 3 | Auth / login / logout / `greg://` deep link | Profile card, profile menu | `actions.rs` auth cluster (680–1010), protocol handler, `drain_handoff` | Complete |
| 4 | Workshop browse / subscribe / favorite / import / sync | Browse page tabs (workshop/subscribed/favorited) | `actions.rs` browse cluster (1860–2119), `packs_apply` (3241) | Complete |
| 5 | Editor + Workshop publish + Modstore publish | Editor page upload readiness + publish | `actions.rs` `update_doc_hints` (1363), `run_editor_checks` (1422), `editor_changed` (1458), `editor_save` (1476), `editor_publish` (1498), `editor_cancel_publish` (1585), `editor_publish_modstore` (1595), `drain_publish` (1738) | Complete |
| 6 | Modstore browse / updates / install | Store page | `actions.rs` store cluster (2119–2283) | Complete |
| 7 | Local content (My Mods / Plugins / Libs / Modpacks) | Local page | `actions.rs` local cluster (2423–2586) | Complete |
| 8 | Problems inbox | Nav `problems` | `actions.rs` `refresh_problems` (2688), `problems_act` (2878) | Complete |
| 9 | Bug report (link-out only) | Nav `report-bug` → chooser dialog | `actions.rs` 148–151, 154–160, 2627–2673; `ui/main.slint` dialog 778–830 | **Partial** (no local capture) |
| 10 | Settings | Settings page | `actions.rs` settings cluster (3300–3450) | Complete |
| 11 | Support bundle | Settings | `actions.rs` `settings_make_bundle` (3431) → `greg_platform::repro::create_bundle` | Complete |
| 12 | Health checks | Settings | `actions.rs` `settings_run_checks` (3391) → `greg_loader::health` | Complete |
| 13 | Logs viewer / open logs | Settings | `actions.rs` `settings_open_logs` (3420) | Complete |
| 14 | Window chrome (min/max/close) | Title bar | `actions.rs` 1033–1058 | Complete |

### 2.2 Domain — `greg-core`

| # | Capability | Evidence | Status |
|---|---|---|---|
| 1 | Preferences port + memory impl | `greg-core/src/prefs.rs` | Complete |
| 2 | Feature flags / endpoints / consent gate | `greg-core/src/models/settings.rs` | Complete (telemetry env-gated, default off) |
| 3 | Telemetry payloads (Loki) | `greg-core/src/models/telemetry.rs` | Partial (types only) |
| 4 | Upload checks & readiness | `greg-core/src/upload.rs`, `models/checks.rs` | Complete |
| 5 | Docs resolution (README/BBCode/changelog) | `greg-core/src/docs/*` | Complete |
| 6 | Limits | `greg-core/src/limits.rs` | Complete |
| 7 | Log buffer | `greg-core/src/log.rs` | Complete |
| 8 | Errors | `greg-core/src/error.rs` | Complete |
| 9 | L10n (EN/DE tables, 360 keys) | `greg-core/src/l10n.rs`, `l10n_strings.rs` | Partial (EN/DE only; UI drift) |

### 2.3 Infrastructure — `greg-platform`

| # | Capability | Evidence | Status |
|---|---|---|---|
| 1 | File log + crash marker | `filelog.rs` | Partial (capture unwired) |
| 2 | Telemetry Loki push | `telemetry.rs` | Partial (sender not wired) |
| 3 | Support bundle | `repro.rs` | Complete |
| 4 | Workspace metadata / publish hash | `workspace.rs` | Complete |
| 5 | Preferences JSON impl | `prefs.rs` | Complete |
| 6 | Paths / instance guard / protocol / process | `paths.rs`, `instance.rs`, `protocol.rs`, `process.rs` | Complete |

### 2.4 Steam — `greg-steam`

| # | Capability | Evidence | Status |
|---|---|---|---|
| 1 | Workshop publish (incl. duplicate adoption, rate limit) | `service.rs`, `rate_limit.rs` | Complete |
| 2 | Browse / search / subscribed / favorited / my published / item details / gallery / import | `service.rs` | Complete |
| 3 | Backend trait + sync orchestrator + poller | `backend.rs`, `poller.rs` | Complete |
| 4 | Dependency inference | `inference.rs` | Complete |

### 2.5 Modstore — `greg-modstore`

| # | Capability | Evidence | Status |
|---|---|---|---|
| 1 | Auth (OAuth callback flow, sessions) | `auth_client.rs`, `intent.rs` | Complete |
| 2 | Catalog / updates / collections | `client.rs`, `collections_client.rs` | Complete |
| 3 | Publish (plan/presign/PUT/submit) | `publish.rs`, `upload_client.rs` | Complete |
| 4 | Install / updates / intents | `install.rs`, `intent.rs` | Complete |

### 2.6 Game integration — `greg-loader`

| # | Capability | Evidence | Status |
|---|---|---|---|
| 1 | Game adapters, launch modes, mods state, park/restore | `game.rs`, `launch.rs` | Complete |
| 2 | Health/dependency checks, native config | `health.rs`, `native_config.rs` | Complete |
| 3 | MelonLoader install, installers | `installers.rs` | Complete |
| 4 | Sync / packs / local content / discovery / content | `sync.rs`, `packs.rs`, `local_content.rs` | Complete |

### 2.7 Cross-cutting

| Capability | Evidence | Status |
|---|---|---|
| Crash capture | `CrashReport`, `FileLog::mark_crash` | Partial (dormant) |
| Telemetry | Loki types, secrets, gate, sender | Partial (dormant sender) |
| Support bundle | `repro.rs`, settings action | Complete |
| Problems inbox | `refresh_problems`, `problems_act` | Complete |
| Update checks (Steam workshop sync states, Modstore, MelonLoader, SteamModfix) | editor/store/launcher surfaces | Complete |

## 3. Status Legend

- **Complete:** end-to-end implemented and wired.
- **Partial:** building blocks exist, wiring/user flow incomplete or dormant.
- **Planned:** roadmap only.

## 4. Report-Reuse Mapping (URA inputs)

| Existing capability | Best URA reuse | Ready-made symbols |
|---|---|---|
| Bug chooser + tracker deep links | Replace/augment with in-app capture | `actions.rs` 148–151, 2627–2673; `ui/main.slint` 778–830 |
| Crash report struct + filelog marker | Extend `CrashReport`, wire `mark_crash` | `greg-core/src/models/misc.rs` (CrashReport), `greg-platform/src/filelog.rs` (mark_crash) |
| Support bundle | Generalize into diagnostics collection | `greg-platform/src/repro.rs` (`create_bundle`), `actions.rs` 3431 |
| Upload readiness checks | "What did the user do / in what state" context | `greg-core/src/upload.rs` (`check`), `models/checks.rs` (`is_ready_to_upload`) |
| Publish flow + rate limiter | Crash/upload correlation, stuck-upload diagnostics | `greg-steam/src/service.rs` (publish/publish_inner), `rate_limit.rs` |
| Docs resolution (README/BBCode/changelog) | Report includes version/change notes | `greg-core/src/docs/*` |
| Problems inbox | Canonical problem feed | `actions.rs` 2688, 2878 |
| Telemetry skeletons | Opt-in diagnostic emission | `greg-platform/src/telemetry.rs`, `greg-core/src/models/telemetry.rs` |
| Log buffer + file log | Sanitized log attachment | `greg-core/src/log.rs`, `greg-platform/src/filelog.rs` |
| Modstore publish/install | Mod-related failure reports | `greg-modstore/src/*` |
| Game launch/health | Crash-report context (pid, mode, loader) | `greg-loader/src/launch.rs`, `health.rs` |

## 5. Explicitly Missing for a URA MVP

- User-facing crash capture (stack trace, mods-in-play, session context).
- User-facing report context collection (version, platform, logs, steps, state).
- Redaction/sanitization of logs before attachment.
- Structured issue intake / preview / confirmation / submission flow.
- Privacy notice, consent storage, telemetry wiring.
- Multi-language completion beyond EN/DE (supported list has 10 cultures, only EN/DE implemented).

## 6. Related Roadmap Items

- `ModStoreAndManager-Roadmap.md`: F-010 (prioritized support), F-032 (source/issue/support links), F-040 (bugtracker/structured issues), F-124 (crash detection & mod bisect), F-037/F-039 (comments/threads, feedback).
- `docs/01_END_USER_GUIDE.md` (§ around 68–105): support request contents and defect-reporting instructions.
- `CONTRIBUTING.md` (§ around 39–44): issue reporting expectations.
- `.github/ISSUE_TEMPLATE/`: `bug_report.md`, `bug_report_de.yml`, `crash_report.yml`, `feature_request.yml`, `installer_problem.yml`, `performance_issue.yml`, `question.yml`, `documentation_improvement.yml`, `config.yml`.
- `current_features.md` / `functions.md`: `/api/report` (POST content/mod report), `/api/admin/gregscan-reports*` endpoints.

## 7. Audit Conclusion

The codebase provides a solid, mostly reusable foundation for report automation.
The gap between "building blocks exist" and "user-facing report capability" is
concentrated in wiring, context collection, redaction, and workflow design —
not in domain logic.
