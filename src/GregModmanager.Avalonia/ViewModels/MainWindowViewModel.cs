using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace GregModmanager.Avalonia.ViewModels;

public sealed class MainWindowViewModel : INotifyPropertyChanged
{
    private bool _steamConnected = true;
    private bool _gregApiOnline;
    private bool _isLoggedIn;
    private string _username = string.Empty;
    private bool _cooldownActive = true;
    private int _cooldownSecondsRemaining = 42;

    public event PropertyChangedEventHandler? PropertyChanged;

    public bool SteamConnected
    {
        get => _steamConnected;
        set
        {
            if (SetField(ref _steamConnected, value))
            {
                Raise(nameof(SteamStatusText));
            }
        }
    }

    public bool GregApiOnline
    {
        get => _gregApiOnline;
        set
        {
            if (SetField(ref _gregApiOnline, value))
            {
                Raise(nameof(GregApiStatusText));
            }
        }
    }

    public bool IsLoggedIn
    {
        get => _isLoggedIn;
        set
        {
            if (SetField(ref _isLoggedIn, value))
            {
                Raise(nameof(LoginStatusText));
            }
        }
    }

    public string Username
    {
        get => _username;
        set
        {
            if (SetField(ref _username, value))
            {
                Raise(nameof(LoginStatusText));
            }
        }
    }

    public bool CooldownActive
    {
        get => _cooldownActive;
        set
        {
            if (SetField(ref _cooldownActive, value))
            {
                Raise(nameof(CooldownMessage));
                Raise(nameof(CooldownFootnote));
                Raise(nameof(CanUpload));
            }
        }
    }

    public int CooldownSecondsRemaining
    {
        get => _cooldownSecondsRemaining;
        set
        {
            if (SetField(ref _cooldownSecondsRemaining, value))
            {
                Raise(nameof(CooldownMessage));
                Raise(nameof(CooldownFootnote));
            }
        }
    }

    public string SteamStatusText => SteamConnected ? "Steam Connected" : "Steam Disconnected";

    public string GregApiStatusText => GregApiOnline ? "gregAPI Online" : "gregAPI Offline";

    public string LoginStatusText
    {
        get
        {
            if (!IsLoggedIn) return "Login To Datacentermods.com";
            var displayName = string.IsNullOrWhiteSpace(Username) ? "User" : Username;
            return $"Logged in as {displayName}";
        }
    }

    public string CooldownMessage => CooldownActive
        ? $"Steam cooldown active - retry in {CooldownSecondsRemaining}s"
        : "Ready for upload";

    public string CooldownFootnote => CooldownActive
        ? $"Rate limit active • next slot in {CooldownSecondsRemaining}s"
        : "Steam rate limiter idle • upload slot available";

    public bool CanUpload => !CooldownActive;

    private bool SetField<T>(ref T field, T value, [CallerMemberName] string? propertyName = null)
    {
        if (EqualityComparer<T>.Default.Equals(field, value))
        {
            return false;
        }

        field = value;
        Raise(propertyName);
        return true;
    }

    private void Raise(string? propertyName)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}
