# Codebase Concerns

> Owner: gregModmanager maintainers. Evidence: current source and build config.
> Update after a risk is resolved, introduced, or changes severity.

## Top Risks

| Severity | Concern | Evidence | Impact | Action |
| --- | --- | --- | --- | --- |
| High | No general transactional deployment/ownership store | `crates/greg-loader/src/` | mixed state or user-file deletion risk | implement P0 deployment/ownership |
| Medium | macOS has no release pipeline, signing, or notarization | `.forgejo/workflows/`, `xtask` | cross-publish output is not distributable | add a real-Mac release design before claiming support |

## Technical Debt

| Item | Where | Risk | Fix |
| --- | --- | --- | --- |
| Mutable JSON persistence | `crates/greg-platform/src/prefs.rs`, `workspace.rs` | partial writes/schema drift | atomic writes and migrations |

## Security

| Risk | Evidence | Mitigation | Gap |
| --- | --- | --- | --- |
| Archive/path traversal | install/archive services | safe copy primitives | common package validator and symlink policy |

## Performance

- Workshop sync copies complete trees; large collections need cache/deduplication.

## Fragile Areas

- `crates/greg-app/src/actions.rs`: startup, auth, protocol and runtime setup
  converge in one module; extract an orchestrator if it grows.

## Evidence

- `crates/greg-loader/src/sync.rs`
- `crates/greg-platform/src/workspace.rs`
