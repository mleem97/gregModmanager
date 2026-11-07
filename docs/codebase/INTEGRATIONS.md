# External Integrations

> Owner: gregModmanager maintainers. Evidence: crate clients, protocol handling,
> configuration. Update after an endpoint, authentication, storage, or data-flow change.

## Inventory

| System | Type | Purpose | Auth | Criticality | Evidence |
| --- | --- | --- | --- | --- | --- |
| Steamworks | native SDK | Steam init and Workshop browse/download/upload | Steam client/session | high | `crates/greg-steam/` |
| Modstore services | HTTPS API | desktop account/session bridge and configured service calls | bearer/session | high | `crates/greg-modstore/` |
| GitHub releases | HTTPS API | release distribution references | public API/User-Agent | medium | release assets |
| Local filesystem | local state/game files | workspace, projects, logs, deployment | OS permissions | high | `crates/greg-platform/` |

## Data Stores

| Store | Role | Access | Risk | Evidence |
| --- | --- | --- | --- | --- |
| JSON preferences | app settings | `crates/greg-platform/src/prefs.rs` | atomic writes | prefs.rs |
| JSON metadata | Workshop project state | `crates/greg-platform/src/workspace.rs` | schema drift | workspace.rs |
| Steam cache | downloaded UGC | Steamworks | external availability | `crates/greg-steam/` |

## Secrets

Passwords are not stored by the desktop app. The current session refresh value is
persisted as `greg_refresh_token` in the user's local preferences JSON so that a
session can be restored; file-system permissions (0600 on Unix) protect it. Logout
removes the stored value.

Tokens, PFX paths/passwords, and telemetry credentials must never be written to
diagnostic logs or committed configuration.

## Reliability

Auth clients use finite HTTP timeouts; authentication failures such as 401/403 are
not retried against another host.

## Observability

`crates/greg-platform/src/filelog.rs` and `telemetry.rs` cover startup and many
external calls.

## Evidence

- `crates/greg-modstore/src/auth_client.rs`
- `crates/greg-platform/src/prefs.rs`
- `crates/greg-platform/src/telemetry.rs`
