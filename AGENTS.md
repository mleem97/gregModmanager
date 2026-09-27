# AGENTS.md — gregModmanager (Rust restart)

This file is the short entry point for agent-oriented repository instructions. Read the referenced companion files before changing code, builds, documentation, or pull requests.

## Important Note

- The C# / Avalonia implementation is frozen under `./archive/modmanager-old/` (buildable via `GregModmanager.sln`). Do not extend it; the active app is Rust.
- The active workspace plan (crates, dependencies, C# → Rust mapping) lives in `./RUST_GRAPH.md`. Consult it before adding crates or dependencies.
- All Linux/Unix relevant changes that cannot be mixed with the other OS go behind `cfg(target_os = "linux")` in `crates/greg-platform` (or a `#[cfg(unix)]` module).
- All Windows relevant changes that cannot be mixed with the other OS go behind `cfg(target_os = "windows")` in `crates/greg-platform`.
- All macOS related changes that cannot be mixed with the other OS go behind `cfg(target_os = "macos")` in `crates/greg-platform`.
- MelonLoader plugins (`SubDirectoryFixer`, `gregPlugin.ModmanagerCompanion`) stay .NET 6 — they load inside MelonLoader and are never rewritten in Rust. Frozen sources: `./archive/modmanager-old/src/GregModmanager.Melons/`. If the Rust app must build them, shell out to the `dotnet` CLI via `xtask`.
- There will be/is a Companion Plugin for Melonloader to integrate the Modmanager into the game. (Maybe with overlays? Plan with atomic tasks)
- Always use Orchestration of several Agents if possible to

## Companion files

- `SOUL.md` — repository personality, collaboration defaults, and communication style.
- `USER.md` — project context, architecture, runtime guardrails, and release expectations.
- `TOOLS.md` — build, CI, Steam Workshop, signing, telemetry, and troubleshooting guidance.
- `EXTERNAL_DEPENDENCIES.md` — external services, tools, native binaries, and third-party dependencies.

## Project context

- Application: cross-platform desktop mod manager for the gregFramework ecosystem.
- UI: `eframe`/`egui` (immediate mode, single binary); see `RUST_GRAPH.md` for the chosen stack and alternatives.
- Target language: latest stable Rust (rustup toolchain); MSRV is "latest stable" unless the maintainer pins one.
- Workspace root: `Cargo.toml` with members under `crates/*` plus `xtask/`.
- Executable crates: `crates/greg-app` (desktop UI), `crates/greg-cli` (headless `publish`/`status`/`list`).
- Domain crates: `crates/greg-core`, `crates/greg-steam`, `crates/greg-modstore`, `crates/greg-loader`, `crates/greg-platform`.

## Required workflow

- Work on a feature branch and open a pull request to `main`; do not push directly to `main` unless the maintainer explicitly requests it.
- Keep commits focused. Combine related changes only when splitting them would reduce reviewability or when the maintainer asks for a single commit.
- Use Conventional Commits for generated commit messages unless the maintainer explicitly requests another format.
- Update `CHANGELOG.md` under `[Unreleased]` for user-facing changes.
- Update `EXTERNAL_DEPENDENCIES.md` when adding, removing, or materially changing external packages, tools, services, or native binaries.

## Architecture guardrails

- `greg-core` must not depend on UI crates (`eframe`/`egui`). If a required fix appears to need that dependency, stop and ask for maintainer confirmation.
- `greg-app` / `greg-cli` may depend on domain crates; they stay thin (composition root + presentation).
- Wire services via constructor injection in the binary `main.rs` composition roots.
- Prefer the `dirs` crate, known paths, or validated user paths over hard-coded platform paths.
- Guard platform-specific code with `cfg(target_os = …)` in `greg-platform` unless the maintainer explicitly confirms a platform-only change.

## Steam Workshop guardrails

- Enforce the publish rate limiter before submitting an item update unless the maintainer explicitly confirms a controlled test path.
- Display cooldown timers in seconds when the UI exposes rate-limit state unless the UX owner confirms a different copy format.
- Keep the vendor Steam native library (`steam_api64` / `libsteam_api`) out of Authenticode signing loops unless the vendor changes the binary format and the maintainer confirms the change.

## Build and release guardrails

- Keep `xtask` commands and CI workflows aligned unless a PR intentionally stages a migration and explains the temporary divergence. `.forgejo/workflows/` (Forgejo Actions) and `.gitea/workflows/` (Gitea Actions) must stay byte-identical — the `parity` job fails the run on drift. GitHub is a read-only archive mirror — no CI lives in `.github/`.
- Preserve binary-size settings unless a measured, reviewed change requires otherwise.
- Breaking changes normally require a major SemVer bump; ask the maintainer before applying a different release policy.
- Manual release promotion is allowed only when the automated workflow is unavailable or the maintainer requests it.

## JSON and localization

- Derive `serde::Serialize` / `Deserialize` on DTOs; keep wire formats backward compatible unless the change is versioned and documented.
- Keep UI strings in the localization module with English plus German at minimum unless the maintainer explicitly limits localization scope.
- Repository documentation should be English unless the user or maintainer explicitly requests another language for the artifact.

## Final checks

- Run the most relevant check available: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
- If a check cannot be run, state that limitation in the PR or final response.
- Verify whether relevant wiki pages need updates; list follow-up pages when the wiki is not available in the working environment.
