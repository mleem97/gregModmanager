# Schnell-„Installation“ ohne Setup-EXE: Publish nach %LOCALAPPDATA%\Programs\… + Verknüpfungen.
# Für einen echten Installer siehe .\build.ps1 (Inno Setup).
#
# .\install-local.ps1
# .\install-local.ps1 -SkipPublish
# .\install-local.ps1 -Uninstall
#Requires -Version 5.1
param(
    [string]$InstallDir = '',
    [switch]$SkipPublish,
    [switch]$Uninstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $repoRoot

$AppFolderName = 'gregModmanager'
$ExeName = 'GregModmanager.exe'
$ShortcutName = 'gregModmanager.lnk'

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$AppFolderName"
}

$publishRel = Join-Path $repoRoot "GregModmanager.Avalonia\bin\Release\net9.0\win-x64\publish"
$exePath = Join-Path $InstallDir $ExeName
$desktop = [Environment]::GetFolderPath([Environment+SpecialFolder]::Desktop)
$startMenuPrograms = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$desktopLnk = Join-Path $desktop $ShortcutName
$startMenuLnk = Join-Path $startMenuPrograms $ShortcutName

function Remove-Shortcuts {
    foreach ($p in @($desktopLnk, $startMenuLnk)) {
        if (Test-Path -LiteralPath $p) {
            Remove-Item -LiteralPath $p -Force
            Write-Host "[install-local] Entfernt: $p"
        }
    }
}

if ($Uninstall) {
    Remove-Shortcuts
    if (Test-Path -LiteralPath $InstallDir) {
        Remove-Item -LiteralPath $InstallDir -Recurse -Force
        Write-Host "[install-local] Installationsordner entfernt: $InstallDir"
    } else {
        Write-Host "[install-local] Kein Ordner vorhanden: $InstallDir"
    }
    exit 0
}

if (-not $SkipPublish) {
    Write-Host '[install-local] dotnet publish -c Release ...'
    & dotnet publish (Join-Path $repoRoot 'GregModmanager.Avalonia\GregModmanager.Avalonia.csproj') -c Release -r win-x64
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not (Test-Path -LiteralPath $publishRel)) {
    throw "Publish-Ausgabe nicht gefunden: $publishRel"
}

# Update/Reinstall: laufende App beenden, sonst bleiben EXE/DLL gesperrt und der Ordner lässt sich nicht sauber ersetzen.
Get-Process -Name 'GregModmanager' -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "[install-local] Beende laufende Instanz (PID $($_.Id)) …"
    Stop-Process -Id $_.Id -Force
}
Start-Sleep -Milliseconds 400

Write-Host "[install-local] Kopiere nach $InstallDir (bestehende Installation wird vollständig ersetzt) …"
if (Test-Path -LiteralPath $InstallDir) {
    Remove-Item -LiteralPath $InstallDir -Recurse -Force
}
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Copy-Item -Path (Join-Path $publishRel '*') -Destination $InstallDir -Recurse -Force

if (-not (Test-Path -LiteralPath $exePath)) {
    throw "Ausführbare Datei fehlt: $exePath"
}

$shell = New-Object -ComObject WScript.Shell
foreach ($lnkPath in @($desktopLnk, $startMenuLnk)) {
    $sc = $shell.CreateShortcut($lnkPath)
    $sc.TargetPath = $exePath
    $sc.WorkingDirectory = $InstallDir
    $sc.Description = 'gregModmanager (Data Center Workshop)'
    $sc.IconLocation = "$exePath,0"
    $sc.Save()
    Write-Host "[install-local] Verknüpfung: $lnkPath"
}
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($shell) | Out-Null

Write-Host ''
Write-Host "[install-local] Fertig. Start: $exePath"

