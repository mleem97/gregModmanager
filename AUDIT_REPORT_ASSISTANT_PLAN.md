# Audit — Universal Report Assistant Plan

> Owner: gregModmanager maintainers. Evidence: source-tree audit (2026-10-28).
> Status: **Audit only — plan. Implementation approved only after user sign-off.**

## 1. Goal

Enable users to create structured reports for the most common problem categories
with minimal effort, and route them into the existing support/issue workflow.

## 2. Intended User Flow (target)

1. User opens Report Assistant (candidate entry points: new nav item, Problems
   inbox entry, Settings entry, in-editor "Report" affordance).
2. User selects category (bug / crash / feature / improvement / compatibility /
   performance / UI-UX / docs / install / config / security / other).
3. User describes the issue in free text (minimal mandatory fields, smart
   follow-ups only when needed).
4. Assistant collects relevant context (app version, platform, OS, game/loader
   state, mod/project involved, upload state, logs, steps).
5. User attaches files (logs, screenshots) within validated limits.
6. Assistant generates structured report preview (title + body, sanitized).
7. User reviews/adjusts and submits (tracker deep link / support bundle / internal
   report store).
8. User receives confirmation + reference.

## 3. UX Concept

- Entry: keep existing `report-bug` chooser pattern as fallback; add assistant
  flow.
- Progressive disclosure: category → description → context → attachments →
  preview → submit.
- Reuse existing dialogs/modals patterns (inline Slint dialogs per current
  implementation).
- Feedback states: collecting → ready → submitted/failed, with job-based
  background where expensive (log collection).

## 4. Design Concept

- Must follow implemented Slint design system (`Palette`, `GregButton`,
  `ListCard`, inline dialogs) — see `AUDIT_DESIGN_SYSTEM.md` for deviations.
- Off-color flat design goals (no pure black/white as regular colors, flat
  surfaces, muted accents) — reconcile with implemented neumorphic touches.
- Dialogs: inline conditional rectangles per current Slint pattern.

## 5. Technical Integration Plan

### 5.1 Reusable components (no rebuild required)

| Need | Reuse | Location |
|---|---|---|
| Logging | `LogBuffer`, `FileLog`, tracing | `greg-core/src/log.rs`, `greg-platform/src/filelog.rs` |
| Crash capture | `CrashReport`, session markers, `mark_crash` | `greg-core/src/models/misc.rs`, `greg-platform/src/filelog.rs` |
| Diagnostics bundle | `create_bundle` | `greg-platform/src/repro.rs` |
| Context checks | `upload::check`, `is_ready_to_upload` | `greg-core/src/upload.rs`, `models/checks.rs` |
| Publish failure context | `WorkshopService::publish*`, rate limiter | `greg-steam/src/service.rs`, `rate_limit.rs` |
| Docs context | `resolve_*_description/changelog` | `greg-core/src/docs/*` |
| Problems feed | `refresh_problems`, `problems_act` | `greg-app/src/actions.rs` |
| Bug deep links | URL building + percent-encoding | `greg-app/src/actions.rs` 148–151, 2627–2673 |
| Telemetry | types/secrets/gate/push skeleton | `greg-platform/src/telemetry.rs`, `greg-core/src/models/telemetry.rs` |
| Upload job machinery | `JobHandle`, progress/log/status | `greg-app/src/worker.rs`, `actions.rs` publish |
| Attachment validation | size limits (`MAX_MOD_BYTES` etc.) | `greg-core/src/limits.rs` |

### 5.2 New components needed (Phase 2+)

- Report state model (category, description, context snapshot, attachments,
  sanitized text, status).
- Collector (log tail, crash marker, environment snapshot, mod context).
- Redaction layer (tokens, paths, secrets) — must run before persistence/submit.
- Preview + submit adapters (tracker deep link, support bundle, internal store).
- Consent/privacy handling.

### 5.3 Suggested phase breakdown

- **Phase 1 – Report context & capture primitives:** wire/extend crash capture,
  log tail selection, context snapshot types in `greg-core`.
- **Phase 2 – Report data model + redaction:** new domain types, sanitizers.
- **Phase 3 – Assistant workflow state machine:** category → description →
  context → attachments → preview → submit in `greg-app`.
- **Phase 4 – Collection & diagnostics:** bundle integration, job-based log
  collection, support bundle hand-off.
- **Phase 5 – Submission adapters:** tracker deep links, internal report API
  (`/api/report`), admin ingestion.
- **Phase 6 – Telemetry/consent:** wire opt-in telemetry, privacy notice.
- **Phase 7 – UI pages/components:** new Slint pages/modals following design
  system, l10n EN/DE.
- **Phase 8 – Tests:** unit tests for redaction/context; screenshot tests for
  new pages via `crates/greg-app/src/shots.rs` pattern.
- **Phase 9 – Docs/release:** user guide, changelog, issue templates alignment.

### 5.4 Architecture fit

- Domain additions in `greg-core`; infra in `greg-platform`; UI/workflow in
  `greg-app`; submission adapters in `greg-modstore` or dedicated client.

### 5.5 Error handling & UX requirements

- Failures must surface via job outcomes / status strings, never silent hangs.
- Cancel flags on long operations.
- Panic containment via `JobHandle` (already present).

## 6. Security Concept (URA-relevant)

- Never persist or transmit raw tokens/session secrets; redaction before logs
  or bundles leave the machine.
- Consent before any diagnostic upload; telemetry default off.
- Fail-closed attachment handling (type/size validation, no path traversal).
- Privacy notice + data minimization (only relevant context).
- Security review per repo `SECURITY.md`; no live probing.

## 7. GitHub Integration Strategy (target)

- Structured issue body (Markdown), sanitized, with version/platform context.
- Deep links already scaffolded (`BUG_REPORT_URL`, percent-encoded titles).
- Templates alignment: `bug_report.md`, `crash_report.yml`, `performance_issue.yml`,
  etc. (only existing starting point).

## 8. Test Strategy

- Unit: redaction, context resolution, checks.
- Integration: bundle creation, publish context.
- UI: screenshot harness extension, l10n key tests (360-key convention).
- E2E: none exist (documented gap).

## 9. Risks

| Risk | Detail | Mitigation |
|---|---|---|
| Stale design docs | DESIGN.md Avalonia-era | Phase ordering: design alignment first |
| Dormant infrastructure | capture/telemetry not wired | Phase 1/6 wiring |
| Privacy/consent complexity | new settings surface | reuse prefs pattern |
| Multi-language scope | EN/DE only implemented | start EN/DE |
| Steam-dependent crash context | needs Steam session state | use existing backend events |

## 10. Acceptance Criteria (audit-level)

- No implementation performed yet; criteria defined for Phase 2 kickoff.
- All six audit artifacts mutually consistent (checked in final report).

## 11. Open Decisions Required Before Phase 2

See `AUDIT_OPEN_QUESTIONS.md`.
