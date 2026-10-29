# Jenkins CI/CD (Rust check / build matrix)

The repository ships a Jenkins declarative pipeline ([`Jenkinsfile`](../../Jenkinsfile))
that runs the Rust quality gates and a workspace build across
Linux, Windows and macOS agents.

## Pipeline stages

1. **Checkout** – `checkout scm`.
2. **Rust-Toolchain** – bootstrap/install Rust toolchain with `rustfmt` and
   `clippy` components; on Linux additionally installs the Slint system
   dependencies (`libxcb-render0-dev`, `libxcb-shape0-dev`, `libxcb-xfixes0-dev`,
   `libxkbcommon-dev`, `libfontconfig1-dev`) via apt.
3. **Format** – `cargo fmt --all -- --check`.
4. **Clippy** – `cargo clippy --workspace --all-targets -- -D warnings`.
5. **Test** – `cargo test --workspace`.
6. **Build** – `cargo build --workspace`.

The matrix covers the operating systems `linux`, `windows` and `macos`
(agent labels of the same names).

## Agent requirements

- Linux agent: Rust via rustup plus `sudo apt-get` access for the Slint
  system dependencies (or a preinstalled Rust toolchain).
- Windows agent: Rust via rustup (`rustup component add rustfmt clippy`).
- macOS agent: Rust via rustup (`rustup component add rustfmt clippy`).

## Relationship to the existing CI

- The Forgejo Actions workflows (`.forgejo/workflows/rust.yml`,
  `.forgejo/workflows/build-and-release.yml`, …) remain the source of truth for
  release automation (cross-builds, packages, releases, promote/train, coverage).
- Jenkins adds the multi-OS check/build matrix and does not replace the
  release path.

## Local parity

- `scripts/check.sh` mirrors the local gate: fmt -> clippy (-D warnings) -> test.
- `cargo build --workspace` mirrors the Jenkins `Build` stage.

## Notes

- Release builds (`cargo xtask dist`) stay with the existing Forgejo/Gitea
  release workflows.
- The pipeline itself requires no repository secrets; any future release-time
  Jenkins jobs (e.g. signing) must use the Jenkins credentials store and must
  not keep secrets in the repository.
