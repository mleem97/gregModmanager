namespace GregModmanager.Models;

public class AppStartupEvent
{
    public bool SteamActive { get; set; }
    public string Culture { get; set; } = "";
    public string OsDescription { get; set; } = "";
    public string DotNetVersion { get; set; } = "";
}
