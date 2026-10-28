# Build scripts

Run the following commands from the repository root unless a command says
otherwise.

## Quality gate

| Task | Command |
| --- | --- |
| rustfmt + clippy (`-D warnings`) + tests, all crates | `./scripts/check.sh` |
| Dev/debug build (all crates) | `./scripts/build.sh` |
| Build app from source and start (debug) | `./scripts/run-app.sh` |
| Run headless CLI from source | `./scripts/run-cli.sh` |
| Release build (host platform) | `./scripts/build-release.sh` |
| Release build Windows x64 | `./scripts/build-windows.sh` |
| Release build Linux x64 | `./scripts/build-linux.sh` |
| Release build macOS x64/arm64 | `./scripts/build-macos.sh` |

All of the above delegate to `cargo` / `cargo xtask` inside the Rust workspace
(`crates/*` + `xtask`). There is no `global.json` or .NET SDK pin anymore; the
Rust toolchain comes from rustup.

## Linux distribution packages

```bash
# after `cargo xtask dist` produced ./dist/gregmodmanager-<version>-linux-x64
bash build/scripts/linux/build-rust-packages.sh \
  ./dist/gregmodmanager-<version>-linux-x64 <version> false [out-dir]
```

The script wraps `nfpm` and emits DEB/RPM/APK/Arch packages plus container
images. The release workflow (`.forgejo/workflows/build-and-release.yml` and its
byte-identical Gitea copy) calls the same script.

## Windows artifacts

- Portable ZIP: produced by `cargo xtask dist` (`--target x86_64-pc-windows-gnu`
  when cross-building from Linux/macOS with mingw).
- Setup installer: `build/installer/gregModmanager.iss` (Inno Setup 6; best-effort
  build under Wine + ISCC, skipped when neither is available — the portable ZIP
  always ships).

## Signing

- `build/installer/sign-authenticode.ps1` with modes `auto`, `self-signed`, and
  `pfx`. See `build/installer/CODE_SIGNING.md` for the full flow.

## Compatibility wrapper

`build.sh` at the repository root is a small compatibility entry point that only
delegates to the Linux packaging script
(`build/scripts/linux/build-rust-packages.sh`). It is not a general build entry
point and must not be used to make Windows installers.

## Development helpers

| Script | Purpose |
| --- | --- |
| `scripts/check.sh` | quality gate: rustfmt + clippy `-D warnings` + tests |
| `scripts/build.sh` | dev/debug build of the whole Rust workspace |
| `scripts/run-app.sh` | build and start the desktop app from source (debug) |
| `scripts/run-cli.sh` | run the headless CLI from source |
| `scripts/build-release.sh`, `scripts/build-windows.sh`, `scripts/build-linux.sh`, `scripts/build-macos.sh` | release builds via `cargo xtask dist` |
| `build/scripts/linux/build-rust-packages.sh` | Linux DEB/RPM/APK/Arch packages + container |
| `build/installer/gregModmanager.iss` | Inno Setup installer (Wine, best effort) |
| `build/installer/sign-authenticode.ps1` | Windows code signing |
