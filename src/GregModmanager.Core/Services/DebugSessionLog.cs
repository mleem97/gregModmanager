using System.Text.Json;

namespace GregModmanager;

/// <summary>NDJSON debug log (Debug builds only; stripped from Release for size and I/O).</summary>
internal static class DebugSessionLog
{
#if DEBUG
	private const string SessionId = "9fc458";
	private static readonly string LogPath = ResolveLogPath();
	private static readonly object Gate = new();

	internal static string LogFilePath => LogPath;

	private static string ResolveLogPath()
	{
		try
		{
			var dir = new DirectoryInfo(AppContext.BaseDirectory);
			for (var i = 0; i < 14 && dir != null; i++, dir = dir.Parent)
			{
				var csproj = Path.Combine(dir.FullName, "GregModmanager", "GregModmanager.csproj");
				if (File.Exists(csproj))
					return Path.Combine(dir.FullName, "debug-9fc458.log");
			}
		}
		catch
		{
			// ignored
		}

		return Path.Combine(Path.GetTempPath(), "debug-9fc458.log");
	}

	public static void Write(string hypothesisId, string location, string message, object? data = null)
	{
		try
		{
			var line = JsonSerializer.Serialize(new
			{
				sessionId = SessionId,
				hypothesisId,
				location,
				message,
				timestamp = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(),
				runId = Environment.GetEnvironmentVariable("WORKSHOP_UPLOADER_DEBUG_RUN") ?? "pre-fix",
				data,
			});
			lock (Gate)
			{
				File.AppendAllText(LogPath, line + Environment.NewLine);
			}
		}
		catch
		{
			// never break startup
		}
	}
#else
	internal static string LogFilePath => string.Empty;

	public static void Write(string hypothesisId, string location, string message, object? data = null) { }
#endif
}

