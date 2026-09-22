using System;
using System.Net.Http;
using System.Net.Http.Json;
using System.Threading.Tasks;
using GregModmanager.Models;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Auth;

public class AuthApiClient : IAuthApiClient
{
    private readonly HttpClient _http;
    private readonly IReadOnlyList<string> _loginUrlFormats;
    private readonly IReadOnlyList<string> _authApiBaseUrls;

    public AuthApiClient()
        : this(CreateHttpClient(), AppSettings.DesktopLoginUrlFormats, AppSettings.AuthApiBaseUrls)
    {
    }

    public AuthApiClient(HttpClient http, IReadOnlyList<string> loginUrlFormats, IReadOnlyList<string> authApiBaseUrls)
    {
        _http = http;
        _loginUrlFormats = loginUrlFormats;
        _authApiBaseUrls = authApiBaseUrls;
    }

    public async Task<string> GetLoginUrlAsync(string requestId)
    {
        var escapedRedirect = Uri.EscapeDataString(AppSettings.AuthCallbackRedirectUri);
        var candidates = _loginUrlFormats
            .Select(format => string.Format(format, escapedRedirect, Uri.EscapeDataString(requestId)))
            .ToArray();
        return await AuthEndpointFallback.ResolveReachableUrlAsync(_http, candidates);
    }

    public async Task<ActiveSession?> ExchangeCallbackCodeAsync(string requestId, string code, string state, string nonce, string signature)
    {
        try
        {
            var payload = new TokenExchangeRequest
            {
                RequestId = requestId,
                Code = code,
                State = state,
                Nonce = nonce,
                Signature = signature,
                RedirectUri = AppSettings.AuthCallbackRedirectUri
            };

            using var response = await AuthEndpointFallback.SendAsync(
                _http,
                _authApiBaseUrls,
                baseUrl => new HttpRequestMessage(HttpMethod.Post, $"{baseUrl}/token")
                {
                    Content = JsonContent.Create(payload, AppJsonContext.Default.TokenExchangeRequest)
                });

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
                            AvatarUrl = tokenResponse.User?.AvatarUrl ?? string.Empty,
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
            using var response = await AuthEndpointFallback.SendAsync(
                _http,
                _authApiBaseUrls,
                baseUrl =>
                {
                    var request = new HttpRequestMessage(HttpMethod.Post, $"{baseUrl}/logout");
                    request.Headers.Add("Authorization", $"Bearer {accessToken}");
                    return request;
                });
            return response.IsSuccessStatusCode;
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Session end request failed", ex);
            return false;
        }
    }

    private static HttpClient CreateHttpClient() => new()
    {
        Timeout = TimeSpan.FromSeconds(10)
    };
}
