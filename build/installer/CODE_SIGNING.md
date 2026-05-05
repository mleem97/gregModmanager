# Code Signing — Anzeigename, „Unbekannter Herausgeber“, SmartScreen

## Zwei verschiedene Dinge

1. **Name in der Signatur** (Zertifikats-Subject, z. B. **mleem97** / **Greg Modding Team**)  
   Den setzt du im **Zertifikat** — bei einer **selbst erstellten** Code-Signing-Zertifikatsvorlage genauso wie bei einer CA. Nach dem Signieren siehst du diesen Namen oft unter *Eigenschaften → Digitale Signaturen → Signaturdetails*.

2. **„Unbekannter Herausgeber“ / kein Vertrauen**  
   Windows zeigt das, wenn die Signatur **nicht** zu einer in Windows **vorinstallierten vertrauenden Stamm-CA** zurückgeführt werden kann — typisch bei **Self-Signed** oder wenn Nutzer dein Stammzertifikat nicht installiert haben.

**Ja:** Du kannst **trotzdem** als *mleem97 / Greg Modding Team* in den Zertifikatsdaten auftauchen **und** gleichzeitig weiterhin als **„Unbekannter Herausgeber“** (oder nicht vertrauenswürdige Signatur) gelten — das ist bei Self-Signed **normal** und **gewollt**, wenn du **kein** gekauftes Zertifikat nutzt.

### Self-Signing und SmartScreen (wichtig)

**Self-Signing ersetzt kein CA-Zertifikat** und **umgeht SmartScreen nicht zuverlässig**. SmartScreen und die Vertrauensbewertung sind auf **öffentlich vertrauenswürdige** Code-Signing-Zertifikate (OV/EV) und **Reputation** ausgelegt. Eine selbst signierte Datei kann weiterhin **Warnungen** auslösen — manchmal sogar **stärker** als „gar nicht signiert“, je nach Heuristik.

Wenn du **bewusst** self-signed nutzt: Ziel ist oft ein **fester Anzeigename** und nachvollziehbare Signatur für Kenner — **nicht** das automatische Wegfiltern aller SmartScreen-Meldungen.

### Schnell: eigenes Code-Signing-Zertifikat mit deinem Namen

```powershell
cd gregModmanager
.\installer\create-selfsigned-codesign-cert.ps1
# optional: -CommonName "mleem97" -Organization "Greg Modding Team"
```

Thumbprint aus der Ausgabe kopieren, dann z. B.:

```powershell
$env:CODE_SIGN_THUMBPRINT = '<Thumbprint>'
.\build.ps1 -SkipPublish -Sign
```

Oder nur die Setup-EXE: `.\installer\sign-authenticode.ps1 -Path "…\gregModmanager-…-Setup.exe" -Thumbprint …`

**Öffentliche CA (kein „Unbekannter Herausgeber“ für die breite Masse)**

Windows zeigt **„Unbekannter Herausgeber“** (SmartScreen, Installer, Eigenschaften), wenn:

- die Datei **nicht signiert** ist, oder  
- sie mit einem Zertifikat signiert wurde, das **nicht** zur **vertrauenden Stamm-CA-Kette** passt (typisch: Self-Signed ohne Import des Stammzerts).

Damit die Signatur **flächendeckend als vertrauenswürdig** gilt, brauchst du ein **Code-Signing-Zertifikat** von einer **öffentlichen Zertifizierungsstelle** (DigiCert, Sectigo, SSL.com, GlobalSign …) — **OV oder EV** Authenticode.

**Nur für dich / Tester:** Du kannst dein **öffentliches** Stammzertifikat (ohne privaten Schlüssel) unter *Vertrauenswürdige Stammzertifizierungsstellen* importieren — dann gilt die Signatur auf **diesem** PC als vertrauenswürdig. Für **alle** Spieler ohne Extra-Import geht das nur mit einer **öffentlichen CA**.

## Ablauf (Kurz)

1. **Zertifikat kaufen** (OV/EV Code Signing, nicht nur TLS/SSL für Webseiten).  
   EV: oft schneller Vertrauensaufbau bei SmartScreen, teurer, USB/YubiKey-HSM je nach Anbieter.
2. **Zertifikat erhalten** meist als **.pfx** + Passwort, oder Installation über Anbieter-Portal in den Windows-Zertifikatspeicher (**Aktueller Benutzer → Eigene Zertifikate** oder **Lokaler Computer** je nach Anleitung).
3. **Thumbprint** des Signaturzertifikats ermitteln (PowerShell):

   ```powershell
   Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.HasPrivateKey } | Format-Table Subject, Thumbprint
   ```

4. Nach dem Build die **Setup-EXE** signieren:

   ```powershell
   cd gregModmanager
   .\installer\sign-authenticode.ps1 -Path ".\installer\Output\gregModmanager-1.5.0-Setup.exe" -Thumbprint DEIN_THUMBPRINT
   ```

   Oder **PFX** direkt:

   ```powershell
   $env:CODE_SIGN_PFX_PASSWORD = '***'
   .\installer\sign-authenticode.ps1 -Path ".\installer\Output\...\Setup.exe" -PfxPath "C:\pfad\codesign.pfx"
   ```

5. **Zeitstempel-URL:** Das Skript nutzt standardmäßig DigiCert (`http://timestamp.digicert.com`). Dein CA-Anbieter kann eine andere RFC-3161-URL vorgeben — dann `-TimestampUrl` setzen.

## Mit `build.ps1` in einem Schritt

Voraussetzung: Inno-Setup-Build war erfolgreich, `gregModmanager-*-Setup.exe` liegt unter `installer\Output\`.

```powershell
# Thumbprint (Zertifikat mit privatem Schlüssel im Benutzerspeicher)
$env:CODE_SIGN_THUMBPRINT = 'DEIN40STELLIGERHEXTHUMBPRINT'
.\build.ps1 -SkipPublish -Sign
```

`-Sign` ohne `-SkipPublish` baut neu, signiert danach die erzeugte Setup-Datei.

### Nur signieren — **ohne Inno Setup** installiert zu haben

Wenn **ISCC.exe** fehlt, aber du eine **Setup-EXE** schon hast (z. B. von anderem Rechner oder ZIP):

```powershell
$env:CODE_SIGN_THUMBPRINT = '4AB58E6A56F4CA5726849BD410151B25321289DC'   # Beispiel — dein echter Wert
.\build.ps1 -SignOnly
```

Nimmt die **neueste** `gregModmanager-*-Setup.exe` unter `installer\Output\`, oder:

```powershell
.\build.ps1 -SignOnly -SetupPath 'D:\dist\gregModmanager-1.5.0-Setup.exe'
```

**Nicht** den Platzhaltertext `<Thumbprint aus der Ausgabe>` verwenden — nur den **40-stelligen Hex-Thumbprint** aus `create-selfsigned-codesign-cert.ps1`.

## Technik

- Tool: **`signtool.exe`** aus dem **Windows SDK** (z. B. unter `Program Files (x86)\Windows Kits\10\bin\...\x64\`).
- Algorithmus: **SHA-256** (`/fd SHA256`), Zeitstempel **SHA-256** (`/td SHA256` mit `/tr`).

## Weitere Dateien signieren

- **Nur die Setup-EXE** zu signieren reicht oft; manche Teams signieren zusätzlich **`GregModmanager.exe`** im Publish-Ordner vor dem Packen — dann Inno mit dem bereits signierten Build erneut laufen lassen. Für den Anfang: **Setup-EXE signieren** ist der übliche Weg.

## Referenzen

- [Microsoft: SignTool](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)  
- [DigiCert: Code Signing](https://www.digicert.com/code-signing/)  
- [Azure Trusted Signing](https://learn.microsoft.com/en-us/azure/trusted-signing/) (Alternative: Signatur aus der Cloud ohne lokalen PFX)

