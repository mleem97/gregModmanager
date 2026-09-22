using System.Net.Http;
using System.Net.Http.Json;
using GregModmanager.Models;
using GregModmanager.Services.Auth;

namespace GregModmanager.Services;

public sealed class BetterAuthService
{
    private readonly HttpClient _http;
    private readonly IReadOnlyList<string> _baseUrls;

    public BetterAuthService()
        : this(new HttpClient { Timeout = TimeSpan.FromSeconds(10) }, AppSettings.BetterAuthApiBaseUrls)
    {
    }

    public BetterAuthService(HttpClient http, IReadOnlyList<string> baseUrls)
    {
        _http = http;
        _baseUrls = baseUrls;
    }

    public async Task<AuthResponse?> LoginAsync(string email, string password)
    {
        try
        {
            var payload = new LoginRequest { Email = email, Password = password };
            using var response = await AuthEndpointFallback.SendAsync(
                _http,
                _baseUrls,
                baseUrl => new HttpRequestMessage(HttpMethod.Post, $"{baseUrl}/sign-in/email")
                {
                    Content = JsonContent.Create(payload, AppJsonContext.Default.LoginRequest)
                });

            if (response.IsSuccessStatusCode)
            {
                return await response.Content.ReadFromJsonAsync(AppJsonContext.Default.AuthResponse);
            }
        }
        catch (Exception)
        {
            // Log error
        }
        return null;
    }

    public async Task<bool> VerifySessionAsync(string token)
    {
        try
        {
            using var response = await AuthEndpointFallback.SendAsync(
                _http,
                _baseUrls,
                baseUrl =>
                {
                    var request = new HttpRequestMessage(HttpMethod.Get, $"{baseUrl}/get-session");
                    request.Headers.Add("Authorization", $"Bearer {token}");
                    return request;
                });
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }
}
