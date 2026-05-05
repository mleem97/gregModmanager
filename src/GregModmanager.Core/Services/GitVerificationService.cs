using System;
using System.Net.Http;
using System.Threading.Tasks;

namespace GregModmanager.Services;

/// <summary>
/// Verifies the current user against the gregFramework Git infrastructure.
/// </summary>
public sealed class GitVerificationService
{
    private const string GitServerUrl = "https://git.datacentermods.com/api/v1/user";
    private readonly HttpClient _http = new();

    public async Task<bool> VerifyUserAsync(string apiToken)
    {
        if (string.IsNullOrWhiteSpace(apiToken)) return false;

        try
        {
            var request = new HttpRequestMessage(HttpMethod.Get, GitServerUrl);
            request.Headers.Add("Authorization", $"token {apiToken}");
            
            var response = await _http.SendAsync(request);
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    public string GetVerificationStatus(bool verified)
    {
        return verified ? "Verified Member" : "Guest / Unauthenticated";
    }
}
