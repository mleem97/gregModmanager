# Development Helper Scripts für GregModmanager
# ===============================================
#
# Diese Datei enthält verschiedene Hilfsfunktionen zur Behebung von XAML/UI-Problemen.
#
# Verwendung:
#   . .\dev-helpers.ps1
#   Fix-AppShellResources
#   Fix-UiPageResources
#   ... usw.

param(
    [ValidateSet('fix-appshell', 'fix-xaml-resources-remove', 'fix-xaml-resources-paths', 'fix-xaml-resources-relative', 'fix-xaml-resources-all')]
    [string]$Action
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ============================================================================
# Fix-AppShell: Ersetzt StaticResource durch DynamicResource in AppShell.xaml
# ============================================================================
function Fix-AppShellResources {
    [CmdletBinding()]
    param()
    
    Write-Host "Fixing AppShell.xaml: StaticResource -> DynamicResource"
    $file = '.\AppShell.xaml'
    
    if (-not (Test-Path $file)) {
        Write-Error "File not found: $file"
        return
    }
    
    $content = Get-Content $file -Raw
    $content = $content -replace "StaticResource ", "DynamicResource "
    Set-Content $file -Value $content
    Write-Host "✓ AppShell.xaml fixed"
}

# ============================================================================
# Fix-UiPageResourcesRemove: Entfernt <ContentPage.Resources> Blöcke
# ============================================================================
function Fix-UiPageResourcesRemove {
    [CmdletBinding()]
    param()
    
    Write-Host "Fixing UI Pages: Removing ContentPage.Resources blocks"
    $files = Get-ChildItem -Path '.\UI\Pages\*.xaml' -Recurse
    
    if ($files.Count -eq 0) {
        Write-Warning "No XAML files found in .\UI\Pages\"
        return
    }
    
    foreach ($f in $files) {
        Write-Host "  Processing $($f.Name)..."
        $content = Get-Content $f.FullName -Raw
        $content = $content -replace "(?i)(?s)<ContentPage\.Resources>.*?</ContentPage\.Resources>\s*", ""
        Set-Content $f.FullName -Value $content
    }
    Write-Host "✓ Removed ContentPage.Resources from $($files.Count) files"
}

# ============================================================================
# Fix-UiPageResourcesPaths: Behebt Ressource-Pfade (/ -> kein /)
# ============================================================================
function Fix-UiPageResourcesPaths {
    [CmdletBinding()]
    param()
    
    Write-Host "Fixing UI Pages: Correcting resource paths (remove leading slash)"
    $files = Get-ChildItem -Path '.\UI\Pages\*.xaml' -Recurse
    
    if ($files.Count -eq 0) {
        Write-Warning "No XAML files found in .\UI\Pages\"
        return
    }
    
    foreach ($f in $files) {
        Write-Host "  Processing $($f.Name)..."
        $content = Get-Content $f.FullName -Raw
        $content = $content -replace 'Source=`"/Resources/', 'Source=`"Resources/'
        Set-Content $f.FullName -Value $content
    }
    Write-Host "✓ Fixed resource paths in $($files.Count) files"
}

# ============================================================================
# Fix-UiPageResourcesRelative: Behebt Ressource-Pfade (../../Resources/)
# ============================================================================
function Fix-UiPageResourcesRelative {
    [CmdletBinding()]
    param()
    
    Write-Host "Fixing UI Pages: Using relative paths (../../Resources/)"
    $files = Get-ChildItem -Path '.\UI\Pages\*.xaml' -Recurse
    
    if ($files.Count -eq 0) {
        Write-Warning "No XAML files found in .\UI\Pages\"
        return
    }
    
    foreach ($f in $files) {
        Write-Host "  Processing $($f.Name)..."
        $content = Get-Content $f.FullName -Raw
        $content = $content -replace 'Source=`"/Resources/', 'Source=`"../../Resources/'
        Set-Content $f.FullName -Value $content
    }
    Write-Host "✓ Fixed resource paths to relative in $($files.Count) files"
}

# ============================================================================
# Fix-UiPageResourcesAll: Kombinierte Reparatur (alles auf einmal)
# ============================================================================
function Fix-UiPageResourcesAll {
    [CmdletBinding()]
    param()
    
    Write-Host "Fixing UI Pages: Complete fix (remove resources + fix paths + dynamic resources)"
    $files = Get-ChildItem -Path '.\UI\Pages\*.xaml' -Recurse
    
    if ($files.Count -eq 0) {
        Write-Warning "No XAML files found in .\UI\Pages\"
        return
    }
    
    foreach ($f in $files) {
        Write-Host "  Processing $($f.Name)..."
        $content = Get-Content $f.FullName -Raw
        $content = $content -replace "(?i)(?s)<ContentPage\.Resources>.*?</ContentPage\.Resources>\s*", ""
        $content = $content -replace "StaticResource ", "DynamicResource "
        Set-Content $f.FullName -Value $content
    }
    Write-Host "✓ Complete fix applied to $($files.Count) files"
}

# ============================================================================
# If script is called with -Action parameter, execute the corresponding function
# ============================================================================
if ($Action) {
    $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
    Set-Location $repoRoot
    
    switch ($Action) {
        'fix-appshell' { Fix-AppShellResources }
        'fix-xaml-resources-remove' { Fix-UiPageResourcesRemove }
        'fix-xaml-resources-paths' { Fix-UiPageResourcesPaths }
        'fix-xaml-resources-relative' { Fix-UiPageResourcesRelative }
        'fix-xaml-resources-all' { Fix-UiPageResourcesAll }
        default { Write-Error "Unknown action: $Action" }
    }
}
