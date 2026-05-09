using System.Net.Http;
using System.Net.Http.Json;
using GregModmanager.Models;

namespace GregModmanager.Services;

public sealed class BetterAuthService
{
    private const string BaseUrl = "https://gregframework.eu/api/auth";
    private readonly HttpClient _http = new();

    public async Task<AuthResponse?> LoginAsync(string email, string password)
    {
        try
        {
            var payload = new LoginRequest { Email = email, Password = password };
            var response = await _http.PostAsJsonAsync($"{BaseUrl}/sign-in/email", payload, AppJsonContext.Default.LoginRequest);

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
            var request = new HttpRequestMessage(HttpMethod.Get, $"{BaseUrl}/get-session");
            request.Headers.Add("Authorization", $"Bearer {token}");

            var response = await _http.SendAsync(request);
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }
}
