using GregModmanager.Localization;

namespace GregModmanager.Services;

public static class AppSettings
{
    public const string ModStoreEnabledKey = "ModStoreEnabled";
    public const string GameRootPathKey = "GameRootPath";
    public const string TelemetryEnabledKey = "TelemetryEnabled";

    public static bool IsLocalBuild => 
        string.Equals(Environment.GetEnvironmentVariable("IS_LOCAL_BUILD"), "TRUE", StringComparison.OrdinalIgnoreCase);

    [System.Diagnostics.CodeAnalysis.SuppressMessage("SonarLint", "S1075", Justification = "Hardcoded fallback URIs are required for core ecosystem stability.")]
    public static string MelonLoaderReleasesUrl => 
        Environment.GetEnvironmentVariable("MELONLOADER_RELEASES_URL") 
        ?? (IsLocalBuild ? "http://localhost:5000/releases" : "https://github.com/LavaGang/MelonLoader/releases");
    
    [System.Diagnostics.CodeAnalysis.SuppressMessage("SonarLint", "S1075", Justification = "Hardcoded fallback URIs are required for core ecosystem stability.")]
    public static string DesktopLoginUrlFormat => 
        Environment.GetEnvironmentVariable("AUTH_LOGIN_URL_FORMAT")
        ?? (IsLocalBuild 
            ? "http://localhost:5001/auth/login?client_id=greg_desktop&response_type=code&redirect_uri={0}&requestId={1}&state=desktop_flow&nonce=mock_nonce" 
            : "https://datacentermods.com/auth/login?client_id=greg_desktop&response_type=code&redirect_uri={0}&requestId={1}&state=desktop_flow&nonce=mock_nonce");
    
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
        return S.Preferences.GetString(GameRootPathKey, "");
    }

    public static bool IsModStoreEnabled()
    {
        return S.Preferences.GetBool(ModStoreEnabledKey, false);
    }

    public static bool IsTelemetryEnabled()
    {
        return S.Preferences.GetBool(TelemetryEnabledKey, true);
    }
}
