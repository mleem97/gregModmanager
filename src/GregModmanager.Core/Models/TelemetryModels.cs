using System.Text.Json.Serialization;

namespace GregModmanager.Models;

/// <summary>
/// Root request payload for the Loki Push API.
/// </summary>
public class LokiPushRequest
{
    [JsonPropertyName("streams")]
    public List<LokiStream> Streams { get; set; } = new();
}

/// <summary>
/// A single log stream in a Loki push request.
/// </summary>
public class LokiStream
{
    [JsonPropertyName("stream")]
    public Dictionary<string, string> Stream { get; set; } = new();

    /// <summary>
    /// Values are [ "nanoseconds", "line" ]
    /// </summary>
    [JsonPropertyName("values")]
    public List<List<string>> Values { get; set; } = new();
}
