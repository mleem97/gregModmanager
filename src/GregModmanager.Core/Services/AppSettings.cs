using GregModmanager.Localization;
using System.Runtime.InteropServices;
using System.Text.RegularExpressions;

namespace GregModmanager.Services;

public static class AppSettings
{
    public static string ModStoreEnabledKey => "ModStoreEnabled";
    public static string AppModeKey => "AppMode";
    public const string AppModeFull = "full";
    public const string AppModeModManagerOnly = "modmanager-only";
    public const string AppModeDecideLater = "decide-later";
    public static string GameRootPathKey => "GameRootPath";
    public static string TelemetryEnabledKey => "TelemetryEnabled";

    public static bool IsLocalBuild => 
        string.Equals(Environment.GetEnvironmentVariable("IS_LOCAL_BUILD"), "TRUE", StringComparison.OrdinalIgnoreCase);

    /// <summary>Uses the local HTTPS webapp at datacentermods.home and its API subdomain.</summary>
    public static bool IsLocalTestBuild =>
        string.Equals(Environment.GetEnvironmentVariable("IS_LOCAL_TEST_BUILD"), "TRUE", StringComparison.OrdinalIgnoreCase);

    public static string ModStoreWebBaseUrl =>
        Environment.GetEnvironmentVariable("MODSTORE_WEB_URL")
        ?? (IsLocalTestBuild ? "https://datacentermods.home" : "https://datacentermods.com");

    public static string ModStoreApiBaseUrl =>
        Environment.GetEnvironmentVariable("MODSTORE_API_URL")
        ?? (IsLocalTestBuild ? "https://api.datacentermods.home" : "https://datacentermods.com");

    public static string AuthApiBaseUrl => $"{ModStoreApiBaseUrl.TrimEnd('/')}/auth";

    [System.Diagnostics.CodeAnalysis.SuppressMessage("SonarLint", "S1075", Justification = "Hardcoded fallback URIs are required for core ecosystem stability.")]
    public static string MelonLoaderReleasesUrl => 
        Environment.GetEnvironmentVariable("MELONLOADER_RELEASES_URL") 
        ?? (IsLocalBuild ? "http://localhost:5000/releases" : "https://github.com/LavaGang/MelonLoader/releases");

    public static string MelonLoaderLatestReleaseApiUrl =>
        Environment.GetEnvironmentVariable("MELONLOADER_LATEST_API_URL")
        ?? "https://api.github.com/repos/LavaGang/MelonLoader/releases/latest";
    
    [System.Diagnostics.CodeAnalysis.SuppressMessage("SonarLint", "S1075", Justification = "Hardcoded fallback URIs are required for core ecosystem stability.")]
    public static string DesktopLoginUrlFormat => 
        Environment.GetEnvironmentVariable("AUTH_LOGIN_URL_FORMAT")
        ?? (IsLocalBuild 
            ? "http://localhost:5001/auth/login?client_id=greg_desktop&response_type=code&redirect_uri={0}&requestId={1}&state=desktop_flow&nonce=mock_nonce" 
            : $"{ModStoreWebBaseUrl}/auth/login?client_id=greg_desktop&response_type=code&redirect_uri={{0}}&requestId={{1}}&state=desktop_flow&nonce=desktop_nonce");
    
    public static string AuthCallbackRedirectUri => 
        Environment.GetEnvironmentVariable("AUTH_CALLBACK_REDIRECT_URI")
        ?? "greg://v1/auth/callback";

    [System.Diagnostics.CodeAnalysis.SuppressMessage("SonarLint", "S1075", Justification = "Hardcoded fallback URIs are required for core ecosystem stability.")]
    public static string DefaultLokiUrl => 
        Environment.GetEnvironmentVariable("TELEMETRY_URL") 
        ?? (IsLocalBuild ? "http://localhost:3100/loki/api/v1/push" : TelemetrySecrets.LokiUrl);

    [System.Diagnostics.CodeAnalysis.SuppressMessage("SonarLint", "S1075", Justification = "Hardcoded fallback URIs are required for core ecosystem stability.")]
    public static string GitServerUrl => 
        Environment.GetEnvironmentVariable("GIT_SERVER_URL") 
        ?? (IsLocalBuild ? "http://localhost:3000/api/v1/user" : "https://git.datacentermods.com/api/v1/user");

    public static string GetGameRootPath()
    {
        var configured = S.Preferences.GetString(GameRootPathKey, "").Trim();
        if (!string.IsNullOrWhiteSpace(configured) && Directory.Exists(configured))
            return Path.GetFullPath(configured);

        var detected = TryDetectGameRoot();
        return detected ?? string.Empty;
    }

    /// <summary>Finds the installed Data Center directory without depending on the current working directory.</summary>
    public static string? TryDetectGameRoot()
    {
        var env = Environment.GetEnvironmentVariable("DATA_CENTER_GAME_DIR")?.Trim();
        if (!string.IsNullOrWhiteSpace(env) && Directory.Exists(env))
            return Path.GetFullPath(env);

        var steamRoots = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        void Add(string? path)
        {
            if (!string.IsNullOrWhiteSpace(path) && Directory.Exists(path))
                steamRoots.Add(Path.GetFullPath(path));
        }

        var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            Add(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86) + "/Steam");
            Add(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles) + "/Steam");
            Add(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData) + "/Steam");
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            Add(Path.Combine(home, "Library/Application Support/Steam"));
        }
        else
        {
            Add(Path.Combine(home, ".steam/steam"));
            Add(Path.Combine(home, ".steam/root"));
            Add(Path.Combine(home, ".local/share/Steam"));
            Add(Path.Combine(home, ".var/app/com.valvesoftware.Steam/.local/share/Steam"));
        }

        var libraries = steamRoots.ToList();
        foreach (var root in libraries)
        {
            var vdf = Path.Combine(root, "steamapps", "libraryfolders.vdf");
            if (!File.Exists(vdf)) continue;

            try
            {
                foreach (Match match in Regex.Matches(File.ReadAllText(vdf), "\\\"path\\\"\\s+\\\"([^\\\"]+)\\\"", RegexOptions.IgnoreCase))
                    Add(match.Groups[1].Value.Replace("\\\\", "\\"));
            }
            catch
            {
                // A malformed or inaccessible library must not prevent other candidates from being checked.
            }
        }

        foreach (var steamRoot in steamRoots)
        {
            var common = Path.Combine(steamRoot, "steamapps", "common");
            foreach (var folderName in new[] { "Data Center", "DataCenter" })
            {
                var candidate = Path.Combine(common, folderName);
                if (Directory.Exists(candidate))
                    return Path.GetFullPath(candidate);
            }
        }

        return null;
    }

    public static bool IsModStoreEnabled()
    {
        return !string.Equals(GetAppMode(), AppModeModManagerOnly, StringComparison.OrdinalIgnoreCase);
    }

    public static string GetAppMode()
    {
        var mode = S.Preferences.GetString(AppModeKey, "").Trim();
        if (string.Equals(mode, AppModeModManagerOnly, StringComparison.OrdinalIgnoreCase))
            return AppModeModManagerOnly;
        if (string.Equals(mode, AppModeDecideLater, StringComparison.OrdinalIgnoreCase))
            return AppModeDecideLater;
        if (string.Equals(mode, AppModeFull, StringComparison.OrdinalIgnoreCase))
            return AppModeFull;

        // Existing installations without the mode setting keep the Modstore enabled by default.
        return AppModeFull;
    }

    public static bool HasAppModeChoice()
        => !string.IsNullOrWhiteSpace(S.Preferences.GetString(AppModeKey, ""));

    public static void SetAppMode(string mode)
    {
        if (mode is not AppModeFull and not AppModeModManagerOnly and not AppModeDecideLater)
            throw new ArgumentOutOfRangeException(nameof(mode));

        S.Preferences.SetString(AppModeKey, mode);
        S.Preferences.SetBool(ModStoreEnabledKey, mode != AppModeModManagerOnly);
    }

    public static bool IsTelemetryEnabled()
    {
        return S.Preferences.GetBool(TelemetryEnabledKey, true);
    }
}
