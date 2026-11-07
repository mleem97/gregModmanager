param(
    [Parameter(Mandatory = $true)][string]$StageDir,
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$OutputDir
)

$ErrorActionPreference = 'Stop'
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    throw 'Windows installers require a three-part numeric release version.'
}
$parts = $Version.Split('.') | ForEach-Object { [int]$_ }
if ($parts[0] -gt 255 -or $parts[1] -gt 255 -or $parts[2] -gt 65535) {
    throw 'Version exceeds Windows Installer version limits.'
}
$stage = (Resolve-Path -LiteralPath $StageDir).Path
foreach ($file in @('gregmodmanager.exe', 'gregcli.exe', 'steam_api64.dll', 'steam_appid.txt')) {
    if (-not (Test-Path -LiteralPath (Join-Path $stage $file) -PathType Leaf)) {
        throw "Missing staged distribution file: $file"
    }
}
if (Test-Path -LiteralPath (Join-Path $stage '.greg-install-kind')) {
    throw 'The portable stage must not contain an installer marker.'
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$output = (Resolve-Path -LiteralPath $OutputDir).Path
$iscc = Get-Command ISCC.exe -ErrorAction SilentlyContinue
if (-not $iscc) {
    $defaultIscc = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'
    if (Test-Path -LiteralPath $defaultIscc) { $iscc = Get-Command $defaultIscc }
}
$wix = Get-Command wix.exe -ErrorAction SilentlyContinue
if (-not $iscc -or -not $wix) {
    throw 'Inno Setup 6 (ISCC.exe) and WiX CLI 5 (wix.exe) are required.'
}
$temp = Join-Path ([System.IO.Path]::GetTempPath()) ("greg-installer-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
try {
    $setupMarker = Join-Path $temp 'setup-marker.txt'
    $msiMarker = Join-Path $temp 'msi-marker.txt'
    [System.IO.File]::WriteAllText($setupMarker, 'setup')
    [System.IO.File]::WriteAllText($msiMarker, 'msi')
    $base = "gregmodmanager-$Version-win-x64"
    & $iscc.Source "/DMyAppVersion=$Version" "/DSourceDir=$stage" "/DMarkerFile=$setupMarker" "/O$output" "/F$base-setup" (Join-Path $PSScriptRoot 'gregModmanager.iss')
    if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed ($LASTEXITCODE)." }
    $msi = Join-Path $output "$base.msi"
    & $wix.Source build -arch x64 -d "ProductVersion=$Version" -d "PublishDir=$stage" -d "MarkerFile=$msiMarker" -o $msi (Join-Path $PSScriptRoot 'wix\Package.wxs')
    if ($LASTEXITCODE -ne 0) { throw "WiX failed ($LASTEXITCODE)." }
    foreach ($name in @("$base-setup.exe", "$base.msi")) {
        $file = Join-Path $output $name
        $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash.ToLowerInvariant()
        [System.IO.File]::WriteAllText("$file.sha256", "$hash  $name`n")
    }
} finally {
    Remove-Item -LiteralPath $temp -Recurse -Force
}
