using System;
using System.Net.Http;
using System.Threading.Tasks;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Auth;

public class AuthApiClient : IAuthApiClient
{
    private const string BaseUrl = "https://gregframework.eu/api/auth";
    private readonly HttpClient _http = new();

    private readonly BetterAuthService _legacyAuth;

    public AuthApiClient(BetterAuthService legacyAuth)
    {
        _legacyAuth = legacyAuth;
    }

    public async Task<string> GetLoginUrlAsync(string requestId)
    {
        // Mock returning a URL to launch
        var redirectUri = System.Uri.EscapeDataString("greg://v1/auth/callback");
        return await Task.FromResult($"https://datacentermods.com/auth/login?client_id=greg_desktop&response_type=code&redirect_uri={redirectUri}&requestId={requestId}&state=desktop_flow&nonce=mock_nonce");
    }

    public async Task<ActiveSession?> ExchangeCallbackCodeAsync(string requestId, string code, string state, string nonce, string signature)
    {
        // TODO: Call web to exchange callback arguments for access/refresh tokens
        await Task.Delay(10);
        return new ActiveSession
        {
            AccessToken = "mock_access_token",
            User = new AccountIdentity { DisplayName = "Mocked User", Roles = new[] { "user" } }
        };
    }

    public async Task<bool> EndSessionAsync(string accessToken)
    {
        await Task.Delay(10);
        return true;
    }
}
