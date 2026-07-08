using System;
using System.Collections.Generic;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Threading.Tasks;
using System.Web;
using GregModmanager.Models.Install;
using GregModmanager.Services.Auth;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Install;

public class InstallIntentClient : IInstallIntentClient
{
    private readonly ISessionManager _sessionManager;
    private readonly HashSet<string> _consumedIntents = new();

    public InstallIntentClient(ISessionManager sessionManager)
    {
        _sessionManager = sessionManager;
    }

    public async Task HandleIntentAsync(string rawUri)
    {
        try
        {
            var uri = new Uri(rawUri);
            var query = HttpUtility.ParseQueryString(uri.Query);

            var intent = new InstallIntentContext
            {
                IntentId = query["intentId"] ?? string.Empty,
                PackageId = query["packageId"] ?? string.Empty,
                SourceUrl = query["sourceUrl"] ?? string.Empty,
                SubjectId = query["subjectId"] ?? string.Empty,
                ExpiresAt = long.TryParse(query["expiresAt"], out var exp) ? exp : 0,
                RequiredRoles = (query["roles"] ?? string.Empty).Split(',', StringSplitOptions.RemoveEmptyEntries),
                Signature = query["sig"] ?? string.Empty
            };

            if (string.IsNullOrEmpty(intent.IntentId))
            {
                AppFileLog.Warn("Install intent missing required 'intentId'.");
                return;
            }

            AppFileLog.Info($"Received install intent: {intent.PackageId} (ID: {intent.IntentId})");

            var validationError = await ValidateIntentSafelyAsync(intent);
            if (!string.IsNullOrEmpty(validationError))
            {
                AppFileLog.Warn($"Install intent validation failed for package {intent.PackageId}: {validationError}");

                AppFileLog.Warn($"Install intent validation failed: {validationError}");
                return;
            }

            AppFileLog.Info($"Install intent validated successfully for package {intent.PackageId}. Queuing installation.");
            _consumedIntents.Add(intent.IntentId);

            // Phase 3 specifies: never silently install untrusted content
            // If non-Steam package install is not fully implemented yet, display safe intent.
            AppFileLog.Info($"Installation queued for package '{intent.PackageId}'.");
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Failed to parse or handle install intent", ex);
        }
    }

    private Task<string?> ValidateIntentSafelyAsync(InstallIntentContext intent)
    {
        // 1. Check Session
        if (_sessionManager.State != SessionState.Authenticated || _sessionManager.CurrentSession == null)
        {
            return Task.FromResult<string?>("User is not authenticated. Please log in first.");
        }

        // 2. Extracted Subject Match
        if (!string.IsNullOrEmpty(intent.SubjectId) && intent.SubjectId != _sessionManager.CurrentSession.User.SubjectId)
        {
            return Task.FromResult<string?>($"Subject mismatch. Intent is for user {intent.SubjectId}, but current session is {_sessionManager.CurrentSession.User.SubjectId}");
        }

        // 3. Expiry
        var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        if (intent.ExpiresAt < now)
        {
            return Task.FromResult<string?>($"Intent expired at timestamp {intent.ExpiresAt}. Current time {now}.");
        }

        // 4. Role constraints
        foreach (var role in intent.RequiredRoles)
        {
            if (!_sessionManager.CurrentSession.User.Roles.Contains(role))
            {
                return Task.FromResult<string?>($"Missing required capability: '{role}'.");
            }
        }

        // 5. Replay protection
        if (_consumedIntents.Contains(intent.IntentId))
        {
            return Task.FromResult<string?>($"Intent ID {intent.IntentId} was already consumed (Replay Protection).");
        }

        // 6. Signature Validation
        if (string.IsNullOrEmpty(intent.Signature))
        {
            return Task.FromResult<string?>("Missing cryptographic signature.");
        }

        // Reject known dummy/placeholder signatures
        if (intent.Signature == "valid_dummy_sig")
        {
            return Task.FromResult<string?>("Signature is a placeholder — cryptographic verification required.");
        }

        // Require minimum length for a valid cryptographic signature (Base64-encoded ECDSA P-256 = ~64 chars)
        if (intent.Signature.Length < 64)
        {
            return Task.FromResult<string?>("Signature too short for a valid cryptographic signature.");
        }

        // 7. Cryptographic ECDSA P-256 signature verification
        var signatureError = VerifyEcdsaSignature(intent);
        if (!string.IsNullOrEmpty(signatureError))
        {
            return Task.FromResult<string?>(signatureError);
        }

        return Task.FromResult<string?>(null); // Null means valid!
    }

    private static string? VerifyEcdsaSignature(InstallIntentContext intent)
    {
        try
        {
            // Load server public key from environment variable (Base64-encoded uncompressed EC point)
            var publicKeyBase64 = Environment.GetEnvironmentVariable("INSTALL_INTENT_PUBLIC_KEY");
            if (string.IsNullOrEmpty(publicKeyBase64))
            {
                return "Server public key not configured (INSTALL_INTENT_PUBLIC_KEY). Cannot verify signature.";
            }

            var publicKeyBytes = Convert.FromBase64String(publicKeyBase64);

            // Create ECDsa instance from the public key
            using var ecdsa = ECDsa.Create();
            try
            {
                ecdsa.ImportSubjectPublicKeyInfo(publicKeyBytes, out _);
            }
            catch
            {
                return "Invalid server public key format. Expected SubjectPublicKeyInfo (SPKI) encoding.";
            }

            // Build the signed payload: intentId|packageId|subjectId|expiresAt
            var payload = $"{intent.IntentId}|{intent.PackageId}|{intent.SubjectId}|{intent.ExpiresAt}";
            var payloadBytes = Encoding.UTF8.GetBytes(payload);

            // Decode the signature from Base64
            byte[] signatureBytes;
            try
            {
                signatureBytes = Convert.FromBase64String(intent.Signature);
            }
            catch
            {
                return "Invalid signature encoding. Expected Base64.";
            }

            // Verify the ECDSA P-256 signature with SHA-256
            var isValid = ecdsa.VerifyData(payloadBytes, signatureBytes, HashAlgorithmName.SHA256);
            if (!isValid)
            {
                return "Cryptographic signature verification failed. Signature does not match payload.";
            }

            return null; // Valid
        }
        catch (FormatException)
        {
            return "Invalid Base64 encoding in signature or public key.";
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Signature verification error", ex);
            return $"Signature verification error: {ex.Message}";
        }
    }
}
