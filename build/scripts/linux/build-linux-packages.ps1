#Requires -Version 5.1
[CmdletBinding()]
param(
    [string]$SourceDir = "",
    [string]$OutputDir = "",
    [string]$Formats = "deb,rpm,apk,archlinux",
    [string]$WslDistro = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$linuxScript = Join-Path $PSScriptRoot 'build-linux-packages.sh'

if (-not (Test-Path -LiteralPath $linuxScript)) {
    throw "Linux packaging script missing: $linuxScript"
}

if ([string]::IsNullOrWhiteSpace($SourceDir)) {
    $SourceDir = Join-Path $repoRoot 'installer\Output'
}

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $repoRoot 'installer\Output\linux-packages'
}

$resolvedSource = (Resolve-Path -LiteralPath $SourceDir).Path
if (-not (Test-Path -LiteralPath $resolvedSource)) {
    throw "Source directory not found: $resolvedSource"
}

if (-not (Test-Path -LiteralPath $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

$resolvedOutput = (Resolve-Path -LiteralPath $OutputDir).Path

function Convert-ToWslPath {
    param(
        [Parameter(Mandatory)][string]$WindowsPath,
        [string]$DistroName
    )

    $argsList = @()
    if (-not [string]::IsNullOrWhiteSpace($DistroName)) {
        $argsList += @('-d', $DistroName)
    }

    $normalizedWindowsPath = $WindowsPath -replace '\\', '/'
    $argsList += @('wslpath', '-a', $normalizedWindowsPath)
    $converted = & wsl.exe @argsList
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($converted)) {
        throw "Failed to convert path to WSL path: $WindowsPath"
    }

    return $converted.Trim()
}

$wslScript = Convert-ToWslPath -WindowsPath $linuxScript -DistroName $WslDistro
$wslSource = Convert-ToWslPath -WindowsPath $resolvedSource -DistroName $WslDistro
$wslOutput = Convert-ToWslPath -WindowsPath $resolvedOutput -DistroName $WslDistro

$wslArgs = @()
if (-not [string]::IsNullOrWhiteSpace($WslDistro)) {
    $wslArgs += @('-d', $WslDistro)
}

$bashCommand = "set -euo pipefail; bash '$wslScript' '$wslSource' '$wslOutput' '$Formats'"

Write-Host "[linux-pack/windows] WSL script: $wslScript"
Write-Host "[linux-pack/windows] Source: $wslSource"
Write-Host "[linux-pack/windows] Output: $wslOutput"
Write-Host "[linux-pack/windows] Formats: $Formats"

& wsl.exe @wslArgs bash -lc $bashCommand
if ($LASTEXITCODE -ne 0) {
    throw "Linux package build failed in WSL (exit code $LASTEXITCODE)."
}

Write-Host "[linux-pack/windows] Done. Packages are in: $resolvedOutput"

