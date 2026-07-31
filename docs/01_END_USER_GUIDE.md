# End-user guide

**Owner:** gregModmanager maintainers. **Evidence:** Avalonia views and Core
services in the current checkout. **Review trigger:** update after a navigation,
Steam workflow, settings/privacy, or release-format change.

## Current scope

gregModmanager is a desktop client for local Workshop projects and Steam-based
workflows in the gregFramework ecosystem. Navigation contains **Projects**,
**New Project**, **My Uploads**, and **Settings**. In **Full** and
**Decide later** app modes, it also contains **Mod Store**. The Mod Store is
hidden only in **Mod Manager-only** mode. Steam and service availability appear
in the application status area.

The following are not released end-user features: profile management, manual
load-order control, automatic dependency/conflict resolution, and an automatic
updater. The in-app Mod Store is a Steam Workshop browser, not a generic public
web catalog. Do not rely on older screenshots or guides that claim otherwise.

## Supported installation formats

| Platform | Use one of these release assets |
| --- | --- |
| Windows x64 | Setup EXE, MSI, or portable ZIP |
| Linux x64 | portable ZIP/tarball, DEB, RPM, APK, or Arch package |
| macOS | no release asset; testing is limited to a manually prepared real Mac |

Release filenames contain the version, platform, and (for preview builds) a
`-pre` suffix. Always choose the matching asset from the release page rather
than copying an example filename from an older document.

## Windows

1. Download either the Setup EXE or MSI from the release page. Use the portable
   ZIP only when an installer is unsuitable.
2. Verify the matching `.sha256` file when one is published.
3. Run the installer or extract the ZIP to a writable location.
4. Start `GregModmanager.exe` and sign in only through the application's
   configured account flow.

**Expected result:** the window opens and the status area reports the available
Steam and service state. A missing Steam login does not prevent local project
work, but Steam browsing, subscriptions, and uploads need a usable Steam session.

A per-build self-signed Windows artifact may still trigger Windows reputation or
publisher warnings. A self-signed certificate identifies a build but does not
make it publicly trusted. See the [signing guide](../build/installer/CODE_SIGNING.md).

## Linux

Install the package format for your distribution, or extract a portable archive
to a writable directory and start `GregModmanager` from that directory. Package
installation commands vary by distribution; use the command shown by your
package manager. An AppImage and an AUR package are not produced by the current
release pipeline.

**Expected result:** a portable archive contains the `GregModmanager` executable.
Run `chmod +x GregModmanager` after extraction if the executable bit was lost.

## Using the application

- **Projects** shows locally known Workshop projects.
- **New Project** starts project creation.
- **My Uploads** is the entry point for upload-related project work.
- **Settings** contains local application configuration.
- **Mod Store** appears outside Mod Manager-only mode. It browses and searches
  Steam Workshop items, supports subscriptions, favorites, bulk subscription,
  and dependency-health actions. Workshop synchronization then handles subscribed
  items through the configured game workflow.

Steam-dependent actions require a usable Steam client/session and the native
Steam library that ships with the matching platform build. Treat network and
Steam failures as recoverable: confirm the account/session state, retry after
connectivity returns, and preserve the error details for a bug report.

## Protect account and diagnostic data

**Session state is stored locally.** The desktop client stores its refresh value
in the local preferences JSON file so that it can restore a session. The current
implementation does not encrypt that value; OS account permissions protect the
file. Log out on shared machines to remove it, and do not copy the preferences
file into a support request.

**Telemetry is enabled by default.** Settings can disable telemetry. The setting
controls application telemetry; it does not change Steam's own data handling.

**Reproduction bundles can contain personal machine data.** The Settings page can
create a ZIP under the local `gregModmanager/repro` folder. It can contain logs,
crash dumps, Windows application events from the previous 48 hours, machine and
user names, and local paths. Inspect the ZIP and remove sensitive material before
sharing it with anyone.

## Troubleshooting

| Problem | First checks |
| --- | --- |
| Application does not start | use the artifact for your CPU/platform; extract portable archives completely; check that the file is executable on Linux |
| Steam action fails | start Steam, sign in, and confirm the configured game and Steam session |
| Installer warning | verify the release source and SHA-256; self-signed builds are not universally trusted |
| A feature is missing | compare against the current-scope section; profiles, manual load order, conflict resolution, and updater are not released |

When reporting a defect, include the application version, platform, exact action,
and non-secret log output. Never include passwords, session tokens, or private
keys.
