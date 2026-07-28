using System.Text.Json.Serialization;
using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using System.Collections.Generic;
using GregModmanager.Models.Auth;

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
[JsonSerializable(typeof(LoginRequest))]
[JsonSerializable(typeof(AuthResponse))]
[JsonSerializable(typeof(TokenExchangeRequest))]
[JsonSerializable(typeof(TokenExchangeResponse))]
[JsonSerializable(typeof(UserInfo))]
[JsonSerializable(typeof(DebugLogPayload))]
[JsonSerializable(typeof(string))]
[JsonSerializable(typeof(bool))]
[JsonSerializable(typeof(int))]
[JsonSerializable(typeof(RalphTaskStatus))]
[JsonSerializable(typeof(AssetModMetadata))]
[JsonSerializable(typeof(object))]
public partial class AppJsonContext : JsonSerializerContext
{
    /// <summary>
    /// Shared options with source-generated resolver. Use this for all JSON operations.
    /// </summary>
    public static JsonSerializerOptions SharedOptions { get; } = CreateSharedOptions();

    private static JsonSerializerOptions CreateSharedOptions()
    {
        var options = new JsonSerializerOptions
        {
            WriteIndented = true,
            PropertyNameCaseInsensitive = true,
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        };

        try
        {
            // Source generator sets TypeInfoResolver on Default.Options
            var defaultResolver = Default?.Options?.TypeInfoResolver;
            if (defaultResolver != null)
            {
                options.TypeInfoResolver = defaultResolver;
            }
        }
        catch
        {
            // Source generator did not produce output — fall back to reflection
        }

        return options;
    }
}
