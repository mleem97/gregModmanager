# Managed workspace runtime contract v1

The Rust manager owns downloads and installations under its selected base:
`GameModding/Mods`, `GameModding/Melons`, `GameModding/UserLibs` and `ModConfigs`.
The game must never independently mirror arbitrary Workshop files into its own
installation. Manager-side import restrictions alone cannot enforce this inside
an already running Unity game.

## Existing runtime evidence

The inspected sources are independent repositories, not the frozen C# reference
implementation in this repository:

- `gregCore/gregCore.main/src/gregCore.Patches/ModLoader/VanillaModloaderSilencer.cs`
  implements best-effort Harmony prefixes for `SyncWorkshopThenLoadAll`,
  `CopyDirectory` and `LoadDll`. Before this patch no caller of `Neutralize`
  existed in that source tree. Unknown builds otherwise retain vanilla behavior.
- `gregPlugin.Melons/gregPlugin.Companion/StartupBootstrap.cs` exposes
  `RegisterBeforeFolderScan`. Calling a normal MelonPlugin after the initial
  folder scan is too late to register external Plugins and UserLibs.
- `GregModmanagerSourceProvider.cs` currently reads `GREGMODMANAGER_SOURCES` and
  expects `Mods/Plugins/UserLibs`. It does not implement the new `Melons` layout.
- Companion configuration currently enables Workshop DLL staging and native
  StreamingAssets synchronization by default.

The supplied [runtime patch](../../patches/managed-workspace-runtime.patch)
changes those runtime paths explicitly. It is a reviewable source integration
artifact; it is not automatically applied to another repository or deployed to a
user's game.

## Contract

The manager sets these per-launch environment variables only for managed runs:

| Variable | Value |
| --- | --- |
| `GREGMODMANAGER_MANAGED` | `1` |
| `GREGMODMANAGER_ROOT` | Absolute selected manager base |
| `GREGMODMANAGER_SESSION` | Unique ASCII alphanumeric/hyphen session identifier |
| `GREGMODMANAGER_READY` | `<base>/Configs/runtime/<session>.ready.json` |

The launcher checks the reviewed runtime descriptor at
`<game>/UserData/gregModmanager/runtime-v1.json`. Paths are relative to the game
root and must identify the deployed patched artifacts, for example:

```json
{
  "schemaVersion": 1,
  "corePath": "Mods/framework/gregCore.dll",
  "companionPath": "Plugins/gregPlugin.Companion.dll",
  "coreSha256": "<SHA256 of deployed patched gregCore.dll>",
  "companionSha256": "<SHA256 of deployed patched gregPlugin.Companion.dll>",
  "startupHook": true
}
```

Replace the example paths with the actual installed paths. Generate hashes after
building and installing the reviewed artifacts. A descriptor with matching hashes
is static deployment evidence, not proof that runtime patching succeeded.
`startupHook: true` is an operator assertion requiring the maintained loader hook
below; setting the flag on an unpatched installation is insufficient.

The early Companion bootstrap writes
`<ready>.sources.json` with `schemaVersion`, matching `session` and
`managedSources: true` only after the supported adapter successfully registers
managed search directories. It disables Steam Workshop/StreamingAssets discovery,
both staging paths, native library injection and implicit plugin sanitization.
`Melons` maps to MelonLoader Plugins; hidden transaction directories and disabled
packages are excluded. Existing explicit game-root framework assemblies remain
available for bootstrap.

gregCore's first initialization action requires that same-session source record,
installs and verifies Harmony ownership of all three dangerous-operation prefixes
plus the managed native-reader prefix, and only then atomically emits:

```json
{
  "schemaVersion": 1,
  "session": "<this launch session>",
  "vanillaBlocked": true,
  "managedSources": true
}
```

Missing methods, changed signatures, unsupported adapter fields or incomplete
patches terminate the managed game process through `Environment.FailFast`.
Merely throwing a Melon initialization exception would be unsafe because the
loader may catch it and continue with vanilla Workshop copying. The manager must
also stop a launched process if no valid same-session acknowledgement arrives
before its startup deadline. Never accept a previous session's file.

For native assets the patch redirects `LoadAllMods` and the blocked Workshop sync
entry to the verified `LoadModPack(string)` read routine using managed
`config.json` packages. Native DLL entries are rejected; executable mods belong
in the Melon mod/plugin paths. Model, texture and icon references are checked for
absolute/traversal/link escapes. No asset copy into StreamingAssets occurs.
`ModConfigs` is a persistent manager-owned config location; existing third-party
mods must opt into this path rather than assuming their game-root UserData paths
have silently moved.

## Apply and build in isolated checkouts

The combined patch retains repository-name prefixes. From each destination root,
strip those two path components and include only that repository's paths.
The artifact uses zero-context hunks; `--unidiff-zero` is required:

```sh
# gregCore/gregCore.main checkout
git apply --check --unidiff-zero -p2 --include='src/*' /path/to/managed-workspace-runtime.patch
git apply --unidiff-zero -p2 --include='src/*' /path/to/managed-workspace-runtime.patch
dotnet build gregCore.csproj -c Release

# gregPlugin.Melons checkout
git apply --check --unidiff-zero -p2 --include='gregPlugin.Companion/*' /path/to/managed-workspace-runtime.patch
git apply --unidiff-zero -p2 --include='gregPlugin.Companion/*' /path/to/managed-workspace-runtime.patch
dotnet build gregPlugin.Companion/gregPlugin.Companion.csproj -c Release

# maintained MelonLoader checkout matching the inspected Core.Initialize source
git apply --check --unidiff-zero -p2 --include='MelonLoader/Core.cs' /path/to/managed-workspace-runtime.patch
git apply --unidiff-zero -p2 --include='MelonLoader/Core.cs' /path/to/managed-workspace-runtime.patch
# Follow that loader checkout's documented build command and dependencies.
```

Keep gregCore and Companion at **.NET 6**, including their references to the
actual deployed MelonLoader, Harmony and generated IL2CPP/game assemblies. The
patch adds no dependency on the Rust UI. The early bootstrap is inserted directly
before `MelonFolderHandler.ScanForFolders()`; it loads the reviewed Companion via
reflection to avoid a compile-time loader/plugin dependency cycle. Stock loader
builds without this hook cannot supply same-launch external Plugin/UserLib
registration. The current Companion adapter is deliberately restricted to
MelonLoader assembly version **0.7.3** and the expected private directory-list
fields. A newer maintained loader needs its own tested adapter; do not bypass
that version/field check simply to make startup continue.

Do not deploy changed loader binaries or overwrite installed framework assemblies
as an implicit manager startup action. Build, review and deploy them as explicit
runtime releases, then update the descriptor hashes. Include all framework
runtime dependencies required by the selected monolithic or modular build.

## Verification and release gates

The patch was checked for application against isolated source snapshots. The
patched Companion was compiled as net6.0 in an isolated `/tmp` copy against its
existing reference assemblies (three pre-existing nullable warnings). A separate
net6.0 compile harness checks the changed guard/silencer against the inspected
game/Harmony references. This does not prove Harmony execution inside Unity.
The complete gregCore/maintained-loader builds and real game launch validation
remain deployment gates.

Before enabling a supported runtime release, verify all of the following with a
known game build and each supported native/Proton platform:

1. Fresh launch yields one matching session acknowledgement after actual prefixes
   and external directories have been installed.
2. Removing gregCore, Companion, a required method, an adapter field or the early
   loader hook fails closed and never allows vanilla Workshop synchronization.
3. `Mods`, `Melons` and `UserLibs` load on the same launch; disabled DLLs/packages
   stay inactive after Workshop updates.
4. Native managed assets load through the read-only reader; path traversal,
   symlinks, native DLL entries and unknown game signatures fail closed.
5. Compare game-tree contents before/after launch: no Workshop payload was copied
   there. Confirm user ModConfigs survive import/update/unsubscribe.
6. Test missing/old/forged-session readiness, startup timeout, interrupted imports,
   simultaneous import attempts and a game started outside the manager.

This contract coordinates trusted local components. It does not sandbox arbitrary
third-party native or .NET code: such code runs with the user's filesystem rights.
