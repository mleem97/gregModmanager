using System;
using System.Net.Http;
using System.Net.Http.Json;
using System.Threading.Tasks;
using System.Text.Json.Serialization;

namespace GregModmanager.Services;

public sealed class BetterAuthService
{
    private const string BaseUrl = "https://gregframework.eu/api/auth";
    private readonly HttpClient _http = new();

    public async Task<AuthResponse?> LoginAsync(string email, string password)
    {
        try
        {
            var payload = new { email, password };
            var response = await _http.PostAsJsonAsync($"{BaseUrl}/sign-in/email", payload);

            if (response.IsSuccessStatusCode)
            {
                return await response.Content.ReadFromJsonAsync<AuthResponse>();
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

public class AuthResponse
{
    [JsonPropertyName("token")]
    public string Token { get; set; } = string.Empty;

    [JsonPropertyName("user")]
    public UserInfo? User { get; set; }
}

public class UserInfo
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("email")]
    public string Email { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;
}
