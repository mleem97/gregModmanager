using System;
using System.Text.Json.Serialization;

namespace GregModmanager.Models.Auth;

public enum SessionState
{
    Anonymous,
    LoginPending,
    Authenticated,
    Refreshing,
    Revoked,
    Error
}

public class AccountIdentity
{
    public string SubjectId { get; set; } = string.Empty;
    public string Email { get; set; } = string.Empty;
    public string DisplayName { get; set; } = string.Empty;
    public string AvatarUrl { get; set; } = string.Empty;
    public string[] Roles { get; set; } = Array.Empty<string>();
    public string Tenant { get; set; } = string.Empty;
}

public class ActiveSession
{
    public AccountIdentity User { get; set; } = new();
    public string AccessToken { get; set; } = string.Empty;
    public string SessionId { get; set; } = string.Empty;
}

public class TokenExchangeRequest
{
    [JsonPropertyName("requestId")]
    public string RequestId { get; set; } = string.Empty;

    [JsonPropertyName("code")]
    public string Code { get; set; } = string.Empty;

    [JsonPropertyName("state")]
    public string State { get; set; } = string.Empty;

    [JsonPropertyName("nonce")]
    public string Nonce { get; set; } = string.Empty;

    [JsonPropertyName("signature")]
    public string Signature { get; set; } = string.Empty;

    [JsonPropertyName("redirectUri")]
    public string RedirectUri { get; set; } = string.Empty;
}

public class TokenExchangeResponse
{
    [JsonPropertyName("accessToken")]
    public string AccessToken { get; set; } = string.Empty;

    [JsonPropertyName("sessionId")]
    public string? SessionId { get; set; }

    [JsonPropertyName("user")]
    public TokenExchangeUser? User { get; set; }
}

public class TokenExchangeUser
{
    [JsonPropertyName("subjectId")]
    public string SubjectId { get; set; } = string.Empty;

    [JsonPropertyName("email")]
    public string Email { get; set; } = string.Empty;

    [JsonPropertyName("displayName")]
    public string DisplayName { get; set; } = string.Empty;

    [JsonPropertyName("avatarUrl")]
    public string AvatarUrl { get; set; } = string.Empty;

    [JsonPropertyName("roles")]
    public string[] Roles { get; set; } = Array.Empty<string>();

    [JsonPropertyName("tenant")]
    public string Tenant { get; set; } = string.Empty;
}
