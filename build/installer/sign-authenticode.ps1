# Signiert eine EXE mit Authenticode (SHA-256 + Zeitstempel), damit Windows einen
# namentlichen Herausgeber anzeigt — nur mit Zertifikat einer vertrauenden CA, nicht selbstsigniert.
#
# Voraussetzung: Windows SDK (signtool.exe) — z. B. „Windows SDK“-Workload in Visual Studio.
#
# Beispiele:
#   .\installer\sign-authenticode.ps1 -Path .\installer\Output\gregModmanager-1.5.0-Setup.exe -Thumbprint ABC123...
#   $env:CODE_SIGN_PFX_PASSWORD = '***'; .\installer\sign-authenticode.ps1 -Path .\foo.exe -PfxPath C:\certs\codesign.pfx
#Requires -Version 5.1
param(
    [Parameter(Mandatory)]
    [string]$Path,
    [string]$Thumbprint = '',
    [string]$PfxPath = '',
    [Security.SecureString]$PfxPassword = $null,
    [string]$TimestampUrl = 'http://timestamp.digicert.com',
    [switch]$NoTimestamp
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $IsWindows) {
    throw "Authenticode signing is only supported on Windows hosts. Use a Windows runner for this step."
}

if (-not (Test-Path -LiteralPath $Path)) {
    throw "Datei nicht gefunden: $Path"
}

$usePfx = -not [string]::IsNullOrWhiteSpace($PfxPath)
$useThumb = -not [string]::IsNullOrWhiteSpace($Thumbprint)
if ($usePfx -eq $useThumb) {
    throw "Genau eine Option angeben: -Thumbprint (Zertifikat im Windows-Zertifikatspeicher) oder -PfxPath (PFX-Datei)."
}

if ($usePfx -and $null -eq $PfxPassword) {
    $plain = $env:CODE_SIGN_PFX_PASSWORD
    if (-not [string]::IsNullOrWhiteSpace($plain)) {
        $PfxPassword = ConvertTo-SecureString -String $plain -AsPlainText -Force
    }
}
if ($usePfx -and $null -eq $PfxPassword) {
    throw "Bei -PfxPath Passwort angeben oder Umgebungsvariable CODE_SIGN_PFX_PASSWORD setzen."
}

function Get-SignToolPath {
    $fromPath = Get-Command 'signtool.exe' -ErrorAction SilentlyContinue
    if ($fromPath) { return $fromPath.Source }
    $candidates = @(
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin",
        "$env:ProgramFiles\Windows Kits\10\bin"
    )
    foreach ($root in $candidates) {
        if (-not (Test-Path -LiteralPath $root)) { continue }
        $found = Get-ChildItem -LiteralPath $root -Filter 'signtool.exe' -Recurse -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match '\\x64\\signtool\.exe$' }
        if ($found) {
            return ($found | Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName)
        }
    }
    return $null
}

$signtool = Get-SignToolPath
if (-not $signtool) {
    throw @"
signtool.exe nicht gefunden (Windows 10/11 SDK).

Installiere das Windows SDK oder die „Desktopentwicklung mit C++“-Workload in Visual Studio,
oder lege signtool.exe auf dem PATH ab.
"@
}

$args = @('sign', '/fd', 'SHA256', '/v')
if (-not $NoTimestamp) {
    $args += @('/tr', $TimestampUrl, '/td', 'SHA256')
}
if ($useThumb) {
    $args += @('/sha1', $Thumbprint)
} else {
    $args += @('/f', $PfxPath, '/p', $PfxPassword)
}
$args += $Path

Write-Host "[sign] $signtool $($args -join ' ')"
& $signtool @args
if ($LASTEXITCODE -ne 0) {
    throw "signtool ExitCode $LASTEXITCODE"
}

$sig = Get-AuthenticodeSignature -FilePath $Path
Write-Host "[sign] Signatur: Status=$($sig.Status) SignaturTyp=$($sig.SignerCertificate.Subject)"

