# External Integrations

> Owner: gregModmanager maintainers. Evidence: current integration code and
> configuration. Update after an endpoint, authentication, storage, or data-flow change.

## Inventory

| System | Type | Purpose | Auth | Criticality | Evidence |
|---|---|---|---|---|---|
| Steamworks | native SDK | Steam init and Workshop browse/download/upload | Steam client/session | high | `SteamWorkshopService.cs` |
| Modstore services | HTTPS API | desktop account/session bridge and configured service calls | bearer/session | high | `Services/Auth/` |
| GitHub releases | HTTPS API | MelonLoader/SteamModfix downloads | public API/User-Agent | medium | installer services |
| Local filesystem | local state/game files | workspace, projects, logs, deployment | OS permissions | high | `WorkspaceService`, `AppSettings` |

## Data Stores

| Store | Role | Access | Risk | Evidence |
|---|---|---|---|---|
| JSON preferences | app settings | `JsonFilePreferences` | partial write/schema drift | `JsonFilePreferences.cs` |
| JSON metadata | Workshop project state | `WorkspaceService` | schema drift | `WorkshopMetadata.cs` |
| Steam cache | downloaded UGC | Steamworks | external availability | `WorkshopDownloadService.cs` |

## Secrets

Passwords are not stored by the desktop app. The current session refresh value
is persisted as `greg_refresh_token` in the user's local preferences JSON so a
session can be restored. The current implementation does not encrypt that value;
file-system permissions are its protection. Logout removes the stored value.

App IDs and public URLs are configuration, not secrets. Tokens, PFX
paths/passwords, and telemetry credentials must never be written to diagnostic
logs or committed configuration.

## Reliability

Installer clients use finite HTTP timeouts. Cancellation and result records exist in download/install services, but no common retry policy or circuit breaker exists. Steam native loading tries multiple platform paths.

## Observability

`AppFileLog`, `AppLogService`, and telemetry cover startup and many external
calls. Telemetry is enabled by default and can be disabled in Settings. A
reproduction bundle may include logs, dumps, environment values such as local
paths/machine/user names, and recent Windows application events; inspect it
before sharing. Missing uniform fields include operation IDs, response status
details, and transaction IDs.

## Evidence

- `src/GregModmanager.Core/Services/Auth/SessionManager.cs`
- `src/GregModmanager.Core/Services/AppSettings.cs`
- `src/GregModmanager.Core/Services/AppFileLog.cs`
