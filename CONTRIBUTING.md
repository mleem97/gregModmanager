# Contributing to gregModmanager

Thank you for your interest in contributing to gregModmanager!

## Getting Started

1. Fork the repository (the primary instance is Forgejo — GitHub is a read-only mirror)
2. Clone your fork locally
3. Install the SDK pinned in [global.json](global.json)
4. Build and test:
   ```bash
   dotnet restore GregModmanager.sln
   dotnet build GregModmanager.sln -c Release
   dotnet test GregModmanager.sln -c Release
   ```
5. Create a feature branch, make your changes, submit a PR against `main`

## Workflow Rules

- Work on a feature branch — never push directly to `main`
- Keep commits focused; one logical change per commit
- Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, `docs:` …)
- Update `CHANGELOG.md` under `[Unreleased]` for user-facing changes
- Update `EXTERNAL_DEPENDENCIES.md` when adding, removing, or materially
  changing external packages, tools, services, or native binaries

## Code Style

- C# latest language version, `Nullable` enabled
- Follow existing naming conventions (PascalCase for public, `_camelCase` for private fields)
- Core must not reference Avalonia — UI code lives in `GregModmanager.Avalonia`,
  shared logic in `GregModmanager.Core`
- Prefer `Path.Combine` and validated user paths over hard-coded platform paths;
  guard platform-specific code with compile-time or runtime platform checks
- UI strings go into `AppStrings.resx` **and** `AppStrings.de.resx` at minimum
- Serialized DTOs must be registered in `AppJsonContext.cs`
- Keep PRs small and focused

## Reporting Issues

- Use the issue tracker with the provided templates
- Include platform, application version, Steam status, and OS
- Attach relevant non-secret logs when possible
- Provide reproducible steps

## Pull Requests

- Target the `main` branch
- Include a clear description of changes
- Ensure the solution builds without errors and all tests pass
- Keep PRs focused on a single feature or fix

## License

By contributing, you agree that your contributions will be licensed under the
Apache License 2.0. See [LICENSE](LICENSE).
