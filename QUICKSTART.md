# gregModmanager QuickStart Guide

> **Version:** 1.6.1
> **Target:** Windows x64 / Linux x64 · .NET 10 · Avalonia UI
> **Status:** Production Ready

---

## 1. Installation (Player / Mod-User)

1. Download the latest release (`-Windows.zip` or `-Linux.tar.gz`) from the
   [releases page](https://github.com/mleem97/gregModmanager/releases).
2. Extract it anywhere and start `GregModmanager` (`GregModmanager.exe` on Windows).
3. Make sure **Steam is running** — the Modmanager connects to the Steam client
   for Workshop access (AppID `4170200`, Data Center).
4. Optional: set a custom workspace folder under **Settings** (default:
   `~/DataCenterWS`), where your local Workshop projects live.

---

## 2. Your First Workshop Project (Mod Creator)

### 2.1 Folder structure

```
<DataCenterWS>/
└── MyFirstMod/
    ├── metadata.json      # title, description, tags, visibility, version
    ├── preview.jpg        # required preview image (Steam limit applies)
    ├── content/           # everything that gets uploaded to the Workshop
    │   └── …your mod files…
    └── screenshots/       # optional additional preview images
```

### 2.2 Create it in the app

1. Open **New Project**, enter a folder name, pick a template.
2. Open the project in the **Editor**: set title, description, tags,
   visibility, and preview image.
3. Run the upload check — it must be green before publishing.
4. **Save & Upload**: creates the Workshop item on first publish and updates
   it on every later publish (a change note is required for updates).

### 2.3 metadata.json (essentials)

```json
{
  "title": "My First Mod",
  "version": "1.0.0",
  "description": "What the mod does, in Steam BBCode.",
  "visibility": "Public",
  "previewImageRelativePath": "preview.jpg",
  "tags": [],
  "needsgreg": true,
  "needsMelonLoader": true
}
```

---

## 3. Feature Overview (selection)

| Area | What it does |
|------|--------------|
| **Projects** | Local projects tab (search, refresh, edit) + Workshop uploads tab (import, update, open on Steam) |
| **Editor** | Title, description, tags, visibility, preview, screenshots, changelog, upload readiness check |
| **Mod Store** | Browse and subscribe Workshop items (Full / Decide-later modes) |
| **Start Game** | Launch Data Center via Steam — with or without mods |
| **Settings** | Workspace folder, game root, language, mod-store mode |
| **Headless** | Publish from scripts/CI: `GregModmanager publish --project <dir> --changelog <text>` |

---

## 4. Build from Source

Requirements:

- .NET SDK pinned in [global.json](global.json)
- A running Steam client (only needed for Workshop features, not for build/test)

```bash
git clone <forgejo-url>/gregModmanager.git
cd gregModmanager
dotnet restore GregModmanager.sln
dotnet build GregModmanager.sln -c Release
dotnet test GregModmanager.sln -c Release
dotnet run --project src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj
```

---

## 5. Troubleshooting

### "Steam Disconnected"
- Steam must be running **before** you start the Modmanager.
- Check `libsteam_api64.so` / `libsteam_api.so` next to the binary (Linux)
  or `steam_api64.dll` (Windows).

### "Steam publish failed: FileNotFound"
- Every file under `content/` and the preview image must exist and be readable.
- Close file managers / image viewers holding the preview image — the app
  stages a temp copy, but locked files can still fail on some systems.

### "Upload check is red"
- Open the editor: title, description, preview image, and `content/` folder
  are mandatory. Fix every error entry, warnings are optional.

### "Workshop legal agreement"
- Accept the Steam Workshop legal agreement once in the Steam client,
  then retry the upload.

---

## 6. Further Links

- [README](README.md) — full overview, badges, compatibility, repository layout
- [User guide](docs/01_END_USER_GUIDE.md) — in-depth usage
- [Creator guide](docs/02_MOD_CREATOR_GUIDE.md) — Workshop publishing details
- [Contributor guide](docs/03_CONTRIBUTOR_GUIDE.md) — code, tests, releases
- [CONTRIBUTING](CONTRIBUTING.md) — workflow rules for pull requests
- [EXTERNAL_DEPENDENCIES](EXTERNAL_DEPENDENCIES.md) — services, tools, binaries

---

**Happy Modding!** – TeamGreg
