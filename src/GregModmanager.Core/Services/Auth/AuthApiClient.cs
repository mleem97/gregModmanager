using System;
using System.Net.Http;
using System.Net.Http.Json;
using System.Threading.Tasks;
using GregModmanager.Models;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Auth;

public class AuthApiClient : IAuthApiClient
{
    private readonly HttpClient _http = new();

    public async Task<string> GetLoginUrlAsync(string requestId)
    {
        var escapedRedirect = Uri.EscapeDataString(AppSettings.AuthCallbackRedirectUri);
        return await Task.FromResult(string.Format(AppSettings.DesktopLoginUrlFormat, escapedRedirect, requestId));
    }

    public async Task<ActiveSession?> ExchangeCallbackCodeAsync(string requestId, string code, string state, string nonce, string signature)
    {
        try
        {
            var baseUrl = AppSettings.IsLocalBuild
                ? "http://localhost:5001/auth"
                : "https://datacentermods.com/auth";

            var payload = new TokenExchangeRequest
            {
                RequestId = requestId,
                Code = code,
                State = state,
                Nonce = nonce,
                Signature = signature,
                RedirectUri = AppSettings.AuthCallbackRedirectUri
            };

            var response = await _http.PostAsJsonAsync($"{baseUrl}/token", payload, AppJsonContext.Default.TokenExchangeRequest);

            if (response.IsSuccessStatusCode)
            {
                var tokenResponse = await response.Content.ReadFromJsonAsync(AppJsonContext.Default.TokenExchangeResponse);
                if (tokenResponse != null && !string.IsNullOrEmpty(tokenResponse.AccessToken))
                {
                    return new ActiveSession
                    {
                        AccessToken = tokenResponse.AccessToken,
                        SessionId = tokenResponse.SessionId ?? string.Empty,
                        User = new AccountIdentity
                        {
                            SubjectId = tokenResponse.User?.SubjectId ?? string.Empty,
                            Email = tokenResponse.User?.Email ?? string.Empty,
                            DisplayName = tokenResponse.User?.DisplayName ?? "User",
                            Roles = tokenResponse.User?.Roles ?? new[] { "user" },
                            Tenant = tokenResponse.User?.Tenant ?? string.Empty
                        }
                    };
                }
            }

            AppFileLog.Warn($"Token exchange failed: {response.StatusCode}");
            return null;
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Token exchange request failed", ex);
            return null;
        }
    }

    public async Task<bool> EndSessionAsync(string accessToken)
    {
        try
        {
            var baseUrl = AppSettings.IsLocalBuild
                ? "http://localhost:5001/auth"
                : "https://datacentermods.com/auth";

            var request = new HttpRequestMessage(HttpMethod.Post, $"{baseUrl}/logout");
            request.Headers.Add("Authorization", $"Bearer {accessToken}");

            var response = await _http.SendAsync(request);
            return response.IsSuccessStatusCode;
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Session end request failed", ex);
            return false;
        }
    }
}
