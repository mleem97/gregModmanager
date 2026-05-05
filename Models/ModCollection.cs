namespace GregModmanager.Models;

public enum ModCollectionSourceKind
{
	Local,
	GregStore,
	SteamWorkshop,
}

public sealed class ModCollectionDefinition
{
	public Guid Id { get; set; } = Guid.NewGuid();
	public string Name { get; set; } = string.Empty;
	public string Description { get; set; } = string.Empty;
	public ModCollectionSourceKind SourceKind { get; set; } = ModCollectionSourceKind.Local;
	public string SourceName { get; set; } = "Local";
	public string? WorkshopCollectionUrl { get; set; }
	public DateTimeOffset CreatedUtc { get; set; } = DateTimeOffset.UtcNow;
	public DateTimeOffset UpdatedUtc { get; set; } = DateTimeOffset.UtcNow;
	public List<ModCollectionEntry> Items { get; set; } = new();
}

public sealed class ModCollectionEntry
{
	public ulong PublishedFileId { get; set; }
	public string Title { get; set; } = string.Empty;
	public string SourceName { get; set; } = "Steam Workshop";
	public string ModType { get; set; } = "PlacableObject";
	public List<ulong> WorkshopDependencyIds { get; set; } = new();
	public string? Notes { get; set; }
}