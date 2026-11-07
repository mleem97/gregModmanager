# Managed workspace and publishing

The selected **manager base** owns application data. Its default is
`$HOME/GregModmanager` (resolved with `dirs` on every desktop platform).
A configured base such as `/home/marvin/DataCenterWS` remains valid.
`GREG_WORKSPACE` overrides the saved location at process startup.

```text
<base>/
  Configs/
    settings.sqlite3          SQLite settings, schema version 1
    collections.json          Modpacks and shareable collections
    runtime/                  Per-launch runtime acknowledgements
  Logs/                       Session and build logs, crash reports
  Workshop/
    <project>/
      metadata.json           Shared Steam + Modstore configuration
      content/                Published payload
      preview.png             Optional project preview
      screenshots/            Optional gallery
    .templates/
  GameModding/
    Mods/<workshop-id>/
    Melons/<workshop-id>/     MelonLoader plugins
    UserLibs/<workshop-id>/   Runtime libraries
    <kind>/GregStore/<id>/    Typed Modstore installations
  ModConfigs/<workshop-id>/  Seeded once; retained across content updates
  Cache/
    Thumbnails/               Bounded downloaded previews
    Builds/                   SDK build outputs
```

The Steam download cache stays owned by Steam. A managed install copies verified
content into `GameModding`; it does not move/delete Steam's files. Downloaded
packages cannot select an arbitrary installation directory. Type markers and
recognized mirror folders determine placement. Unknown explicit types, unsafe
paths, symlinks and overwrites of unowned directories fail. Updates use staged
replacement with rollback per content category; multi-category packages are not
one filesystem-wide transaction. Disabled files/packages stay disabled.

**Install in GameModding** and **Import** serve different actions in Workshop
Browse. The former installs content for local management after a review dialog;
the latter creates an editable publishing project in `Workshop`. Neither writes
Workshop payloads into the game installation. Library/plugin installation dialogs
also target managed UserLibs/Melons. The runtime itself still needs an explicitly
installed MelonLoader/framework bootstrap in the game installation.

## Existing data and changing the base

Legacy publishing projects directly under a custom base remain discoverable in
place. New projects always use `Workshop/`. When using the default base, legacy
`~/DataCenterWS` publishing projects remain discoverable too. Startup does not
move, overwrite or delete existing projects or game-installed mods.

Settings are imported once from the former OS application-data
`gregModmanager/preferences.json` and optional `<base>/Configs/preferences.json`.
The typed SQLite key/value table uses WAL, prepared row updates and a memory
cache for reads. Unchanged writes are skipped. A small OS application-data
`workspace.path` locator is needed to find a custom base on the next launch;
it does not contain settings or tokens. OAuth values retain their JSON types;
Configs and database permissions are restricted on Unix.

A base change first shows a modal containing the target and directory layout.
It is refused during game sessions and conflicting background jobs. Settings are
copied to the target database and logs are retargeted. The previous database,
mods and publishing projects remain in the old base. Choose the old base again
to access those files; this operation is not a bulk content migration. Existing
mods in the game installation require deliberate migration/review, not silent
startup copying.

## One publishing configuration

Both providers read the same `<project>/metadata.json`. Provider identifiers
remain distinct: `publishedFileId` for Steam and `modstoreId` for datacentermods.
Title, version, description, visibility, tags, requirements, content type and
project build settings belong to this common document. README/CHANGELOG
resolution can format the same source text for each provider.

```json
{
  "title": "Example library",
  "version": "1.0.0",
  "description": "Shared library for my mods",
  "modType": "Userlib",
  "publishedFileId": 0,
  "modstoreId": "",
  "category": "Core Framework",
  "license": "MIT",
  "build": {
    "project": "src/Example.csproj",
    "configuration": "Release"
  }
}
```

Supported canonical types:

| `modType` | Managed folder | Catalog class |
| --- | --- | --- |
| `PlacableObject` | `Mods` | Native asset mod |
| `DataCenterMod` | `Mods` | Mod |
| `MelonloaderPlugin` | `Melons` | MelonLoader plugin |
| `Userlib` | `UserLibs` | Library |

Both providers publish the same `greg-mod-type:<canonical-type>` tag and payload
marker `greg-modmanager.meta.json`. The existing Modstore backend persists these
tags; topic categories remain separate from installation types. The manager
catalog filters Mods, Libraries and Plugins without confusing filtered row
positions with install targets. Legacy untagged Modstore entries default to
Mods; conflicting or unknown explicit classification is rejected.

The existing Modstore submit backend inspected in `greg-store` currently strips
`modId`/`releaseId` and inserts new records. The client preserves the common
`modstoreId`, but true update publication requires a coordinated server change.
No server database migration or website deployment is included here.

## Build tests

The editor's **Build test** dialog resolves a project and local dependency paths
before running an explicit SDK command on a background worker. It supports
`.csproj`, `.sln`, `.slnx` and `Cargo.toml`. Ambiguous source trees require
`build.project`; a single solution can aggregate multiple projects. A build
project must resolve inside its publishing project. Native assets without source
have no compilation test.

.NET builds require SDK 8 or newer, use the configured build configuration and
receive assembly search directories from managed UserLibs/Melons and the game's
MelonLoader/managed assemblies. Projects can also use the supplied `GameDir` and
`UserLibsDir` MSBuild properties. Existing project/NuGet dependencies still apply;
missing references produce a failed build with the actual compiler diagnostics.
Rust builds use `cargo build --release`. Output/artifacts go to `Cache/Builds`,
and complete logs go to `Logs`. The dialog shows up to the last 256 KiB, exit
status and duration. Cancellation and a five-minute timeout stop the build.
Build scripts/tasks execute local project code; tests run only on explicit user
action. Compilation success does not prove game compatibility or runtime safety.

## Performance and images

Local scans run in background workers, coalesce repeated requests and discard
outdated results after navigation or root changes. Searches filter cached rows
without rescanning directories. Publish hashes stream through a 64 KiB buffer;
content statistics use one traversal. Log file handles and HTTP clients are
reused instead of reopening/recreating them for each item.

PNG/JPEG/GIF/WebP previews use actual bounded pixel decoding. Limits include
8 MiB encoded input, 8192 pixels per edge, 16 million source pixels and 64 MiB
of decoder allocation. Oversized previews are downsampled to 512 pixels on the
longest edge; tiny images are not enlarged. The decoded LRU cache holds at most
128 entries/32 MiB and invalidates when a local file's size or timestamp changes.
Provider cache keys and changed preview URLs invalidate stale remote previews.
Corrupt images produce a placeholder and cached failures avoid repeated decoding.

## Runtime protection

Manager-side path rules do not disable Unity's own Workshop copier. Managed
launches therefore require the verified external runtime described in
[managed-runtime.md](integration/managed-runtime.md). Until patched artifacts and
the maintained loader startup hook are explicitly installed, modded launches are
refused. Descriptor hashes alone are insufficient: every run needs a fresh
nonce acknowledgement that source registration and all required Harmony
prefixes succeeded. A missing/invalid acknowledgement stops that run.

The source patch is included for review and was compiled in isolated test
harnesses. Full framework/loader builds and live Unity tests are still required
before claiming the external runtime integration is production-ready. Vanilla
launches do not load this managed runtime and are not covered by its Workshop
copy protection. Third-party code is not sandboxed by this path contract.

Repository documentation lives under `docs`; there is no separate wiki checkout
to update. Future runtime deployment should update this page and the integration
contract together.
