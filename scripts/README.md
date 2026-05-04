# GregModmanager Scripts
## Zentrale Übersicht

Dieses Verzeichnis enthält alle wichtigen Build-, Deploy- und Development-Hilfsskripte.

### 🚀 Wichtigste Scripts

#### `build.ps1` - Kompilieren & Installer erstellen
```powershell
.\build.ps1              # Release-Publish + Inno Setup Installer
.\build.ps1 -SkipPublish # Nur Setup (wenn Publish schon vorhanden)
.\build.ps1 -SignOnly    # Nur Authenticode-Signatur (CODE_SIGN_THUMBPRINT nötig)
.\build.ps1 -Sign        # Build + Sign
```

#### `run.ps1` - Direkter Start im Debug-Modus
```powershell
.\run.ps1                # Startet GregModmanager mit `dotnet run`
.\run.ps1 -- -h          # Argumente nach `--` werden an die App weitergeleitet
```

#### `install-local.ps1` - Schnelle lokale Installation
```powershell
.\install-local.ps1              # Publish nach %LOCALAPPDATA%\Programs\gregModmanager + Shortcuts
.\install-local.ps1 -SkipPublish # Nur Shortcuts (wenn Publish schon vorhanden)
.\install-local.ps1 -Uninstall   # Alles entfernen
```

#### `start.ps1` - Release-Anwendung direkt starten
```powershell
.\start.ps1                                          # Startet GregModmanager.exe aus Release-Build
.\start.ps1 -Configuration Release -Platform win10-x64
```

### 🛠️ Development Helper

#### `dev-helpers.ps1` - Reparatur-Hilfsfunktionen
Sammlung von Funktionen zur Behebung von UI/XAML-Problemen:

```powershell
# Laden und verwenden:
. .\dev-helpers.ps1
Fix-AppShellResources              # StaticResource -> DynamicResource in AppShell.xaml
Fix-UiPageResourcesRemove          # ContentPage.Resources Blöcke entfernen
Fix-UiPageResourcesPaths           # Ressourcen-Pfade korrigieren
Fix-UiPageResourcesRelative        # Relative Pfade verwenden (../../Resources/)
Fix-UiPageResourcesAll             # Alle Fixes kombiniert
```

Oder mit Parameter direkt aufrufen:
```powershell
.\dev-helpers.ps1 -Action fix-appshell
.\dev-helpers.ps1 -Action fix-xaml-resources-all
```

#### `fix-csharp-strings.csx` - C# String-Interpolation reparieren
```powershell
dotnet script fix-csharp-strings.csx
```

### 📦 Linux-Unterstützung

#### `linux/build-linux-packages.ps1` - Linux-Pakete (von Windows)
```powershell
.\linux\build-linux-packages.ps1
```

#### `linux/build-linux-packages.sh` - Linux-Pakete (native Linux)
```bash
./linux/build-linux-packages.sh
```

---

### 📝 Quick-Start

**Einfache Schritte zum Starten:**

1. **Entwicklung & Schneller Test:**
   ```powershell
   .\run.ps1
   ```

2. **Installieren auf diesem PC (schnell):**
   ```powershell
   .\install-local.ps1
   ```

3. **Release-Installer bauen (mit Inno Setup):**
   ```powershell
   .\build.ps1
   ```

4. **UI-Probleme beheben:**
   ```powershell
   . .\dev-helpers.ps1
   Fix-UiPageResourcesAll
   ```

---

### 📋 Vom Root-Verzeichnis aufrufen

Alle Scripts können auch direkt aus dem Root-Verzeichnis aufgerufen werden (Convenience-Wrapper):
```powershell
cd c:\Users\marvi\Desktop\gregModmanager
.\build.ps1
.\run.ps1
.\install-local.ps1
```

Diese leiten automatisch an die Versionen in `/scripts` weiter.
