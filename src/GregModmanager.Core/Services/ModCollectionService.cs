using System.Text.Json;
using GregModmanager.Models;

namespace GregModmanager.Services;

public sealed class ModCollectionService
{
	private readonly object _gate = new();
	private readonly JsonSerializerOptions _jsonOptions = AppJsonContext.SharedOptions;
	private readonly TelemetryService _telemetry;
	private readonly string _storagePath;
	private CollectionCatalog _catalog;

	public ModCollectionService(TelemetryService telemetry)
	{
		_telemetry = telemetry;
		var root = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "GregModmanager");
		Directory.CreateDirectory(root);
		_storagePath = Path.Combine(root, "collections.json");
		_catalog = LoadCatalog();
	}

	public IReadOnlyList<ModCollectionDefinition> GetCollections()
	{
		lock (_gate)
		{
			return _catalog.Collections
				.OrderByDescending(x => x.UpdatedUtc)
				.ToList();
		}
	}

	public ModCollectionDefinition? GetCollection(Guid id)
	{
		lock (_gate)
		{
			return _catalog.Collections.FirstOrDefault(x => x.Id == id);
		}
	}

	public ModCollectionDefinition EnsureCollection(string name, string? description = null, ModCollectionSourceKind sourceKind = ModCollectionSourceKind.Local, string? sourceName = null)
	{
		lock (_gate)
		{
			var existing = _catalog.Collections.FirstOrDefault(x => string.Equals(x.Name, name, StringComparison.OrdinalIgnoreCase));
			if (existing is not null)
			{
				if (!string.IsNullOrWhiteSpace(description)) existing.Description = description;
				existing.SourceKind = sourceKind;
				existing.SourceName = sourceName ?? existing.SourceName;
				existing.UpdatedUtc = DateTimeOffset.UtcNow;
				SaveCatalog();
				return existing;
			}

			var created = new ModCollectionDefinition
			{
				Name = name.Trim(),
				Description = description?.Trim() ?? string.Empty,
				SourceKind = sourceKind,
				SourceName = sourceName ?? sourceKind.ToString(),
			};
			_catalog.Collections.Add(created);
			SaveCatalog();
			return created;
		}
	}

	public ModCollectionDefinition EnsureCollectionForItem(string collectionName, ulong publishedFileId, string title, string sourceName, string modType = "PlacableObject")
	{
		var collection = EnsureCollection(collectionName, sourceName: sourceName);
		AddItem(collection.Id, publishedFileId, title, sourceName, modType);
		return collection;
	}

	public bool AddItem(Guid collectionId, ulong publishedFileId, string title, string sourceName, string modType = "PlacableObject", IEnumerable<ulong>? workshopDependencyIds = null, string? notes = null)
	{
		lock (_gate)
		{
			var collection = _catalog.Collections.FirstOrDefault(x => x.Id == collectionId);
			if (collection is null) return false;
			if (collection.Items.Any(x => x.PublishedFileId == publishedFileId)) return true;

			collection.Items.Add(new ModCollectionEntry
			{
				PublishedFileId = publishedFileId,
				Title = title.Trim(),
				SourceName = sourceName.Trim(),
				ModType = string.IsNullOrWhiteSpace(modType) ? "PlacableObject" : modType.Trim(),
				WorkshopDependencyIds = workshopDependencyIds?.Distinct().ToList() ?? new List<ulong>(),
				Notes = notes,
			});
			collection.UpdatedUtc = DateTimeOffset.UtcNow;
			SaveCatalog();
			return true;
		}
	}

	public bool RemoveItem(Guid collectionId, ulong publishedFileId)
	{
		lock (_gate)
		{
			var collection = _catalog.Collections.FirstOrDefault(x => x.Id == collectionId);
			if (collection is null) return false;
			var removed = collection.Items.RemoveAll(x => x.PublishedFileId == publishedFileId) > 0;
			if (removed)
			{
				collection.UpdatedUtc = DateTimeOffset.UtcNow;
				SaveCatalog();
			}
			return removed;
		}
	}

	public bool RenameCollection(Guid id, string newName)
	{
		lock (_gate)
		{
			var collection = _catalog.Collections.FirstOrDefault(x => x.Id == id);
			if (collection is null) return false;
			collection.Name = newName.Trim();
			collection.UpdatedUtc = DateTimeOffset.UtcNow;
			SaveCatalog();
			return true;
		}
	}

	public bool DeleteCollection(Guid id)
	{
		lock (_gate)
		{
			var removed = _catalog.Collections.RemoveAll(x => x.Id == id) > 0;
			if (removed) SaveCatalog();
			return removed;
		}
	}

	public async Task<bool> SyncCollectionAsync(
		Guid collectionId,
		WorkshopDownloadService downloader,
		ModsFolderSyncService sync,
		string gameRoot,
		IProgress<string>? log = null,
		CancellationToken ct = default)
	{
		ModCollectionDefinition? collection;
		lock (_gate)
		{
			collection = _catalog.Collections.FirstOrDefault(x => x.Id == collectionId);
		}

		if (collection is null)
		{
			log?.Report("Collection not found.");
			return false;
		}

		if (string.IsNullOrWhiteSpace(gameRoot) || !Directory.Exists(gameRoot))
		{
			log?.Report("Game root not configured.");
			return false;
		}

		var seen = new HashSet<ulong>();
		var queue = new Queue<ModCollectionEntry>(collection.Items);
		var anySucceeded = false;

		var sw = System.Diagnostics.Stopwatch.StartNew();
		while (queue.Count > 0)
		{
			ct.ThrowIfCancellationRequested();
			var entry = queue.Dequeue();
			if (!seen.Add(entry.PublishedFileId))
			{
				continue;
			}

			log?.Report($"Downloading {entry.Title} ({entry.PublishedFileId})…");
			var download = await downloader.DownloadItemAsync(entry.PublishedFileId, null, log, ct).ConfigureAwait(false);
			if (!download.Success || string.IsNullOrWhiteSpace(download.LocalDirectory))
			{
				log?.Report(download.ErrorMessage ?? $"Download failed for {entry.PublishedFileId}.");
				continue;
			}

			var synced = sync.SyncItem(entry.PublishedFileId, download.LocalDirectory, gameRoot);
			if (synced.Success)
			{
				anySucceeded = true;
				log?.Report($"Synced {entry.PublishedFileId} → {synced.DestinationPath}");
			}
			else
			{
				log?.Report(synced.ErrorMessage ?? $"Sync failed for {entry.PublishedFileId}.");
			}

			foreach (var dependencyId in entry.WorkshopDependencyIds)
			{
				if (!seen.Contains(dependencyId))
				{
					queue.Enqueue(new ModCollectionEntry
					{
						PublishedFileId = dependencyId,
						Title = $"Dependency {dependencyId}",
						SourceName = entry.SourceName,
						ModType = "Userlib",
					});
				}
			}
		}
		sw.Stop();

		_ = _telemetry.TrackEventAsync("sync_collection", new SyncCollectionEvent
		{
			collectionId = collectionId,
			collectionName = collection.Name,
			success = anySucceeded,
			itemCount = seen.Count,
			durationMs = sw.ElapsedMilliseconds
		}, new Dictionary<string, string>
		{
			{ "outcome", anySucceeded ? "success" : "failed" }
		});

		return anySucceeded;
	}

	private CollectionCatalog LoadCatalog()
	{
		try
		{
			if (!File.Exists(_storagePath))
			{
				return new CollectionCatalog();
			}

			var json = File.ReadAllText(_storagePath);
			return JsonSerializer.Deserialize(json, AppJsonContext.Default.CollectionCatalog) ?? new CollectionCatalog();
		}
		catch
		{
			return new CollectionCatalog();
		}
	}

	private void SaveCatalog()
	{
		File.WriteAllText(_storagePath, JsonSerializer.Serialize(_catalog, AppJsonContext.Default.CollectionCatalog));
	}
}