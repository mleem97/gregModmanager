# Audit Open Questions — Universal Report Assistant

> Owner: gregModmanager maintainers. Evidence: source-tree audit (2026-10-28).
> Status: **Audit only** — open items awaiting decisions.

## 1. Open Technical Questions

1. **Crash capture wiring:** should `mark_crash` be wired to panic hook /
   session end / game exit paths, and what stack-trace sources are available
   (native FFI boundaries, loader events)?
2. **Report context schema:** which fields are mandatory (version, platform,
   OS, game/loader versions, mod/project, upload state)?
3. **Redaction policy:** which paths/usernames/secrets are redacted before
   bundle submission?
4. **Attachment handling:** allowed types/sizes for user-attached files
   (logs/screenshots) beyond existing upload limits?
5. **Internal report store:** use `/api/report` now or tracker deep links first?

## 2. Open Product/UX Questions

1. Primary assistant entry point: new nav item vs. Problems inbox vs.
   in-editor vs. settings.
2. Category list and follow-up question strategy (minimal mandatory fields).
3. Confirmation/submission flow (preview, edit, submit, cancel).
4. Success/failure feedback surfaces.

## 3. Open Design Questions

1. Design source of truth after Slint migration (`DESIGN.md` is Avalonia-era).
2. Accessibility support status in Slint 1.18.1.
3. Visual verification needs (headless screenshot harness available).

## 4. Open Security Questions

1. Telemetry consent storage (prefs key vs. env) and notice copy.
2. Keyring migration timing for OAuth tokens (known limit).
3. Log redaction coverage before support-bundle sharing.

## 5. Open Integration Questions

1. Should report context reuse upload-check results, and where stored?
2. Rate limits for report submission (GitHub/Modstore API).
3. Multi-language rollout (EN/DE implemented; others fallback).

## 6. Audit-Open Questions Requiring User Decision (Phase 2 Go/No-Go)

1. Proceed to Phase 2 implementation after design alignment?
2. Target tracker: GitLab (`git.datacentermods.com/teamGreg/gregModmanager`)
   deep links as implemented vs. GitHub issues.
3. Telemetry: opt-in now, or later?
4. Visual/design verification before implementation?

## 7. Unresolved External Facts

- Live behavior of Steam/Modstore APIs could not be probed (no live access in
  audit environment) — needs owner-observed testing.
- Actual rendered font fallback and layout metrics — needs visual check.

## 8. Audit Conclusion on Open Items

All open questions must be resolved or explicitly deferred before Phase 2
implementation begins; no open question authorizes implementation in this pass.
