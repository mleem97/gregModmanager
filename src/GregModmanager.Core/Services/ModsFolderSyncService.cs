using GregModmanager.Models;

namespace GregModmanager.Services;

/// <summary>
/// Synchronizes downloaded Workshop content from Steam's cache into the live game folders
/// using atomic copy.
/// </summary>
public sealed class ModsFolderSyncService
{
	public event Action<SyncProgressArgs>? SyncProgress;

	/// <summary>
	/// Sync a single downloaded Workshop item into the game's folder based on its <see cref="ModContentType"/>.
	/// </summary>
	public SyncResult SyncItem(ulong publishedFileId, string steamLocalDir, string gameRoot)
	{
		if (string.IsNullOrEmpty(gameRoot))
			return SyncResult.Fail("Game root path is not configured.");

		if (!Directory.Exists(steamLocalDir))
			return SyncResult.Fail($"Source directory does not exist: {steamLocalDir}");

		var modType = ReadModTypeFromDirectory(steamLocalDir);
		var destDir = ResolveDestinationPath(gameRoot, publishedFileId, modType);
		var tempDir = destDir + ".tmp";

		try
		{
			if (Directory.Exists(tempDir))
				Directory.Delete(tempDir, recursive: true);

			Directory.CreateDirectory(tempDir);
			CopyDirectoryRecursive(steamLocalDir, tempDir);

			if (Directory.Exists(destDir))
				Directory.Delete(destDir, recursive: true);

			Directory.Move(tempDir, destDir);

			SyncProgress?.Invoke(new SyncProgressArgs(publishedFileId, true, destDir));
			return SyncResult.Ok(destDir);
		}
		catch (Exception ex)
		{
			try { if (Directory.Exists(tempDir)) Directory.Delete(tempDir, recursive: true); }
			catch { /* cleanup best-effort */ }

			SyncProgress?.Invoke(new SyncProgressArgs(publishedFileId, false, null));
			return SyncResult.Fail($"Sync failed for {publishedFileId}: {ex.Message}");
		}
	}

	private static ModContentType ReadModTypeFromDirectory(string dir)
	{
		var path = Path.Combine(dir, "greg-modmanager.meta.json");
		if (!File.Exists(path))
			return ModContentType.PlacableObject;

		try
		{
			using var doc = System.Text.Json.JsonDocument.Parse(File.ReadAllText(path));
			if (doc.RootElement.TryGetProperty("modType", out var prop) &&
			    prop.ValueKind == System.Text.Json.JsonValueKind.String)
			{
				return ParseModType(prop.GetString());
			}
		}
		catch { /* ignore parse errors */ }

		return ModContentType.PlacableObject;
	}

	private static ModContentType ParseModType(string? value)
		=> value switch
		{
			"MelonloaderPlugin" or "MelonLoaderPlugin" => ModContentType.MelonloaderPlugin,
			"Userlib" => ModContentType.Userlib,
			"DataCenterMod" or "DataCenterMods" => ModContentType.DataCenterMod,
			_ => ModContentType.PlacableObject,
		};

	private static string ResolveDestinationPath(string gameRoot, ulong publishedFileId, ModContentType modType)
	{
		var id = publishedFileId.ToString();
		return modType switch
		{
			ModContentType.MelonloaderPlugin => Path.Combine(gameRoot, "Plugins", id),
			ModContentType.Userlib => Path.Combine(gameRoot, "Plugins", "Dependencies", id),
			ModContentType.DataCenterMod => Path.Combine(gameRoot, "Mods", id),
			_ => Path.Combine(gameRoot, "Data Center_Data", "StreamingAssets", "Mods", id),
		};
	}

	/// <summary>
	/// Sync multiple items downloaded via <see cref="WorkshopDownloadService"/>.
	/// </summary>
	public IReadOnlyList<SyncResult> SyncItems(
		IReadOnlyList<(ulong Id, string LocalDir)> items,
		string gameRoot,
		IProgress<string>? log = null)
	{
		var results = new List<SyncResult>(items.Count);
		foreach (var (id, localDir) in items)
		{
			log?.Report($"Syncing {id}…");
			var result = SyncItem(id, localDir, gameRoot);
			if (result.Success)
				log?.Report($"Synced {id} → {result.DestinationPath}");
			else
				log?.Report($"Failed {id}: {result.ErrorMessage}");
			results.Add(result);
		}

		return results;
	}

	/// <summary>
	/// Removes a Workshop item from all known installation directories.
	/// </summary>
	public bool RemoveItem(ulong publishedFileId, string gameRoot)
	{
		var id = publishedFileId.ToString();
		var candidates = new[]
		{
			Path.Combine(gameRoot, "Data Center_Data", "StreamingAssets", "Mods", id),
			Path.Combine(gameRoot, "Mods", "Workshop", id),
			Path.Combine(gameRoot, "Mods", id),
			Path.Combine(gameRoot, "Plugins", id),
			Path.Combine(gameRoot, "Plugins", "Dependencies", id),
			Path.Combine(gameRoot, "Userlibs", id),
		};

		bool anyRemoved = false;
		foreach (var destDir in candidates)
		{
			if (!Directory.Exists(destDir)) continue;
			try
			{
				Directory.Delete(destDir, recursive: true);
				anyRemoved = true;
			}
			catch { /* best-effort */ }
		}

		return anyRemoved;
	}

	private static void CopyDirectoryRecursive(string sourceDir, string destDir)
	{
		foreach (var dir in Directory.GetDirectories(sourceDir, "*", SearchOption.AllDirectories))
		{
			var relative = Path.GetRelativePath(sourceDir, dir);
			Directory.CreateDirectory(Path.Combine(destDir, relative));
		}

		foreach (var file in Directory.GetFiles(sourceDir, "*", SearchOption.AllDirectories))
		{
			var relative = Path.GetRelativePath(sourceDir, file);
			File.Copy(file, Path.Combine(destDir, relative), overwrite: true);
		}
	}
}

public readonly record struct SyncResult(bool Success, string? DestinationPath, string? ErrorMessage)
{
	public static SyncResult Ok(string path) => new(true, path, null);
	public static SyncResult Fail(string error) => new(false, null, error);
}

public readonly record struct SyncProgressArgs(ulong PublishedFileId, bool Success, string? DestinationPath);

