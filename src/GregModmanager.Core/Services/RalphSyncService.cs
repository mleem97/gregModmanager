using System.Text.Json;
using GregModmanager.Models;

namespace GregModmanager.Services;

public sealed class RalphSyncService
{
	public void WriteStatus(string projectRoot, string command, bool ok, string message)
	{
		var dir = Path.Combine(projectRoot, ".ralph", "tasks");
		Directory.CreateDirectory(dir);
		var path = Path.Combine(dir, "status.json");
		var payload = new RalphTaskStatus
		{
			LastCommand = command,
			Ok = ok,
			Message = message,
			TimestampUtc = DateTime.UtcNow,
		};
		File.WriteAllText(path, JsonSerializer.Serialize(payload, AppJsonContext.SharedOptions));
	}
}

