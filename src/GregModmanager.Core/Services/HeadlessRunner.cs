namespace GregModmanager.Services;

public static class HeadlessRunner
{
	public static bool TryHandle(IReadOnlyList<string> args, out int exitCode)
	{
		exitCode = 0;
		if (args.Any(a => string.Equals(a, "--help", StringComparison.OrdinalIgnoreCase) || string.Equals(a, "-h", StringComparison.OrdinalIgnoreCase)))
		{
			Console.WriteLine("""
				gregCoreMF Workshop Uploader (headless)

				  --mode publish       Publish a local workshop project
				  --upload             Same as --mode publish
				  --path <dir>         Project root (must contain content/ and metadata.json)
				  --autocommit         Write .ralph/tasks/status.json on completion

				Example (path is usually <game>/workshop/<project>):
				  GregModmanager.exe --mode publish --path "D:\Steam\steamapps\common\Data Center\workshop\MyMod" --autocommit
				""");
			exitCode = 0;
			return true;
		}

		if (!IsPublishInvocation(args))
		{
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

		exitCode = RunPublishAsync(path, autocommit).GetAwaiter().GetResult();
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

	private static async Task<int> RunPublishAsync(string projectRoot, bool autocommit)
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
			var progress = new Progress<string>(Console.WriteLine);
			var upload = new Progress<float>(p => Console.WriteLine($"Upload {p:P0}"));

			var outcome = await steam.PublishAsync(
				projectRoot,
				metadata,
				content,
				null,
				upload,
				progress,
				CancellationToken.None).ConfigureAwait(false);

			if (!outcome.Success)
			{
				Console.Error.WriteLine(outcome.Message);
				if (autocommit)
				{
					ralph.WriteStatus(projectRoot, "publish", false, outcome.Message);
				}

				return 1;
			}

			workspace.SaveMetadata(projectRoot, metadata);
			Console.WriteLine($"Published. Workshop file id: {outcome.PublishedFileId}");

			if (autocommit)
			{
				ralph.WriteStatus(projectRoot, "publish", true, $"Published file id {outcome.PublishedFileId}");
			}

			return 0;
		}
		finally
		{
			steam.Shutdown();
		}
	}
}

