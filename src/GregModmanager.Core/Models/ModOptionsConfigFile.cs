using System.Text.Json;
using System.Text.Json.Serialization;

namespace GregModmanager.Models;

/// <summary>Optional <c>content/modconfig.json</c> for runtime options used by standalone or greg mods (MelonLoader / framework loaders).</summary>
/// <remarks><c>config.json</c> remains the required native Workshop definition when using shop/static assets.</remarks>
public sealed class ModOptionsConfigFile
{
	[JsonPropertyName("schemaVersion")]
	public int SchemaVersion { get; set; } = 1;

	/// <summary><c>standalone</c> (MelonLoader-only) or <c>greg</c> (gregCoreModFramework).</summary>
	[JsonPropertyName("modKind")]
	public string ModKind { get; set; } = "standalone";

	[JsonPropertyName("notes")]
	public string Notes { get; set; } = "";

	/// <summary>Open-ended key/value settings (strings, numbers, booleans — stored as JSON values).</summary>
	[JsonPropertyName("settings")]
	public Dictionary<string, JsonElement> Settings { get; set; } = new();
}

