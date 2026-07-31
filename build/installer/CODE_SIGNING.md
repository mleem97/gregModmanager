# Windows code signing

## Trust boundary

The CI and default local build generate a fresh, short-lived self-signed
certificate for each signed build. That certificate identifies the generated
artifact but is not trusted by other Windows installations and does not avoid
SmartScreen or publisher warnings. Never export or publish its private key.

Use a publicly trusted code-signing certificate when broad Windows trust is a
release requirement. A self-signed root may be imported on a controlled test
machine, but that does not establish trust for other users.

## Build with a new self-signed certificate

From the repository root on Windows:

```powershell
.\build\scripts\build.ps1 -SigningMode self-signed
```

The script signs Windows PE payloads, the Setup EXE, and the MSI when those
artifacts are built. Certificate subjects include a random build value so each
build uses a distinct certificate.

CI exposes the same per-build `auto`, `self-signed`, and `none` choices. It does
not accept PFX/Thumbprint secrets, by design.

## Create a reusable local test certificate

`build/installer/create-selfsigned-codesign-cert.ps1` creates a manually managed
local certificate with its own subject and lifetime. It is separate from the
ephemeral certificate created by `build.ps1`; use it only for controlled test
machines and keep its private key out of the repository.

## Build with an existing certificate

Use **exactly one** of the following environment values and select `pfx` mode:

```powershell
$env:CODE_SIGN_THUMBPRINT = '40_HEX_CHARACTER_THUMBPRINT'
.\build\scripts\build.ps1 -SigningMode pfx
```

```powershell
$env:CODE_SIGN_PFX = 'D:\secure\codesign.pfx'
$env:CODE_SIGN_PFX_PASSWORD = 'read-from-your-secret-store'
.\build\scripts\build.ps1 -SigningMode pfx
```

Do not put the PFX, password, thumbprint, or base64-encoded certificate in the
repository, a checked-in `.env`, logs, or documentation. `auto` and
`self-signed` intentionally discard existing PFX/thumbprint environment values;
choose `pfx` when reusing a certificate.

## Sign an existing Setup EXE

```powershell
.\build\scripts\build.ps1 -SignOnly -SigningMode pfx `
  -SetupPath 'D:\dist\gregModmanager-1.6.1-Windows.exe'
```

`-SignOnly` does not restore, publish, or invoke Inno Setup/WiX. It requires a
Windows host, an existing Setup EXE, and usable Authenticode tooling.

## Verification

On Windows, inspect the file's **Digital Signatures** tab or run:

```powershell
Get-AuthenticodeSignature 'D:\dist\gregModmanager-1.6.1-Windows.exe' |
  Format-List Status, StatusMessage, SignerCertificate
```

`Valid` on a controlled computer only means that computer trusts the certificate
chain. It does not guarantee public reputation.
