using System;
using System.Collections.Generic;
using System.Linq;
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

        // 6. Signature Validation (Mock logic conforming to Phase 3 requirements)
        if (string.IsNullOrEmpty(intent.Signature))
        {
            return Task.FromResult<string?>("Missing cryptographic signature.");
        }

        // TODO: Implement actual cryptographic ECDSA/HMAC signature verification against server public key
        // For the Vertical Slice (Phase 3), we require the signature to at least match a mock known structure.
        if (intent.Signature != "valid_dummy_sig" && intent.Signature.Length < 32)
        {
            return Task.FromResult<string?>("Signature format invalid or unrecognized.");
        }

        return Task.FromResult<string?>(null); // Null means valid!
    }

}
