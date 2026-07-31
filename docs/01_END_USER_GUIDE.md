# End-user guide

## Current scope

gregModmanager is a desktop client for local Workshop projects and Steam-based
workflows in the gregFramework ecosystem. The visible navigation contains
**Projects**, **New Project**, **My Uploads**, and **Settings**. Steam and service
availability are shown in the application status area.

The following are not released end-user features: profile management, manual
load-order control, automatic dependency/conflict resolution, an automatic
updater, and a public in-app mod-store browser. Do not rely on older screenshots
or guides that describe them.

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

A per-build self-signed Windows artifact may still trigger Windows reputation or
publisher warnings. A self-signed certificate identifies a build but does not
make it publicly trusted. See the [signing guide](../build/installer/CODE_SIGNING.md).

## Linux

Install the package format for your distribution, or extract a portable archive
to a writable directory and start `GregModmanager` from that directory. Package
installation commands vary by distribution; use the command shown by your
package manager. An AppImage and an AUR package are not produced by the current
release pipeline.

## Using the application

- **Projects** shows locally known Workshop projects.
- **New Project** starts project creation.
- **My Uploads** is the entry point for upload-related project work.
- **Settings** contains local application configuration.

Steam-dependent actions require a usable Steam client/session and the native
Steam library that ships with the matching platform build. Treat network and
Steam failures as recoverable: confirm the account/session state, retry after
connectivity returns, and preserve the error details for a bug report.

## Troubleshooting

| Problem | First checks |
| --- | --- |
| Application does not start | use the artifact for your CPU/platform; extract portable archives completely; check that the file is executable on Linux |
| Steam action fails | start Steam, sign in, and confirm the configured game and Steam session |
| Installer warning | verify the release source and SHA-256; self-signed builds are not universally trusted |
| A feature is missing | compare against the current-scope section; profiles, load order, updater, and store browsing are not released |

When reporting a defect, include the application version, platform, exact action,
and non-secret log output. Never include passwords, session tokens, or private
keys.
