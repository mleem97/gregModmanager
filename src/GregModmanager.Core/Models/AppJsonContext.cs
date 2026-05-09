using System.Text.Json.Serialization;
using System.Text.Json;
using System.Collections.Generic;

namespace GregModmanager.Models;

/// <summary>
/// Central registry for JSON Source Generation.
/// All models that require serialization in a trimmed/AOT environment MUST be registered here.
/// </summary>
[JsonSourceGenerationOptions(
    WriteIndented = true, 
    PropertyNameCaseInsensitive = true, 
    DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(WorkshopMetadata))]
[JsonSerializable(typeof(CrashReport))]
[JsonSerializable(typeof(NativeModConfig))]
[JsonSerializable(typeof(ModOptionsConfigFile))]
[JsonSerializable(typeof(ModCollectionDefinition))]
[JsonSerializable(typeof(ModCollectionEntry))]
[JsonSerializable(typeof(CollectionCatalog))]
[JsonSerializable(typeof(List<PluginPackageInfo>))]
[JsonSerializable(typeof(Dictionary<string, JsonElement>))]
[JsonSerializable(typeof(ModStoreMarker))]
[JsonSerializable(typeof(LokiPushRequest))]
[JsonSerializable(typeof(SyncCollectionEvent))]
public partial class AppJsonContext : JsonSerializerContext
{
}
