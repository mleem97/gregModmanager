# gregModmanager — Mod & Plugin Creator Guide

**Last Updated:** May 2026  
**Version:** v1.5.1

---

## Table of Contents

1. [Modding Ecosystem Overview](#modding-ecosystem-overview)
2. [Creating MelonLoader Universal / Game-Specific Plugins](#creating-melonloader-universal--game-specific-plugins)
3. [Creating Game Mods](#creating-game-mods)
4. [Creating gregCore Mods](#creating-gregcore-mods)
5. [Creating gregCore Extensions](#creating-gregcore-extensions)
6. [Creating UserLibs / Universal Extensions](#creating-userlibs--universal-extensions)
7. [Defining Dependencies](#defining-dependencies)
8. [Plugin Manifest Files](#plugin-manifest-files)
9. [Publishing to datacentermods.com](#publishing-to-datacentermods.com)

---

## Modding Ecosystem Overview

### The MelonLoader & gregFramework Stack

gregModmanager manages mods and plugins built for the following ecosystem:

```text
┌─────────────────────────────────────────────────────┐
│                    Game (Unity Engine)              │
├─────────────────────────────────────────────────────┤
│                   MelonLoader (Mod Framework)       │
├──────────────────────┬──────────────────────────────┤
│   User Mods          │    gregCore / gregFramework  │
│   Plugins            │    (Extended Features)       │
│                      │                              │
│  ┌────────────────┐  │  ┌──────────────────────┐    │
│  │ Mod A (Modded  │  │  │ gregCore Mod         │    │
│  │ content)       │  │  │ (Extends gregCore)   │    │
│  │                │  │  │                      │    │
│  │ Depends on:    │  │  │ Depends on gregCore  │    │
│  │ - UserLib B    │  │  │ and UserLib Y        │    │
│  │ - Plugin C     │  │  │                      │    │
│  │                │  │  │                      │    │
│  └────────────────┘  │  └──────────────────────┘    │
│                      │                              │
│  ┌────────────────┐  │  ┌──────────────────────┐    │
│  │ UserLib B      │  │  │ UserLib Y            │    │
│  │ (Shared code)  │  │  │ (Shared utilities)   │    │
│  │                │  │  │                      │    │
│  │ No game mods   │  │  │ No game mods         │    │
│  │ depend on it   │  │  │ depend on it         │    │
│  │                │  │  │                      │    │
│  └────────────────┘  │  └──────────────────────┘    │
│                      │                              │
│  ┌────────────────┐  │                              │
│  │ Plugin C       │  │                              │
│  │ (Runtime hook) │  │                              │
│  │                │  │                              │
│  │ Universal or   │  │                              │
│  │ game-specific  │  │                              │
│  │                │  │                              │
│  └────────────────┘  │                              │
└──────────────────────┴──────────────────────────────┘
```

### Package Types & Decision Flowchart

```
START: "I want to create a mod/plugin"
  │
  ├─ "I'm modifying game content (textures, balance, gameplay)"
  │  └─> GAME MOD
  │
  ├─ "I'm adding runtime hooks (logging, debugging, network interception)"
  │  ├─ "My plugin works with ANY game"
  │  │  └─> MELONLOADER UNIVERSAL PLUGIN
  │  │
  │  └─ "My plugin is for one specific game"
  │     └─> MELONLOADER GAME-SPECIFIC PLUGIN
  │
  ├─ "I'm using gregFramework APIs (game abstraction, mod loading hooks)"
  │  ├─ "I'm creating game modifications with gregCore support"
  │  │  └─> GREGCORE MOD
  │  │
  │  └─ "I'm extending gregCore's features or services"
  │     └─> GREGCORE EXTENSION
  │
  └─ "I have shared code used by multiple mods/plugins"
     └─> USERLIB / UNIVERSAL EXTENSION
```

### Package Type Reference

| Type | Purpose | Load Point | Scope | Typical Use Case |
| :--- | :--- | :--- | :--- | :--- |
| **Game Mod** | Modify game content and behavior | Game startup (via MelonLoader) | Single game | New weapons, balance changes, new gameplay systems |
| **MelonLoader Plugin (Universal)** | Hook into any game's runtime | MelonLoader injection point | Any game using MelonLoader | Debug console, profiling tools, API bridges |
| **MelonLoader Plugin (Game-Specific)** | Hook into specific game runtime | MelonLoader injection point | Single game | Game-specific debugging, performance optimization |
| **gregCore Mod** | Extend game via gregFramework API | gregCore initialization | Single game (with gregCore) | High-level game modifications, cross-system features |
| **gregCore Extension** | Add features/services to gregCore | gregCore plugin system | System-wide (via gregCore) | Custom game abstraction layers, shared services |
| **UserLib** | Shared utility library | Link-time (referenced by other mods) | Multiple mods/plugins | Common utilities, shared data structures |

### Ecosystem Links & Resources

- **[datacentermods.com](https://datacentermods.com)** — Central mod repository and distribution
- **[gregframework.eu](https://gregframework.eu)** — gregCore/gregFramework documentation and APIs
- **[melonwiki.xyz](https://melonwiki.xyz)** — MelonLoader documentation and plugin examples
- **[GitHub: MelonLoader](https://github.com/LavaGang/MelonLoader)** — MelonLoader source and issue tracker

---

## Creating MelonLoader Universal / Game-Specific Plugins

### What is a MelonLoader Plugin?

A **MelonLoader Plugin** is a .NET assembly (DLL) that hooks into the game's runtime before or after MelonLoader's initialization. Plugins can:

- Intercept method calls and events
- Patch game code at runtime
- Log and debug game state
- Provide utilities to other mods
- Integrate external systems (e.g., Discord, Twitch, APIs)

**Universal vs. Game-Specific:**

| Aspect | Universal | Game-Specific |
| :--- | :--- | :--- |
| **Works with** | Any game using MelonLoader | Only one specific game |
| **Folder name** | `PluginID/` | `PluginID-GameID/` |
| **Metadata** | No game compatibility field | Specifies target game ID |
| **Use Case** | Debug tools, profilers, bridges | Game-specific optimizations, hooks |

### Folder Structure

```
MyPlugin/                                 # Package root
├── Plugins/
│   ├── MyPlugin.dll                     # Main plugin assembly
│   └── (optional dependencies)
├── metadata.json                        # Plugin manifest
└── README.md                            # Documentation
```

### Naming Conventions

- **Plugin ID:** Lowercase alphanumeric + hyphens (e.g., `my-awesome-plugin`)
- **DLL Name:** Match plugin ID in PascalCase (e.g., `MyAwesomePlugin.dll`)
- **Game-Specific:** Append game ID (e.g., `my-plugin-datacenter` for Data Center game)

### Annotating Plugin Metadata

Create a `metadata.json` file:

```json
{
  "id": "my-awesome-plugin",
  "name": "My Awesome Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "A brief description of what the plugin does",
  "type": "melonloader_plugin",
  "melonloader_version_min": "0.6.0",
  "melonloader_version_max": "0.7.x",
  "games": []                          # Empty = universal; or ["datacenter", "otherGame"]
}
```

### Minimal "Hello World" Plugin Example

#### C# Code: `HelloWorldPlugin.cs`

```csharp
using System;
using MelonLoader;

[assembly: MelonInfo(typeof(HelloWorldPlugin.HelloWorldPlugin), "Hello World Plugin", "1.0.0", "Your Name")]
[assembly: MelonGame("Company", "GameName")] // Empty strings for universal

namespace HelloWorldPlugin;

public class HelloWorldPlugin : MelonPlugin
{
    public override void OnInitializeMelon()
    {
        LoggerInstance.Msg("Hello from MelonLoader!");
    }

    public override void OnUpdate()
    {
        // Called every frame
    }

    public override void OnLateUpdate()
    {
        // Called after Update
    }
}
```

#### Project File: `HelloWorldPlugin.csproj`

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net6.0</TargetFramework>
    <AssemblyName>HelloWorldPlugin</AssemblyName>
    <Version>1.0.0</Version>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="MelonLoader" Version="0.6.1" />
  </ItemGroup>
</Project>
```

#### Metadata: `metadata.json`

```json
{
  "id": "hello-world-plugin",
  "name": "Hello World Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "A simple Hello World plugin for MelonLoader",
  "type": "melonloader_plugin",
  "melonloader_version_min": "0.6.0"
}
```

### Building and Packaging Plugins

```bash
# Build the plugin
dotnet build HelloWorldPlugin.csproj -c Release

# Create package structure
mkdir -p MyPlugin/Plugins
cp bin/Release/net6.0/HelloWorldPlugin.dll MyPlugin/Plugins/
cp metadata.json MyPlugin/
cp README.md MyPlugin/

# Create distribution ZIP
zip -r MyPlugin-1.0.0.zip MyPlugin/
```

### Distributing Plugins

#### Local Installation

Users install via **Mod Browser** → **Install from File** → select `MyPlugin-1.0.0.zip`

#### Publishing to datacentermods.com

1. Go to [datacentermods.com](https://datacentermods.com)
2. Create account and click **Upload Mod**
3. Select **MelonLoader Plugin** as type
4. Fill in metadata (name, description, version, screenshot)
5. Upload the `.zip` package
6. Submit for review
7. Once approved, mod appears in gregModmanager's Mod Browser

### Testing Plugins in gregModmanager

1. Place plugin ZIP in a test folder
2. Open gregModmanager
3. Select target game
4. Click **Mod Browser** → **Install from File**
5. Select plugin ZIP
6. Click **Install**
7. Launch game to test plugin
8. Check logs in gregModmanager for plugin output

---

## Creating Game Mods

### What is a Game Mod?

A **Game Mod** is a package of game assets and metadata that modifies or extends game content. Game mods typically include:

- Modified assets (textures, models, sounds)
- Configuration files for gameplay balance
- Metadata describing the mod (name, version, dependencies)

Game mods are loaded directly by the game engine, not via MelonLoader (though they may optionally depend on MelonLoader plugins for advanced features).

### Folder Structure

```
MyGameMod/                                    # Package root
├── content/                                  # Game mod content
│   ├── (any game-specific folder structure)
│   └── (depends on game's mod loader)
├── metadata.json                             # Mod manifest
└── README.md                                 # Documentation
```

### Naming Conventions

- **Mod ID:** Lowercase alphanumeric + hyphens (e.g., `my-game-mod`)
- **Version:** Semantic versioning (e.g., `1.2.3`)
- **Folder names:** Use descriptive names matching mod content

### Metadata File (`metadata.json`)

```json
{
  "id": "my-game-mod",
  "name": "My Game Mod",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "A brief description of what the mod adds or changes",
  "type": "game_mod",
  "games": ["datacenter"],
  "dependencies": {
    "optional": [
      { "id": "some-plugin", "version_min": "1.0.0" }
    ]
  }
}
```

### Minimal Example: Balance Mod

#### Folder Structure

```
BalanceMod/
├── content/
│   ├── balance/
│   │   ├── weapons.json
│   │   └── upgrades.json
│   └── descriptions/
│       └── en.json
├── metadata.json
└── README.md
```

#### `metadata.json`

```json
{
  "id": "balance-mod",
  "name": "Game Balance Rebalance",
  "version": "2.1.0",
  "author": "Balance Team",
  "description": "Rebalances weapons and upgrades for better game progression",
  "type": "game_mod",
  "games": ["datacenter"],
  "release_date": "2026-05-01",
  "dependencies": {}
}
```

#### `content/balance/weapons.json`

```json
{
  "weapons": [
    {
      "id": "rifle",
      "damage": 35,
      "fire_rate": 0.1,
      "cost": 1500
    },
    {
      "id": "pistol",
      "damage": 15,
      "fire_rate": 0.08,
      "cost": 750
    }
  ]
}
```

### Packaging Game Mods

```bash
# Create the structure
mkdir -p BalanceMod/content/balance
cp *.json BalanceMod/content/balance/
cp metadata.json BalanceMod/
cp README.md BalanceMod/

# Package as ZIP
zip -r BalanceMod-2.1.0.zip BalanceMod/
```

### Distributing Game Mods

Same process as plugins:

1. **Local:** Users install via gregModmanager's **Install from File**
2. **datacentermods.com:** Upload, review, then available in Mod Browser

---

## Creating gregCore Mods

### What is a gregCore Mod?

A **gregCore Mod** leverages the [gregFramework API](https://gregframework.eu) to create advanced game modifications that:

- Use game abstraction layers from gregCore
- Integrate with gregCore's mod loading and configuration systems
- Access cross-system features (logging, configuration, events)
- Are updated and managed by gregCore

### Folder Structure

```
MyGreatMod/
├── Plugins/
│   ├── MyGreatMod.dll                # Your compiled mod
│   └── (optional dependencies)
├── metadata.json                     # gregCore mod manifest
├── config.json                       # Default mod configuration
└── README.md
```

### Minimal gregCore Mod Example

#### C# Code: `MyGreatMod.cs`

```csharp
using System;
using GregFramework;
using GregFramework.Logging;

namespace MyGreatMod;

[gregMod("my-great-mod", "1.0.0")]
public class MyGreatMod : gregModBase
{
    private ILogger _logger;

    public override void OnInitialize()
    {
        _logger = LogProvider.GetLogger("MyGreatMod");
        _logger.Info("Initializing My Great Mod v1.0.0");

        // Register event handlers, initialize systems, etc.
        GetService<IGameEventBus>().Subscribe<GameStartedEvent>(OnGameStarted);
    }

    private void OnGameStarted(GameStartedEvent evt)
    {
        _logger.Info($"Game started on level {evt.LevelName}");
    }

    public override void OnShutdown()
    {
        _logger.Info("My Great Mod shutting down");
    }
}
```

#### Metadata: `metadata.json`

```json
{
  "id": "my-great-mod",
  "name": "My Great Mod",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "An advanced mod using gregCore features",
  "type": "gregcore_mod",
  "games": ["datacenter"],
  "gregcore_version_min": "2.0.0",
  "dependencies": {
    "required": [
      { "id": "gregcore", "version_min": "2.0.0" }
    ],
    "optional": [
      { "id": "logging-plugin", "version_min": "1.0.0" }
    ]
  }
}
```

#### Configuration: `config.json`

```json
{
  "mod": "my-great-mod",
  "settings": {
    "enabled": true,
    "debug_logging": false,
    "feature_x_enabled": true
  }
}
```

### Using gregCore APIs

#### Logging

```csharp
_logger.Debug("Debug message");
_logger.Info("Information message");
_logger.Warn("Warning message");
_logger.Error("Error message", exception);
```

#### Configuration Management

```csharp
var config = GetService<IConfigProvider>().GetModConfig("my-great-mod");
var debugMode = config.GetBool("debug_logging", false);
var maxItems = config.GetInt("max_items", 100);
```

#### Event System

```csharp
// Subscribe to events
var eventBus = GetService<IGameEventBus>();
eventBus.Subscribe<PlayerDiedEvent>(OnPlayerDied);
eventBus.Subscribe<ItemPickedUpEvent>(OnItemPickedUp);

// Emit events
eventBus.Emit(new CustomEvent { Data = "..." });
```

#### Service Resolution

```csharp
var gameState = GetService<IGameStateService>();
var inventory = GetService<IInventoryService>();
var ui = GetService<IUIService>();
```

### Testing gregCore Mods

1. Install gregCore in target game
2. Place mod DLL in `Plugins/` folder
3. Place `metadata.json` and `config.json` in mod root
4. Open gregModmanager, select game
5. Install mod via **Install from File**
6. Launch game and check logs for mod initialization

### Publishing gregCore Mods

Upload to [datacentermods.com](https://datacentermods.com) with type `gregcore_mod`. Ensure `metadata.json` lists gregCore as a required dependency.

---

## Creating gregCore Extensions

### What is a gregCore Extension?

A **gregCore Extension** is a specialized plugin that doesn't modify game content, but instead extends gregCore's capabilities by:

- Providing new game abstraction services
- Implementing feature hooks or extension points
- Offering utility APIs to other mods
- Adding logging, monitoring, or profiling features

### Folder Structure

```
MyGreatExtension/
├── Plugins/
│   ├── MyGreatExtension.dll
│   └── (optional dependencies)
├── metadata.json
└── README.md
```

### Example: Custom Game State Service Extension

#### C# Code: `MyGameStateExtension.cs`

```csharp
using GregFramework;
using GregFramework.Extensibility;

namespace MyGameStateExtension;

[gregExtension("my-game-state-ext", "1.0.0")]
public class MyGameStateExtension : gregExtensionBase
{
    public override void OnRegisterServices(IServiceCollection services)
    {
        // Register custom service
        services.AddSingleton<ICustomGameStateService, CustomGameStateService>();
    }

    public override void OnInitialize()
    {
        // Extension initialized
    }
}

public interface ICustomGameStateService
{
    string GetCurrentPhase();
    int GetEnemyCount();
}

public class CustomGameStateService : ICustomGameStateService
{
    public string GetCurrentPhase() => "Phase 1";
    public int GetEnemyCount() => 42;
}
```

#### Metadata: `metadata.json`

```json
{
  "id": "my-game-state-ext",
  "name": "Custom Game State Extension",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Provides custom game state tracking for gregCore",
  "type": "gregcore_extension",
  "gregcore_version_min": "2.0.0",
  "dependencies": {
    "required": [
      { "id": "gregcore", "version_min": "2.0.0" }
    ]
  }
}
```

### Publishing gregCore Extensions

Similar to gregCore mods, upload to [datacentermods.com](https://datacentermods.com) with type `gregcore_extension`.

---

## Creating UserLibs / Universal Extensions

### What is a UserLib?

A **UserLib** (User Library) is a shared library DLL that is NOT itself a game mod or plugin, but is instead referenced by multiple mods to:

- Avoid code duplication
- Provide common utilities and algorithms
- Share data structures across mods
- Centralize logging, configuration, or API wrappers

### When to Create a UserLib

- You find yourself copying the same utility code to multiple mods
- You want to provide a common API to plugin developers
- You're managing frequently changing shared code

### Folder Structure

```
CommonUtils/
├── Plugins/
│   ├── Dependencies/                 # Shared libraries go here
│   │   └── CommonUtils.dll
├── metadata.json
└── README.md
```

### Naming Convention

- **Library ID:** Lowercase with hyphens (e.g., `common-utils`)
- **DLL Name:** PascalCase matching ID (e.g., `CommonUtils.dll`)

### Example: Common Utilities Library

#### C# Code: `CommonUtils.cs`

```csharp
namespace CommonUtils;

public static class StringHelpers
{
    public static string Truncate(string text, int maxLength)
    {
        return text.Length > maxLength
            ? text.Substring(0, maxLength) + "..."
            : text;
    }
}

public static class MathHelpers
{
    public static int Clamp(int value, int min, int max)
    {
        return Math.Max(min, Math.Min(max, value));
    }
}

public interface ILogger
{
    void Log(string message);
    void Error(string message, Exception ex);
}

public class ConsoleLogger : ILogger
{
    public void Log(string message) => Console.WriteLine($"[LOG] {message}");
    public void Error(string message, Exception ex) => Console.WriteLine($"[ERROR] {message}\n{ex}");
}
```

#### Project File: `CommonUtils.csproj`

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net6.0</TargetFramework>
    <AssemblyName>CommonUtils</AssemblyName>
    <Version>1.0.0</Version>
  </PropertyGroup>
</Project>
```

#### Metadata: `metadata.json`

```json
{
  "id": "common-utils",
  "name": "Common Utilities Library",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Shared utilities for mods and plugins",
  "type": "userlib",
  "dependencies": {}
}
```

### Using a UserLib in Another Mod

In another mod's `.csproj`, reference the UserLib:

```xml
<ItemGroup>
  <ProjectReference Include="..\CommonUtils\CommonUtils.csproj" />
  <!-- Or via NuGet if published as a package -->
  <PackageReference Include="CommonUtils" Version="1.0.0" />
</ItemGroup>
```

Then in your mod code:

```csharp
using CommonUtils;

var truncated = StringHelpers.Truncate("Hello World", 5); // "Hello..."
var clamped = MathHelpers.Clamp(50, 0, 100);               // 50
```

### Publishing UserLibs

1. If distributing as DLL (local or via gregModmanager):
   - Package as ZIP with `Plugins/Dependencies/CommonUtils.dll`
   - Include `metadata.json`
   - Upload to [datacentermods.com](https://datacentermods.com)

2. If distributing via NuGet:
   - Create a NuGet package (`.nupkg`)
   - Push to NuGet.org or private feed
   - Mods reference via `<PackageReference>`

### Version Conflict Resolution

If multiple mods depend on different versions of the same UserLib:

- gregModmanager detects the conflict and alerts the user
- User must choose one version or find mods compatible with the same UserLib version
- Best practice: Maintain backward compatibility in UserLib minor/patch versions

---

## Defining Dependencies

### Dependency Model

Mods can depend on:

- **Other mods** (e.g., "Advanced Gameplay Mod" depends on "Base Framework Mod")
- **Plugins** (e.g., "My Mod" depends on "Logging Plugin")
- **UserLibs** (e.g., "My Mod" depends on "Common Utils")
- **gregCore** (e.g., "gregCore Mod" depends on gregCore v2.0+)

### Dependency Declaration Syntax

In `metadata.json`:

```json
{
  "dependencies": {
    "required": [
      { "id": "base-mod", "version_min": "1.0.0", "version_max": "2.x" },
      { "id": "common-utils", "version_min": "1.0.0" }
    ],
    "optional": [
      { "id": "logging-plugin", "version_min": "1.0.0" }
    ],
    "conflicts": [
      { "id": "incompatible-mod", "reason": "Uses conflicting game systems" }
    ]
  }
}
```

### Semantic Versioning & Version Ranges

gregModmanager uses [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`

**Version Range Operators:**

| Operator | Example | Matches |
|----------|---------|---------|
| `exact` | `"1.2.3"` | Only 1.2.3 |
| `^` (caret) | `"^1.2.3"` | 1.2.3 to <2.0.0 (compatible with 1.x) |
| `~` (tilde) | `"~1.2.3"` | 1.2.3 to <1.3.0 (patch-level updates) |
| `*` | `"1.2.*"` | Any 1.2.x version |
| `x` | `"1.x"` | Any 1.x version |
| Range | `">=1.0.0, <2.0.0"` | Between 1.0.0 and 2.0.0 (exclusive) |

### Examples

#### Simple Dependency

```json
{
  "dependencies": {
    "required": [
      { "id": "framework-mod", "version_min": "1.0.0" }
    ]
  }
}
```

Requires `framework-mod` version 1.0.0 or newer.

#### Multi-Dependency with Constraints

```json
{
  "dependencies": {
    "required": [
      { "id": "gregcore", "version_min": "2.0.0", "version_max": "2.x" },
      { "id": "common-utils", "version_min": "1.0.0" },
      { "id": "logging-plugin", "version_min": "1.5.0" }
    ]
  }
}
```

Requires:
- gregCore 2.0.0 to 2.99.99
- common-utils 1.0.0+
- logging-plugin 1.5.0+

#### Optional Dependencies

```json
{
  "dependencies": {
    "optional": [
      { "id": "advanced-ui-plugin", "version_min": "1.0.0" }
    ]
  }
}
```

If `advanced-ui-plugin` is installed, use it; otherwise, continue without it.

#### Conflict Declarations

```json
{
  "dependencies": {
    "conflicts": [
      { "id": "incompatible-balance-mod", "reason": "Modifies the same game systems" },
      { "id": "old-framework", "version_max": "0.9.x", "reason": "Uses obsolete API" }
    ]
  }
}
```

Warns users if incompatible mods are installed simultaneously.

### Dependency Resolution in gregModmanager

When a user installs a mod:

1. gregModmanager reads `metadata.json` and extracts dependencies
2. It builds a **dependency graph** (which mods depend on what)
3. It resolves the graph:
   - Missing required dependencies → Installation blocked, user prompted to install them
   - Version conflicts → User alerted; must choose compatible version or uninstall one mod
   - Circular dependencies → Installation blocked with error
   - Satisfied dependencies → Installation proceeds
4. On game launch, mods are loaded in dependency order

### Best Practices for Dependencies

1. **Be Specific:** Use `version_min` to specify minimum compatible version
2. **Use Semantic Versioning:** Follow MAJOR.MINOR.PATCH
3. **Avoid Over-Specification:** Don't lock to exact versions unless necessary
4. **Test Dependency Resolution:** Test with multiple dependency combinations before releasing
5. **Document:** In README, explain why each dependency is required
6. **Minimize:** Only depend on truly necessary mods/plugins

---

## Plugin Manifest Files

### Introduction

The **manifest file** (`metadata.json`) declares all metadata about a mod or plugin, enabling gregModmanager to:

- Display mod information in the UI
- Resolve dependencies before installation
- Detect conflicts and incompatibilities
- Track versions and updates
- Enable filtering and searching

### Manifest Format & Field Reference

gregModmanager uses **JSON** for manifest files. Below is a comprehensive field reference:

#### Core Identity

| Field | Type | Required | Example | Purpose |
|-------|------|----------|---------|---------|
| `id` | string | ✓ | `"my-mod"` | Unique identifier (lowercase, hyphens) |
| `name` | string | ✓ | `"My Great Mod"` | Display name |
| `version` | string | ✓ | `"1.2.0"` | Semantic version (MAJOR.MINOR.PATCH) |
| `author` | string | ✓ | `"John Doe"` | Creator name |
| `description` | string | ✓ | `"A mod that does X"` | Brief description (<160 chars) |
| `type` | string | ✓ | `"game_mod"` | Package type (see below) |

#### Package Type Codes

| Code | Meaning |
|------|---------|
| `game_mod` | Game modification (content/balance) |
| `melonloader_plugin` | MelonLoader plugin (universal or game-specific) |
| `gregcore_mod` | gregCore-based mod |
| `gregcore_extension` | gregCore service extension |
| `userlib` | Shared library/utility |

#### Version & Compatibility

| Field | Type | Example | Purpose |
|-------|------|---------|---------|
| `melonloader_version_min` | string | `"0.6.0"` | Minimum MelonLoader version (plugins only) |
| `melonloader_version_max` | string | `"0.7.x"` | Maximum MelonLoader version (plugins only) |
| `gregcore_version_min` | string | `"2.0.0"` | Minimum gregCore version (gregCore mods only) |
| `games` | array | `["datacenter"]` | Target games (empty = universal) |

#### Release & Distribution

| Field | Type | Example | Purpose |
|-------|------|---------|---------|
| `release_date` | string (ISO 8601) | `"2026-05-01"` | Release date |
| `release_notes` | string | `"Added X, fixed Y"` | Changelog for this version |
| `homepage` | string | `"https://example.com"` | Mod homepage/documentation |
| `source_url` | string | `"https://github.com/..."` | Source repository (if open-source) |

#### Content & Metadata

| Field | Type | Example | Purpose |
|-------|------|---------|---------|
| `dependencies` | object | (see Dependencies section) | Mod/plugin dependencies |
| `tags` | array | `["balance", "gameplay"]` | Search tags |
| `license` | string | `"MIT"` | License type |
| `screenshots` | array | `[{ "url": "...", "caption": "..." }]` | Visual previews |

### Manifest: Schema Versioning

```json
{
  "schema_version": "1.0",       // Optional: explicitly declare schema version
  "id": "my-mod",
  "name": "My Mod",
  // ... rest of fields
}
```

If gregModmanager encounters an unknown `schema_version`, it warns the user that the mod may require a newer version of gregModmanager.

### Manifest: Example Configurations

#### Example 1: Simple Mod (No Dependencies)

```json
{
  "id": "simple-balance",
  "name": "Simple Balance Tweaks",
  "version": "1.0.0",
  "author": "BalanceTeam",
  "description": "Tweaks weapon and upgrade costs",
  "type": "game_mod",
  "games": ["datacenter"],
  "release_date": "2026-05-01"
}
```

#### Example 2: Mod with Multiple Dependencies

```json
{
  "id": "advanced-gameplay",
  "name": "Advanced Gameplay System",
  "version": "2.0.0",
  "author": "GameDesigners",
  "description": "Adds new gameplay mechanics with gregCore support",
  "type": "gregcore_mod",
  "games": ["datacenter"],
  "release_date": "2026-05-01",
  "release_notes": "Added UI improvements, fixed balance issues",
  "gregcore_version_min": "2.0.0",
  "dependencies": {
    "required": [
      { "id": "gregcore", "version_min": "2.0.0" },
      { "id": "common-utils", "version_min": "1.0.0" }
    ],
    "optional": [
      { "id": "advanced-ui-plugin", "version_min": "1.0.0" }
    ]
  },
  "homepage": "https://github.com/...",
  "source_url": "https://github.com/...",
  "license": "MIT",
  "tags": ["gameplay", "gregcore", "advanced"]
}
```

#### Example 3: MelonLoader Plugin (Multi-Game)

```json
{
  "id": "universal-debug-plugin",
  "name": "Universal Debug Plugin",
  "version": "1.5.0",
  "author": "DebugTeam",
  "description": "Debug console and profiling for any MelonLoader game",
  "type": "melonloader_plugin",
  "games": [],                                    // Empty = universal
  "melonloader_version_min": "0.6.0",
  "melonloader_version_max": "0.7.x",
  "release_date": "2026-05-01",
  "homepage": "https://github.com/...",
  "license": "MIT"
}
```

#### Example 4: Game-Specific Plugin

```json
{
  "id": "datacenter-perf-optimizer",
  "name": "Data Center Performance Optimizer",
  "version": "1.0.0",
  "author": "PerfTeam",
  "description": "Optimizes Data Center rendering and physics performance",
  "type": "melonloader_plugin",
  "games": ["datacenter"],                       // Game-specific
  "melonloader_version_min": "0.6.0",
  "release_date": "2026-05-01",
  "tags": ["performance", "optimization"]
}
```

#### Example 5: gregCore Extension

```json
{
  "id": "custom-game-state",
  "name": "Custom Game State Extension",
  "version": "1.0.0",
  "author": "ExtensionTeam",
  "description": "Extends gregCore with custom game state tracking",
  "type": "gregcore_extension",
  "gregcore_version_min": "2.0.0",
  "dependencies": {
    "required": [
      { "id": "gregcore", "version_min": "2.0.0" }
    ]
  },
  "homepage": "https://github.com/..."
}
```

#### Example 6: Package with Placeable Objects & Assets

```json
{
  "id": "new-weapons-pack",
  "name": "New Weapons & Equipment Pack",
  "version": "1.0.0",
  "author": "ContentTeam",
  "description": "10 new weapons and 5 new equipment items",
  "type": "game_mod",
  "games": ["datacenter"],
  "release_date": "2026-05-01",
  "release_notes": "Initial release with 10 weapons",
  "screenshots": [
    {
      "url": "https://cdn.example.com/screenshots/weapon1.png",
      "caption": "Laser Rifle"
    }
  ],
  "tags": ["content", "weapons", "equipment"]
}
```

#### Example 7: UserLib with Version Conflict Handling

```json
{
  "id": "common-utils",
  "name": "Common Utilities Library",
  "version": "2.0.0",
  "author": "UtilityTeam",
  "description": "Shared utilities for mods",
  "type": "userlib",
  "release_date": "2026-05-01",
  "release_notes": "Breaking changes: StringHelpers API refactored",
  "dependencies": {
    "conflicts": [
      { "id": "common-utils", "version_max": "1.x", "reason": "API incompatible" }
    ]
  },
  "license": "MIT"
}
```

### Manifest Validation

In gregModmanager, you can validate a manifest before publishing:

1. Go to **My Uploads** tab
2. Click **Validate Manifest**
3. Select `metadata.json` file
4. gregModmanager checks for:
   - Required fields present
   - Semantic versioning format
   - ID uniqueness (against known mods)
   - Dependency version syntax
5. Shows errors or "Validation OK"

---

## Publishing Mods to datacentermods.com

### Account Registration & Login

1. Go to [datacentermods.com](https://datacentermods.com)
2. Click **Sign Up**
3. Enter email, username, password
4. Verify email address
5. Create API token: Account → **API Tokens** → **Generate Token**

### Publishing via gregModmanager

1. Open gregModmanager and go to **My Uploads** tab
2. Click **Login to datacentermods.com**
3. Enter username and API token
4. Click **New Mod Release**
5. Fill in metadata:
   - **Mod ID:** Must match your mod's `id` in `metadata.json`
   - **Version:** Semantic version (e.g., `1.0.0`)
   - **Type:** Select package type
   - **Description:** Markdown-formatted description
   - **Screenshot:** Upload PNG/JPG (recommended 800x600)
   - **Files:** Select mod package ZIP
6. Review and submit
7. Mod enters review queue (typically 1-2 days)
8. Once approved, mod appears in Mod Browser

### Publishing via Web (datacentermods.com)

Alternatively, use the web interface:

1. Log in to [datacentermods.com](https://datacentermods.com)
2. Go to **Dashboard** → **Upload Mod**
3. Follow the same steps as above

### Metadata Requirements

For successful submission:

- **Mod ID:** Lowercase alphanumeric + hyphens only
- **Version:** Must follow `MAJOR.MINOR.PATCH` format
- **Description:** Clear, concise, in English (translations welcome)
- **Type:** Must match one of the defined types
- **Screenshot:** High-resolution image showing mod in action
- **Dependencies:** Accurately list all required/optional mods
- **License:** Choose from MIT, GPL-3.0, Apache-2.0, etc.

### Review Process

datacentermods.com moderators review submissions for:

- ✓ Correct metadata format
- ✓ No malware or suspicious code (scanned automatically)
- ✓ Appropriate content (follows community guidelines)
- ✓ Working download link and valid package
- ✓ Accurate description and tags

Rejection reasons are typically:

- ✗ Missing dependencies listed
- ✗ Incompatible with current gregModmanager version
- ✗ Suspicious code or external downloads
- ✗ Offensive or inappropriate content

### Updating an Existing Release

1. Go to **My Uploads** → select mod
2. Click **New Version**
3. Upload updated `.zip` package
4. Update `version` and `release_notes` fields
5. Submit for review
6. Once approved, new version is available to users as an update

### Version History & Rollback

- Users can see all past versions in mod details
- Users can manually downgrade to an older version if needed
- Current recommended version is marked in the browser

### Rate Limiting & Cooldown

gregModmanager enforces upload rate limiting to prevent spam:

- **Minimum:** 30 seconds between upload attempts
- **Window:** 5 uploads per 10 minutes
- **Feedback:** UI displays countdown timer if rate limit hit

See `wiki/Steam-Limits-and-Cooldown.md` for details.

---

## Support & Resources

- **gregFramework Docs:** [gregframework.eu](https://gregframework.eu)
- **MelonLoader Docs:** [melonwiki.xyz](https://melonwiki.xyz)
- **datacentermods.com:** [datacentermods.com](https://datacentermods.com)
- **GitHub Issues:** [github.com/mleem97/gregModmanager/issues](https://github.com/mleem97/gregModmanager/issues)
- **Example Manifests:** `docs/examples/manifests/` in this repository

---

**Last Updated:** May 2026  
**Version:** v1.5.1
