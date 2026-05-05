using System.Text.Json;

namespace GregModmanager.Services;

public sealed class RalphSyncService
{
	private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };

	public void WriteStatus(string projectRoot, string command, bool ok, string message)
	{
		var dir = Path.Combine(projectRoot, ".ralph", "tasks");
		Directory.CreateDirectory(dir);
		var path = Path.Combine(dir, "status.json");
		var payload = new
		{
			lastCommand = command,
			ok,
			message,
			timestampUtc = DateTime.UtcNow,
		};
		File.WriteAllText(path, JsonSerializer.Serialize(payload, JsonOptions));
	}
}

