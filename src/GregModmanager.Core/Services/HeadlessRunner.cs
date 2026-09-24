namespace GregModmanager.Services;

using GregModmanager.Models;

/// <summary>
/// Handles the supported non-interactive Steam Workshop publish command before
/// Avalonia starts. A project must contain both <c>content/</c> and readable
/// <c>metadata.json</c>; Steam authentication is still required.
/// </summary>
public static class HeadlessRunner
{
	/// <summary>
	/// Recognizes help and publish arguments and returns their process exit code.
	/// Returns <see langword="false"/> when the caller should continue with normal UI startup.
	/// </summary>
	public static bool TryHandle(IReadOnlyList<string> args, out int exitCode)
	{
		exitCode = 0;
		if (args.Any(a => string.Equals(a, "--help", StringComparison.OrdinalIgnoreCase) || string.Equals(a, "-h", StringComparison.OrdinalIgnoreCase)))
		{
			var executableName = Path.GetFileName(Environment.ProcessPath) ?? "GregModmanager";
			Console.WriteLine($$"""
				gregCoreMF Workshop Uploader (headless)

				  --mode publish       Publish a local workshop project
				  --upload             Same as --mode publish
				  --path <dir>         Project root (must contain content/ and metadata.json)
				  --changelog <text>   Change note attached to the Steam update
				  --autocommit         Write .ralph/tasks/status.json on completion
				  --mode status        Show sync state of one project (needs --path).
				                       Exit code: 0 = synced, 1 = publish needed
				                       (modified/unpublished/unknown), 2 = error.
				  --mode list          List all workspace projects with sync state.
				                       No Steam connection needed for status/list.
				  --verbose            Mirror the file log to stdout (also works for the GUI)
				  --console-log        Same as --verbose
				  --log-file <path>    Write the log file to <path> instead of the default

				Live log location (GUI and headless):
				  logs/app-YYYYMMDD.log next to the binary (or LocalAppData fallback)

				Example (path is usually <game>/workshop/<project>):
				  {{executableName}} --mode publish --path "<project-path>" --changelog "Fixed X" --autocommit
				Example (publish only when changed):
				  {{executableName}} --mode status --path "<project-path>" || {{executableName}} --mode publish --path "<project-path>" --changelog "Update"
				""");
			exitCode = 0;
			return true;
		}

		if (!IsPublishInvocation(args))
		{
			if (IsStatusInvocation(args))
			{
				var statusPath = GetArgValue(args, "--path");
				if (string.IsNullOrWhiteSpace(statusPath))
				{
					Console.Error.WriteLine("Missing --path <dir>.");
					exitCode = 2;
					return true;
				}

				exitCode = RunStatus(Path.GetFullPath(statusPath.Trim().Trim('"')));
				return true;
			}

			if (IsListInvocation(args))
			{
				exitCode = RunList();
				return true;
			}

			return false;
		}

		var path = GetArgValue(args, "--path");
		if (string.IsNullOrWhiteSpace(path))
		{
			Console.Error.WriteLine("Missing --path <dir>.");
			exitCode = 2;
			return true;
		}

		path = Path.GetFullPath(path.Trim().Trim('"'));
		var autocommit = args.Any(a => string.Equals(a, "--autocommit", StringComparison.OrdinalIgnoreCase));
		var changeLog = GetArgValue(args, "--changelog") ?? GetArgValue(args, "--change-note");

		exitCode = RunPublishAsync(path, autocommit, changeLog).GetAwaiter().GetResult();
		return true;
	}

	private static bool IsPublishInvocation(IReadOnlyList<string> args)
	{
		if (args.Any(a => string.Equals(a, "--upload", StringComparison.OrdinalIgnoreCase)))
		{
			return true;
		}

		var mode = GetArgValue(args, "--mode");
		return string.Equals(mode, "publish", StringComparison.OrdinalIgnoreCase);
	}

	private static bool IsStatusInvocation(IReadOnlyList<string> args)
	{
		var mode = GetArgValue(args, "--mode");
		return string.Equals(mode, "status", StringComparison.OrdinalIgnoreCase);
	}

	private static bool IsListInvocation(IReadOnlyList<string> args)
	{
		var mode = GetArgValue(args, "--mode");
		return string.Equals(mode, "list", StringComparison.OrdinalIgnoreCase);
	}

	private static string? GetArgValue(IReadOnlyList<string> args, string name)
	{
		for (var i = 0; i < args.Count - 1; i++)
		{
			if (string.Equals(args[i], name, StringComparison.OrdinalIgnoreCase))
			{
				return args[i + 1];
			}
		}

		return null;
	}

	/// <summary>
	/// Prints the sync state of one project. Exit codes: 0 = synced (nothing
	/// to do), 1 = publish needed (modified/unpublished/unknown), 2 = error.
	/// Needs no Steam connection.
	/// </summary>
	private static int RunStatus(string projectRoot)
	{
		var workspace = new WorkspaceService();
		if (!Directory.Exists(projectRoot))
		{
			Console.Error.WriteLine($"Project path not found: {projectRoot}");
			return 2;
		}

		var meta = workspace.LoadMetadata(projectRoot);
		var state = workspace.GetSyncState(projectRoot);
		Console.WriteLine($"project: {Path.GetFileName(projectRoot.TrimEnd(Path.DirectorySeparatorChar))}");
		Console.WriteLine($"path: {projectRoot}");
		Console.WriteLine($"workshop-id: {(meta.PublishedFileId == 0 ? "none" : meta.PublishedFileId.ToString())}");
		Console.WriteLine($"sync-state: {state.ToString().ToUpperInvariant()}");
		Console.WriteLine($"last-published-hash: {(string.IsNullOrEmpty(meta.LastPublishedHash) ? "none" : meta.LastPublishedHash[..Math.Min(16, meta.LastPublishedHash.Length)] + "…")}");

		return state == WorkspaceService.ProjectSyncState.Synced ? 0 : 1;
	}

	/// <summary>
	/// Lists all workspace projects with their sync state. Needs no Steam
	/// connection. Always exits 0 unless the workspace itself is unreadable.
	/// </summary>
	private static int RunList()
	{
		var workspace = new WorkspaceService();
		IReadOnlyList<WorkshopProject> projects;
		try
		{
			projects = workspace.ScanProjects();
		}
		catch (Exception ex)
		{
			Console.Error.WriteLine($"Workspace unreadable ({workspace.WorkspaceRoot}): {ex.Message}");
			return 2;
		}

		Console.WriteLine($"workspace: {workspace.WorkspaceRoot}");
		Console.WriteLine($"projects: {projects.Count}");
		foreach (var project in projects)
		{
			WorkspaceService.ProjectSyncState state;
			ulong id;
			try
			{
				state = workspace.GetSyncState(project.RootPath);
				id = workspace.LoadMetadata(project.RootPath).PublishedFileId;
			}
			catch (Exception ex)
			{
				Console.WriteLine($"{project.Name} | id=? | ERROR: {ex.Message}");
				continue;
			}

			Console.WriteLine($"{project.Name} | id={(id == 0 ? "none" : id.ToString())} | {state.ToString().ToUpperInvariant()}");
		}

		return 0;
	}

	/// <summary>
	/// Publishes one local project and returns 0 on success, 1 for project or
	/// Steam failures. When requested, writes automation status beside the project.
	/// </summary>
	private static async Task<int> RunPublishAsync(string projectRoot, bool autocommit, string? changeLog)
	{
		var workspace = new WorkspaceService();
		var steam = new SteamWorkshopService();
		var ralph = new RalphSyncService();

		try
		{
			if (!Directory.Exists(projectRoot))
			{
				Console.Error.WriteLine($"Project path not found: {projectRoot}");
				if (autocommit)
				{
					ralph.WriteStatus(projectRoot, "publish", false, "Project path not found.");
				}

				return 1;
			}

			var content = Path.Combine(projectRoot, "content");
			if (!Directory.Exists(content))
			{
				Console.Error.WriteLine($"Missing content folder: {content}");
				if (autocommit)
				{
					ralph.WriteStatus(projectRoot, "publish", false, "Missing content folder.");
				}

				return 1;
			}

			var metadata = workspace.LoadMetadata(projectRoot);
			// Progress<T> posts async — mirror to the file log too so fast exits
			// can't lose trailing lines (Environment.Exit doesn't flush the queue).
			var progress = new Progress<string>(s => { try { Console.WriteLine(s); } catch { } AppFileLog.Info(s); });
			var upload = new Progress<float>(p => { try { Console.WriteLine($"Upload {p:P0}"); } catch { } });

			var outcome = await steam.PublishAsync(
				projectRoot,
				metadata,
				content,
				changeLog,
				upload,
				progress,
				CancellationToken.None).ConfigureAwait(false);

			if (!outcome.Success)
			{
				Console.Error.WriteLine(outcome.Message);
				AppFileLog.Error($"Headless publish failed: {outcome.Message}");
				if (autocommit)
				{
					ralph.WriteStatus(projectRoot, "publish", false, outcome.Message);
				}

				// Let queued Progress<T> callbacks flush before the process exits.
				try { await Task.Delay(500).ConfigureAwait(false); } catch { /* shutting down */ }
				return 1;
			}

			WorkspaceService.SaveMetadata(projectRoot, metadata);
			Console.WriteLine($"Published. Workshop file id: {outcome.PublishedFileId}");

			if (autocommit)
			{
				ralph.WriteStatus(projectRoot, "publish", true, $"Published file id {outcome.PublishedFileId}");
			}

			// Let queued Progress<T> callbacks flush before the process exits.
			try { await Task.Delay(500).ConfigureAwait(false); } catch { /* shutting down */ }
			return 0;
		}
		finally
		{
			steam.Shutdown();
		}
	}
}

