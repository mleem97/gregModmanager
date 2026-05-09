# AGENTS.md — gregModmanager

This file contains agent-oriented instructions for the `gregModmanager` repository.
Read this before modifying code, building, or creating pull requests.

---

## 1. Project Context

- **Application**: Cross-platform desktop Mod Manager for the gregFramework ecosystem.
- **UI Framework**: Avalonia UI 11.2 (replaces legacy MAUI).
- **Target Framework**: .NET 9 (`net9.0`).
- **Primary RID**: `win-x64` (Windows), `linux-x64` (Linux).
- **Steam AppID**: `4170200` (Data Center).
- **Solution**: `GregModmanager.sln`
- **Executable Project**: `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`
- **Shared Library**: `src/GregModmanager.Core/GregModmanager.Core.csproj`

---

## 2. Architecture Rules

### 2.1 Project Layout

- `src/GregModmanager.Avalonia/` — Avalonia executable, Views, DI setup, custom chrome window.
- `src/GregModmanager.Core/` — Shared services, models, Steam integration, localization.
- `src/SubDirectoryFixer/` — net6.0 helper project (MelonLoader plugin).
- `tests/GregModmanager.Tests/` — xUnit test project.
- `build/scripts/` — All build and automation scripts.
- `build/installer/` — Inno Setup script (`gregModmanager.iss`) and signing tools.
- `wiki/` — **Git submodule** pointing to `https://github.com/mleem97/gregModmanager.wiki.git`.
  - Do NOT commit wiki contents into the main repo.
  - Update the submodule pointer after wiki changes.
- All Documentation and Correspondence must use the English Language.

### 2.2 Dependency Direction

- Core must never reference Avalonia.
- Avalonia depends on Core via `<ProjectReference>`.
- Use `Microsoft.Extensions.DependencyInjection` for service registration in `Program.cs`.

### 2.3 Cross-Platform Constraints

- Prefer `System.IO.Path.Combine` over string concatenation.
- Use `Environment.SpecialFolder` or known paths; never hard-code Windows paths in Core.
- Platform-specific code must be guarded with `#if WINDOWS` or runtime checks.
- Linux packaging requires `tar`, `dotnet`, and optionally `wsl` + `nfpm`.

---

## 3. Avalonia UI Best Practices

### 3.1 Window Chrome

- Use custom borderless window: `SystemDecorations="None"`.
- Do NOT use `ExtendClientAreaToDecorationsHint="True"` if it breaks dragging.
- Implement drag-to-move via `PointerPressed` + `BeginMoveDrag` on the title bar `Border`.

### 3.2 Data Binding

- Prefer `CompiledBindings` where possible to reduce reflection and trimming warnings.
- Avoid `ReflectionBindingExtension` in performance-critical paths.

### 3.3 Styling (Terminal Core)

- Design system: **Terminal Core**.
- 8px grid alignment.
- Sharp geometry: radius `<= 8px` on containers.
- Hairline separators (`#122131` on `#051424`).
- Monospace for metadata and status values.
- Color tokens:
  - Surface base: `#051424`
  - Primary: `#8AEBFF`
  - Secondary: `#4DE082`
  - On-surface: `#D4E4FA`
- No soft shadows, no rounded bubble UI.

### 3.5 Telemetry & Health Monitoring

- **Backend**: Self-hosted Loki at `telemetry.datacentermods.com`.
- **Authentication**: Uses `X-Scope-OrgID: managerclient` for multi-tenancy.
- **Privacy**: Respects `AppSettings.TelemetryEnabled`. Anonymized `machine_id` is used for installation tracking.
- **Implementation**: `TelemetryService.cs` handles structured JSON events and crash report flushing.
- **Startup Resilience**: Eager reporting in `Program.Main` sends previous crash reports before full UI initialization.

### 3.6 Online Installer (Windows)

- **Tool**: Inno Setup 6.
- **Dependencies**: Automatically detects and installs Microsoft Visual C++ Redistributable (x64).
- **Online Capability**: Downloads required runtimes via PowerShell during the installation process if missing.

---

## 4. Steam & Workshop Rules

### 4.1 Rate Limiting

- `SteamPublishRateLimiter.Shared` enforces:
  - **30 seconds** minimum between publish attempts.
  - **5 attempts** per **10-minute** rolling window.
- Always check `TryAcquire(out retryAfter)` before calling `SubmitAsync()`.
- UI must display explicit cooldown timers in seconds.

### 4.2 Native DLL Handling

- `steam_api64.dll` must NEVER be Authenticode-signed (it is not a valid PE for signing).
- Exclude it from `signtool` / `Get-AuthenticodeSignature` loops.

### 4.3 Game Root & Paths

- Game root resolves via Steam API (`SteamApps.AppInstallDir(4170200)`) or user preference.
- Workshop sync targets depend on `ModContentType`:
  - `PlacableObject` → `{GameRoot}/Mods/Workshop/{id}/`
  - `MelonloaderPlugin` → `{GameRoot}/Plugins/{id}/`
  - `Userlib` → `{GameRoot}/Userlibs/{id}/`
  - `DataCenterMod` → `{GameRoot}/Mods/{id}/`

---

## 5. Build & Release Rules

### 5.1 Size Optimization (Critical)

Publish settings must include:

```xml
<PublishTrimmed>true</PublishTrimmed>
<TrimMode>full</TrimMode>
<PublishSingleFile>true</PublishSingleFile>
<SelfContained>true</SelfContained>
<DebugType>none</DebugType>
<DebugSymbols>false</DebugSymbols>
```

- Target size: **< 50 MB** (currently ~38 MB).
- Avoid adding large native dependencies without measuring publish output.

### 5.2 Build Orchestration

- **Local builds**: use `build/scripts/build.ps1` (mirrors CI exactly).
- **Interactive builder**: use `build/builder.ps1` or `build/builder.sh`.
- CI workflow: `.github/workflows/build-and-release.yml`.
- The PowerShell script and the GitHub Actions workflow must stay in sync.

### 5.3 Signing

- Windows binaries: Authenticode via `build/installer/sign-authenticode.ps1`.
- Environment: `CODE_SIGN_THUMBPRINT` (store cert) or `CODE_SIGN_PFX` + `CODE_SIGN_PFX_PASSWORD`.
- If no cert is configured, the build script creates an ephemeral self-signed cert.

### 5.4 Versioning

- Version lives in `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj` `<Version>` property.
- Numeric version: `x.y.z` → `x.y.z.0` for Inno Setup.
- Single source of truth: the `.csproj` file.

---

## 6. Coding Standards

### 6.1 General

- Use `init` or `required` for DTO properties where applicable.
- Prefer `record struct` for small immutable value types.
- Use `CancellationToken` on all async service methods.
- Keep UI components compact and data-dense.

### 6.2 JSON Serialization

- Use `System.Text.Json` with `[JsonPropertyName(...)]` attributes.
- Avoid polymorphic deserialization without source generators (trimming-safe).
- If trimming warnings appear for JSON code, consider adding the type to `TrimmerRoots.xml`.

### 6.3 Localization

- Strings are stored in `Resources/Strings/AppStrings.resx` + satellite `.de.resx`, `.es.resx`, etc.
- Access via `GregModmanager.Localization.S.Get("Key")` or `S.Format("Key", args)`.
- When adding UI text, add entries to `AppStrings.resx` and `AppStrings.de.resx` at minimum.

### 6.4 Git Workflow

- Create a **feature branch** for every change.
- Open a **Pull Request** to `main`; do not push directly to `main`.
- Keep commits atomic and focused.
- Update `AGENTS.md` if you change build steps, architecture, or agent instructions.

---

## 7. Troubleshooting Quick Reference

| Symptom | Fix |
|---------|-----|
| Avalonia window has duplicate title bars | Remove `ExtendClientAreaToDecorationsHint` or set to `False`. |
| `steam_api64.dll` signing fails with `0x800700C1` | Skip it in the signing loop. |
| Linux package build fails | Ensure `nfpm` is installed and in `PATH`. |
| Steam upload blocked | Check `SteamPublishRateLimiter` cooldown banner. |
| Trimming warnings in JSON code | Use source generators or add roots to `TrimmerRoots.xml`. |

---

## 8. External Resources

- **Wiki (submodule)**: `wiki/Home.md`
- **Inno Setup**: <https://jrsoftware.org/isdl.php>
- **Avalonia Docs**: <https://docs.avaloniaui.net/>
- **nfpm**: <https://nfpm.goreleaser.com/>

## Core Runtime Guardrails

- Keep all gameplay/runtime-facing components compatible with `.NET 6.x`.
- This also applies to any shared libraries, Melonloader Plugins or Game Mods that may be used, within this Project, and/or in Unity IL2CPP + MelonLoader contexts.
- Do not retarget runtime projects beyond `net6.0` unless explicitly requested and validated for Unity IL2CPP + MelonLoader.

## Mandatory System Architecture Prompt

- Apply `.github/instructions/gregframework_system_architecture.instructions.md` to all implementation and design decisions.
- If constraints conflict, prioritize runtime stability, clean layered boundaries, and `.NET 6` compatibility.

## SonarQube MCP Rules

- Apply `.github/instructions/sonarqube_mcp.instructions.md` whenever SonarQube MCP tooling is used.

## Collaboration Defaults

- Respond in technical German unless a file or repository policy explicitly requires English-only artifacts.
- Summarize intent before code changes.
- Keep refactors minimal and architecture-safe.

## Wiki Currency Check (Mandatory)

- At the end of every change request, verify whether relevant wiki pages are up to date.
- If updates are required, list the pages and include them in follow-up recommendations.

---

## 9. Versioning, Commits & Changelog

### 9.1 Semantic Versioning (SemVer)

This project follows [Semantic Versioning 2.0.0](https://semver.org/).

- **Single source of truth:** `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj` `<Version>` property.
- Format: `MAJOR.MINOR.PATCH[-prerelease]` (e.g. `1.5.0`, `1.6.0-alpha1`).
- **Bump rules:**
  - `MAJOR` — breaking API or behavior changes (public interfaces, save-data formats, CLI arguments).
  - `MINOR` — new features, new mod content types, new UI pages, backward-compatible additions.
  - `PATCH` — bug fixes, security patches, performance improvements, docs corrections.
  - Prerelease identifiers (`-alpha`, `-beta`, `-pre`) produce artifacts with `-pre` suffix.

### 9.2 Conventional Commits

All commits **MUST** follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

**Structure:**

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types and SemVer mapping:**

| Type | SemVer impact | Use for |
|------|---------------|---------|
| `feat` | MINOR | New features, new UI pages, new public APIs |
| `fix` | PATCH | Bug fixes, crash fixes, security patches |
| `perf` | PATCH | Performance improvements |
| `refactor` | PATCH | Code restructuring without behavior change |
| `docs` | — | Documentation and comment changes |
| `style` | — | Formatting, whitespace, semicolons |
| `test` | — | Adding or correcting tests |
| `build` | — | Build scripts, CI, dependencies |
| `ci` | — | GitHub Actions, workflow changes |
| `chore` | — | Routine maintenance, dependency bumps |

**Breaking changes:**

- Append `!` after type/scope: `feat(api)!: remove legacy upload endpoint`
- OR add footer: `BREAKING CHANGE: old mod format no longer supported`
- Breaking changes **always** trigger a **MAJOR** bump.

**Examples:**

```
feat(workshop): add PlacableObject mod type routing

Implements subdirectory routing for PlacableObject mods
to {GameRoot}/Mods/Workshop/{id}/.

Refs: #42
```

```
fix(steam): prevent rate limit bypass in rapid publish

SteamPublishRateLimiter now correctly resets the rolling
window after 10 minutes.
```

```
feat(auth)!: replace username/password with Steam OpenID

BREAKING CHANGE: local credential store is no longer read.
Users must re-authenticate via Steam.
```

**Agent rules when generating commits:**

- Every commit must have a type prefix.
- Scope is recommended for commits touching `Steam`, `Workshop`, `UI`, `Build`, `CI`.
- Description uses imperative mood (`add`, not `added` or `adds`).
- Body explains **why**, not just what.
- Footer references issues/PRs with `Refs: #123`, `Closes: #456`, `Fixes: #789`.
- **Never** combine unrelated changes in a single commit.

### 9.3 Changelog Maintenance

This project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

- **File:** `CHANGELOG.md` at repository root.
- **Format:** Markdown with `## [Version] - YYYY-MM-DD` headers.
- **Sections per version:** `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

**Agent workflow:**

1. When implementing a feature/fix, add an entry to the `[Unreleased]` section immediately.
2. Entries should be concise, user-facing descriptions (not commit messages).
3. Group related entries under the correct subsection.
4. **Do not** edit released version sections.

**Release promotion:**

- The `[Unreleased]` header is promoted to a versioned header **only** by the automated workflow (`.github/workflows/promote-changelog.yml`).
- The workflow is triggered manually via `workflow_dispatch` with the target version.
- The workflow:
  1. Renames `[Unreleased]` to `## [x.y.z] - YYYY-MM-DD`.
  2. Inserts a new empty `[Unreleased]` block at the top.
  3. Bumps `<Version>` in `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`.
  4. Commits to `main` and pushes.
  5. Optionally creates and pushes git tag `vx.y.z` (if `create_tag` is true).
- After promotion, the `build-and-release.yml` workflow triggers automatically from the tag.

**Manual emergency override:**
If the automated workflow is unavailable, an agent **may** perform the promotion manually by editing `CHANGELOG.md` and the `.csproj` file in a single commit with message:

```
chore(release): promote changelog and bump version to x.y.z
```

### 9.4 Release Checklist (Agent Responsibility)

Before triggering the promote workflow:

- [ ] `CHANGELOG.md` `[Unreleased]` section is complete and accurate.
- [ ] All referenced issues/PRs are closed or documented.
- [ ] `AGENTS.md` updated if architecture or workflow changed.
- [ ] `EXTERNAL_DEPENDENCIES.md` updated if new packages added.
- [ ] Version in `.csproj` matches intended release (workflow will overwrite anyway, but verify).
- [ ] `main` branch CI is green.
