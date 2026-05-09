using System.Text.Json.Serialization;

namespace GregModmanager.Models;

public class AssetModMetadata
{
    [JsonPropertyName("assetType")]
    public string AssetType { get; set; } = string.Empty;

    [JsonPropertyName("tags")]
    public string[] Tags { get; set; } = Array.Empty<string>();

    [JsonPropertyName("isStandalone")]
    public bool IsStandalone { get; set; }
}
