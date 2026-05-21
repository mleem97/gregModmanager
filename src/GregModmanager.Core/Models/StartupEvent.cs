namespace GregModmanager.Models;

public sealed record StartupEvent(
    bool SteamActive,
    string Culture,
    string OsDescription,
    string DotNetVersion
);
