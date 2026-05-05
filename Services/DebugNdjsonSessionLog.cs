using System.Text.Json;
using System.Text.Json.Nodes;

namespace GregModmanager;

/// <summary>Session 9fc458: NDJSON to %TEMP%\debug-9fc458.log (writable from Program Files). Remove after root cause confirmed.</summary>
internal static class DebugNdjsonSessionLog
{
	private const string SessionId = "9fc458";

	internal static string LogPath => Path.Combine(Path.GetTempPath(), "debug-9fc458.log");

	#region agent log
	internal static void Write(string hypothesisId, string location, string message, object? data = null)
	{
		try
		{
			var root = new JsonObject
			{
				["sessionId"] = SessionId,
				["hypothesisId"] = hypothesisId,
				["location"] = location,
				["message"] = message,
				["timestamp"] = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(),
				["runId"] = Environment.GetEnvironmentVariable("WORKSHOP_UPLOADER_DEBUG_RUN") ?? "repro-1",
			};
			if (data != null)
			{
				root["data"] = JsonSerializer.SerializeToNode(data, data.GetType());
			}

			File.AppendAllText(LogPath, root.ToJsonString(new JsonSerializerOptions { WriteIndented = false }) + Environment.NewLine);
		}
		catch
		{
			// never break startup
		}
	}
	#endregion
}

