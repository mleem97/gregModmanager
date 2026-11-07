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
.\build\installer\create-selfsigned-codesign-cert.ps1
```

The script creates a self-signed code-signing certificate in the current user
certificate store and prints the certificate thumbprint. Use that thumbprint with
`build/installer/sign-authenticode.ps1`:

```powershell
$env:CODE_SIGN_THUMBPRINT = '<thumbprint>'
.\build\installer\sign-authenticode.ps1 -Path 'D:\dist\gregmodmanager-<version>-Windows\gregmodmanager.exe'
```

The CI workflow (`.forgejo/workflows/build-and-release.yml` and its byte-identical
Gitea copy) performs the same per-build self-signed step via `osslsigncode` for the
Windows portable executable (best-effort; skipped when the signing secret or the
tooling is missing). CI does not accept PFX/Thumbprint secrets, by design.

## Create a reusable local test certificate

`build/installer/create-selfsigned-codesign-cert.ps1` creates a manually managed
local certificate with its own subject and lifetime. It is separate from the
ephemeral certificate flow above; use it only for controlled test machines and
keep its private key out of the repository.

## Build with an existing certificate

Use **exactly one** of the following environment values and select `pfx` mode:

```powershell
$env:CODE_SIGN_PFX = 'D:\secure\codesign.pfx'
$env:CODE_SIGN_PFX_PASSWORD = 'read-from-your-secret-store'
.\build\installer\sign-authenticode.ps1 -Path 'D:\dist\gregmodmanager-<version>-Windows\gregmodmanager.exe' -PfxPath 'D:\secure\codesign.pfx'
```

Do not put the PFX, password, thumbprint, or base64-encoded certificate in the
repository, a checked-in `.env`, logs, or documentation. `auto` and `self-signed`
intentionally discard existing PFX/thumbprint environment values; choose `pfx`
when reusing a certificate.

## Sign an existing file

```powershell
.\build\installer\sign-authenticode.ps1 -Path 'D:\dist\gregmodmanager-<version>-Windows\gregmodmanager.exe' -Thumbprint '40_HEX_CHARACTER_THUMBPRINT'
```

`-Path` selects the file to sign; `-Thumbprint` signs with a certificate from the
Windows certificate store, `-PfxPath` signs with a PFX file. The script requires a
Windows host with the Windows SDK (`signtool.exe`) available.

## Verification

On Windows, inspect the file's **Digital Signatures** tab or run:

```powershell
Get-AuthenticodeSignature 'D:\dist\gregmodmanager-<version>-Windows\gregmodmanager.exe' |
  Format-List Status, StatusMessage, SignerCertificate
```

`Valid` on a controlled computer only means that computer trusts the certificate
chain. It does not guarantee public reputation.
