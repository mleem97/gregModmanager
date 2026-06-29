# De-Mock Journal

## 2026-06-29 - Initial Analysis
**Mock Pattern:** Repository uses .NET 9 with Avalonia UI, Steam integration via Facepunch.Steamworks
**Learning:** Cross-platform desktop app with platform-specific code (Windows/Linux). Steam API integration is core functionality.
**Future Handling:** Focus on service stubs, placeholder implementations, and hardcoded demo data in productive code paths.

## 2026-06-29 - AuthApiClient Mock Replacement
**Mock Pattern:** `AuthApiClient.ExchangeCallbackCodeAsync` returned hardcoded mock tokens (`mock_access_token`, `mocked User`) instead of making real API calls. `SessionManager.InitializeAsync` created mock sessions on refresh. `SessionManager.HandleProtocolCallbackAsync` stored `mock_refresh` as token.
**Learning:** Real auth API exists at `https://datacentermods.com/auth` (production) and `http://localhost:5001/auth` (local). `BetterAuthService.cs` shows existing HTTP client pattern with `HttpClient`.
**Future Handling:** Auth flow now uses real HTTP calls. Token exchange, session refresh, and logout all hit real API endpoints. Remaining mock: `InstallIntentClient.cs` accepts signatures >= 64 chars without cryptographic verification (TODO in code).

## 2026-06-29 - InstallIntentClient Signature Verification
**Mock Pattern:** `InstallIntentClient.ValidateIntentSafelyAsync` accepted all signatures >= 64 chars without cryptographic verification. TODO comment: "Implement actual cryptographic ECDSA/HMAC signature verification against server public key".
**Learning:** Server public key must be provided via `INSTALL_INTENT_PUBLIC_KEY` environment variable (Base64-encoded SubjectPublicKeyInfo). Signed payload format: `intentId|packageId|subjectId|expiresAt`.
**Future Handling:** ECDSA P-256 with SHA-256 verification now implemented. Missing public key returns clear error instead of accepting mock. All remaining productive mocks removed.
