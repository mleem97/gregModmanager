#!/usr/bin/env pwsh
#Requires -Version 5.1
# gregModmanager Interactive Builder
# Styled CLI UI for local builds, run, and test workflows.

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $repoRoot

# ---------------------------------------------------------------------------
# Terminal Core Theme
# ---------------------------------------------------------------------------
$CPrimary    = [ConsoleColor]::Cyan
$CSecondary  = [ConsoleColor]::Green
$CWarn       = [ConsoleColor]::Yellow
$CError      = [ConsoleColor]::Red
$CDefault    = [ConsoleColor]::Gray
$COnSurface  = [ConsoleColor]::White

function Write-Banner {
    Clear-Host
    Write-Host ''
    Write-Host '  ================================================' -ForegroundColor $CPrimary
    Write-Host '     gregModmanager  -  Interactive Builder'       -ForegroundColor $COnSurface
    Write-Host '     Avalonia UI  |  .NET 9  |  Cross-Platform'    -ForegroundColor $CDefault
    Write-Host '  ================================================' -ForegroundColor $CPrimary
    Write-Host ''
}

function Write-MenuItem {
    param(
        [string]$Key,
        [string]$Label,
        [switch]$Active
    )
    if ($Active) {
        Write-Host "  > [$Key]  $Label" -ForegroundColor $CPrimary -BackgroundColor ([ConsoleColor]::DarkGray)
    } else {
        Write-Host "    [$Key]  $Label" -ForegroundColor $CDefault
    }
}

function Show-Menu {
    param([int]$Selected = 0)
    Write-Banner

    $items = @(
        @('B', 'Build All (CI mirror)'),
        @('W', 'Build Windows Only'),
        @('L', 'Build Linux Only'),
        @('P', 'Build Linux Packages (WSL)'),
        @('R', 'Run Avalonia (Debug)'),
        @('D', 'Run Avalonia (Release)'),
        @('T', 'Test (dotnet test)'),
        @('C', 'Clean artifacts'),
        @('I', 'Install locally (win-x64)'),
        @('Q', 'Quit')
    )

    for ($i = 0; $i -lt $items.Count; $i++) {
        $active = ($i -eq $Selected)
        Write-MenuItem -Key $items[$i][0] -Label $items[$i][1] -Active:$active
    }

    Write-Host ''
    Write-Host '  Use arrow keys or number to select, Enter to confirm' -ForegroundColor $CDefault
}

function Invoke-Choice {
    param([int]$Selected)
    switch ($Selected) {
        0 { Invoke-BuildAll }
        1 { Invoke-BuildWindows }
        2 { Invoke-BuildLinux }
        3 { Invoke-BuildLinuxPackages }
        4 { Invoke-RunDebug }
        5 { Invoke-RunRelease }
        6 { Invoke-Test }
        7 { Invoke-Clean }
        8 { Invoke-InstallLocal }
        9 { exit 0 }
    }
}

# ---------------------------------------------------------------------------
# Actions
# ---------------------------------------------------------------------------

function Invoke-BuildAll {
    Write-Banner
    Write-Host '  [BUILD ALL]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & (Join-Path $PSScriptRoot 'scripts\build.ps1')
        Write-Host ''
        Write-Host '  Build completed successfully.' -ForegroundColor $CSecondary
    } catch {
        Write-Host ''
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Invoke-BuildWindows {
    Write-Banner
    Write-Host '  [BUILD WINDOWS]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & (Join-Path $PSScriptRoot 'scripts\build.ps1') -SkipLinux -SkipLinuxPackages
        Write-Host ''
        Write-Host '  Windows build completed.' -ForegroundColor $CSecondary
    } catch {
        Write-Host ''
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Invoke-BuildLinux {
    Write-Banner
    Write-Host '  [BUILD LINUX]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & (Join-Path $PSScriptRoot 'scripts\build.ps1') -SkipWindows -SkipLinuxPackages
        Write-Host ''
        Write-Host '  Linux build completed.' -ForegroundColor $CSecondary
    } catch {
        Write-Host ''
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Invoke-BuildLinuxPackages {
    Write-Banner
    Write-Host '  [BUILD LINUX PACKAGES]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & (Join-Path $PSScriptRoot 'scripts\build.ps1') -SkipWindows
        Write-Host ''
        Write-Host '  Linux packages built.' -ForegroundColor $CSecondary
    } catch {
        Write-Host ''
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Invoke-RunDebug {
    Write-Banner
    Write-Host '  [RUN DEBUG]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & dotnet run --project (Join-Path $repoRoot 'src\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj') -c Debug
    } catch {
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Invoke-RunRelease {
    Write-Banner
    Write-Host '  [RUN RELEASE]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & dotnet run --project (Join-Path $repoRoot 'src\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj') -c Release
    } catch {
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Invoke-Test {
    Write-Banner
    Write-Host '  [TEST]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & dotnet test (Join-Path $repoRoot 'GregModmanager.sln') --verbosity normal
        Write-Host ''
        Write-Host '  Tests completed.' -ForegroundColor $CSecondary
    } catch {
        Write-Host ''
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Invoke-Clean {
    Write-Banner
    Write-Host '  [CLEAN]' -ForegroundColor $CPrimary
    Write-Host ''
    $dirs = @(
        'artifacts',
        'build\installer\Output',
        'src\GregModmanager.Avalonia\bin',
        'src\GregModmanager.Avalonia\obj',
        'src\GregModmanager.Core\bin',
        'src\GregModmanager.Core\obj',
        'src\SubDirectoryFixer\bin',
        'src\SubDirectoryFixer\obj',
        'tests\GregModmanager.Tests\bin',
        'tests\GregModmanager.Tests\obj'
    )
    foreach ($d in $dirs) {
        $p = Join-Path $repoRoot $d
        if (Test-Path -LiteralPath $p) {
            Remove-Item -LiteralPath $p -Recurse -Force
            Write-Host "  removed: $d" -ForegroundColor $CWarn
        }
    }
    Write-Host ''
    Write-Host '  Clean finished.' -ForegroundColor $CSecondary
    Pause-AnyKey
}

function Invoke-InstallLocal {
    Write-Banner
    Write-Host '  [INSTALL LOCAL]' -ForegroundColor $CPrimary
    Write-Host ''
    try {
        & (Join-Path $PSScriptRoot 'scripts\install-local.ps1')
    } catch {
        Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor $CError
    }
    Pause-AnyKey
}

function Pause-AnyKey {
    Write-Host ''
    Write-Host '  Press any key to continue...' -ForegroundColor $CDefault -NoNewline
    $null = [Console]::ReadKey($true)
}

# ---------------------------------------------------------------------------
# Input Loop
# ---------------------------------------------------------------------------

$selected = 0
$count = 10

while ($true) {
    Show-Menu -Selected $selected
    $key = [Console]::ReadKey($true)

    switch ($key.Key) {
        'UpArrow'    { if ($selected -gt 0) { $selected-- } }
        'DownArrow'  { if ($selected -lt ($count - 1)) { $selected++ } }
        'Enter'      { Invoke-Choice -Selected $selected }
        'B'          { Invoke-BuildAll }
        'W'          { Invoke-BuildWindows }
        'L'          { Invoke-BuildLinux }
        'P'          { Invoke-BuildLinuxPackages }
        'R'          { Invoke-RunDebug }
        'D'          { Invoke-RunRelease }
        'T'          { Invoke-Test }
        'C'          { Invoke-Clean }
        'I'          { Invoke-InstallLocal }
        'Q'          { exit 0 }
    }
}
