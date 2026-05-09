namespace GregModmanager.Models;

/// <summary>
/// Telemetry event payload for mod collection synchronization.
/// </summary>
public class SyncCollectionEvent
{
    public Guid collectionId { get; set; }
    public string? collectionName { get; set; }
    public bool success { get; set; }
    public int itemCount { get; set; }
    public long durationMs { get; set; }
}
