# gregModmanager — Contributor Guide

**Last Updated:** May 2026  
**Version:** v1.5.1

---

## Table of Contents

1. [Project Overview & Architecture](#project-overview--architecture)
2. [Development Environment Setup](#development-environment-setup)
3. [Build System & Scripts](#build-system--scripts)
4. [Coding Standards & Conventions](#coding-standards--conventions)
5. [Testing](#testing)
6. [Debugging & Logging](#debugging--logging)
7. [Contributing: Workflow & Pull Requests](#contributing-workflow--pull-requests)
8. [Localization & Translations](#localization--translations)
9. [Release & Versioning Process](#release--versioning-process)
10. [External Dependencies](#external-dependencies)
11. [Security & Responsible Disclosure](#security--responsible-disclosure)

---

## Project Overview & Architecture

### Solution Structure

The gregModmanager repository is organized as follows:

```
gregModmanager.sln
├── GregModmanager.Core/                    # Core business logic (net9.0)
│   ├── Models/                             # Data models (ContentStats, DependencyCheckResult, etc.)
│   ├── Services/                           # Service layer (Steam, Workshop, Mods sync, dependencies)
│   ├── Steam/                              # Steam API integration (SteamAPI native loader)
│   ├── Localization/                       # Localization system (S.cs, TranslateExtension.cs)
│   └── GregModmanager.Core.csproj
│
├── GregModmanager.Avalonia/                # Avalonia UI application (net9.0-windows, net9.0-linux)
│   ├── Views/                              # Avalonia Views (ItemDetailPage, SettingsPage, etc.)
│   ├── Services/                           # UI-specific services (DialogService, SubDirectoryFixerInstallerService)
│   ├── Program.cs                          # Entry point, DI setup
│   ├── MainWindow.axaml(.cs)              # Main window chrome
│   ├── AppShell.xaml(.cs)                 # Shell/layout
│   ├── App.xaml(.cs)                      # Application root
│   └── GregModmanager.Avalonia.csproj
│
├── SubDirectoryFixer/                      # Helper utility (net6.0)
│   ├── SubDirectoryFixerBootstrap.cs      # Entry point
│   └── SubDirectoryFixer.csproj
│
├── GregModmanager.Tests/                   # Unit and integration tests (net9.0)
│   ├── ContentStatsTests.cs
│   ├── ProjectSanityTests.cs
│   └── GregModmanager.Tests.csproj
│
├── scripts/                                # Build and automation scripts
│   ├── build.ps1                          # Main build orchestration (Windows, Linux)
│   ├── builder.ps1 / builder.sh           # Interactive builder
│   └── linux/                             # Linux-specific packaging scripts
│
├── installer/                              # Installer and signing scripts
│   ├── gregModmanager.iss                 # Inno Setup script (Windows)
│   ├── sign-authenticode.ps1              # Code signing helper
│   └── CODE_SIGNING.md                    # Signing documentation
│
├── wiki/                                   # Documentation (Git submodule)
│   └── (synced from gregModmanager.wiki repo)
│
├── docs/                                   # Generated/supplementary docs
│   ├── 01_END_USER_GUIDE.md
│   ├── 02_MOD_CREATOR_GUIDE.md
│   ├── 03_CONTRIBUTOR_GUIDE.md
│   └── examples/                          # Example manifests and mods
│
├── Localization/                           # (Legacy, deprecated in favor of Core/Localization)
├── Models/                                 # (Legacy model definitions)
├── Services/                               # (Shared services used by both Core and legacy UI)
├── Steam/                                  # (Shared Steam integration)
│
├── Resources/                              # Shared resources
│   ├── Strings/                           # Localization strings (AppStrings.resx)
│   ├── Fonts/                             # Vendored fonts
│   ├── Styles/                            # Shared styling
│   └── AppIcon/                           # Application icon
│
├── Properties/                             # MSBuild properties
├── Platforms/                              # Platform-specific code
├── TrimmerRoots.xml                       # IL Trimmer roots (AOT safety)
├── GregModmanager.sln                     # Solution file
├── README.md                              # Main repository README
├── AGENTS.md                              # Agent instructions (build, release, structure)
└── EXTERNAL_DEPENDENCIES.md              # Third-party dependency inventory
```

### Architectural Patterns

#### Layered Architecture

```
┌─────────────────────────────────────────┐
│      UI Layer (Avalonia Views)          │  GregModmanager.Avalonia
├─────────────────────────────────────────┤
│  Service/ViewModel Layer                │  Mostly GregModmanager.Core.Services
├─────────────────────────────────────────┤
│  Model/Business Logic Layer             │  GregModmanager.Core.Models
├─────────────────────────────────────────┤
│  External Integration (Steam, IO)       │  GregModmanager.Core.Steam, Platform APIs
└─────────────────────────────────────────┘
```

#### Dependency Direction

```
GregModmanager.Avalonia
       ↓
GregModmanager.Core
       ↓
(External: MelonLoader, Steam, System.Net, etc.)

⚠️  Core MUST NEVER reference Avalonia
```

#### Service Architecture

Services follow a common pattern:

```csharp
namespace GregModmanager.Services;

public interface IMyService
{
    Task<Result> DoSomethingAsync(CancellationToken ct);
}

public class MyService : IMyService
{
    private readonly ILogger _logger;
    private readonly IExternalDependency _dep;

    public MyService(ILogger logger, IExternalDependency dep)
    {
        _logger = logger;
        _dep = dep;
    }

    public async Task<Result> DoSomethingAsync(CancellationToken ct)
    {
        // Implementation
    }
}
```

#### Dependency Injection

All services are registered in `GregModmanager.Avalonia/Program.cs`:

```csharp
services.AddSingleton<IMyService, MyService>();
services.AddTransient<IMyEphemeralService, MyEphemeralService>();
```

### Key Architectural Decisions

1. **No Avalonia in Core:** Core library is strictly business logic; UI components are isolated in Avalonia project
2. **Service-Based:** Heavy use of dependency injection and interfaces for testability
3. **Async Throughout:** All I/O operations use `async/await` and `CancellationToken`
4. **Structured Logging:** Use `DebugSessionLog` and `DebugNdjsonSessionLog` for debug output
5. **Cross-Platform:** Code uses `Path.Combine`, `Environment.SpecialFolder`, and platform guards (`#if WINDOWS`)

---

## Development Environment Setup

### Required Tools

| Tool | Version | Purpose |
| :--- | :--- | :--- |
| **.NET SDK** | 9.0+ | Build, compile, test |
| **Visual Studio** / **Rider** / **VS Code** | Latest | Editor and debugging |
| **Git** | Latest | Version control |
| **PowerShell** | 5.1+ (Core 7+ for Linux) | Build scripts |
| **Inno Setup** (Windows only) | 6.0+ | Build Windows installer |
| **nfpm** (Linux packaging) | Latest | Package Linux distributions |

### Installation

#### Windows

```powershell
# Install .NET 9 SDK
# Download from: https://dotnet.microsoft.com/download/dotnet/9.0

# Install Visual Studio (or Rider/VS Code)
# Download from: https://visualstudio.microsoft.com/

# Install Inno Setup (optional, for building installer)
choco install innosetup
# Or download from: https://jrsoftware.org/isdl.php

# Clone repository
git clone https://github.com/mleem97/gregModmanager.git
cd gregModmanager
```

#### Linux (Ubuntu/Debian)

```bash
# Install .NET 9 SDK
wget https://dot.net/v1/dotnet-install.sh -O dotnet-install.sh
chmod +x dotnet-install.sh
./dotnet-install.sh --channel 9.0

# Install editor (optional)
sudo apt-get install code
# Or install Rider: https://www.jetbrains.com/rider/

# Install nfpm (for Linux packaging)
echo 'deb [trusted=yes] https://repo.goreleaser.com/apt/ /' | sudo tee /etc/apt/sources.list.d/goreleaser.list
sudo apt-get update
sudo apt-get install nfpm

# Clone repository
git clone https://github.com/mleem97/gregModmanager.git
cd gregModmanager
```

### Initialize Local Development Environment

```powershell
# Windows
.\scripts\install-local.ps1

# Linux
bash ./scripts/install-local.sh
```

This script:
1. Installs .NET dependencies via NuGet
2. Verifies build environment
3. Compiles both projects
4. Runs basic sanity checks

### Running the Application Locally

```powershell
# Via script (recommended, mirrors CI)
.\scripts\run.ps1
# or
.\scripts\builder.ps1

# Or directly with dotnet
dotnet run --project .\src\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj

# Or via batch file (Windows)
.\Start-GregModmanager.bat
```

### Environment Variables & Configuration

Optional environment variables for development:

```powershell
# Code Signing (for installer signing)
$env:CODE_SIGN_THUMBPRINT = '<thumbprint>'
# or
$env:CODE_SIGN_PFX = 'C:\path\to\cert.pfx'
$env:CODE_SIGN_PFX_PASSWORD = 'password'

# Steam Integration
$env:DATA_CENTER_GAME_DIR = 'C:\path\to\game' # Override game root detection

# Build Options
$env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE = 'true'
$env:DOTNET_CLI_TELEMETRY_OPTOUT = 'true'
```

### From Zero to Running App (Quickstart)

```powershell
# 1. Install .NET 9 SDK (if not present)
dotnet --version

# 2. Clone repository
git clone https://github.com/mleem97/gregModmanager.git
cd gregModmanager

# 3. Restore dependencies
dotnet restore GregModmanager.sln

# 4. Build solution
dotnet build GregModmanager.sln -c Debug

# 5. Run app
dotnet run --project GregModmanager.Avalonia

# 6. (Or use the convenience script)
.\scripts\run.ps1
```

---

## Build System & Scripts

### Main Build Script: `build.ps1`

Located at `scripts/build.ps1`, this is the primary build orchestration for Windows and Linux.

#### Purpose

- Builds both Core and Avalonia projects
- Runs tests
- Creates Windows installer (via Inno Setup)
- Creates portable ZIP archive
- Optionally builds Linux packages (via WSL or native Linux)
- Signs binaries (optional)

#### Usage

```powershell
# Build everything
.\scripts\build.ps1

# Skip tests
.\scripts\build.ps1 -SkipTest

# Skip Linux packaging
.\scripts\build.ps1 -SkipLinux

# Skip Windows packaging
.\scripts\build.ps1 -SkipWindows

# Sign binaries (requires cert installed)
.\scripts\build.ps1 -Sign

# Only sign (skip build)
.\scripts\build.ps1 -SignOnly

# Use specific Inno Setup installer path
.\scripts\build.ps1 -SetupPath 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'

# Use specific WSL distro for Linux packaging
.\scripts\build.ps1 -WslDistro 'Ubuntu-22.04'
```

#### Build Targets

The script produces:

| Target | Location | Purpose |
| :--- | :--- | :--- |
| **Debug (local)** | `bin/Debug/` | Local testing |
| **Release (single-file)** | `bin/Release/` | Trimmed, self-contained executable |
| **Windows Installer** | `dist/gregModmanager-v1.5.1-setup.exe` | Inno Setup installer |
| **Windows Portable** | `dist/gregModmanager-v1.5.1-portable.zip` | Portable archive |
| **Linux AppImage** | `dist/GregModmanager-v1.5.1.AppImage` | Self-contained Linux image |
| **Linux .deb** | `dist/gregmodmanager_1.5.1_amd64.deb` | Debian/Ubuntu package |
| **Linux .rpm** | `dist/gregmodmanager-1.5.1-1.x86_64.rpm` | Fedora/RHEL package |

#### Key Functions

```powershell
Get-ProjectVersion              # Extracts version from .csproj
New-Sha256File                 # Creates SHA256 checksum files
Test-ZipArchiveExtractable     # Validates ZIP archives
Assert-AuthenticodeSignaturePresent  # Verifies code signing
New-EphemeralCodeSignThumbprint # Creates temporary signing cert
Invoke-BuildSign               # Authenticode signs executables
```

### SubDirectoryFixer Integration

The build script automatically:

1. Builds the `SubDirectoryFixer` project
2. Copies `SubDirectoryFixer.dll` into Avalonia assets
3. Includes the DLL in the installer package
4. Avalonia app auto-installs it on first run

### Manual Build Steps

If you want more control, you can build each component separately:

#### Build Core Only

```powershell
dotnet build GregModmanager.Core.csproj -c Release
```

#### Build Avalonia Only

```powershell
dotnet build GregModmanager.Avalonia/GregModmanager.Avalonia.csproj -c Release
```

#### Publish Single-File Executable (Windows)

```powershell
dotnet publish GregModmanager.Avalonia/GregModmanager.Avalonia.csproj `
  -c Release `
  -f net9.0-windows10.0.19041.0 `
  -r win-x64 `
  --self-contained `
  -o bin/Release/publish
```

#### Create Installer (Windows)

Requires Inno Setup to be installed:

```powershell
$isccPath = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
& $isccPath installer/gregModmanager.iss
```

#### Build Linux Packages

On Linux or via WSL:

```bash
# AppImage
fpm -s dir -t appimage \
  -n gregmodmanager \
  -v 1.5.1 \
  ./publish/=/opt/gregmodmanager
```

### CI/CD Pipeline

See `.github/workflows/build-and-release.yml` for the GitHub Actions workflow that:

1. Resolves version from `.csproj`
2. Runs tests on Ubuntu
3. Builds Windows artifacts on `windows-latest`
4. Builds Linux artifacts on `ubuntu-latest`
5. Creates GitHub release on tag push
6. Uploads artifacts to release

---

## Coding Standards & Conventions

### Naming Conventions

| Category | Convention | Example |
| :--- | :--- | :--- |
| **Classes** | PascalCase | `WorkshopSyncService`, `ModItem` |
| **Methods** | PascalCase | `GetModsAsync()`, `ResolveConflicts()` |
| **Properties** | PascalCase | `IsEnabled`, `ModCount` |
| **Fields (private)** | _camelCase | `_logger`, `_cache` |
| **Parameters** | camelCase | `modId`, `cancellationToken` |
| **Constants** | UPPER_SNAKE_CASE | `DEFAULT_MOD_PATH`, `TIMEOUT_MS` |
| **Interfaces** | IPascalCase | `IModService`, `ILogger` |

### File Organization

```
Services/
├── IMyService.cs          # Interface definition (optional, separate file)
├── MyService.cs           # Implementation
└── MyServiceException.cs  # Service-specific exceptions (if needed)

Models/
├── Mod.cs                 # DTO/model
├── ModDependency.cs       # Related model
└── ModCollection.cs       # Collection model
```

### Async Patterns

All asynchronous operations use `async/await` and include `CancellationToken`:

```csharp
public interface IModService
{
    Task<List<Mod>> GetModsAsync(CancellationToken cancellationToken);
    Task<bool> InstallModAsync(string modId, CancellationToken cancellationToken);
}

public async Task<List<Mod>> GetModsAsync(CancellationToken cancellationToken)
{
    var result = await _httpClient.GetAsync(url, cancellationToken);
    // Handle response
    return mods;
}
```

### Null Handling

Use modern C# nullable reference types:

```csharp
#nullable enable

public class Mod
{
    public string Id { get; set; } = "";              // Non-nullable, must be set
    public string? Description { get; set; }          // Nullable
    
    public string GetDescription() => Description ?? "No description";
}

#nullable restore
```

### Logging

Use structured logging via `DebugSessionLog` and `AppFileLog`:

```csharp
// Human-readable debug log
DebugSessionLog.Write("H1", "LocationTag", "event_name", new { detail1 = "value", detail2 = 42 });

// File logging
AppFileLog.Info("Installation started");
AppFileLog.Warn("Dependency missing: X");
AppFileLog.Error("Installation failed", exception);

// Structured NDJSON logging
DebugNdjsonSessionLog.Write("H1", "LocationTag", "event_name", new { structured = "data" });
```

### Localization Strings

Always use the localization system for user-facing strings:

```csharp
// Add to Resources/Strings/AppStrings.resx
// Key: "InstallationStarted"
// Value: "Installation started for {0}"

// In code
string message = GregModmanager.Localization.S.Format("InstallationStarted", modName);
// or
string message = GregModmanager.Localization.S.Get("InstallationStarted");
```

### Adding New Services

1. Create interface in `Services/IMyService.cs`
2. Implement in `Services/MyService.cs`
3. Register in `GregModmanager.Avalonia/Program.cs`:

```csharp
services.AddSingleton<IMyService, MyService>();
```

4. Inject via constructor in consuming classes

### Adding New Models

1. Create in `Models/MyModel.cs`
2. Use proper nullability annotations
3. Use `record` for immutable DTOs when appropriate:

```csharp
public record ModInfo(string Id, string Name, string Version)
{
    public override string ToString() => $"{Name} v{Version}";
}
```

### Error Handling

Use custom exceptions for domain-specific errors:

```csharp
public class ModInstallationException : Exception
{
    public ModInstallationException(string message, Exception? inner = null)
        : base(message, inner) { }
}

// Usage
throw new ModInstallationException($"Failed to install mod {modId}", ex);
```

### Code Style

- Use `var` for obvious types; explicit types for clarity
- Prefer LINQ over loops where readable
- Use expression-bodied members for simple properties/methods
- Keep methods focused and small (<30 lines preferred)
- Add XML documentation comments for public API

---

## Testing

### Test Project Structure

`GregModmanager.Tests/` contains:

```
GregModmanager.Tests/
├── ContentStatsTests.cs          # Unit tests for ContentStats model
├── ProjectSanityTests.cs         # Project structure validation tests
└── (More test files as needed)
```

### Running Tests

#### All Tests

```powershell
dotnet test GregModmanager.sln -c Release

# Or with verbose output
dotnet test GregModmanager.sln -c Release --verbosity detailed
```

#### Specific Test File

```powershell
dotnet test GregModmanager.Tests/GregModmanager.Tests.csproj -c Release
```

#### Specific Test Method

```powershell
dotnet test GregModmanager.Tests/GregModmanager.Tests.csproj `
  -c Release `
  --filter "ClassName.TestMethodName"
```

#### With Code Coverage

```powershell
dotnet test GregModmanager.sln -c Release `
  /p:CollectCoverage=true `
  /p:CoverageFormat=cobertura `
  /p:CoverageFileName=coverage.xml
```

### Unit Testing Conventions

```csharp
using Xunit;

namespace GregModmanager.Tests;

public class ModServiceTests
{
    [Fact]
    public async Task GetMods_WithValidId_ReturnsModList()
    {
        // Arrange
        var service = new ModService();
        var modId = "test-mod";

        // Act
        var result = await service.GetModAsync(modId, CancellationToken.None);

        // Assert
        Assert.NotNull(result);
        Assert.Equal("test-mod", result.Id);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    public async Task GetMods_WithInvalidId_ThrowsArgumentException(string invalidId)
    {
        // Arrange
        var service = new ModService();

        // Act & Assert
        await Assert.ThrowsAsync<ArgumentException>(
            () => service.GetModAsync(invalidId, CancellationToken.None));
    }
}
```

### Test Coverage Goals

- **Target:** 70%+ code coverage for Core services
- **Critical:** 100% coverage for dependency resolution and mod installation logic
- **Nice-to-have:** UI view tests (Avalonia testing is complex)

### Adding New Tests

1. Create `MyFeatureTests.cs` in `GregModmanager.Tests/`
2. Follow Arrange-Act-Assert pattern
3. Use `[Fact]` for single tests, `[Theory]` + `[InlineData]` for parameterized tests
4. Mock external dependencies (use Moq or similar)
5. Run locally before submitting PR

---

## Debugging & Logging

### Debug Logs

#### `DebugSessionLog` (Human-Readable)

Located in root directory as `_debug_session_*.txt`:

```csharp
DebugSessionLog.Write("H1", "Category", "EventName", new { detail = "value" });

// Example output:
// [H1] Category.EventName: {"detail":"value"}
```

#### `DebugNdjsonSessionLog` (Structured)

Located in root directory as `_debug_session_*.ndjson`:

```csharp
DebugNdjsonSessionLog.Write("H1", "Category", "EventName", new { detail = "value" });

// Example output (one JSON object per line):
// {"level":"H1","category":"Category","event":"EventName","data":{"detail":"value"},"timestamp":"2026-05-01T12:00:00Z"}
```

### File Logging

Via `AppFileLog` (written to `%AppData%/gregModmanager/logs/`):

```csharp
AppFileLog.Info("Operation completed");
AppFileLog.Warn("Warning message");
AppFileLog.Error("Error occurred", exception);
AppFileLog.MarkCrash("CrashType", exception);
```

### Attaching Debugger (Windows)

#### Visual Studio

1. Open solution in Visual Studio
2. Set breakpoints
3. Run via **Debug** → **Start Debugging** (F5)

#### Rider

1. Open solution in Rider
2. Set breakpoints
3. Click **Run** → **Debug** (Shift+F9)

#### VS Code with C# Dev Kit

1. Install C# Dev Kit extension
2. Open folder
3. Click **Run and Debug** → select "net6.0" target
4. Set breakpoints and run

### Attaching Debugger (Linux)

```bash
# Install vsdbg (VS Code debugger)
curl -sSL https://aka.ms/getvsdbg | bash -s latest

# Run app with debugging
dotnet run --project GregModmanager.Avalonia -- --wait-for-debugger

# Attach debugger (VS Code)
# Press F5, select ".NET Core" environment
```

### Diagnostic Checklist

When investigating a crash:

1. **Check log files:**
   - `%AppData%/gregModmanager/logs/` (Windows)
   - `~/.config/gregModmanager/logs/` (Linux)

2. **Enable verbose logging:**
   - Settings → **Debug Logging** ☑

3. **Look for `[ERROR]` or `[EXCEPTION]` lines** in logs

4. **Reproduce the issue:**
   - Exact steps to reproduce
   - System environment (OS, .NET version)
   - Installed mods/plugins

5. **Collect debug logs:**
   - Logs panel → **Export Logs**
   - Attach to bug report

---

## Contributing: Workflow & Pull Requests

### Branch Naming

```
feature/add-new-mod-browser       # New feature
fix/resolve-dependency-conflicts   # Bug fix
docs/update-readme                 # Documentation
chore/update-dependencies          # Maintenance
test/add-integration-tests         # Testing
refactor/reorganize-services       # Code cleanup
```

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): subject

body (optional)

footer (optional)
```

**Examples:**

```
feat(mod-browser): add search filters for mod category

fix(dependencies): resolve circular dependency detection

docs(readme): clarify installation on Linux

test(mod-service): add unit tests for mod installation

chore(deps): update MelonLoader to 0.6.2
```

### Creating a Pull Request

1. **Create a feature branch:**
   ```powershell
   git checkout -b feature/your-feature-name
   ```

2. **Make changes and commit:**
   ```powershell
   git add .
   git commit -m "feat(scope): your commit message"
   ```

3. **Push to your fork:**
   ```powershell
   git push origin feature/your-feature-name
   ```

4. **Open PR on GitHub:**
   - Go to [GitHub](https://github.com/mleem97/gregModmanager)
   - Click **New Pull Request**
   - Select your branch
   - Fill in PR template (see below)

### Pull Request Template

```markdown
## Description
Brief description of the changes in 2-3 sentences.

## Linked Issue
Fixes #123
Related to #456

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update

## Testing Done

- [ ] Unit tests added/updated
- [ ] Manual testing on Windows
- [ ] Manual testing on Linux
- [ ] No new warnings or errors

## Screenshots (if UI change)
(Attach screenshots or GIFs showing the change)

## Checklist
- [ ] My code follows the coding style of this project
- [ ] I have added documentation for new features
- [ ] I have updated localization strings (if applicable)
- [ ] I have tested my changes
- [ ] All tests pass locally
- [ ] No breaking changes (or documented in PR description)
```

### Review Process

1. **Automated Checks:**
   - CI builds project
   - Tests run (must pass)
   - Code formatting checked
   - Dependency security scanned

2. **Code Review:**
   - At least 1 maintainer review required
   - Reviewers check for:
     - Code quality and style compliance
     - Architecture adherence
     - Test coverage
     - Documentation completeness

3. **Merge:**
   - All checks must pass
   - At least 1 approval from maintainer
   - Branch can be merged by maintainer

### Handling Merge Conflicts

```bash
# Fetch latest main
git fetch origin main

# Rebase onto main
git rebase origin/main

# Resolve conflicts in editor
# Then mark as resolved
git add .
git rebase --continue

# Force push to PR branch
git push origin feature/your-branch --force
```

---

## Localization & Translations

### Adding New Localized Strings

1. **Open `Resources/Strings/AppStrings.resx`** (base English strings)
2. **Add new entry:**
   - **Name:** `ModInstallationComplete` (descriptive, PascalCase)
   - **Value:** `Installation of {0} completed successfully` (supports `{0}`, `{1}` placeholders)
   - **Comment:** `Shown when mod installation finishes` (optional, for translators)

3. **Add to translations:**
   - `AppStrings.de.resx` (German)
   - `AppStrings.es.resx` (Spanish)
   - etc.

### Using Localized Strings in Code

```csharp
using GregModmanager.Localization;

// Get simple string
string message = S.Get("ModInstallationComplete");

// Get with format placeholders
string message = S.Format("ModInstallationComplete", modName);
```

### Adding Support for New Language

1. **Create new `.resx` file:**
   - Copy `AppStrings.resx` → `AppStrings.xx.resx` (where `xx` is language code)
   - Language codes: `de` (German), `es` (Spanish), `ja` (Japanese), etc.

2. **Translate all entries:**
   - Use a `.resx` editor (Visual Studio or VS Code extension)
   - Or edit XML directly

3. **Update `Localization/S.cs`:**
   ```csharp
   var culture = CultureInfo.GetCultureInfo("xx");
   ```

4. **Test:**
   - Settings → **Language** → select new language
   - App restarts in new language
   - Verify all UI strings are translated

### Translation Guidelines

- **Be concise:** Keep UI strings short and scannable
- **Match terminology:** Use consistent terminology across all strings
- **Respect placeholders:** Don't remove `{0}` placeholders; they're filled by code
- **Test context:** Some strings appear in dialogs, some in tooltips
- **RTL languages:** Note if you're adding Arabic or Hebrew (special handling needed)

### Reporting Translation Issues

1. Create GitHub issue: [Issues](https://github.com/mleem97/gregModmanager/issues)
2. Title: `Translation: [Language] [Issue Description]`
3. Include:
   - Language and location (menu, dialog, etc.)
   - Incorrect translation
   - Suggested correction
   - Screenshot if context unclear

---

## Release & Versioning Process

### Semantic Versioning

gregModmanager follows [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH

1.5.1
│ │ └─ Patch: bug fixes, minor improvements (no breaking changes)
│ └─── Minor: new features (backward compatible)
└───── Major: breaking changes, major rewrites
```

**Examples:**
- `1.5.0` → `1.5.1` — bug fixes only
- `1.5.1` → `1.6.0` — new features added
- `1.6.0` → `2.0.0` — breaking changes (e.g., new architecture)

### Release Workflow

#### 1. Update Version Number

Edit `GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`:

```xml
<PropertyGroup>
  <Version>1.6.0</Version>
  <!-- ... -->
</PropertyGroup>
```

This propagates to:
- Installer (Inno Setup)
- Build scripts
- GitHub Actions workflows

#### 2. Update Changelog

Edit `RELEASENOTE.md`:

```markdown
## v1.6.0 (2026-05-15)

### New Features
- Added mod search filters
- Integrated gregCore mod support
- New Italian localization

### Bug Fixes
- Fixed crash on empty mod list
- Improved dependency resolution performance
- Fixed Steam API integration on Linux

### Breaking Changes
- Removed legacy MAUI UI code

### Contributors
- @john-doe
- @jane-smith
```

#### 3. Update Wiki (if needed)

If documentation needs updating:
- Edit wiki pages in `wiki/` folder
- Test in local wiki preview
- Commit submodule changes

#### 4. Create Release Commit

```bash
git checkout -b release/v1.6.0
git add RELEASENOTE.md GregModmanager.Avalonia/GregModmanager.Avalonia.csproj wiki/
git commit -m "release: v1.6.0 - new features and bug fixes"
git push origin release/v1.6.0
```

#### 5. Create Pull Request

- Open PR for `release/v1.6.0` → `main`
- Let CI build all artifacts
- Get final approval from maintainers
- Merge to `main`

#### 6. Tag Release

```bash
git checkout main
git pull origin main
git tag -a v1.6.0 -m "Release v1.6.0"
git push origin v1.6.0
```

#### 7. GitHub Release

- Go to [Releases](https://github.com/mleem97/gregModmanager/releases)
- Click **Draft a new release**
- Select tag `v1.6.0`
- Title: `gregModmanager v1.6.0`
- Description: Paste RELEASENOTE.md content
- Upload artifacts (if not auto-uploaded by CI)
- Click **Publish release**

### Release Types

| Type | Version | Use Case |
| :--- | :--- | :--- |
| **Stable** | `1.5.0` | Production release, well-tested |
| **Pre-release** | `1.6.0-alpha.1` | Early preview, may have bugs |
| **Patch** | `1.5.1` | Bug fix only |
| **Hotfix** | `1.5.0.1` | Emergency fix (rare) |

---

## External Dependencies

See `EXTERNAL_DEPENDENCIES.md` for a comprehensive inventory of all third-party libraries.

### Policy for Adding Dependencies

Before adding a new external dependency, ensure:

1. **Justification:** Why is this needed? Is there no built-in alternative?
2. **License:** Compatible with MIT/Apache-2.0 license?
3. **Security:** No known vulnerabilities? Active maintenance?
4. **Size:** Won't significantly increase installer size?
5. **AOT Compatibility:** Will it work with trimmed/self-contained builds?

### Updating Dependencies

```bash
# Check for updates
dotnet outdated GregModmanager.sln

# Update specific package
dotnet package update PackageName

# Update all packages
dotnet package update

# Review changelog for breaking changes
# Run tests
dotnet test

# Commit with conventional message
git commit -m "deps: update PackageName to 2.0.0"
```

### SteamAPI Integration

- **DLL:** `steam_api64.dll` (Windows) or `libsteam_api.so` (Linux)
- **Version:** Managed by Steam client
- **Preloading:** Handled by `SteamApiNativeLoader.cs`
- **Note:** Never Authenticode-sign `steam_api64.dll` (it's not a valid PE)

---

## Security & Responsible Disclosure

### Security Policy

**If you discover a security vulnerability:**

1. **Do NOT open a public issue**
2. **Email:** [security@gregmodmanager.eu](mailto:security@gregmodmanager.eu)
3. **Include:**
   - Vulnerability description
   - Steps to reproduce
   - Affected versions
   - Suggested fix (optional)
4. **Response time:** We aim to respond within 48 hours

### Handling Secrets

**NEVER commit secrets to the repository:**

- API keys
- Code signing certificates
- Database passwords
- OAuth tokens

**Use environment variables or secret managers:**

```csharp
var apiKey = Environment.GetEnvironmentVariable("API_KEY");
if (string.IsNullOrEmpty(apiKey)) throw new InvalidOperationException("API_KEY not set");
```

### Code Scanning

The repository has:

- **GitHub Advanced Security:** Dependency scanning, secret scanning enabled
- **Trivy:** Container image scanning (if Docker builds added)
- **OWASP:** Static analysis for common vulnerabilities

### SSL/TLS Certificates

All external API calls use HTTPS:

```csharp
var client = new HttpClient();
// .NET automatically validates certificates
var response = await client.GetAsync("https://api.example.com/data");
```

---

## Supporting Documents

### Related Files

- **Architecture Guide:** `AGENTS.md` (system architecture & constraints)
- **External Dependencies:** `EXTERNAL_DEPENDENCIES.md`
- **Code Signing:** `installer/CODE_SIGNING.md`
- **Release Notes:** `RELEASENOTE.md`
- **License:** MIT (see LICENSE file)
- **Sponsors:** `SPONSORS.md`

### Additional Resources

- **MelonLoader:** [melonwiki.xyz](https://melonwiki.xyz)
- **gregFramework:** [gregframework.eu](https://gregframework.eu)
- **Avalonia UI:** [docs.avaloniaui.net](https://docs.avaloniaui.net/)
- **Inno Setup:** [jrsoftware.org/isinfo.php](https://jrsoftware.org/isinfo.php)

---

## Support & Contact

- **GitHub Issues:** [Report bugs](https://github.com/mleem97/gregModmanager/issues)
- **Discussions:** [Ask questions](https://github.com/mleem97/gregModmanager/discussions)
- **Email:** [support@gregmodmanager.eu](mailto:support@gregmodmanager.eu)
- **Discord:** (if community server exists)

---

---

###### Last Updated: May 2026
###### Version: v1.5.1
