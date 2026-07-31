# Codebase Concerns

> Owner: gregModmanager maintainers. Evidence: current source and build config.
> Update after a risk is resolved, introduced, or changes severity.

## Top Risks

| Severity | Concern | Evidence | Impact | Action |
|---|---|---|---|---|
| High | No general transaction/ownership store for deployment | `ModsFolderSyncService.cs` | mixed state or user-file deletion risk | implement P0 deployment/ownership |
| High | Steam Workshop Mod Store exists only outside Mod Manager-only mode and is not a generic web catalog | `MainWindow.axaml.cs`, `ModManagerPage.axaml.cs` | users may misunderstand available discovery workflows | document the mode-dependent Steam scope precisely |
| Medium | Adapter profiles and launch remain explicitly unsupported; local/workshop installation plans are supported | `DataCenterGameAdapter.cs` | callers may assume unsupported game-control features | keep capability checks at call sites |
| Medium | macOS has no release pipeline, signing, or notarization | build and CI configuration | cross-publish output is not distributable | add a real-Mac release design before claiming support |

## Technical Debt

| Item | Where | Risk | Fix |
|---|---|---|---|
| Direct Data-Center fallback paths | `SteamApiNativeLoader.cs`, `ModDependencyService.cs` | future adapters diverge | move remaining resolution behind adapter |
| Mutable JSON persistence | `JsonFilePreferences.cs`, `ModCollectionService.cs` | partial writes/schema drift | atomic writes and migrations |

## Security

| Risk | Evidence | Mitigation | Gap |
|---|---|---|---|
| Archive/path traversal | installer/archive services | safe ZIP extraction exists | common package validator and symlink policy |
| Token/log exposure | auth/logging services | redaction helpers | audit all diagnostic payloads |
| Recursive secret upload | Workshop/editor paths | partial preflight | sensitive-file review and confirmation |

## Performance

- Workshop sync copies complete trees in `ModsFolderSyncService`; large collections need cache/deduplication.
- Startup performs runtime checks and network-capable setup from `MainWindow`; offline status should become a non-blocking surface.

## Fragile Areas

- `MainWindow.axaml.cs`: startup, auth, protocol and runtime setup converge in one file; extract an orchestrator.
- `EditorPage.axaml.cs`: large UI/business integration surface; move upload planning to Core.

## Evidence

- `src/GregModmanager.Core/Services/ModsFolderSyncService.cs`
- `src/GregModmanager.Core/Services/GameAdapters/DataCenterGameAdapter.cs`
