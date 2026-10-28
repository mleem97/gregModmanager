# Contributing to gregModmanager

Thank you for your interest in contributing to gregModmanager!

## Getting Started

1. Fork the repository (the primary instance is Forgejo — GitHub is a read-only mirror)
2. Clone your fork locally
3. Install the Rust stable toolchain via [rustup](https://rustup.rs) and the native
   Linux X11 / font / GL development packages required by Slint
   (see [the devcontainer Dockerfile](../.devcontainer/Dockerfile))
4. Build and test:
   ```bash
   cargo build --workspace
   cargo test --workspace
   git diff --check
   ```
5. Create a feature branch, make your changes, submit a PR against `main`

## Workflow Rules

- Work on a feature branch — never push directly to `main`
- Keep commits focused; one logical change per commit
- Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, `docs:`, …)
- Update `CHANGELOG.md` under `[Unreleased]` for user-facing changes
- Update `EXTERNAL_DEPENDENCIES.md` when adding, removing, or materially
  changing external packages, tools, services, or native binaries

## Code Style

- Rust: latest stable edition, rustfmt defaults, clippy `-D warnings` in CI
- Follow existing naming conventions (module/file PascalCase, items `snake_case`,
  constants `SCREAMING_SNAKE_CASE`)
- Core must not depend on UI crates (`slint`); UI code lives in `crates/greg-app`
- Prefer `dirs`-provided known paths or validated user paths over hard-coded
  platform paths; guard platform-specific code with `cfg(target_os = …)` in
  `crates/greg-platform`
- UI strings go through the `greg-core` l10n modules (English plus German at
  minimum) — no hardcoded UI strings
- Keep PRs small and focused

## Reporting Issues

- Use the issue tracker with the provided templates
- Include platform, application version, Steam status, and OS
- Attach relevant non-secret logs when possible
- Provide reproducible steps

## Pull Requests

- Target the `main` branch
- Include a clear description of changes
- Ensure the workspace builds without errors and all tests pass (`cargo test --workspace`)
- Keep PRs focused on a single feature or fix

## License

By contributing, you agree that your contributions will be licensed under the
Apache License 2.0. See [LICENSE](LICENSE).
