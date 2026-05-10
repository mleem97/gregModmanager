# Startet GregModmanager direkt aus dem Build-Output
# ===================================================
#
# Verwendung: .\start.ps1
# oder:       .\start.ps1 -Configuration Release -Platform win10-x64
#
# Diese Datei startet die bereits kompilierte Anwendung direkt.
# Für einen Development-Run siehe: .\run.ps1

#Requires -Version 5.1
param(
    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Release',
    
    [ValidateSet('win-x64', 'win-arm64')]
    [string]$Platform = 'win-x64'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$buildPath = Join-Path $repoRoot "bin\$Configuration\net9.0-windows10.0.19041.0\$Platform"
$exePath = Join-Path $buildPath 'GregModmanager.exe'

if (-not (Test-Path $exePath)) {
    Write-Error ("GregModmanager.exe nicht gefunden: $exePath`nBitte zuerst bauen mit: .\build.ps1")
    exit 1
}

Write-Host "Starting GregModmanager from: $exePath"
& $exePath
