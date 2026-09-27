using System.Text.Json.Serialization;

namespace GregModmanager.Models;

public class RalphTaskStatus
{
    [JsonPropertyName("lastCommand")]
    public string LastCommand { get; set; } = string.Empty;

    [JsonPropertyName("ok")]
    public bool Ok { get; set; }

    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;

    [JsonPropertyName("timestampUtc")]
    public DateTime TimestampUtc { get; set; }
}
