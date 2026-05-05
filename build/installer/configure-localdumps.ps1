# Configure Windows Error Reporting (WER) LocalDumps for GregModmanager.exe
# Usage examples:
#   .\installer\configure-localdumps.ps1
#   .\installer\configure-localdumps.ps1 -Scope Machine -DumpCount 20
#   .\installer\configure-localdumps.ps1 -Disable

[CmdletBinding()]
param(
	[ValidateSet('CurrentUser', 'Machine')]
	[string]$Scope = 'CurrentUser',
	[ValidateSet('Mini', 'Full')]
	[string]$DumpType = 'Mini',
	[ValidateRange(1, 100)]
	[int]$DumpCount = 10,
	[string]$DumpFolder,
	[switch]$Disable
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-BaseRegPath {
	param([string]$TargetScope)
	if ($TargetScope -eq 'Machine') {
		return 'HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps'
	}

	return 'HKCU:\Software\Microsoft\Windows\Windows Error Reporting\LocalDumps'
}

function Ensure-AdminIfNeeded {
	param([string]$TargetScope)
	if ($TargetScope -ne 'Machine') { return }

	$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
	$principal = New-Object Security.Principal.WindowsPrincipal($identity)
	$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
	if (-not $isAdmin) {
		throw "Scope 'Machine' requires an elevated PowerShell session (Run as Administrator)."
	}
}

Ensure-AdminIfNeeded -TargetScope $Scope

$baseRegPath = Get-BaseRegPath -TargetScope $Scope
$exeName = 'GregModmanager.exe'
$targetRegPath = Join-Path $baseRegPath $exeName

if ([string]::IsNullOrWhiteSpace($DumpFolder)) {
	if ($Scope -eq 'Machine') {
		$DumpFolder = 'C:\ProgramData\gregModmanager\dumps'
	}
	else {
		$DumpFolder = Join-Path $env:LOCALAPPDATA 'gregModmanager\dumps'
	}
}

if ($Disable) {
	if (Test-Path -LiteralPath $targetRegPath) {
		Remove-Item -LiteralPath $targetRegPath -Recurse -Force
		Write-Host "[ok] LocalDumps disabled for $exeName at $Scope scope."
	}
	else {
		Write-Host "[info] No LocalDumps configuration found for $exeName at $Scope scope."
	}

	return
}

New-Item -Path $targetRegPath -Force | Out-Null
New-Item -ItemType Directory -Path $DumpFolder -Force | Out-Null

$dumpTypeValue = if ($DumpType -eq 'Full') { 2 } else { 1 }

New-ItemProperty -Path $targetRegPath -Name DumpFolder -PropertyType ExpandString -Value $DumpFolder -Force | Out-Null
New-ItemProperty -Path $targetRegPath -Name DumpType -PropertyType DWord -Value $dumpTypeValue -Force | Out-Null
New-ItemProperty -Path $targetRegPath -Name DumpCount -PropertyType DWord -Value $DumpCount -Force | Out-Null

Write-Host "[ok] LocalDumps enabled for $exeName"
Write-Host "     Scope:     $Scope"
Write-Host "     DumpType:  $DumpType ($dumpTypeValue)"
Write-Host "     DumpCount: $DumpCount"
Write-Host "     Folder:    $DumpFolder"

