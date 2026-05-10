# Erstellt ein SELBSTSIGNIERTES Code-Signing-Zertifikat im Speicher Aktueller Benutzer.
# Der Name erscheint in der Signatur — Windows kann trotzdem „Unbekannter Herausgeber“ anzeigen,
# weil kein vertrauender Stamm (öffentliche CA) beteiligt ist. Siehe installer\CODE_SIGNING.md
#
# Einmal ausführen, Thumbprint notieren, dann sign-authenticode.ps1 oder build.ps1 -Sign nutzen.
#Requires -Version 5.1
param(
    [string]$CommonName = 'mleem97',
    [string]$Organization = 'Greg Modding Team',
    [int]$YearsValid = 5
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$subject = "CN=$CommonName, O=$Organization"
$notAfter = (Get-Date).AddYears($YearsValid)

Write-Host "[selfsign] Erstelle Code-Signing-Zertifikat: $subject (gültig bis $notAfter)"

$cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $subject `
    -KeyUsage DigitalSignature `
    -KeyAlgorithm RSA `
    -KeyLength 2048 `
    -HashAlgorithm SHA256 `
    -NotAfter $notAfter `
    -CertStoreLocation Cert:\CurrentUser\My `
    -FriendlyName "Greg Modding Team — self-signed code signing ($CommonName)"

Write-Host ''
Write-Host '[selfsign] Fertig. Für Signatur diesen Thumbprint verwenden:'
Write-Host "         $($cert.Thumbprint)"
Write-Host ''
Write-Host '[selfsign] Beispiel:'
Write-Host "         `$env:CODE_SIGN_THUMBPRINT='$($cert.Thumbprint)'"
Write-Host '         .\installer\sign-authenticode.ps1 -Path ".\installer\Output\gregModmanager-1.5.0-Setup.exe" -Thumbprint $env:CODE_SIGN_THUMBPRINT'
Write-Host ''
Write-Host '[selfsign] Hinweis: Auf ANDEREN PCs bleibt die Signatur „nicht vertrauenswürdig“, solange dieses Stammzertifikat nicht importiert wurde (normal für Self-Signed).'

