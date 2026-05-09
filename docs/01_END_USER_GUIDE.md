# gregModmanager — End-User Guide

**Last Updated:** May 2026  
**Version:** v1.5.1

---

## Table of Contents

1. [What is gregModmanager?](#what-is-gregmodmanager)
2. [System Requirements](#system-requirements)
3. [Installation & Platform Support](#installation--platform-support)
4. [Getting Started](#getting-started)
5. [User Interface Overview](#user-interface-overview)
6. [Managing Games](#managing-games)
7. [Managing Mods & Plugins](#managing-mods--plugins)
8. [Troubleshooting & FAQ](#troubleshooting--faq)
9. [Localization & Language Support](#localization--language-support)

---

## What is gregModmanager?

### Purpose & Scope

**gregModmanager** is a cross-platform desktop application designed to simplify the management of mods and plugins for games powered by MelonLoader and the gregFramework ecosystem. It provides an intuitive interface to:

- **Browse and install mods** from [datacentermods.com](https://datacentermods.com) directly within the application
- **Manage mod load order** and dependencies automatically
- **Handle multiple games** and mod profiles seamlessly
- **Track and resolve conflicts** between incompatible mods
- **Update mods** with a single click
- **Package and distribute** your own mods and plugins

### Supported Use Cases

1. **Installing and enabling mods** from datacentermods.com or local files
2. **Managing load order** and mod dependencies
3. **Switching between profiles** for different gameplay experiences
4. **Resolving dependency conflicts** with clear error messages
5. **Updating mods** to newer versions
6. **Packaging mods** for sharing or publishing
7. **Publishing mods** to datacentermods.com

### Supported Package Types

gregModmanager recognizes and manages four primary package types:

| Type | Purpose | Example |
|------|---------|---------|
| **Game Mods** | Content modifications for specific games | New gameplay mechanics, balance changes |
| **MelonLoader Plugins** | Universal or game-specific plugins that hook into the game runtime | Debug tools, UI enhancers, integration layers |
| **gregCore Mods** | Mods and extensions built on the gregFramework API | Advanced gameplay systems, cross-game features |
| **UserLibs** | Shared libraries used by multiple mods to avoid duplication | Common utilities, data structures, logging frameworks |

### Supported Platforms

- **Windows 10/11** (x64) — primary platform, fully supported
- **Linux** (x64) — Debian, Ubuntu, Fedora, Arch, and other distributions supported via AppImage, .deb, .rpm
- **macOS** — planned for future releases (Intel and Apple Silicon)

### External Ecosystems

gregModmanager integrates with the following key resources:

- **[datacentermods.com](https://datacentermods.com)** — Central mod repository and distribution platform
- **[gregframework.eu](https://gregframework.eu)** — Official gregCore/gregFramework documentation
- **[melonwiki.xyz](https://melonwiki.xyz)** — MelonLoader documentation and resources

---

## System Requirements

### Hardware

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 2-core @ 2.0 GHz | 4-core @ 2.5 GHz or better |
| **RAM** | 2 GB | 4 GB or more |
| **Disk Space** | 500 MB free | 2+ GB free (for mod cache) |
| **Display** | 1024×768 @ 96 DPI | 1920×1080 @ 96 DPI or higher |

### Software Requirements

#### Windows 10/11 (x64)

- **OS:** Windows 10 (version 1909+) or Windows 11
- **.NET Runtime:** .NET 9 (bundled in installer; standalone install recommended if using portable)
- **Visual C++ Redistributable:** MSVC Runtime v14.2+ (Visual Studio 2019 redistributable)
- **Steam Client:** Optional, required only for Steam integration
- **Administrator Privileges:** Not required for per-user installation; required for per-machine installation

#### Linux (x64)

Tested and supported on:

- **Debian 11+** / **Ubuntu 20.04 LTS+** / **Linux Mint 20+**
- **Fedora 36+** / **CentOS Stream 8+** / **RHEL 8+**
- **Arch Linux** and derivatives
- **openSUSE Leap 15.4+**

Required libraries (usually pre-installed):

```bash
# Debian/Ubuntu
sudo apt-get install libx11-6 libxrandr2 libxcursor1 libxi6

# Fedora/RHEL
sudo dnf install libX11 libXrandr libXcursor libXi

# Arch
sudo pacman -S libx11 libxrandr libxcursor libxi
```

Display server support:

- **X11** — fully supported
- **Wayland** — partially supported; some UI rendering may differ

#### macOS (Planned)

- **Target OS:** macOS 11+ (Big Sur and later)
- **Architectures:** Intel x86-64, Apple Silicon (ARM64)
- **Timeline:** Q3 2026 (estimated)

### Steam Integration

- **Steam Client:** Version 2024-01-01 or later
- **SteamAPI:** Automatically loaded from game installation
- **SteamAPI Requirements:** `steam_api64.dll` (Windows) or `libsteam_api.so` (Linux)

---

## Installation & Platform Support

### Windows

#### Option 1: Installer (Recommended)

1. Download the latest `GregModmanager-v1.5.1-setup.exe` from [releases](https://github.com/mleem97/gregModmanager/releases)
2. Run the installer
3. Choose installation mode:
   - **Per-user** — installs to `%LocalAppData%\gregModmanager` (default, no admin needed)
   - **Per-machine** — installs to `C:\Program Files\gregModmanager` (requires admin)
4. Follow the setup wizard
5. Launch gregModmanager from the Start menu or desktop shortcut

**Default Paths:**

- Per-user: `C:\Users\<Username>\AppData\Local\gregModmanager`
- Per-machine: `C:\Program Files\gregModmanager`
- Config/logs: `%AppData%\gregModmanager`

**To Uninstall:**

- Control Panel → Programs → Uninstall a program → select gregModmanager
- Or: Run the installer again and select "Uninstall"

#### Option 2: Portable (No Installation)

1. Download `GregModmanager-v1.5.1-portable.zip`
2. Extract to any folder (e.g., `C:\Tools\gregModmanager`)
3. Run `GregModmanager.exe` directly
4. (Optional) Create a shortcut on the desktop or Start menu

**Advantages:**

- No registry changes
- Easy to keep multiple versions
- Can run from USB drive
- No admin privileges required

**Note:** First run may prompt for .NET runtime installation if not present on system.

### Linux

#### Option 1: AppImage (Easiest)

```bash
# Download
wget https://github.com/mleem97/gregModmanager/releases/download/v1.5.1/GregModmanager-v1.5.1.AppImage

# Make executable
chmod +x GregModmanager-v1.5.1.AppImage

# Run
./GregModmanager-v1.5.1.AppImage
```

**Advantages:**

- Self-contained, no dependencies
- Works on any Linux distro
- No installation needed
- Easy to run from anywhere

#### Option 2: .deb (Debian/Ubuntu)

```bash
# Download
wget https://github.com/mleem97/gregModmanager/releases/download/v1.5.1/gregmodmanager_1.5.1_amd64.deb

# Install
sudo apt-get install ./gregmodmanager_1.5.1_amd64.deb

# Launch
gregmodmanager
```

#### Option 3: .rpm (Fedora/RHEL)

```bash
# Download
wget https://github.com/mleem97/gregModmanager/releases/download/v1.5.1/gregmodmanager-1.5.1-1.x86_64.rpm

# Install
sudo dnf install ./gregmodmanager-1.5.1-1.x86_64.rpm

# Launch
gregmodmanager
```

#### Option 4: Arch

```bash
# Using AUR (if available)
yay -S gregmodmanager

# Or manual .pkg.tar.zst
wget https://github.com/mleem97/gregModmanager/releases/download/v1.5.1/gregmodmanager-1.5.1-1-x86_64.pkg.tar.zst
sudo pacman -U ./gregmodmanager-1.5.1-1-x86_64.pkg.tar.zst
```

#### Option 5: Portable (Manual)

```bash
# Download and extract
wget https://github.com/mleem97/gregModmanager/releases/download/v1.5.1/gregmodmanager-v1.5.1-linux-x64.tar.gz
tar -xzf gregmodmanager-v1.5.1-linux-x64.tar.gz
cd gregmodmanager

# Run
./GregModmanager
```

**Default Paths (Linux):**

- Config/logs: `~/.config/gregModmanager` or `~/.local/share/gregModmanager`
- Mods cache: `~/.cache/gregModmanager`

### First-Run Setup

Upon first launch, gregModmanager will guide you through:

1. **Language Selection** — Choose your preferred language (currently English, German, Spanish, more planned)
2. **Game Library Scan** — Auto-detect installed games (Steam, custom paths)
3. **Game Configuration** — Set game root directory, MelonLoader version
4. **MelonLoader Installation** — Optionally install MelonLoader if not present
5. **Privacy & Telemetry** — Opt-in/out of diagnostic data collection

### Auto-Updater

gregModmanager includes an auto-update system:

- **Check Frequency:** Daily on startup
- **Notification:** Popup alert if a new version is available
- **Download & Install:** One-click update (restarts the app)
- **Portable Update:** Manual download and extract (same as fresh install)
- **Rollback:** Keep previous version in a separate folder and switch back if needed

**Disable Auto-Update:**
Settings → Privacy → ☐ Check for updates automatically

### Troubleshooting Installation

| Problem | Solution |
|---------|----------|
| **Windows SmartScreen warning** | Click "More info" → "Run anyway" |
| **"Missing .NET runtime" error** | Download [.NET 9 Runtime](https://dotnet.microsoft.com/download/dotnet/9.0) and install |
| **"Permission denied" on Linux** | Run `chmod +x ./gregmodmanager` |
| **AppImage won't run on Wayland** | Try running with `QT_QPA_PLATFORM=xcb` environment variable |
| **AV/antivirus blocking** | Add gregModmanager folder to AV whitelist |

---

## Getting Started

### Minimal End-to-End Example

This section walks through installing a mod from start to finish.

#### Step 1: Launch gregModmanager

- **Windows:** Click the Start menu shortcut or run `GregModmanager.exe`
- **Linux:** Run `gregmodmanager` from terminal or click the desktop launcher

#### Step 2: Detect or Add a Game

On first launch:

1. Go to **Games** tab (left sidebar)
2. Click **+ Add Game**
3. gregModmanager scans for Steam or manual paths
4. Select the game you want to mod
5. Click **Add** to confirm

Or if your game is already detected, click **Select** to activate it.

#### Step 3: Install MelonLoader (if needed)

1. Go to **Settings** → **Game Setup**
2. If MelonLoader is not installed, click **Install MelonLoader**
3. Choose MelonLoader version (stable recommended)
4. Wait for installation to complete
5. Close the game if running

#### Step 4: Browse and Install a Mod

1. Click **Mod Browser** tab (center)
2. Browse mods from datacentermods.com or search for a specific mod
3. Click on a mod to view details (description, dependencies, version)
4. Click **Install** button
5. Review dependencies (if any) and confirm
6. Wait for download and installation to complete
7. Mod appears in your **Installed Mods** list

#### Step 5: Launch the Game

1. Click **Launch Game** button (main toolbar or Settings)
2. Game launches with MelonLoader and your installed mods active
3. Enjoy!

---

## User Interface Overview

### Main Window Layout

```
┌─────────────────────────────────────────────────────────┐
│  gregModmanager v1.5.1                        ⚙ ? _ □ ✕ │
├────────────┬─────────────────────────────────────────────┤
│            │                                             │
│  ≡ MENU    │  📦 Installed Mods (Game: [Selector ▼])     │
│            │                                             │
│ 🎮 Games   │  ┌─────────────────────────────────────┐    │
│ 📦 Mods    │  │ Mod Name         │ Version │ Status  │    │
│ 👤 Profile │  │ ✓ ModA           │ 1.2.0   │ ✓ On    │    │
│ ⚙ Settings │  │ ✓ ModB           │ 2.0.1   │ ✓ On    │    │
│ 📋 Logs    │  │ ○ Disabled Mod   │ 1.0.0   │ ○ Off   │    │
│            │  │                  │         │         │    │
│            │  └─────────────────────────────────────┘    │
│            │                                             │
│            │  [Install] [Remove] [Update] [Launch Game]  │
│            │                                             │
│  [Console] │  ┌─────────────────────────────────────┐    │
│            │  │ Last 10 log entries...              │    │
│            │  └─────────────────────────────────────┘    │
│            │                                             │
└────────────┴─────────────────────────────────────────────┘
```

### Main Areas

#### Left Sidebar (Navigation)

- **≡ Menu** — Toggle navigation sidebar
- **🎮 Games** — Manage installed games, switch active game
- **📦 Mods** — Browse mod browser, search, filter by category
- **👤 Profile** — Create, switch, import/export mod profiles
- **⚙ Settings** — Game paths, MelonLoader version, language, update settings
- **📋 Logs** — View debug logs, export logs for bug reports

#### Center Panel (Main Content)

- **Game Selector** — Dropdown to quickly switch active game
- **Installed Mods List** — Shows all mods for the active game
  - Checkboxes enable/disable individual mods
  - Drag-and-drop to reorder load order
  - Right-click for context menu (delete, view details, open folder)
- **Search & Filter** — Filter mods by name, category, or enabled status
- **Action Buttons** — Install, Update, Remove, Launch Game

#### Bottom Panel (Console/Logs)

- Real-time output from operations (install, uninstall, game launch)
- Click to expand, scroll to view history
- Double-click to open full log viewer

### Navigation Tabs (Top)

- **Installed Mods** — Currently installed mods for the active game
- **Mod Browser** — Browse/search datacentermods.com
- **My Uploads** — Mods you have published (if logged in)

### Toolbar Actions

| Action | Shortcut | Purpose |
|--------|----------|---------|
| Install | — | Install selected mod from browser |
| Remove | Delete | Uninstall selected mod |
| Update All | Ctrl+U | Update all mods with available updates |
| Launch Game | Ctrl+L | Launch the active game with mods |
| Settings | Ctrl+, | Open settings dialog |
| Help | F1 | Open documentation (online) |
| About | — | View version info and credits |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+L` | Launch game |
| `Ctrl+U` | Update all mods |
| `Ctrl+,` | Open settings |
| `Ctrl+F` | Focus search bar |
| `Del` | Delete selected mod |
| `F1` | Open help |
| `F5` | Refresh mod list |

### Context Menu (Right-Click on Mod)

- **View Details** — Open mod information panel
- **Open Folder** — Open mod folder in file explorer
- **Check for Updates** — Check if update is available
- **Uninstall** — Remove mod
- **Move to Profile** — Move mod to different profile
- **Report Issue** — Report mod as broken/incompatible

---

## Managing Games

### Adding a Game

1. Click **Games** in left sidebar
2. Click **+ Add Game**
3. Choose detection method:
   - **Auto-Detect (Steam)** — Scan Steam library
   - **Manual Path** — Browse to game folder
4. Select the game from the list or enter custom path
5. Click **Add** to confirm
6. gregModmanager scans for game metadata and MelonLoader

### Switching Active Game

- Use the **Game Selector** dropdown in the top toolbar
- Or click **Games** sidebar and double-click a game name
- Mod list updates to show mods for the selected game

### Editing Game Configuration

1. Click **Games** sidebar
2. Right-click the game name → **Settings**
3. Configure:
   - **Game Root Path** — Install directory of the game
   - **MelonLoader Version** — Which version to use
   - **Custom Mod Paths** — Non-standard mod folder locations
   - **Launch Options** — Custom command-line arguments
4. Click **Save**

### Steam Integration

gregModmanager integrates with Steam:

- **Auto-detect games** from Steam library
- **Launch games** via Steam (preserves overlay, achievements)
- **Track playtime** integration
- **Workshop support** for datcentermods.com content

**Requirements:**

- Steam client must be installed and running
- Game must be in your Steam library

**To Launch via Steam:**
Settings → **Launch via Steam** ☑ → Save

### Removing a Game

1. Go to **Games** sidebar
2. Right-click game → **Remove**
3. (Optional) Keep local mod data or delete completely
4. Confirm

**Note:** This only removes the game from gregModmanager, not from your system.

---

## Managing Mods & Plugins

### Installing Mods

#### From datacentermods.com (Recommended)

1. Click **Mod Browser** tab (center panel)
2. Search or browse for a mod
3. Click on the mod card to view details
4. Review dependencies, version, compatibility notes
5. Click **Install** button
6. Confirm any dependencies (will be auto-installed)
7. Wait for download and installation

#### From Local File

1. In **Installed Mods** tab, right-click in empty space
2. Select **Install from File**
3. Choose a `.zip`, `.rar`, or folder containing mod files
4. gregModmanager validates the package
5. Click **Install** to confirm
6. Mod is extracted to the correct game folder

#### Drag & Drop

1. Locate a mod `.zip` file on your desktop or file explorer
2. Drag the file onto the gregModmanager window
3. Drop onto **Installed Mods** tab
4. Follow the installation prompt

### Enabling & Disabling Mods

- **Enable:** Click the checkbox next to mod name (☑)
- **Disable:** Uncheck the checkbox (☐)
- **Batch:** Select multiple mods with Ctrl+Click, then right-click → **Enable All** or **Disable All**

Disabled mods remain installed but won't load when the game launches.

### Mod Load Order

The order mods are listed determines the load order:

1. **Auto-Sort** — Click **Sort** button to auto-arrange by dependency
2. **Manual Drag & Drop** — Click and drag a mod to reorder
3. **Numbered Prefix** — Use `01_ModA`, `02_ModB`, etc. for explicit ordering

**Why Load Order Matters:**

- Some mods depend on others loading first
- Conflicting features may cause issues if loaded in wrong order
- Initialization of systems may require specific sequences

### Managing Profiles

Profiles let you save different mod configurations per game.

#### Creating a Profile

1. Click **Profile** in left sidebar
2. Click **+ New Profile**
3. Enter profile name (e.g., "Vanilla", "Gameplay Overhaul", "Speedrun")
4. Choose base profile (copy existing profile or start fresh)
5. Click **Create**

#### Switching Profiles

1. Click **Profile** sidebar
2. Double-click profile name to activate
3. Mod list updates to show mods in that profile
4. Launch game to use this profile

#### Importing & Exporting Profiles

- **Export:** Right-click profile → **Export** → Choose location → Save as `.gregprofile`
- **Import:** Click **+ Import Profile** → Choose `.gregprofile` file → Confirm

Use this to share profiles with friends or backup your configurations.

### Checking for Updates

1. Click **Update All** button in toolbar (or Ctrl+U)
2. gregModmanager checks datacentermods.com for updates
3. A list of available updates is shown
4. Click **Update All** or select specific mods to update
5. Wait for download and installation

**Auto-Update:**
Settings → **Auto-Update Mods** ☑ → Mods update automatically when launching game

### Uninstalling Mods

1. In **Installed Mods** list, right-click the mod
2. Select **Uninstall**
3. Confirm removal
4. Mod files are deleted; dependencies are NOT auto-removed

### Batch Operations

**Select Multiple Mods:** Ctrl+Click to select multiple, Shift+Click for range

**Batch Actions:**

- **Enable/Disable All** → Right-click → select action
- **Delete Selected** → Right-click → Uninstall All
- **Update Selected** → Right-click → Update Selected

### Dependency Resolution

If a mod requires other mods to function:

1. When installing, a **Dependency Resolution** dialog appears
2. Shows all required and optional dependencies
3. Click **Install Dependencies** to auto-install (recommended)
4. Or manually install dependencies first, then install the mod

If dependencies are missing:

- **Yellow warning icon** appears next to mod name
- Click icon to see missing dependencies
- Go to Mod Browser and install required mods

### Viewing Mod Details

1. In **Installed Mods** list, click mod name
2. Details panel opens showing:
   - Description and version
   - Author and download count
   - Dependencies and conflicts
   - Installation date and mod folder path
3. Right-click menu offers: Update, Uninstall, Open Folder

---

## Troubleshooting & FAQ

### Common Problems & Solutions

#### "Mod failed to install"

**Cause:** Network error, corrupted package, or permission issue

**Solution:**

1. Check internet connection
2. Try installing again
3. If error persists, check mod folder permissions (should be read/write)
4. Check log file for detailed error message

#### "Missing dependency: X"

**Cause:** A mod requires another mod that isn't installed

**Solution:**

1. Go to **Mod Browser**
2. Search for the missing dependency
3. Click **Install**
4. After installing, retry installing the original mod

#### "Mod conflict detected"

**Cause:** Two mods modify the same game system and may be incompatible

**Solution:**

1. Read the conflict warning carefully
2. Check mod documentation for compatibility info
3. Disable one of the conflicting mods
4. Or test both and report if they actually work together

#### "Game won't launch after installing mods"

**Cause:** Mod incompatibility, corrupt install, or missing MelonLoader

**Solution:**

1. Check MelonLoader is installed: Settings → Game Setup → MelonLoader Status
2. Try disabling recently installed mods
3. Launch game without mods to confirm it runs
4. Check game logs for error messages
5. Reinstall the problematic mod

#### "MelonLoader won't install"

**Cause:** Permission issue, incompatible game version, or missing .NET runtime

**Solution:**

1. Close the game completely
2. Run gregModmanager as administrator (Windows)
3. Check .NET 6+ runtime is installed: `dotnet --version` in terminal
4. Try manual MelonLoader installation from [melonwiki.xyz](https://melonwiki.xyz)

#### "Mods work offline, but datacentermods.com won't load"

**Cause:** Network connectivity, firewall, or DNS issue

**Solution:**

1. Check internet connection (try opening [datacentermods.com](https://datacentermods.com) in browser)
2. Check firewall allows gregModmanager (Windows Defender → Allow app)
3. Try a different DNS (8.8.8.8 or 1.1.1.1)
4. Check if datacentermods.com is experiencing downtime

#### "Performance is slow after installing mods"

**Cause:** Too many mods, conflicts, or poorly optimized mods

**Solution:**

1. Disable non-essential mods
2. Check mod documentation for performance impact
3. Review game logs for errors that might indicate conflicts
4. Ensure game meets minimum hardware requirements

### Reading Logs

Logs contain detailed information about every operation:

1. Click **Logs** in left sidebar
2. Scroll through recent entries
3. Look for `[ERROR]`, `[WARNING]`, or `[EXCEPTION]` lines
4. Right-click to copy log text or **Export** to file

**Log Locations:**

- **Windows:** `C:\Users\<Username>\AppData\Roaming\gregModmanager\logs`
- **Linux:** `~/.config/gregModmanager/logs` or `~/.local/share/gregModmanager/logs`

### Reporting Issues

If you encounter a bug:

1. Export logs: Logs panel → **Export Logs**
2. Gather system info:
   - OS version and architecture (Windows 10/11, Ubuntu 22.04, etc.)
   - gregModmanager version
   - .NET runtime version (`dotnet --version`)
   - Game and game version
3. Go to [GitHub Issues](https://github.com/mleem97/gregModmanager/issues)
4. Create a new issue with:
   - Clear title describing the problem
   - Reproduction steps (what did you do when the bug occurred)
   - Expected vs. actual behavior
   - System info and logs attached

### FAQ

**Q: Can I use gregModmanager on macOS?**  
A: Not yet. macOS support is planned for Q3 2026.

**Q: Do I need to be online to use gregModmanager?**  
A: Not for installed mods. Browsing and installing from datacentermods.com requires internet.

**Q: Can I use gregModmanager for other games besides those using MelonLoader?**  
A: No, gregModmanager is specifically designed for MelonLoader-based games.

**Q: Where are my mods installed?**  
A: Default: `<GameRoot>/Mods/` or `<GameRoot>/Plugins/` depending on mod type. See "Managing Games" section for details.

**Q: Can I back up my mod configuration?**  
A: Yes, use **Export Profile** to save your mod list and settings. See "Managing Profiles" section.

**Q: Is gregModmanager open source?**  
A: Yes! Source code is available on [GitHub](https://github.com/mleem97/gregModmanager).

**Q: How do I contribute?**  
A: See the Contributor Guide in `docs/03_CONTRIBUTOR_GUIDE.md`.

---

## Localization & Language Support

### Supported Languages

| Language | Code | Status |
|----------|------|--------|
| English | `en` | ✓ Fully supported |
| German | `de` | ✓ Fully supported |
| Spanish | `es` | ✓ Partial support |
| French | `fr` | 🔄 In progress |
| Japanese | `ja` | 🔄 Planned |
| Russian | `ru` | 🔄 Planned |

**Legend:** ✓ = Complete | 🔄 = In progress | — = Not started

### Changing Language

1. Go to **Settings** (left sidebar)
2. Click **Language** dropdown
3. Select desired language
4. Click **Apply**
5. gregModmanager restarts in the new language

### System Language Detection

On first launch, gregModmanager automatically detects your system language and uses it if supported. Fallback to English if your language is not yet supported.

### Localization File Format

Localization files use `ResX` format (standard .NET resource files) and are located at:

```
GregModmanager.Core/
  Localization/
    AppStrings.resx          (English - base)
    AppStrings.de.resx       (German)
    AppStrings.es.resx       (Spanish)
    AppStrings.fr.resx       (French)
```

### Contributing Translations

To help translate gregModmanager:

1. Go to [GitHub Issues](https://github.com/mleem97/gregModmanager/issues)
2. Search for translation/localization issues
3. Fork the repository
4. Edit the relevant `.resx` file (e.g., `AppStrings.de.resx` for German)
5. Submit a pull request with your translations

**Guidelines:**

- Keep translations concise and context-aware
- Maintain formatting codes (e.g., `{0}` for placeholders)
- Test the translation in-app before submitting
- Link to any terminology glossary or style guide

### Reporting Translation Issues

If you find incorrect or missing translations:

1. Go to [GitHub Issues](https://github.com/mleem97/gregModmanager/issues)
2. Click **New Issue** → **Translation Error**
3. Specify language, incorrect text, and suggested translation
4. Include screenshot if context is unclear

---

## Support & Contact

- **Documentation:** [docs/](../docs/) in this repository
- **Wiki:** [gregModmanager Wiki](https://github.com/mleem97/gregModmanager/wiki)
- **Bug Reports:** [GitHub Issues](https://github.com/mleem97/gregModmanager/issues)
- **Discussions:** [GitHub Discussions](https://github.com/mleem97/gregModmanager/discussions)
- **Email:** [support@gregModmanager.eu](mailto:support@gregModmanager.eu)

---

**Last Updated:** May 2026  
**Version:** v1.5.1
