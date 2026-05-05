namespace GregModmanager.Models;

/// <summary>Metadata for a distributable greg plugin artifact (stable or beta channel).</summary>
public sealed class PluginPackageInfo
{
	public required string PluginId { get; init; }
	public required string Version { get; init; }
	public required string Channel { get; init; }
	public string? DownloadUrl { get; init; }
	public string? Sha256 { get; init; }
}

