using System;
using System.Diagnostics;
using System.Threading.Tasks;
using GregModmanager.Localization;
using GregModmanager.Services;
using GregModmanager.Models.Auth;

namespace GregModmanager.Services.Auth;
public class SessionManager : ISessionManager
{
    private readonly IAuthApiClient _apiClient;
    private string _currentRequestId = string.Empty;

    public SessionState State { get; private set; } = SessionState.Anonymous;
    public ActiveSession? CurrentSession { get; private set; }

    public event Action? StateChanged;
    public event Action<string>? ProtocolInvoked;

    public SessionManager(IAuthApiClient apiClient)
    {
        _apiClient = apiClient;
    }

    public Task InitializeAsync()
    {
        ProtocolSingleInstance.StartListening(uri =>
        {
            ProtocolInvoked?.Invoke(uri);
        });

        var refresh = S.Preferences.GetString("greg_refresh_token", "");
        if (!string.IsNullOrEmpty(refresh))
        {
            State = SessionState.Refreshing;
            StateChanged?.Invoke();

            // Exchange refresh token for new session via API
            _ = Task.Run(async () =>
            {
                try
                {
                    var session = await _apiClient.ExchangeCallbackCodeAsync(
                        requestId: string.Empty,
                        code: string.Empty,
                        state: "refresh",
                        nonce: string.Empty,
                        signature: refresh);

                    if (session != null)
                    {
                        CurrentSession = session;
                        State = SessionState.Authenticated;
                        AppFileLog.Info("Session restored from refresh token.");
                    }
                    else
                    {
                        S.Preferences.Remove("greg_refresh_token");
                        State = SessionState.Anonymous;
                        AppFileLog.Warn("Refresh token exchange failed. Cleared stored token.");
                    }
                }
                catch (Exception ex)
                {
                    AppFileLog.Error("Refresh token exchange failed", ex);
                    S.Preferences.Remove("greg_refresh_token");
                    State = SessionState.Anonymous;
                }
                StateChanged?.Invoke();
            });
        }
        else
        {
            State = SessionState.Anonymous;
        }
        StateChanged?.Invoke();
        return Task.CompletedTask;
    }

    public async Task StartBrowserLoginAsync()
    {
        State = SessionState.LoginPending;
        StateChanged?.Invoke();

        _currentRequestId = Guid.NewGuid().ToString("N");
        var loginUrl = await _apiClient.GetLoginUrlAsync(_currentRequestId);
        if (!string.IsNullOrEmpty(loginUrl))
        {
            try
            {
                await SafeProcess.OpenUrlAsync(loginUrl);
            }
            catch (Exception ex)
            {
                AppFileLog.Warn($"Browser launch failed: {ex.Message}");
                State = SessionState.Error;
                StateChanged?.Invoke();
            }
        }
        else
        {
            State = SessionState.Error;
            StateChanged?.Invoke();
        }
    }

    public async Task HandleProtocolCallbackAsync(string rawUri)
    {
        if (State != SessionState.LoginPending) return;

        try
        {
            var uri = new Uri(rawUri);
            var query = System.Web.HttpUtility.ParseQueryString(uri.Query);

            var code = query["code"] ?? string.Empty;
            var reqId = query["requestId"] ?? string.Empty;
            var state = query["state"] ?? string.Empty;
            var nonce = query["nonce"] ?? string.Empty;
            var sig = query["sig"] ?? string.Empty;

            if (reqId != _currentRequestId)
            {
                AppFileLog.Warn("Callback request ID mismatch or replay attempt.");
                State = SessionState.Error;
                StateChanged?.Invoke();
                return;
            }

            var session = await _apiClient.ExchangeCallbackCodeAsync(reqId, code, state, nonce, sig);
            if (session != null)
            {
                CurrentSession = session;
                State = SessionState.Authenticated;
                S.Preferences.SetString("greg_refresh_token", session.AccessToken);
                StateChanged?.Invoke();
                AppFileLog.Info("User session authenticated successfully.");
            }
            else
            {
                State = SessionState.Error;
                StateChanged?.Invoke();
                AppFileLog.Warn("Callback exchange failed API validation.");
            }
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Callback handling failed", ex);
            State = SessionState.Error;
            StateChanged?.Invoke();
        }
    }

    public async Task LogoutAsync()
    {
        if (CurrentSession != null)
        {
            await _apiClient.EndSessionAsync(CurrentSession.AccessToken);
        }
        S.Preferences.Remove("greg_refresh_token");
        CurrentSession = null;
        State = SessionState.Anonymous;
        StateChanged?.Invoke();
        AppFileLog.Info("User session logged out.");
    }
}


