# Startet GregModmanager (Avalonia Windows). Ausführen aus diesem Ordner: .\run.ps1
# Optional: .\run.ps1 -- -h  (Argumente nach -- gehen an die App)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $repoRoot
dotnet run --project (Join-Path $repoRoot 'src\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj') -c Release @args

