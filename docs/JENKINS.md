# Jenkins CI and releases

Jenkins runs formatting, Clippy, tests and a workspace build on native Linux,
Windows and macOS agents. `BUILD_TEST_PACKAGES` builds and archives Windows and
Linux packages on any branch without publishing them. The `PUBLISH_RELEASE`
parameter additionally builds the packages on `main`. Publication requires the
`RELEASE_VALIDATED` parameter after installation, upgrade and runtime checks
have been completed on all supported platforms and package formats.
Publication is currently blocked because the Alpine APK is known not to run.
Fix and validate the APK before removing the explicit publication block. Linux
release binaries must require glibc 2.39 or older; use an Ubuntu 24.04 build
agent or an equivalent baseline. Test builds can use a newer host and report
their actual glibc requirement in the job log.

The release job must be a multibranch job with `BRANCH_NAME=main`. The Linux
agent needs the Slint development libraries, `nfpm`, `binutils`, `sha256sum`,
`git` and `gh`. The Windows agent needs Rust, Inno Setup 6 and WiX CLI 5 on
PATH. The release stage uses the Jenkins secret text credential
`github-release-token` with permission to create releases in
`mleem97/gregModmanager`. The source commit must already be mirrored to GitHub.
The pipeline refuses to reuse an existing version tag or continue when the tag
check fails. It publishes the archives, installers, packages and SHA-256 files
to GitHub Releases; it does not push directly to `main`.

Keep `VERSION` and the Cargo workspace version identical. Before enabling
publication, verify all artifacts on their target systems, including Steam and
the game, and verify update and uninstall preserve user data. In particular,
the APK uses a glibc-built binary and depends on Alpine's `gcompat`; its actual
compatibility and package trust must be verified on Alpine. Windows installers
are not publicly trusted unless signed with a trusted code-signing certificate.

Run `scripts/check.sh` locally for the Rust gates. Native installation and
upgrade tests remain necessary; building an archive alone does not validate it.
