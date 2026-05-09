using System;
using System.Net.Http;
using System.Threading.Tasks;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Auth;

public class AuthApiClient : IAuthApiClient
{
    public async Task<string> GetLoginUrlAsync(string requestId)
    {
        // Mock returning a URL to launch
        var escapedRedirect = Uri.EscapeDataString(AppSettings.AuthCallbackRedirectUri);
        return await Task.FromResult(string.Format(AppSettings.DesktopLoginUrlFormat, escapedRedirect, requestId));
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
