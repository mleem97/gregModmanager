# Architecture

> Owner: gregModmanager maintainers. Evidence: workspace crate boundaries,
> `greg-core` rules, `greg-app` composition root.
> Update after a dependency-injection, protocol, service-boundary, or
> startup-flow change.

## Architectural Style

Layered desktop client: `greg-app` (composition root + presentation) → domain
crates (`greg-core`, `greg-steam`, `greg-modstore`, `greg-loader`) →
infrastructure (`greg-platform`). Primary constraints are Steam initialization
order, platform-specific native libraries, and local game-file safety.

## System Flow

```text
greg-app main -> AppState -> actions/state/worker -> domain services -> greg-platform -> Steam/files/HTTPS
```

Startup preloads the Steam backend and builds the state; actions drain jobs on a
UI-thread timer; background results arrive via channels. Persistence is JSON
(preferences, workspace metadata, publish hash).

## Responsibilities

| Module | Owns | Must not own |
| --- | --- | --- |
| `greg-app` | UI navigation, user actions, presentation | Domain rules |
| Domain crates | Workshop, Modstore, game/deploy logic | UI layout |
| `greg-platform` | Filesystem/log/telemetry/workspace/process/platform shims | Domain rules |
| `greg-loader` | Game detection, paths, capabilities, launch | generic execution |

## Patterns

| Pattern | Location | Reason |
| --- | --- | --- |
| State | `crates/greg-app/src/state.rs` | shared state behind mutex + timer |
| Job handling | `crates/greg-app/src/worker.rs` | background isolation, failed outcomes |
| Adapter registry | `greg-loader` | future games without UI rewrites |
| Errors | `thiserror` in domain, `anyhow` at shells | typed boundaries |

## Risks

- General transactional deployment/ownership is not implemented yet.

## Evidence

- `crates/greg-app/src/main.rs`
- `crates/greg-core/src/`
- `crates/greg-loader/src/launch.rs`
