# AGENTS.md — gregModmanager

This file is the short entry point for agent-oriented repository instructions. Read the referenced companion files before changing code, builds, documentation, or pull requests.

## Important Note

- All Linux/Unix relevant changes that cannot be mixed with the other OS need to go in "./src/GregModmanager.Unix"
- All Windows relevant changes that cannot be mixed with the other OS need to go in "./src/GregModmanager.Windows"
- All MacOs Related changes that cannot be mixed with the other os need to go in "./src/GregModmanager.MacOs"
- There will be/is a Companion Plugin for Melonloader to integrate the Modmanager into the game. (Maybe with overlays? Plan with atomic tasks) - Located here: "./src/GregModmanager.Melons/gregPlugin.ModmanagerCompanion"
- Always use Orchestration of several Agents if possible to 

## Companion files

- `SOUL.md` — repository personality, collaboration defaults, and communication style.
- `USER.md` — project context, architecture, runtime guardrails, and release expectations.
- `TOOLS.md` — build, CI, Steam Workshop, signing, telemetry, and troubleshooting guidance.
- `EXTERNAL_DEPENDENCIES.md` — external services, tools, native binaries, and third-party dependencies.

## Project context

- Application: cross-platform desktop mod manager for the gregFramework ecosystem.
- UI: Latest Avalonia if possible.
- Target framework: Latest Stable .net for the desktop app; runtime-facing helper projects stay compatible with .NET 6 unless explicitly requested and validated, but only for Melonloader Integrations.
- Solution: `GregModmanager.sln`.
- Executable project: `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`.
- Shared library: `src/GregModmanager.Core/GregModmanager.Core.csproj`.

## Required workflow

- Work on a feature branch and open a pull request to `main`; do not push directly to `main` unless the maintainer explicitly requests it.
- Keep commits focused. Combine related changes only when splitting them would reduce reviewability or when the maintainer asks for a single commit.
- Use Conventional Commits for generated commit messages unless the maintainer explicitly requests another format.
- Update `CHANGELOG.md` under `[Unreleased]` for user-facing changes.
- Update `EXTERNAL_DEPENDENCIES.md` when adding, removing, or materially changing external packages, tools, services, or native binaries.

## Architecture guardrails

- Core should not reference Avalonia. If a required fix appears to need that dependency, stop and ask for maintainer confirmation.
- Avalonia may depend on Core through `<ProjectReference>`.
- Use dependency injection in `Program.cs` for services.
- Prefer `Path.Combine`, `Environment.SpecialFolder`, known paths, or validated user paths over hard-coded platform paths.
- Guard platform-specific code with compile-time or runtime platform checks unless the maintainer explicitly confirms a platform-only change.

## Steam Workshop guardrails

- Check `SteamPublishRateLimiter.Shared.TryAcquire(out retryAfter)` before `SubmitAsync()` unless the maintainer explicitly confirms a controlled test path.
- Display cooldown timers in seconds when the UI exposes rate-limit state unless the UX owner confirms a different copy format.
- Keep `steam_api64.dll` out of Authenticode signing loops unless the vendor changes the binary format and the maintainer confirms the change.

## Build and release guardrails

- Keep PowerShell build scripts and GitHub Actions workflows aligned unless a PR intentionally stages a migration and explains the temporary divergence.
- Preserve publish-size settings unless a measured, reviewed change requires otherwise.
- Breaking changes normally require a major SemVer bump; ask the maintainer before applying a different release policy.
- Manual release promotion is allowed only when the automated workflow is unavailable or the maintainer requests it.

## JSON and localization

- Register serialized DTOs in `src/GregModmanager.Core/Models/AppJsonContext.cs` unless the type is intentionally excluded and documented.
- Add UI strings to `AppStrings.resx` and `AppStrings.de.resx` at minimum unless the maintainer explicitly limits localization scope.
- Repository documentation should be English unless the user or maintainer explicitly requests another language for the artifact.

## Final checks

- Run the most relevant build, test, or static check available in the environment.
- If a check cannot be run, state that limitation in the PR or final response.
- Verify whether relevant wiki pages need updates; list follow-up pages when the wiki is not available in the working environment.
