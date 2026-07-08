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

	public ModCollectionDefinition EnsureCollection(string name)
	{
		return EnsureCollection(name, description: null, ModCollectionSourceKind.Local, sourceName: null);
	}

	public ModCollectionDefinition EnsureCollection(string name, string? description)
	{
		return EnsureCollection(name, description, ModCollectionSourceKind.Local, sourceName: null);
	}

	public ModCollectionDefinition EnsureCollection(string name, string? description, ModCollectionSourceKind sourceKind)
	{
		return EnsureCollection(name, description, sourceKind, sourceName: null);
	}

	public ModCollectionDefinition EnsureCollection(string name, string? description, ModCollectionSourceKind sourceKind, string? sourceName)
	{
		lock (_gate)
		{
			var existing = _catalog.Collections.FirstOrDefault(x => string.Equals(x.Name, name, StringComparison.OrdinalIgnoreCase));
			if (existing is not null)
			{
				UpdateExistingCollection(existing, description, sourceKind, sourceName);
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

	public ModCollectionDefinition EnsureCollectionForItem(string collectionName, ulong publishedFileId, string title, string sourceName)
	{
		return EnsureCollectionForItem(collectionName, publishedFileId, title, sourceName, "PlacableObject");
	}

	public ModCollectionDefinition EnsureCollectionForItem(string collectionName, ulong publishedFileId, string title, string sourceName, string modType)
	{
		var collection = EnsureCollection(collectionName, description: null, ModCollectionSourceKind.Local, sourceName);
		AddItem(collection.Id, publishedFileId, title, sourceName, modType);
		return collection;
	}

	public bool AddItem(Guid collectionId, ulong publishedFileId, string title, string sourceName)
	{
		return AddItem(collectionId, publishedFileId, title, sourceName, "PlacableObject", workshopDependencyIds: null, notes: null);
	}

	public bool AddItem(Guid collectionId, ulong publishedFileId, string title, string sourceName, string modType)
	{
		return AddItem(collectionId, publishedFileId, title, sourceName, modType, workshopDependencyIds: null, notes: null);
	}

	public bool AddItem(Guid collectionId, ulong publishedFileId, string title, string sourceName, string modType, IEnumerable<ulong>? workshopDependencyIds)
	{
		return AddItem(collectionId, publishedFileId, title, sourceName, modType, workshopDependencyIds, notes: null);
	}

	public bool AddItem(Guid collectionId, ulong publishedFileId, string title, string sourceName, string modType, IEnumerable<ulong>? workshopDependencyIds, string? notes)
	{
		lock (_gate)
		{
			var collection = _catalog.Collections.FirstOrDefault(x => x.Id == collectionId);
			if (collection is null) return false;
			if (collection.Items.Any(x => x.PublishedFileId == publishedFileId)) return true;

			collection.Items.Add(CreateEntry(publishedFileId, title, sourceName, modType, workshopDependencyIds, notes));
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

	public Task<bool> SyncCollectionAsync(Guid collectionId, WorkshopDownloadService downloader, ModsFolderSyncService sync, string gameRoot)
	{
		return SyncCollectionAsync(collectionId, downloader, sync, gameRoot, log: null, CancellationToken.None);
	}

	public Task<bool> SyncCollectionAsync(Guid collectionId, WorkshopDownloadService downloader, ModsFolderSyncService sync, string gameRoot, IProgress<string>? log)
	{
		return SyncCollectionAsync(collectionId, downloader, sync, gameRoot, log, CancellationToken.None);
	}

	public async Task<bool> SyncCollectionAsync(
		Guid collectionId,
		WorkshopDownloadService downloader,
		ModsFolderSyncService sync,
		string gameRoot,
		IProgress<string>? log,
		CancellationToken ct)
	{
		var collection = GetCollectionForSync(collectionId, log);
		if (collection is null || !IsGameRootConfigured(gameRoot, log))
		{
			return false;
		}

		var context = new CollectionSyncContext(collection, downloader, sync, gameRoot, log, ct);
		var result = await ProcessCollectionQueueAsync(context).ConfigureAwait(false);
		TrackSyncResult(collectionId, collection, result);
		return result.AnySucceeded;
	}

	private void UpdateExistingCollection(ModCollectionDefinition existing, string? description, ModCollectionSourceKind sourceKind, string? sourceName)
	{
		if (!string.IsNullOrWhiteSpace(description)) existing.Description = description;
		existing.SourceKind = sourceKind;
		existing.SourceName = sourceName ?? existing.SourceName;
		existing.UpdatedUtc = DateTimeOffset.UtcNow;
		SaveCatalog();
	}

	private static ModCollectionEntry CreateEntry(ulong publishedFileId, string title, string sourceName, string modType, IEnumerable<ulong>? workshopDependencyIds, string? notes)
	{
		return new ModCollectionEntry
		{
			PublishedFileId = publishedFileId,
			Title = title.Trim(),
			SourceName = sourceName.Trim(),
			ModType = string.IsNullOrWhiteSpace(modType) ? "PlacableObject" : modType.Trim(),
			WorkshopDependencyIds = workshopDependencyIds?.Distinct().ToList() ?? new List<ulong>(),
			Notes = notes,
		};
	}

	private ModCollectionDefinition? GetCollectionForSync(Guid collectionId, IProgress<string>? log)
	{
		lock (_gate)
		{
			var collection = _catalog.Collections.FirstOrDefault(x => x.Id == collectionId);
			if (collection is not null) return collection;
		}

		log?.Report("Collection not found.");
		return null;
	}

	private static bool IsGameRootConfigured(string gameRoot, IProgress<string>? log)
	{
		if (!string.IsNullOrWhiteSpace(gameRoot) && Directory.Exists(gameRoot)) return true;

		log?.Report("Game root not configured.");
		return false;
	}

	private static async Task<CollectionSyncResult> ProcessCollectionQueueAsync(CollectionSyncContext context)
	{
		var seen = new HashSet<ulong>();
		var queue = new Queue<ModCollectionEntry>(context.Collection.Items);
		var sw = System.Diagnostics.Stopwatch.StartNew();
		var anySucceeded = false;

		while (queue.Count > 0)
		{
			context.CancellationToken.ThrowIfCancellationRequested();
			anySucceeded |= await ProcessCollectionEntryAsync(context, queue, seen).ConfigureAwait(false);
		}

		sw.Stop();
		return new CollectionSyncResult(anySucceeded, seen.Count, sw.ElapsedMilliseconds);
	}

	private static async Task<bool> ProcessCollectionEntryAsync(CollectionSyncContext context, Queue<ModCollectionEntry> queue, HashSet<ulong> seen)
	{
		var entry = queue.Dequeue();
		if (!seen.Add(entry.PublishedFileId)) return false;

		context.Log?.Report($"Downloading {entry.Title} ({entry.PublishedFileId})…");
		var download = await context.Downloader.DownloadItemAsync(entry.PublishedFileId, progress: null, context.Log, context.CancellationToken).ConfigureAwait(false);
		if (!download.Success || string.IsNullOrWhiteSpace(download.LocalDirectory))
		{
			context.Log?.Report(download.ErrorMessage ?? $"Download failed for {entry.PublishedFileId}.");
			return false;
		}

		var succeeded = SyncDownloadedEntry(context, entry, download.LocalDirectory);
		EnqueueDependencies(queue, seen, entry);
		return succeeded;
	}

	private static bool SyncDownloadedEntry(CollectionSyncContext context, ModCollectionEntry entry, string localDirectory)
	{
		var synced = context.Sync.SyncItem(entry.PublishedFileId, localDirectory, context.GameRoot);
		if (synced.Success)
		{
			context.Log?.Report($"Synced {entry.PublishedFileId} → {synced.DestinationPath}");
			return true;
		}

		context.Log?.Report(synced.ErrorMessage ?? $"Sync failed for {entry.PublishedFileId}.");
		return false;
	}

	private static void EnqueueDependencies(Queue<ModCollectionEntry> queue, HashSet<ulong> seen, ModCollectionEntry entry)
	{
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

	private void TrackSyncResult(Guid collectionId, ModCollectionDefinition collection, CollectionSyncResult result)
	{
		_ = _telemetry.TrackEventAsync("sync_collection", new SyncCollectionEvent
		{
			collectionId = collectionId,
			collectionName = collection.Name,
			success = result.AnySucceeded,
			itemCount = result.SeenCount,
			durationMs = result.ElapsedMilliseconds
		}, new Dictionary<string, string>
		{
			{ "outcome", result.AnySucceeded ? "success" : "failed" }
		});
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

	private sealed record CollectionSyncContext(
		ModCollectionDefinition Collection,
		WorkshopDownloadService Downloader,
		ModsFolderSyncService Sync,
		string GameRoot,
		IProgress<string>? Log,
		CancellationToken CancellationToken);

	private readonly record struct CollectionSyncResult(bool AnySucceeded, int SeenCount, long ElapsedMilliseconds);
}
