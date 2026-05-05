using System;

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
    public string[] Roles { get; set; } = Array.Empty<string>();
    public string Tenant { get; set; } = string.Empty;
}

public class ActiveSession
{
    public AccountIdentity User { get; set; } = new();
    public string AccessToken { get; set; } = string.Empty;
    public string SessionId { get; set; } = string.Empty;
}
