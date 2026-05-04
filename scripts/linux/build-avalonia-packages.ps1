#Requires -Version 5.1
[CmdletBinding()]
param(
    [string]$OutputDir = "",
    [string]$Version = "1.1.0",
    [string]$WslDistro = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$bashScript = Join-Path $PSScriptRoot 'build-avalonia-packages.sh'

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $repoRoot 'artifacts\avalonia-linux'
}

if (-not (Test-Path -LiteralPath $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

function Convert-ToWslPath {
    param(
        [Parameter(Mandatory)][string]$WindowsPath,
        [string]$DistroName
    )

    $argsList = @()
    if (-not [string]::IsNullOrWhiteSpace($DistroName)) {
        $argsList += @('-d', $DistroName)
    }

    $argsList += @('wslpath', '-a', ($WindowsPath -replace '\\', '/'))
    $converted = & wsl.exe @argsList
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($converted)) {
        throw "Failed to convert path to WSL path: $WindowsPath"
    }

    return $converted.Trim()
}

$wslScript = Convert-ToWslPath -WindowsPath $bashScript -DistroName $WslDistro
$wslOutput = Convert-ToWslPath -WindowsPath $OutputDir -DistroName $WslDistro

$wslArgs = @()
if (-not [string]::IsNullOrWhiteSpace($WslDistro)) {
    $wslArgs += @('-d', $WslDistro)
}

$bashCommand = "set -euo pipefail; bash '$wslScript' '$wslOutput' '$Version'"
& wsl.exe @wslArgs bash -lc $bashCommand

if ($LASTEXITCODE -ne 0) {
    throw "Avalonia Linux packaging failed (exit code $LASTEXITCODE)."
}

Write-Host "Done. Linux packages: $OutputDir"
