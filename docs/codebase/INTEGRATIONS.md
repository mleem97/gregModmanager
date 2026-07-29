# External Integrations

## Inventory

| System | Type | Purpose | Auth | Criticality | Evidence |
|---|---|---|---|---|---|
| Steamworks | native SDK | Steam init and Workshop browse/download/upload | Steam client/session | high | `SteamWorkshopService.cs` |
| Modstore Webapp | HTTPS API | auth bridge and future catalog/creator integration | bearer/session | high | `api-compatibility.md`, `Services/Auth/` |
| GitHub releases | HTTPS API | MelonLoader/SteamModfix downloads | public API/User-Agent | medium | installer services |
| Local filesystem | local state/game files | workspace, projects, logs, deployment | OS permissions | high | `WorkspaceService`, `AppSettings` |

## Data Stores

| Store | Role | Access | Risk | Evidence |
|---|---|---|---|---|
| JSON preferences | app settings | `JsonFilePreferences` | partial write/schema drift | `JsonFilePreferences.cs` |
| JSON metadata | Workshop project state | `WorkspaceService` | schema drift | `WorkshopMetadata.cs` |
| Steam cache | downloaded UGC | Steamworks | external availability | `WorkshopDownloadService.cs` |

## Secrets

Session tokens are obtained through the Better-Auth desktop bridge; passwords are not stored by the desktop app. App IDs/public URLs are configuration, not secrets. Token lifecycle follows the webapp bridge; broader API-key lifecycle is `[TODO]`.

## Reliability

Installer clients use finite HTTP timeouts. Cancellation and result records exist in download/install services, but no common retry policy or circuit breaker exists. Steam native loading tries multiple platform paths.

## Observability

`AppFileLog`, `AppLogService` and telemetry cover startup and many external calls. Missing uniform fields include operation IDs, response status details and transaction IDs.

## Evidence

- `src/GregModmanager.Core/Services/Auth/SessionManager.cs`
- `src/GregModmanager.Core/Services/AppSettings.cs`
- `src/GregModmanager.Core/Services/AppFileLog.cs`
