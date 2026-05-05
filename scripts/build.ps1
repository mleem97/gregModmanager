# Erstellt ein Release-Publish und kompiliert eine echte Setup-EXE mit Inno Setup 6
# (Assistent, Eintrag unter "Apps", Deinstallieren, Desktop-Verknuepfung optional).
#
# Voraussetzung: Inno Setup 6 installieren - https://jrsoftware.org/isdl.php
#
# Aus diesem Ordner: .\build.ps1
# Nur Setup neu bauen (Publish schon vorhanden): .\build.ps1 -SkipPublish
# Nur signieren (ohne Inno Setup): .\build.ps1 -SignOnly  (+ CODE_SIGN_THUMBPRINT)
# Nach dem Setup Authenticode-Signatur: .\build.ps1 -Sign
#   Umgebung: CODE_SIGN_THUMBPRINT=... oder CODE_SIGN_PFX=... + CODE_SIGN_PFX_PASSWORD
#Requires -Version 5.1
param(
    [switch]$SkipPublish,
    [switch]$Sign,
    [switch]$SignOnly,
    [string]$SetupPath = '',
    [string[]]$LinuxDistros = @()
)

Set-StrictMode -Version Latest
Import-Module Microsoft.PowerShell.Security -ErrorAction SilentlyContinue
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $repoRoot
$isWindowsHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)
$script:AutoSignThumbprint = $null

function New-Sha256File {
    param([Parameter(Mandatory)][string]$TargetPath)

    if (-not (Test-Path -LiteralPath $TargetPath)) {
        throw "Datei fuer SHA256 nicht gefunden: $TargetPath"
    }

    $hash = ""
    if (Get-Command Get-FileHash -ErrorAction SilentlyContinue) {
        $hash = (Get-FileHash -LiteralPath $TargetPath -Algorithm SHA256).Hash.ToLowerInvariant()
    } else {
        # Fallback to .NET for environments without Get-FileHash
        $stream = [System.IO.File]::OpenRead($TargetPath)
        $sha = [System.Security.Cryptography.SHA256]::Create()
        $hashBytes = $sha.ComputeHash($stream)
        $stream.Close()
        $hash = (($hashBytes | ForEach-Object { "{0:x2}" -f $_ }) -join "")
    }

    $leafName = [System.IO.Path]::GetFileName($TargetPath)
    $hashFile = "$TargetPath.sha256"
    Set-Content -LiteralPath $hashFile -Value "$hash *$leafName" -NoNewline -Encoding ascii
    Write-Host "[build] SHA256: $hashFile"
    return $hashFile
}

function Test-ZipArchiveExtractable {
    param(
        [Parameter(Mandatory)][string]$ZipPath,
        [string]$ExpectedRelativePath = ''
    )

    if (-not (Test-Path -LiteralPath $ZipPath)) {
        throw "ZIP nicht gefunden: $ZipPath"
    }

    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("gregtools-zipcheck-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null

    try {
        Expand-Archive -LiteralPath $ZipPath -DestinationPath $tempRoot -Force
        $anyFile = Get-ChildItem -LiteralPath $tempRoot -Recurse -File | Select-Object -First 1
        if (-not $anyFile) {
            throw "ZIP ist leer oder konnte nicht korrekt entpackt werden: $ZipPath"
        }

        if (-not [string]::IsNullOrWhiteSpace($ExpectedRelativePath)) {
            $expectedLeaf = [System.IO.Path]::GetFileName($ExpectedRelativePath)
            $found = Get-ChildItem -LiteralPath $tempRoot -Recurse -File | Where-Object { $_.Name -ieq $expectedLeaf } | Select-Object -First 1
            if (-not $found) {
                throw "Erwartete Datei '$ExpectedRelativePath' fehlt im Archiv: $ZipPath"
            }
        }

        Write-Host "[build] Entpack-Test OK: $ZipPath"
    }
    finally {
        if (Test-Path -LiteralPath $tempRoot) {
            Remove-Item -LiteralPath $tempRoot -Recurse -Force
        }
    }
}

function Assert-AuthenticodeSignaturePresent {
    param([Parameter(Mandatory)][string]$TargetPath)

    if (-not (Test-Path -LiteralPath $TargetPath)) {
        throw "Datei fuer Signaturpruefung fehlt: $TargetPath"
    }

    $signature = Get-AuthenticodeSignature -FilePath $TargetPath
    if ($signature.Status -eq [System.Management.Automation.SignatureStatus]::NotSigned -or -not $signature.SignerCertificate) {
        throw "Signaturpruefung fehlgeschlagen (NotSigned): $TargetPath"
    }

    Write-Host "[build] Signaturpruefung OK: $TargetPath (Status=$($signature.Status))"
}

function Get-CodeSigningCertificate {
    $thumb = $env:CODE_SIGN_THUMBPRINT
    $pfxPath = $env:CODE_SIGN_PFX
    $pfxPassword = $env:CODE_SIGN_PFX_PASSWORD

    if ([string]::IsNullOrWhiteSpace($thumb) -and [string]::IsNullOrWhiteSpace($pfxPath)) {
        $thumb = New-EphemeralCodeSignThumbprint
    }

    if (-not [string]::IsNullOrWhiteSpace($thumb) -and -not [string]::IsNullOrWhiteSpace($pfxPath)) {
        throw "Ungueltige Signierkonfiguration: entweder CODE_SIGN_THUMBPRINT oder CODE_SIGN_PFX setzen."
    }

    if (-not [string]::IsNullOrWhiteSpace($thumb)) {
        $trimmed = $thumb.Trim()
        $candidatePaths = @(
            "Cert:\CurrentUser\My\$trimmed",
            "Cert:\LocalMachine\My\$trimmed"
        )

        foreach ($certPath in $candidatePaths) {
            $cert = Get-Item -LiteralPath $certPath -ErrorAction SilentlyContinue
            if ($cert -and $cert.HasPrivateKey) {
                return $cert
            }
        }

        throw "Code-Signing-Zertifikat mit privatem Schluessel nicht gefunden: $trimmed"
    }

    if (-not (Test-Path -LiteralPath $pfxPath)) {
        throw "PFX-Datei nicht gefunden: $pfxPath"
    }

    $flags = [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::Exportable
    $flags = $flags -bor [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::PersistKeySet
    if ($isWindowsHost) {
        $flags = $flags -bor [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::MachineKeySet
    }

    return New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($pfxPath.Trim(), $pfxPassword, $flags)
}

function New-DetachedArtifactSignature {
    param([Parameter(Mandatory)][string]$TargetPath)

    if (-not (Test-Path -LiteralPath $TargetPath)) {
        throw "Artefakt fuer Detached-Signatur nicht gefunden: $TargetPath"
    }

    $cert = Get-CodeSigningCertificate
    $rsa = $null
    try {
        $rsa = [System.Security.Cryptography.X509Certificates.RSACertificateExtensions]::GetRSAPrivateKey($cert)
    } catch {
        $rsa = $null
    }

    if (-not $rsa -and $cert.PrivateKey -is [System.Security.Cryptography.RSACryptoServiceProvider]) {
        $rsa = [System.Security.Cryptography.RSACryptoServiceProvider]$cert.PrivateKey
    }

    if (-not $rsa -and $cert.PrivateKey -is [System.Security.Cryptography.RSA]) {
        $rsa = [System.Security.Cryptography.RSA]$cert.PrivateKey
    }
    if (-not $rsa) {
        throw "RSA Private Key fuer Detached-Signatur nicht verfuegbar."
    }

    $payloadBytes = [System.IO.File]::ReadAllBytes($TargetPath)
    $signatureBytes = $rsa.SignData($payloadBytes, [System.Security.Cryptography.HashAlgorithmName]::SHA256, [System.Security.Cryptography.RSASignaturePadding]::Pkcs1)
    $signaturePath = "$TargetPath.sig"
    $certificatePath = "$TargetPath.sig.cer"

    [System.IO.File]::WriteAllBytes($signaturePath, $signatureBytes)
    [System.IO.File]::WriteAllBytes($certificatePath, $cert.Export([System.Security.Cryptography.X509Certificates.X509ContentType]::Cert))

    Write-Host "[build] Detached-Signatur: $signaturePath"
    Write-Host "[build] Signatur-Zertifikat: $certificatePath"
}

function Invoke-SignWindowsPayloadBinaries {
    param([Parameter(Mandatory)][string]$PublishDirectory)

    $files = Get-ChildItem -LiteralPath $PublishDirectory -Recurse -File |
        Where-Object { $_.Extension -in @('.exe', '.dll') } |
        Sort-Object FullName

    foreach ($file in $files) {
        $sig = Get-AuthenticodeSignature -FilePath $file.FullName
        if ($sig.Status -eq [System.Management.Automation.SignatureStatus]::Valid) {
            Write-Host "[build] Bereits signiert: $($file.FullName)"
            continue
        }

        Invoke-BuildSign -TargetPath $file.FullName
    }
}

function New-LinuxSourceBundle {
    param(
        [Parameter(Mandatory)][string]$Version,
        [Parameter(Mandatory)][string]$OutputDirectory,
        [Parameter(Mandatory)][string[]]$Distros
    )

    $gitCmd = Get-Command git -ErrorAction SilentlyContinue
    if (-not $gitCmd) {
        throw "git wurde nicht gefunden. Linux-Source-Bundles benoetigen git archive."
    }

    foreach ($distro in $Distros) {
        $cleanDistro = ($distro -replace '[^A-Za-z0-9._-]', '').Trim()
        if ([string]::IsNullOrWhiteSpace($cleanDistro)) {
            continue
        }

        $bundlePath = Join-Path $OutputDirectory ("Linux-{0}-v{1}-signed.zip" -f $cleanDistro, $Version)
        if (Test-Path -LiteralPath $bundlePath) {
            Remove-Item -LiteralPath $bundlePath -Force
        }

        Write-Host "[build] Linux-Source-Bundle: $bundlePath"
        & git -C $repoRoot archive --format=zip --output=$bundlePath HEAD
        if ($LASTEXITCODE -ne 0) {
            throw "git archive fehlgeschlagen fuer Linux-$cleanDistro"
        }

        Test-ZipArchiveExtractable -ZipPath $bundlePath -ExpectedRelativePath 'README.md'

        New-Sha256File -TargetPath $bundlePath | Out-Null
        if ($wantSign) {
            New-DetachedArtifactSignature -TargetPath $bundlePath
        }
    }
}

function New-EphemeralCodeSignThumbprint {
    if (-not $isWindowsHost) {
        throw "Ephemeral code signing is only supported on Windows."
    }

    if (-not [string]::IsNullOrWhiteSpace($script:AutoSignThumbprint)) {
        return $script:AutoSignThumbprint
    }

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
    $signScript = Join-Path $repoRoot 'installer\sign-authenticode.ps1'
    if (-not (Test-Path -LiteralPath $signScript)) {
        throw "Signierskript fehlt: $signScript"
    }
    $thumb = $env:CODE_SIGN_THUMBPRINT
    $pfx = $env:CODE_SIGN_PFX
    if ([string]::IsNullOrWhiteSpace($thumb) -and [string]::IsNullOrWhiteSpace($pfx)) {
        $thumb = New-EphemeralCodeSignThumbprint
    }
    if ([string]::IsNullOrWhiteSpace($thumb) -eq [string]::IsNullOrWhiteSpace($pfx)) {
        throw "Ungueltige Signierkonfiguration: entweder CODE_SIGN_THUMBPRINT oder CODE_SIGN_PFX setzen."
    }
    if (-not (Test-Path -LiteralPath $TargetPath)) {
        throw "Datei zum Signieren nicht gefunden: $TargetPath"
    }
    Write-Host "[build] Authenticode-Signatur: $TargetPath"
    if (-not [string]::IsNullOrWhiteSpace($thumb)) {
        $t = $thumb.Trim()
        if ($t -match '<|>') {
            throw "CODE_SIGN_THUMBPRINT ist noch ein Platzhalter - den echten 40-stelligen Hex-Thumbprint aus create-selfsigned-codesign-cert.ps1 einsetzen."
        }
        & $signScript -Path $TargetPath -Thumbprint $t
    } else {
        & $signScript -Path $TargetPath -PfxPath $pfx.Trim()
    }
}

if ($SignOnly) {
    if (-not $isWindowsHost) {
        throw "-SignOnly is only supported on Windows (Authenticode requires Windows tooling)."
    }

    $outDir = Join-Path $repoRoot 'installer\Output'
    $resolved = $SetupPath
    if ([string]::IsNullOrWhiteSpace($resolved)) {
        if (-not (Test-Path -LiteralPath $outDir)) {
            throw "Ordner fehlt: $outDir - Setup-EXE bereitstellen oder -SetupPath `"C:\pfad\Setup.exe`" angeben."
        }
        $latest = Get-ChildItem -LiteralPath $outDir -Filter '*-Setup*.exe' -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1
        if (-not $latest) {
            throw "Keine gregModmanager-*-Setup.exe unter $outDir. Inno-Build ausfuehren oder -SetupPath verwenden."
        }
        $resolved = $latest.FullName
    }
    Write-Host "[build] -SignOnly -> $resolved"
    Invoke-BuildSign -TargetPath $resolved
    exit 0
}

$isccCandidates = @(
    (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
    (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
    (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe')
)
$iscc = $isccCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $iscc) {
    throw @"
Inno Setup 6 Compiler (ISCC.exe) nicht gefunden.

Erwartet unter:
  $($isccCandidates[0])
  $($isccCandidates[1])
  $($isccCandidates[2]) (z. B. winget: JRSoftware.InnoSetup)

Download: https://jrsoftware.org/isdl.php
Nur signieren (ohne Inno): .\build.ps1 -SignOnly

Nach der Installation ggf. PowerShell neu starten.
"@
}

$projPath = Join-Path $repoRoot 'GregModmanager.Avalonia\GregModmanager.Avalonia.csproj'
$csproj = [xml](Get-Content -LiteralPath $projPath -Raw)
$ver = (
    $csproj.Project.PropertyGroup |
    ForEach-Object { $_.Version } |
    Where-Object { $_ } |
    Select-Object -First 1
).Trim()
if ([string]::IsNullOrWhiteSpace($ver)) {
    $ver = '1.0.0'
}

$publishDir = Join-Path $repoRoot 'GregModmanager.Avalonia\bin\Release\net9.0\win-x64\publish'
$iss = Join-Path $repoRoot 'installer\gregModmanager.iss'
$outDir = Join-Path $repoRoot 'installer\Output'
$linuxRequested = $LinuxDistros.Count -gt 0
$wantSign = $false

if (-not $isWindowsHost) {
    if ($Sign) {
        throw "-Sign is only supported on Windows (Authenticode/signing certificate store is Windows-specific)."
    }

    if (Test-Path -LiteralPath $publishDir) {
        Write-Host "[build] Cleaning old publish output: $publishDir"
        Remove-Item -LiteralPath $publishDir -Recurse -Force
    }

    New-Item -ItemType Directory -Path $outDir -Force | Out-Null

    if (-not $linuxRequested) {
        Write-Host '[build] Non-Windows host: kein Windows-Build moeglich. Uebergib -LinuxDistros Debian,Kali,... fuer Linux-Source-Bundles.'
        exit 0
    }

    New-LinuxSourceBundle -Version $ver -OutputDirectory $outDir -Distros $LinuxDistros
    Write-Host '[build] Linux-Source-Bundles erstellt (inkl. SHA256-Dateien).'
    exit 0
}

New-Item -ItemType Directory -Path $outDir -Force | Out-Null

if (-not $SkipPublish) {
    if (Test-Path -LiteralPath $publishDir) {
        Write-Host "[build] Bereinige alte Publish-Ausgabe: $publishDir"
        Remove-Item -LiteralPath $publishDir -Recurse -Force
    }
    Write-Host '[build] dotnet publish -c Release -r win-x64 ...'
    & dotnet publish $projPath -c Release -r win-x64
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} else {
    Write-Warning "-SkipPublish aktiv: Es wird eine bestehende Publish-Ausgabe verpackt. Bei Startproblemen bitte ohne -SkipPublish neu bauen."
}

if (-not (Test-Path -LiteralPath $publishDir)) {
    throw "Publish-Ausgabe nicht gefunden: $publishDir"
}

if (-not (Test-Path -LiteralPath $iss)) {
    throw "Inno-Skript fehlt: $iss"
}

$portableZipPath = Join-Path $outDir ("win64-v{0}-portable.zip" -f $ver)
if (Test-Path -LiteralPath $portableZipPath) {
    Remove-Item -LiteralPath $portableZipPath -Force
}

if ($wantSign) {
    Write-Host ''
    Write-Host '[build] Signiere alle PE-Binaries im Portable-Payload (EXE + DLL) ...'
    Invoke-SignWindowsPayloadBinaries -PublishDirectory $publishDir
}

Write-Host "[build] Packe Win64 Portable ZIP: $portableZipPath"
Compress-Archive -Path (Join-Path $publishDir '*') -DestinationPath $portableZipPath -CompressionLevel Optimal
Test-ZipArchiveExtractable -ZipPath $portableZipPath -ExpectedRelativePath 'GregModmanager.exe'
New-Sha256File -TargetPath $portableZipPath | Out-Null
if ($wantSign) {
    New-DetachedArtifactSignature -TargetPath $portableZipPath
}

Write-Host "[build] Inno Setup ($iscc) - Version $ver ..."
$numericVer = $ver -replace '-.*$', ''
if ($numericVer -notmatch '^\d+\.\d+\.\d+\.\d+$') {
    if ($numericVer -match '^\d+\.\d+\.\d+$') { $numericVer += ".0" }
    else { $numericVer = "1.0.0.0" }
}
$argList = @(
    $iss
    "/DMyAppVersion=$ver"
    "/DMyAppNumericVersion=$numericVer"
)
& $iscc @argList
if ($LASTEXITCODE -ne 0) {
    throw "ISCC beendet mit Code $LASTEXITCODE"
}

$setupName = "gregModmanager-$ver-Setup.exe"
$setupPath = Join-Path $outDir $setupName
if (Test-Path -LiteralPath $setupPath) {
    $len = (Get-Item -LiteralPath $setupPath).Length
    $mb = [math]::Round($len / 1MB, 2)
    Write-Host ''
    Write-Host "[build] Fertig: $setupPath ($mb MB)"
} else {
    Write-Host '[build] ISCC ohne Fehler - Ausgabedatei bitte unter installer\Output pruefen.'
}

if ($wantSign) {
    Write-Host ''
    Write-Host '[build] Signiere Setup-EXE ...'
    Invoke-BuildSign -TargetPath $setupPath
    Assert-AuthenticodeSignaturePresent -TargetPath $setupPath
}

New-Sha256File -TargetPath $setupPath | Out-Null
if ($wantSign) {
    New-DetachedArtifactSignature -TargetPath $setupPath
}

$setupAlias = if ($wantSign) {
    Join-Path $outDir ("win64-v{0}-setup-signed.exe" -f $ver)
} else {
    Join-Path $outDir ("win64-v{0}-setup.exe" -f $ver)
}

$oppositeSetupAlias = if ($wantSign) {
    Join-Path $outDir ("win64-v{0}-setup.exe" -f $ver)
} else {
    Join-Path $outDir ("win64-v{0}-setup-signed.exe" -f $ver)
}

if (Test-Path -LiteralPath $oppositeSetupAlias) {
    Remove-Item -LiteralPath $oppositeSetupAlias -Force
}

Copy-Item -LiteralPath $setupPath -Destination $setupAlias -Force
Write-Host "[build] Setup Alias: $setupAlias"
if ($wantSign) {
    Assert-AuthenticodeSignaturePresent -TargetPath $setupAlias
}
New-Sha256File -TargetPath $setupAlias | Out-Null
if ($wantSign) {
    New-DetachedArtifactSignature -TargetPath $setupAlias
}

if ($linuxRequested) {
    New-LinuxSourceBundle -Version $ver -OutputDirectory $outDir -Distros $LinuxDistros
}
