using GregModmanager.Localization;

namespace GregModmanager.Services;

public static class AppSettings
{
    public const string ModStoreEnabledKey = "ModStoreEnabled";
    public const string GameRootPathKey = "GameRootPath";

    public static string GetGameRootPath()
    {
        return S.Preferences.GetString(GameRootPathKey, "");
    }

    public static bool IsModStoreEnabled()
    {
        return S.Preferences.GetBool(ModStoreEnabledKey, false);
    }
}
