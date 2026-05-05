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
- **Executable Project**: `GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`
- **Shared Library**: `GregModmanager.Core.csproj`

---

## 2. Architecture Rules

### 2.1 Project Layout
- `GregModmanager.Avalonia/` — Avalonia executable, Views, DI setup, custom chrome window.
- `GregModmanager.Core/` — Shared services, models, Steam integration, localization.
- `scripts/` — All build and automation scripts.
- `installer/` — Inno Setup script (`gregModmanager.iss`).
- `wiki/` — **Git submodule** pointing to `https://github.com/mleem97/gregModmanager.wiki.git`.
  - Do NOT commit wiki contents into the main repo.
  - Update the submodule pointer after wiki changes.

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

### 3.4 Dialogs
- Never use native `MessageBox`.
- Use the injected `IDialogService` (`Window.ShowDialog`) for all user prompts.

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
- **Local builds**: use `scripts/build.ps1` (mirrors CI exactly).
- **Interactive builder**: use `scripts/builder.ps1` or `scripts/builder.sh`.
- CI workflow: `.github/workflows/build-and-release.yml`.
- The PowerShell script and the GitHub Actions workflow must stay in sync.

### 5.3 Signing
- Windows binaries: Authenticode via `installer/sign-authenticode.ps1`.
- Environment: `CODE_SIGN_THUMBPRINT` (store cert) or `CODE_SIGN_PFX` + `CODE_SIGN_PFX_PASSWORD`.
- If no cert is configured, the build script creates an ephemeral self-signed cert.

### 5.4 Versioning
- Version lives in `GregModmanager.Avalonia.csproj` `<Version>` property.
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
- **Inno Setup**: https://jrsoftware.org/isdl.php
- **Avalonia Docs**: https://docs.avaloniaui.net/
- **nfpm**: https://nfpm.goreleaser.com/
