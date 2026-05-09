using System;
using System.Net.Http;
using System.Threading.Tasks;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Auth;

public class AuthApiClient : IAuthApiClient
{
    private const string BaseAuthUrl = "https://gregframework.eu/api/auth";
    private const string DesktopLoginUrl = "https://datacentermods.com/auth/login?client_id=greg_desktop&response_type=code&redirect_uri={0}&requestId={1}&state=desktop_flow&nonce=mock_nonce";
    private const string RedirectUriValue = "greg://v1/auth/callback";

    public async Task<string> GetLoginUrlAsync(string requestId)
    {
        // Mock returning a URL to launch
        var escapedRedirect = Uri.EscapeDataString(RedirectUriValue);
        return await Task.FromResult(string.Format(DesktopLoginUrl, escapedRedirect, requestId));
    }

    public async Task<ActiveSession?> ExchangeCallbackCodeAsync(string requestId, string code, string state, string nonce, string signature)
    {
        // Placeholder: Implementation required for web callback argument exchange
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
