using System;
using System.Threading.Tasks;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Auth;

public interface IAuthApiClient
{
    Task<string> GetLoginUrlAsync(string requestId);
    Task<ActiveSession?> ExchangeCallbackCodeAsync(string requestId, string code, string state, string nonce, string signature);
    Task<bool> EndSessionAsync(string accessToken);
}

public interface ISessionManager
{
    SessionState State { get; }
    ActiveSession? CurrentSession { get; }
    event Action? StateChanged;
    event Action<string>? ProtocolInvoked;

    Task InitializeAsync();
    Task StartBrowserLoginAsync();
    Task HandleProtocolCallbackAsync(string rawUri);
    Task LogoutAsync();
}
