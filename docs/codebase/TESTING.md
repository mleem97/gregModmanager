# Testing Patterns

## Stack and Commands

- xUnit 2.9.3, Microsoft.NET.Test.Sdk 18.8.1 and Coverlet collector 10.0.1.

```bash
DOTNET_ROLL_FORWARD=Major dotnet test tests/GregModmanager.Tests/GregModmanager.Tests.csproj -c Release
dotnet test tests/GregModmanager.Tests/GregModmanager.Tests.csproj -c Release --filter FullyQualifiedName~GameAdapterTests
dotnet test --collect:"XPlat Code Coverage"
```

Use the SDK selected in `global.json`. Do not rely on runtime roll-forward as a
substitute for the supported .NET 9 application runtime.

## Layout

Central test project `tests/GregModmanager.Tests/`, files named `*Tests.cs`. Tests create unique temporary roots and clean them in `Dispose`; no global fixture or mocking framework was found.

## Scope

| Scope | Covered | Notes |
|---|---|---|
| Unit | yes | models, installers, adapter and filesystem behavior |
| Integration | partial | local filesystem/service boundaries; no live Steam/Webapp |
| E2E | no in desktop repo | UI, native Steam and real game flows remain `[TODO]` |

## Gaps

No enforced coverage threshold. Missing tests cover native Steam, auth/network, UI picker, package installation and transactional deployment.

## Evidence

- `tests/GregModmanager.Tests/GregModmanager.Tests.csproj`
- `tests/GregModmanager.Tests/GameAdapterTests.cs`
- `.github/workflows/build-and-release.yml`
