using System.Diagnostics;
using System.IO.Compression;
using System.Reflection;
using System.Text;

namespace GregModmanager.Services;

/// <summary>
/// Creates a support ZIP from local diagnostics. The archive can contain logs,
/// dumps, paths, machine/user names, and recent Windows application events;
/// callers must ask users to inspect it before sharing.
/// </summary>
public sealed class ReproBundleService
{
	private const string AppFolderName = "gregModmanager";

	/// <summary>Creates a timestamped archive in the current user's local application-data directory.</summary>
	public Task<string> CreateBundleAsync()
	{
		return CreateBundleAsync(CancellationToken.None);
	}

	/// <summary>Creates the archive and observes cancellation after local collection completes.</summary>
	public Task<string> CreateBundleAsync(CancellationToken cancellationToken)
	{
		var now = DateTime.Now;
		var root = Path.Combine(
			Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
			AppFolderName,
			"repro");
		Directory.CreateDirectory(root);

		var stamp = now.ToString("yyyyMMdd-HHmmss");
		var stagingDir = Path.Combine(root, $"bundle-{stamp}");
		var zipPath = Path.Combine(root, $"repro-bundle-{stamp}.zip");

		if (Directory.Exists(stagingDir))
		{
			Directory.Delete(stagingDir, true);
		}
		Directory.CreateDirectory(stagingDir);

		WriteSummary(stagingDir, now);
		CopyDiagnostics(stagingDir);
		WriteRecentEventLog(stagingDir);

		if (File.Exists(zipPath))
		{
			File.Delete(zipPath);
		}

		ZipFile.CreateFromDirectory(stagingDir, zipPath, CompressionLevel.Optimal, includeBaseDirectory: false);
		Directory.Delete(stagingDir, true);

		cancellationToken.ThrowIfCancellationRequested();
		return Task.FromResult(zipPath);
	}

	private static void WriteSummary(string stagingDir, DateTime createdAt)
	{
		var version = Assembly.GetExecutingAssembly().GetName().Version?.ToString() ?? "unknown";
		var summary = new StringBuilder();
		summary.AppendLine("gregModmanager - Repro Bundle");
		summary.AppendLine($"CreatedAt: {createdAt:O}");
		summary.AppendLine($"AppVersion: {version}");
		summary.AppendLine($"OSVersion: {Environment.OSVersion}");
		summary.AppendLine($"Framework: {System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription}");
		summary.AppendLine($"Architecture: {System.Runtime.InteropServices.RuntimeInformation.OSArchitecture}");
		summary.AppendLine($"ProcessArchitecture: {System.Runtime.InteropServices.RuntimeInformation.ProcessArchitecture}");
		summary.AppendLine($"BaseDirectory: {AppContext.BaseDirectory}");
		summary.AppendLine($"CurrentDirectory: {Environment.CurrentDirectory}");
		summary.AppendLine($"MachineName: {Environment.MachineName}");
		summary.AppendLine($"UserName: {Environment.UserName}");

		File.WriteAllText(Path.Combine(stagingDir, "environment.txt"), summary.ToString());
	}

	private static void CopyDiagnostics(string stagingDir)
	{
		var userLogs = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), AppFolderName, "logs");
		CopyDirectorySafe(userLogs, Path.Combine(stagingDir, "logs"));

		var userDumps = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), AppFolderName, "dumps");
		CopyDirectorySafe(userDumps, Path.Combine(stagingDir, "dumps-user"));

		var machineDumps = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), AppFolderName, "dumps");
		CopyDirectorySafe(machineDumps, Path.Combine(stagingDir, "dumps-machine"));
	}

	private static void CopyDirectorySafe(string sourceDir, string targetDir)
	{
		try
		{
			if (!Directory.Exists(sourceDir))
			{
				return;
			}

			Directory.CreateDirectory(targetDir);
			foreach (var file in Directory.GetFiles(sourceDir, "*", SearchOption.AllDirectories))
			{
				CopyDiagnosticFile(sourceDir, targetDir, file);
			}
		}
		catch
		{
			// ignore copy failures in diagnostics collection
		}
	}

	private static void CopyDiagnosticFile(string sourceDir, string targetDir, string file)
	{
		try
		{
			var relativePath = Path.GetRelativePath(sourceDir, file);
			var dest = Path.Combine(targetDir, relativePath);
			var destDir = Path.GetDirectoryName(dest);
			if (!string.IsNullOrWhiteSpace(destDir))
			{
				Directory.CreateDirectory(destDir);
			}
			File.Copy(file, dest, true);
		}
		catch
		{
			// skip unreadable files
		}
	}

	private static void WriteRecentEventLog(string stagingDir)
	{
		if (!OperatingSystem.IsWindows())
		{
			return;
		}

		var outputPath = Path.Combine(stagingDir, "eventlog.txt");
		const string psCommand = "$events = Get-WinEvent -FilterHashtable @{LogName='Application'; StartTime=(Get-Date).AddDays(-2)} | Where-Object { ($_.ProviderName -in @('Application Error','.NET Runtime','Windows Error Reporting')) -and $_.Message -match 'GregModmanager' } | Select-Object -First 200 TimeCreated,ProviderName,Id,LevelDisplayName,Message; if (-not $events) { 'No matching events found.' } else { $events | Format-List | Out-String -Width 240 }";

		try
		{
			using var proc = StartPowerShell(psCommand);
			if (proc is null)
			{
				File.WriteAllText(outputPath, "Could not start powershell.exe for event log export.");
				return;
			}

			var output = proc.StandardOutput.ReadToEnd();
			var error = proc.StandardError.ReadToEnd();
			proc.WaitForExit(7000);

			WriteEventLogOutput(outputPath, output, error);
		}
		catch (Exception ex)
		{
			File.WriteAllText(outputPath, $"Could not read Event Log: {ex.Message}");
		}
	}

	private static Process? StartPowerShell(string command)
	{
		var startInfo = new ProcessStartInfo
		{
			FileName = "powershell.exe",
			UseShellExecute = false,
			RedirectStandardOutput = true,
			RedirectStandardError = true,
			CreateNoWindow = true
		};
		startInfo.ArgumentList.Add("-NoProfile");
		startInfo.ArgumentList.Add("-ExecutionPolicy");
		startInfo.ArgumentList.Add("Bypass");
		startInfo.ArgumentList.Add("-Command");
		startInfo.ArgumentList.Add(command);

		return Process.Start(startInfo);
	}

	private static void WriteEventLogOutput(string outputPath, string output, string error)
	{
		var sb = new StringBuilder();
		sb.AppendLine("Recent Application log events for GregModmanager (last 48h)");
		sb.AppendLine();
		sb.AppendLine(output);

		if (!string.IsNullOrWhiteSpace(error))
		{
			sb.AppendLine("--- STDERR ---");
			sb.AppendLine(error);
		}

		File.WriteAllText(outputPath, sb.ToString());
	}
}
