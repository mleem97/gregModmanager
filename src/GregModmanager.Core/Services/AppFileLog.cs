using GregModmanager.Models;

namespace GregModmanager.Services;

public static class AppFileLog
{
	private static readonly object Gate = new();
	private static string? _logPath;
	private static int _sessionStarted;

	public static string LogPath
	{
		get
		{
			if (!string.IsNullOrEmpty(_logPath))
			{
				return _logPath;
			}

			try
			{
				// Try installation folder first (requested by user)
				var baseDir = AppContext.BaseDirectory;
				if (!string.IsNullOrEmpty(baseDir) && IsDirectoryWritable(baseDir))
				{
					var root = Path.Combine(baseDir, "logs");
					Directory.CreateDirectory(root);
					_logPath = Path.Combine(root, $"app-{DateTime.Now:yyyyMMdd}.log");
					return _logPath;
				}

				// Fallback to LocalApplicationData
				var appData = Path.Combine(
					Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
					"gregModmanager",
					"logs");
				Directory.CreateDirectory(appData);
				_logPath = Path.Combine(appData, $"app-{DateTime.Now:yyyyMMdd}.log");
			}
			catch
			{
				_logPath = Path.Combine(Path.GetTempPath(), $"gregModmanager-app-{DateTime.Now:yyyyMMdd}.log");
			}

			return _logPath;
		}
	}

	private static bool IsDirectoryWritable(string directoryPath)
	{
		try
		{
			var testFile = Path.Combine(directoryPath, $".write-test-{Environment.ProcessId}.tmp");
			File.WriteAllText(testFile, "ok");
			File.Delete(testFile);
			return true;
		}
		catch
		{
			return false;
		}
	}

	private static string SessionMarkerPath
	{
		get
		{
			var logDir = Path.GetDirectoryName(LogPath) ?? Path.GetTempPath();
			return Path.Combine(logDir, "session-active.marker");
		}
	}

	public static string ReportsDir
	{
		get
		{
			var root = Path.GetDirectoryName(LogPath) ?? Path.GetTempPath();
			var reports = Path.Combine(root, "reports");
			Directory.CreateDirectory(reports);
			return reports;
		}
	}

	public static void Info(string message) => Write("INFO", message, null);

	public static void Warn(string message) => Write("WARN", message, null);

	public static void Error(string message, Exception? ex = null) => Write("ERROR", message, ex);

	public static void StartSession()
	{
		if (Interlocked.Exchange(ref _sessionStarted, 1) == 1)
		{
			return;
		}

		try
		{
			if (File.Exists(SessionMarkerPath))
			{
				var marker = File.ReadAllText(SessionMarkerPath);
				Warn($"Previous run ended unexpectedly. Marker: {marker}");
			}

			var payload = $"pid={Environment.ProcessId};utc={DateTime.UtcNow:O}";
			File.WriteAllText(SessionMarkerPath, payload);
			Info("Session started.");
		}
		catch
		{
			// logging must never crash the app
		}
	}

	public static void EndSession()
	{
		try
		{
			Info("Session ended cleanly.");
			if (File.Exists(SessionMarkerPath))
			{
				File.Delete(SessionMarkerPath);
			}
		}
		catch
		{
			// logging must never crash the app
		}
	}

	public static void MarkCrash(string source, Exception? ex = null)
	{
		try
		{
			Error($"Crash marker from {source}", ex);
			var payload = $"pid={Environment.ProcessId};utc={DateTime.UtcNow:O};source={source};message={ex?.Message}";
			File.WriteAllText(SessionMarkerPath, payload);

			var report = new GregModmanager.Models.CrashReport
			{
				Timestamp = DateTime.UtcNow,
				AppVersion = GetAppVersion(),
				OsVersion = Environment.OSVersion.ToString(),
				ExceptionType = ex?.GetType().FullName,
				Message = ex?.Message,
				StackTrace = ex?.StackTrace,
				Source = source,
				ProcessId = Environment.ProcessId
			};

			SaveCrashReport(report);
		}
		catch
		{
			// logging must never crash the app
		}
	}

	private static string GetAppVersion()
	{
		try
		{
			return System.Reflection.Assembly.GetExecutingAssembly().GetName().Version?.ToString() ?? "0.0.0.0";
		}
		catch
		{
			return "unknown";
		}
	}

	public static void SaveCrashReport(GregModmanager.Models.CrashReport report)
	{
		try
		{
			var path = Path.Combine(ReportsDir, $"crash-{DateTime.UtcNow:yyyyMMdd-HHmmss}.json");
			var json = System.Text.Json.JsonSerializer.Serialize(report, AppJsonContext.Default.CrashReport);
			File.WriteAllText(path, json);
		}
		catch
		{
			// logging must never crash the app
		}
	}

	private static void Write(string level, string message, Exception? ex)
	{
		try
		{
			var line = $"{DateTime.Now:yyyy-MM-dd HH:mm:ss.fff} [{level}] [pid:{Environment.ProcessId}] {message}";
			if (ex is not null)
			{
				line += $" | {ex.GetType().FullName}: {ex.Message}";
			}

			lock (Gate)
			{
				File.AppendAllText(LogPath, line + Environment.NewLine);
				if (ex is not null && !string.IsNullOrWhiteSpace(ex.StackTrace))
				{
					File.AppendAllText(LogPath, ex.StackTrace + Environment.NewLine);
				}
			}
		}
		catch
		{
			// logging must never crash the app
		}
	}
}

