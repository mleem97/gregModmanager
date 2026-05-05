using GregModmanager;

namespace GregModmanager.Services;

/// <summary>
/// Orchestrates the full Workshop sync pipeline: poll for subscription changes,
/// download new items, and sync them into the game's Mods folder.
/// </summary>
public sealed class WorkshopSyncOrchestrator : IDisposable
{
	private readonly SubscriptionPoller _poller;
	private readonly WorkshopDownloadService _downloader;
	private readonly ModsFolderSyncService _sync;
	private bool _running;

	public event Action<WorkshopSyncEvent>? StatusChanged;

	public WorkshopSyncOrchestrator(
		SubscriptionPoller poller,
		WorkshopDownloadService downloader,
		ModsFolderSyncService sync)
	{
		_poller = poller;
		_downloader = downloader;
		_sync = sync;

		_poller.NewSubscriptionsDetected += OnNewSubscriptions;
		_poller.UnsubscriptionsDetected += OnUnsubscriptions;
	}

	public void Start()
	{
		if (_running) return;
		_running = true;
		_poller.Start();
		StatusChanged?.Invoke(WorkshopSyncEvent.PollerStarted());
	}

	public void Stop()
	{
		_poller.Stop();
		_running = false;
		StatusChanged?.Invoke(WorkshopSyncEvent.PollerStopped());
	}

	private async void OnNewSubscriptions(IReadOnlyList<ulong> newIds)
	{
		var gameRoot = GetGameRoot();
		if (string.IsNullOrEmpty(gameRoot))
		{
			StatusChanged?.Invoke(WorkshopSyncEvent.Warning(
				"Game root not configured — skipping Workshop sync."));
			return;
		}

		StatusChanged?.Invoke(WorkshopSyncEvent.DownloadStarted(newIds.Count));

		var log = new Progress<string>(msg =>
			StatusChanged?.Invoke(WorkshopSyncEvent.Info(msg)));

		var results = await _downloader.DownloadItemsAsync(newIds, log).ConfigureAwait(false);

		var toSync = new List<(ulong Id, string LocalDir)>();
		for (var i = 0; i < results.Count; i++)
		{
			var r = results[i];
			if (r.Success && r.LocalDirectory is not null)
				toSync.Add((newIds[i], r.LocalDirectory));
		}

		if (toSync.Count > 0)
		{
			StatusChanged?.Invoke(WorkshopSyncEvent.SyncStarted(toSync.Count));
			_sync.SyncItems(toSync, gameRoot, log);
		}

		StatusChanged?.Invoke(WorkshopSyncEvent.Complete(
			toSync.Count, results.Count - toSync.Count));
	}

	private void OnUnsubscriptions(IReadOnlyList<ulong> removedIds)
	{
		var gameRoot = GetGameRoot();
		if (string.IsNullOrEmpty(gameRoot)) return;

		foreach (var id in removedIds)
		{
			_sync.RemoveItem(id, gameRoot);
		}

		StatusChanged?.Invoke(WorkshopSyncEvent.Removed(removedIds.Count));
	}

	private string? GetGameRoot()
	{
#if WINDOWS || ANDROID || IOS || MACCATALYST
		var gameRoot = SettingsPage.GetGameRootPath();
		if (!string.IsNullOrEmpty(gameRoot))
		{
			return gameRoot;
		}
#endif
		return Steam.SteamApiNativeLoader.GetGameRoot();
	}

	public void Dispose()
	{
		Stop();
		_poller.NewSubscriptionsDetected -= OnNewSubscriptions;
		_poller.UnsubscriptionsDetected -= OnUnsubscriptions;
	}
}

public readonly record struct WorkshopSyncEvent(string Kind, string Message)
{
	public static WorkshopSyncEvent PollerStarted() => new("started", "Workshop sync active.");
	public static WorkshopSyncEvent PollerStopped() => new("stopped", "Workshop sync stopped.");
	public static WorkshopSyncEvent DownloadStarted(int count) => new("download", $"Downloading {count} new item(s)…");
	public static WorkshopSyncEvent SyncStarted(int count) => new("sync", $"Syncing {count} item(s) to Mods…");
	public static WorkshopSyncEvent Complete(int ok, int failed) => new("complete", $"Sync complete: {ok} OK, {failed} failed.");
	public static WorkshopSyncEvent Removed(int count) => new("removed", $"Removed {count} unsubscribed item(s).");
	public static WorkshopSyncEvent Warning(string msg) => new("warning", msg);
	public static WorkshopSyncEvent Info(string msg) => new("info", msg);
}

