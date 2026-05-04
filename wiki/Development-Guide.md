# Development Guide

## Prerequisites
- .NET SDK 9.x
- Windows: Visual Studio / Rider with Avalonia support
- Linux packaging: Docker or `nfpm`

## Build
```powershell
dotnet build GregModmanager.sln -c Release
```

## Run MAUI Client
```powershell
.\scripts\start.ps1
```

## Run Avalonia Client
```powershell
dotnet run --project .\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj
```

## Build Linux Packages (WSL from Windows)
```powershell
.\scripts\linux\build-avalonia-packages.ps1
```

## Coding Standards
- Keep UI components compact and data dense.
- Preserve Steam-safe behavior for upload flows.
- Prefer clear user-facing status text and explicit retry timers.
