namespace GregModmanager.Models;

public sealed class TelemetryStartupEvent
{
	public bool SteamActive { get; set; }
	public string Culture { get; set; } = string.Empty;
	public string OsDescription { get; set; } = string.Empty;
	public string DotNetVersion { get; set; } = string.Empty;
}
