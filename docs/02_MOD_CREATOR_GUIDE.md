# Mod and plugin creator guide

**Owner:** gregModmanager maintainers. **Evidence:** `WorkshopMetadata`,
`WorkspaceService`, and `HeadlessRunner` in this repository. **Review trigger:**
update after a metadata, upload, Steam, or headless-command change.

## What this client supports today

gregModmanager contains project and upload workflows backed by Steam Workshop
services. It is not the published specification for every gregFramework package
type, and it does not expose a general “validate manifest” command.

Use the target game's and MelonLoader's current documentation for runtime plugin
requirements. For gregCore contracts, use the gregCore repository/API reference.
This guide documents the boundary that is visible in this repository, not an
invented universal package format.

## Workshop project metadata

The desktop client persists Workshop project metadata in `metadata.json` through the
`WorkshopMetadata` model in
`src/GregModmanager.Core/Models/WorkshopMetadata.cs`. Its fields and serialization
are the source of truth for data created by this client. In particular, it uses
fields such as `publishedFileId`, `workshop_dependency`, and `modType`; that
contract does not match the historical *sample schema* in this repository, even
though both use the `metadata.json` filename.

Before automating metadata generation, inspect the model and an application-
created project on the version you target. Do not submit an example JSON file as
though it were a validated manifest.

## Publish a prepared project from the command line

The executable has a headless Steam Workshop publish command. The project path
must contain both `content/` and application-readable `metadata.json`; Steam
must be available and logged in.

```text
GregModmanager.exe --mode publish --path "D:\path\to\project"
```

`--upload` is an alias for `--mode publish`. Add `--autocommit` only when an
automation workflow expects `.ralph/tasks/status.json` in the project. Exit code
`0` means Steam reported success; exit code `1` reports a project or publish
failure; exit code `2` means `--path` was omitted. Test against a disposable
Workshop item before automating a production upload.

## Build a MelonLoader integration

The repository keeps MelonLoader-specific code under `src/GregModmanager.Melons/`.
`SubDirectoryFixer` targets `net6.0` because it must load in the MelonLoader
environment. Its local project currently references `MelonLoader.dll` from a
Data Center installation; adapt that reference for your own game environment
without committing a machine-specific path or third-party binaries.

The release build compiles this helper separately and copies
`SubDirectoryFixer.dll` into the Avalonia assets. A normal solution build does
not automatically build every Melon integration.

## Publishing safely

1. Build and test against the exact game, MelonLoader, and Steam environment you
   intend to support.
2. Keep source, licence notices, and third-party binaries clearly separated.
3. Test an upload from a disposable project before publishing to a public item.
4. Review every file that will be included; exclude credentials, tokens, local
   configuration, and unrelated game files.
5. Record supported game and loader versions in your own release notes.

Steam Workshop availability depends on the Steam client, the configured App ID,
and Steamworks permissions. A successful local build is not evidence that an
upload will be accepted by Steam.

## Historical manifest samples

Files under `examples/manifests/` are historical samples, not an input contract
for the present desktop application. They remain available for research only.
See [their status note](examples/manifests/README.md) and replace them with
tested examples once a stable, public manifest schema exists.
