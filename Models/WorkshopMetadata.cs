using System.Text.Json.Serialization;

namespace GregModmanager.Models;

public sealed class WorkshopMetadata
{
	[JsonPropertyName("publishedFileId")]
	public ulong PublishedFileId { get; set; }

	[JsonPropertyName("title")]
	public string Title { get; set; } = "";

	[JsonPropertyName("description")]
	public string Description { get; set; } = "";

	/// <summary>Public, FriendsOnly, or Private.</summary>
	[JsonPropertyName("visibility")]
	public string Visibility { get; set; } = "Public";

	[JsonPropertyName("previewImageRelativePath")]
	public string PreviewImageRelativePath { get; set; } = "preview.png";

	[JsonPropertyName("tags")]
	public List<string> Tags { get; set; } = new();

	[JsonPropertyName("needsgreg")]
	public bool Needsgreg { get; set; }

	/// <summary>When true, upload appends a MelonLoader requirement + link to the Steam description if not already mentioned.</summary>
	[JsonPropertyName("needsMelonLoader")]
	public bool NeedsMelonLoader { get; set; }

	/// <summary>
	/// <c>decoration</c> — native shop/static items in <c>config.json</c>. <c>code</c> — only native DLL entries; code mods use MelonLoader, not shop/static in this file.
	/// </summary>
	[JsonPropertyName("nativeConfigProfile")]
	public string NativeConfigProfile { get; set; } = "decoration";

	[JsonPropertyName("additionalPreviews")]
	public List<string> AdditionalPreviews { get; set; } = new();

	/// <summary>Other Steam Workshop file ids this item depends on (subscribe order / tooling).</summary>
	[JsonPropertyName("workshop_dependency")]
	public List<ulong> WorkshopDependencyIds { get; set; } = new();

	/// <summary>
	/// <c>PlacableObject</c>, <c>MelonloaderPlugin</c>, <c>Userlib</c>, or <c>DataCenterMod</c>.
	/// Determines the installation folder when the item is synced from Steam.
	/// </summary>
	[JsonPropertyName("modType")]
	public string ModType { get; set; } = "PlacableObject";
}

