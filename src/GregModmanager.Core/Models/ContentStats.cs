namespace GregModmanager.Models;

/// <summary>Size breakdown for <c>content/</c>.</summary>
public sealed class ContentStats
{
	public bool Exists { get; init; }

	public long TotalBytes { get; init; }

	/// <summary>Largest first; files directly under <c>content/</c> are named <c>(files)</c>.</summary>
	public IReadOnlyList<ContentFolderSize> TopEntries { get; init; } = Array.Empty<ContentFolderSize>();
}

public readonly record struct ContentFolderSize(string Name, long Bytes);

