# Lokaler Build - spiegelt den GitHub-Workflow build-and-release.yml wider.
# Unterstuetzt Windows (Setup + Portable), Linux (tar.gz) und Linux-Packages (via WSL).
#
# Voraussetzungen:
#   - .NET 9 SDK
#   - Inno Setup 6 (nur fuer Windows-Setup)
#   - WSL mit bash + dotnet + nfpm (optional fuer Linux-Packages)
#
# Ausfuehren:
#   .\scripts\build.ps1                    # Alles bauen
#   .\scripts\build.ps1 -SkipTest         # Ohne Unit-Tests
#   .\scripts\build.ps1 -SkipLinux        # Nur Windows
#   .\scripts\build.ps1 -SkipWindows      # Nur Linux
#   .\scripts\build.ps1 -Sign             # Mit Code-Signing
#   .\scripts\build.ps1 -SkipPublish      # Publish ueberspringen (nur Setup neu)
#Requires -Version 5.1
param(
    [switch]$SkipTest,
    [switch]$SkipWindows,
    [switch]$SkipLinux,
    [switch]$SkipLinuxPackages,
    [switch]$SkipPublish,
    [switch]$Sign,
    [switch]$SignOnly,
    [string]$SetupPath = '',
    [string]$WslDistro = ''
)

Set-StrictMode -Version Latest
Import-Module Microsoft.PowerShell.Security -ErrorAction SilentlyContinue
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $repoRoot

# Load .env if it exists
$envFile = Join-Path $repoRoot '.env'
if (Test-Path -LiteralPath $envFile) {
    Write-Host "[build] Lade Umgebungsvariablen aus .env ..."
    Get-Content $envFile | Where-Object { $_ -match '=' -and $_ -notmatch '^#' } | ForEach-Object {
        $name, $value = $_.Split('=', 2)
        Set-Item -Path "env:$($name.Trim())" -Value $value.Trim()
    }
}

$isWindowsHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)
$script:AutoSignThumbprint = $null

# ---------------------------------------------------------------------------
# Hilfsfunktionen
# ---------------------------------------------------------------------------

function Get-ProjectVersion {
    $projPath = Join-Path $repoRoot 'src\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj'
    $csproj = [xml](Get-Content -LiteralPath $projPath -Raw)
    $ver = (
        $csproj.Project.PropertyGroup |
        ForEach-Object { $_.Version } |
        Where-Object { $_ } |
        Select-Object -First 1
    ).Trim()
    if ([string]::IsNullOrWhiteSpace($ver)) { $ver = '1.5.0' }

    $numeric = $ver -replace '-.*$', ''
    if ($numeric -notmatch '^\d+\.\d+\.\d+\.\d+$') {
        if ($numeric -match '^\d+\.\d+\.\d+$') { $numeric += '.0' }
        else { $numeric = '1.5.0.0' }
    }

    $isPre = $ver -match '-'
    if (-not $isPre) {
        try {
            $branch = & git -C $repoRoot rev-parse --abbrev-ref HEAD 2>$null
            if ($LASTEXITCODE -eq 0 -and $branch -ne 'main' -and $branch -ne 'master') {
                $isPre = $true
            }
        }
        catch { }
    }

    return [PSCustomObject]@{
        Version        = $ver
        NumericVersion = $numeric
        IsPre          = $isPre
        PreSuffix      = if ($isPre) { '-pre' } else { '' }
    }
}

function New-Sha256File {
    param([Parameter(Mandatory)][string]$TargetPath)
    if (-not (Test-Path -LiteralPath $TargetPath)) {
        throw "Datei fuer SHA256 nicht gefunden: $TargetPath"
    }
    $hash = (Get-FileHash -LiteralPath $TargetPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $leafName = [System.IO.Path]::GetFileName($TargetPath)
    $hashFile = "$TargetPath.sha256"
    Set-Content -LiteralPath $hashFile -Value "$hash *$leafName" -NoNewline -Encoding ascii
    Write-Host "[build] SHA256: $hashFile"
}

function Test-ZipArchiveExtractable {
    param(
        [Parameter(Mandatory)][string]$ZipPath,
        [string]$ExpectedRelativePath = ''
    )
    if (-not (Test-Path -LiteralPath $ZipPath)) { throw "ZIP nicht gefunden: $ZipPath" }
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("gregtools-zipcheck-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null
    try {
        Expand-Archive -LiteralPath $ZipPath -DestinationPath $tempRoot -Force
        $anyFile = Get-ChildItem -LiteralPath $tempRoot -Recurse -File | Select-Object -First 1
        if (-not $anyFile) { throw "ZIP ist leer: $ZipPath" }
        if (-not [string]::IsNullOrWhiteSpace($ExpectedRelativePath)) {
            $expectedLeaf = [System.IO.Path]::GetFileName($ExpectedRelativePath)
            $found = Get-ChildItem -LiteralPath $tempRoot -Recurse -File | Where-Object { $_.Name -ieq $expectedLeaf } | Select-Object -First 1
            if (-not $found) { throw "Erwartete Datei '$ExpectedRelativePath' fehlt im Archiv: $ZipPath" }
        }
        Write-Host "[build] Entpack-Test OK: $ZipPath"
    }
    finally {
        if (Test-Path -LiteralPath $tempRoot) { Remove-Item -LiteralPath $tempRoot -Recurse -Force }
    }
}

function Assert-AuthenticodeSignaturePresent {
    param([Parameter(Mandatory)][string]$TargetPath)
    if (-not (Test-Path -LiteralPath $TargetPath)) { throw "Datei fehlt: $TargetPath" }
    $signature = Get-AuthenticodeSignature -FilePath $TargetPath
    if ($signature.Status -eq [System.Management.Automation.SignatureStatus]::NotSigned -or -not $signature.SignerCertificate) {
        throw "Signaturpruefung fehlgeschlagen: $TargetPath"
    }
    Write-Host "[build] Signaturpruefung OK: $TargetPath (Status=$($signature.Status))"
}

function New-EphemeralCodeSignThumbprint {
    if (-not $isWindowsHost) { throw "Ephemeral code signing is only supported on Windows." }
    if (-not [string]::IsNullOrWhiteSpace($script:AutoSignThumbprint)) { return $script:AutoSignThumbprint }
    $subject = "CN=GregTools Local Build " + (Get-Date -Format 'yyyyMMdd-HHmmss')
    $notAfter = (Get-Date).AddDays(7)
    Write-Host "[build] Erzeuge temporaeres Self-Signed-Code-Signing-Zertifikat: $subject"
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $subject `
        -KeyUsage DigitalSignature `
        -KeyAlgorithm RSA `
        -KeyLength 2048 `
        -HashAlgorithm SHA256 `
        -NotAfter $notAfter `
        -CertStoreLocation Cert:\CurrentUser\My `
        -FriendlyName "GregTools ephemeral build signing"
    $script:AutoSignThumbprint = $cert.Thumbprint
    Write-Host "[build] Temporaerer Signing-Thumbprint: $($script:AutoSignThumbprint)"
    return $script:AutoSignThumbprint
}

function Invoke-BuildSign {
    param([Parameter(Mandatory)][string]$TargetPath)
    $signScript = Join-Path $repoRoot 'build\installer\sign-authenticode.ps1'
    if (-not (Test-Path -LiteralPath $signScript)) { throw "Signierskript fehlt: $signScript" }
    $thumb = $env:CODE_SIGN_THUMBPRINT
    $pfx = $env:CODE_SIGN_PFX
    if ([string]::IsNullOrWhiteSpace($thumb) -and [string]::IsNullOrWhiteSpace($pfx)) {
        $thumb = New-EphemeralCodeSignThumbprint
    }
    if ([string]::IsNullOrWhiteSpace($thumb) -eq [string]::IsNullOrWhiteSpace($pfx)) {
        throw "Ungueltige Signierkonfiguration: entweder CODE_SIGN_THUMBPRINT oder CODE_SIGN_PFX setzen."
    }
    if (-not (Test-Path -LiteralPath $TargetPath)) { throw "Datei zum Signieren nicht gefunden: $TargetPath" }
    Write-Host "[build] Authenticode-Signatur: $TargetPath"
    if (-not [string]::IsNullOrWhiteSpace($thumb)) {
        $t = $thumb.Trim()
        if ($t -match '<|>') { throw "CODE_SIGN_THUMBPRINT ist noch ein Platzhalter." }
        & $signScript -Path $TargetPath -Thumbprint $t
    }
    else {
        & $signScript -Path $TargetPath -PfxPath $pfx.Trim()
    }
}

function Invoke-SignWindowsPayloadBinaries {
    param([Parameter(Mandatory)][string]$PublishDirectory)
    $files = Get-ChildItem -LiteralPath $PublishDirectory -Recurse -File |
    Where-Object { $_.Extension -in @('.exe', '.dll') -and $_.Name -ne 'steam_api64.dll' } |
    Sort-Object FullName
    foreach ($file in $files) {
        $sig = Get-AuthenticodeSignature -FilePath $file.FullName
        if ($sig.Status -eq [System.Management.Automation.SignatureStatus]::Valid) {
            Write-Host "[build] Bereits signiert: $($file.FullName)"
            continue
        }
        try {
            Invoke-BuildSign -TargetPath $file.FullName
        }
        catch {
            Write-Warning "[build] Signieren fehlgeschlagen fuer $($file.Name): $($_.Exception.Message)"
        }
    }
}

function New-DetachedArtifactSignature {
    param([Parameter(Mandatory)][string]$TargetPath)
    if (-not (Test-Path -LiteralPath $TargetPath)) { throw "Artefakt nicht gefunden: $TargetPath" }
    $openssl = Get-Command openssl -ErrorAction SilentlyContinue
    if (-not $openssl) {
        Write-Warning "[build] openssl nicht gefunden - ueberspringe detached Signature."
        return
    }
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("greg-sign-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    try {
        $keyPath = Join-Path $tmpDir 'key.pem'
        $certPath = Join-Path $tmpDir 'cert.pem'
        & openssl genrsa -out $keyPath 2048 2>$null
        & openssl req -new -x509 -key $keyPath -out $certPath -days 7 -subj "/CN=GregTools Local Build" 2>$null
        $sigPath = "$TargetPath.sig"
        & openssl dgst -sha256 -sign $keyPath -out $sigPath $TargetPath 2>$null
        Copy-Item -LiteralPath $certPath -Destination "$TargetPath.sig.cer" -Force
        Write-Host "[build] Detached-Signatur: $sigPath"
    }
    finally {
        if (Test-Path -LiteralPath $tmpDir) { Remove-Item -LiteralPath $tmpDir -Recurse -Force }
    }
}

function Invoke-BuildSubDirectoryFixer {
    $fixerProjPath = Join-Path $repoRoot 'src\SubDirectoryFixer\SubDirectoryFixer.csproj'
    $fixerAssetsDir = Join-Path $repoRoot 'src\GregModmanager.Avalonia\Assets\SubDirectoryFixer'
    $fixerDllPath = Join-Path $fixerAssetsDir 'SubDirectoryFixer.dll'
    if (-not (Test-Path -LiteralPath $fixerProjPath)) {
        Write-Warning '[build] SubDirectoryFixer-Projekt nicht gefunden - wird uebersprungen.'
        return
    }
    Write-Host '[build] Baue SubDirectoryFixer (net6.0) ...'
    & dotnet build $fixerProjPath -c Release
    if ($LASTEXITCODE -ne 0) { throw 'SubDirectoryFixer Build fehlgeschlagen.' }
    $builtDll = Join-Path $repoRoot 'src\SubDirectoryFixer\bin\Release\net6.0\SubDirectoryFixer.dll'
    if (-not (Test-Path -LiteralPath $builtDll)) { throw 'SubDirectoryFixer DLL nicht gefunden.' }
    New-Item -ItemType Directory -Path $fixerAssetsDir -Force | Out-Null
    Copy-Item -LiteralPath $builtDll -Destination $fixerDllPath -Force
    Write-Host "[build] SubDirectoryFixer (plugin) bereitgestellt: $fixerDllPath"
}

function Add-TelemetrySecrets {
    $secretsPath = Join-Path $repoRoot 'src\GregModmanager.Core\Services\TelemetrySecrets.cs'
    if (-not (Test-Path -LiteralPath $secretsPath)) {
        Write-Warning "[build] TelemetrySecrets.cs nicht gefunden - ueberspringe Injektion."
        return
    }

    $url = $env:TELEMETRY_URL
    $user = $env:TELEMETRY_USER
    $pass = $env:TELEMETRY_PASS
    $tenant = $env:TELEMETRY_TENANT

    if ([string]::IsNullOrWhiteSpace($url)) {
        Write-Host "[build] TELEMETRY_URL nicht gesetzt - Telemetrie-Platzhalter bleiben bestehen."
        return
    }

    Write-Host "[build] Injektiere Telemetrie-Geheimnisse aus Umgebungsvariablen ..."
    $content = Get-Content -LiteralPath $secretsPath -Raw
    $content = $content.Replace('__LOKI_URL__', $url)
    if (-not [string]::IsNullOrWhiteSpace($user)) { $content = $content.Replace('__LOKI_USER__', $user) }
    if (-not [string]::IsNullOrWhiteSpace($pass)) { $content = $content.Replace('__LOKI_PASS__', $pass) }
    if (-not [string]::IsNullOrWhiteSpace($tenant)) { $content = $content.Replace('__LOKI_TENANT__', $tenant) }

    Set-Content -LiteralPath $secretsPath -Value $content -Force
}

# ---------------------------------------------------------------------------
# Phase 1 - Build & Test
# ---------------------------------------------------------------------------

$verInfo = Get-ProjectVersion
Add-TelemetrySecrets
$ver = $verInfo.Version
$numericVer = $verInfo.NumericVersion
Write-Host ''
Write-Host '========================================'
Write-Host '  GregModmanager Build'
Write-Host "  Version: $ver ($numericVer)"
Write-Host "  OS:      $([System.Runtime.InteropServices.RuntimeInformation]::OSDescription)"
Write-Host '========================================'
Write-Host ''

if (-not $SkipTest) {
    Write-Host '[build] dotnet restore ...'
    & dotnet restore (Join-Path $repoRoot 'GregModmanager.sln')
    if ($LASTEXITCODE -ne 0) { throw 'dotnet restore fehlgeschlagen.' }

    Write-Host '[build] dotnet build -c Release ...'
    & dotnet build (Join-Path $repoRoot 'GregModmanager.sln') --no-restore -c Release
    if ($LASTEXITCODE -ne 0) { throw 'dotnet build fehlgeschlagen.' }

    Write-Host '[build] dotnet test ...'
    & dotnet test (Join-Path $repoRoot 'GregModmanager.sln') --no-build -c Release --verbosity normal
    if ($LASTEXITCODE -ne 0) { throw 'dotnet test fehlgeschlagen.' }
}
else {
    Write-Host '[build] Tests uebersprungen (-SkipTest).'
}

$wantSign = $Sign
# Always use self-signed certificate if no explicit signing is configured
if (-not $wantSign -and -not $env:CODE_SIGN_THUMBPRINT -and -not $env:CODE_SIGN_PFX) {
    $wantSign = $true
    Write-Host '[build] Using self-signed certificate for code signing (no CODE_SIGN_THUMBPRINT or CODE_SIGN_PFX set).'
}
$projPath = Join-Path $repoRoot 'src\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj'
$iss = Join-Path $repoRoot 'build\installer\gregModmanager.iss'
$winPublishDir = Join-Path $repoRoot 'src\GregModmanager.Avalonia\bin\Release\net9.0\win-x64\publish'
$linuxPublishDir = Join-Path $repoRoot 'artifacts\publish\linux-x64'
$installerOutDir = Join-Path $repoRoot 'build\installer\Output'
$artifactsDir = Join-Path $repoRoot 'artifacts'

New-Item -ItemType Directory -Path $installerOutDir -Force | Out-Null
New-Item -ItemType Directory -Path $artifactsDir -Force | Out-Null

# ---------------------------------------------------------------------------
# Phase 2 - Windows Build
# ---------------------------------------------------------------------------

if ($isWindowsHost -and -not $SkipWindows) {
    if (-not $SkipPublish) {
        Invoke-BuildSubDirectoryFixer
        if (Test-Path -LiteralPath $winPublishDir) {
            Write-Host "[build] Bereinige alte Windows-Publish-Ausgabe: $winPublishDir"
            Remove-Item -LiteralPath $winPublishDir -Recurse -Force
        }
        Write-Host '[build] dotnet publish Windows (win-x64) ...'
        & dotnet publish $projPath -c Release -r win-x64
        if ($LASTEXITCODE -ne 0) { throw 'Windows publish fehlgeschlagen.' }
    }
    else {
        Write-Warning '[build] -SkipPublish: bestehende Windows-Publish-Ausgabe wird verwendet.'
    }

    if (-not (Test-Path -LiteralPath $winPublishDir)) { throw "Windows Publish-Ausgabe nicht gefunden: $winPublishDir" }

    $portableZipPath = Join-Path $installerOutDir ("gregModmanager-{0}{1}-Windows.zip" -f $ver, $verInfo.PreSuffix)
    if (Test-Path -LiteralPath $portableZipPath) { Remove-Item -LiteralPath $portableZipPath -Force }

    if ($wantSign) {
        Write-Host '[build] Signiere Windows PE-Binaries ...'
        Invoke-SignWindowsPayloadBinaries -PublishDirectory $winPublishDir
    }

    Write-Host "[build] Packe Win64 Portable ZIP: $portableZipPath"
    Compress-Archive -Path (Join-Path $winPublishDir '*') -DestinationPath $portableZipPath -CompressionLevel Optimal
    Test-ZipArchiveExtractable -ZipPath $portableZipPath -ExpectedRelativePath 'GregModmanager.exe'
    New-Sha256File -TargetPath $portableZipPath
    if ($wantSign) { New-DetachedArtifactSignature -TargetPath $portableZipPath }

    # Inno Setup
    $isccCandidates = @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
        (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe')
    )
    $iscc = $isccCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $iscc) {
        Write-Warning '[build] Inno Setup 6 nicht gefunden - Setup-EXE wird nicht erstellt.'
    }
    else {
        if (-not (Test-Path -LiteralPath $iss)) { throw "Inno-Skript fehlt: $iss" }
        Write-Host "[build] Inno Setup ($iscc) - Version $ver ..."
        $outputBaseName = "gregModmanager-$ver$($verInfo.PreSuffix)-Windows"
        $argList = @($iss, "/DMyAppVersion=$ver", "/DMyAppNumericVersion=$numericVer", "/DMyAppOutputBaseFilename=$outputBaseName")
        & $iscc @argList
        if ($LASTEXITCODE -ne 0) { throw "ISCC beendet mit Code $LASTEXITCODE" }

        $setupName = "gregModmanager-$ver$($verInfo.PreSuffix)-Windows.exe"
        $setupPath = Join-Path $installerOutDir $setupName
        if (Test-Path -LiteralPath $setupPath) {
            $mb = [math]::Round((Get-Item -LiteralPath $setupPath).Length / 1MB, 2)
            Write-Host ('[build] Setup erstellt: ' + $setupPath + ' (' + $mb + ' MB)')
        }
        else {
            Write-Warning '[build] Setup-Datei nicht an erwartetem Ort gefunden.'
        }

        if ($wantSign) {
            Write-Host '[build] Signiere Setup-EXE ...'
            Invoke-BuildSign -TargetPath $setupPath
            Assert-AuthenticodeSignaturePresent -TargetPath $setupPath
        }
        New-Sha256File -TargetPath $setupPath
        if ($wantSign) { New-DetachedArtifactSignature -TargetPath $setupPath }
    }
}
elseif ($SkipWindows) {
    Write-Host '[build] Windows-Build uebersprungen (-SkipWindows).'
}
else {
    Write-Host '[build] Nicht-Windows-Host: Windows-Build wird uebersprungen.'
}

# ---------------------------------------------------------------------------
# Phase 3 - Linux Build
# ---------------------------------------------------------------------------

if (-not $SkipLinux) {
    if (Test-Path -LiteralPath $linuxPublishDir) {
        Write-Host "[build] Bereinige alte Linux-Publish-Ausgabe: $linuxPublishDir"
        Remove-Item -LiteralPath $linuxPublishDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $linuxPublishDir -Force | Out-Null

    Invoke-BuildSubDirectoryFixer
    Write-Host '[build] dotnet publish Linux (linux-x64) ...'
    & dotnet publish $projPath -c Release -r linux-x64 `
        --self-contained true /p:PublishTrimmed=true /p:TrimMode=full `
        /p:PublishSingleFile=true /p:DebugType=none /p:DebugSymbols=false `
        -o $linuxPublishDir
    if ($LASTEXITCODE -ne 0) { throw 'Linux publish fehlgeschlagen.' }

    $linuxBin = Join-Path $linuxPublishDir 'GregModmanager'
    if (-not (Test-Path -LiteralPath $linuxBin)) { throw "Linux-Binary nicht gefunden: $linuxBin" }

    # Linux Standalone ZIP
    $linuxZipPath = Join-Path $installerOutDir ("gregModmanager-{0}{1}-Linux.zip" -f $ver, $verInfo.PreSuffix)
    if (Test-Path -LiteralPath $linuxZipPath) { Remove-Item -LiteralPath $linuxZipPath -Force }
    Write-Host "[build] Packe Linux Standalone ZIP: $linuxZipPath"
    Compress-Archive -Path (Join-Path $linuxPublishDir '*') -DestinationPath $linuxZipPath -CompressionLevel Optimal
    New-Sha256File -TargetPath $linuxZipPath

    # Linux Tarball
    $tarName = "gregModmanager-$ver$($verInfo.PreSuffix)-Linux.tar.gz"
    $tarPath = Join-Path $artifactsDir $tarName
    if (Test-Path -LiteralPath $tarPath) { Remove-Item -LiteralPath $tarPath -Force }

    Write-Host "[build] Erstelle Tarball: $tarPath"
    $tarExe = Get-Command tar -ErrorAction SilentlyContinue
    if (-not $tarExe) { throw 'tar nicht gefunden. Windows 10/11 sollte tar.exe enthalten.' }
    & tar.exe -czf $tarPath -C $linuxPublishDir .
    if ($LASTEXITCODE -ne 0) { throw 'tar fehlgeschlagen.' }

    New-Sha256File -TargetPath $tarPath
    Write-Host "[build] Linux Tarball fertig: $tarPath"
}
else {
    Write-Host '[build] Linux-Build uebersprungen (-SkipLinux).'
}

# ---------------------------------------------------------------------------
# Phase 4 - Linux Packages (optional via WSL)
# ---------------------------------------------------------------------------

if (-not $SkipLinuxPackages -and -not $SkipLinux) {
    $wsl = Get-Command wsl.exe -ErrorAction SilentlyContinue
    if (-not $wsl) {
        Write-Warning '[build] wsl.exe nicht gefunden - Linux-Packages werden uebersprungen.'
    }
    else {
        $linuxPkgScript = Join-Path $repoRoot 'build\scripts\linux\build-avalonia-packages.ps1'
        if (-not (Test-Path -LiteralPath $linuxPkgScript)) {
            Write-Warning "[build] Linux-Package-Skript nicht gefunden: $linuxPkgScript"
        }
        else {
            Write-Host '[build] Baue Linux-Packages (DEB/RPM/Arch) via WSL ...'
            $pkgOut = Join-Path $artifactsDir 'avalonia-linux'
            try {
                $preFlag = if ($verInfo.IsPre) { $true } else { $false }
                & $linuxPkgScript -OutputDir $pkgOut -Version $ver -WslDistro $WslDistro -IsPre:$preFlag
                if ($LASTEXITCODE -ne 0) { throw 'Linux-Packages fehlgeschlagen.' }
                $pkgDir = Join-Path $pkgOut 'packages'
                if (Test-Path -LiteralPath $pkgDir) {
                    $pkgs = Get-ChildItem -LiteralPath $pkgDir -File -ErrorAction SilentlyContinue
                    foreach ($pkg in $pkgs) {
                        New-Sha256File -TargetPath $pkg.FullName
                    }
                }
                Write-Host "[build] Linux-Packages fertig: $pkgOut\packages"
            }
            catch {
                Write-Warning "[build] Linux-Packages fehlgeschlagen: $($_.Exception.Message)"
            }
        }
    }
}
else {
    Write-Host '[build] Linux-Packages uebersprungen.'
}

# ---------------------------------------------------------------------------
# Zusammenfassung
# ---------------------------------------------------------------------------

Write-Host ''
Write-Host '========================================'
Write-Host '  Build abgeschlossen'
Write-Host '========================================'
Write-Host ''

if ($isWindowsHost -and -not $SkipWindows) {
    if (Test-Path -LiteralPath $installerOutDir) {
        Get-ChildItem -LiteralPath $installerOutDir -File | ForEach-Object {
            Write-Host ('  [Windows] ' + $_.Name + '  (' + ([math]::Round($_.Length / 1KB, 1)) + ' KB)')
        }
    }
}
if (-not $SkipLinux) {
    if (Test-Path -LiteralPath $artifactsDir) {
        Get-ChildItem -LiteralPath $artifactsDir -File | ForEach-Object {
            Write-Host ('  [Linux]   ' + $_.Name + '  (' + ([math]::Round($_.Length / 1KB, 1)) + ' KB)')
        }
    }
    $pkgDir = Join-Path $artifactsDir 'avalonia-linux\packages'
    if (Test-Path -LiteralPath $pkgDir) {
        Get-ChildItem -LiteralPath $pkgDir -File | ForEach-Object {
            Write-Host ('  [Pkg]     ' + $_.Name + '  (' + ([math]::Round($_.Length / 1KB, 1)) + ' KB)')
        }
    }
}
Write-Host ''

exit 0