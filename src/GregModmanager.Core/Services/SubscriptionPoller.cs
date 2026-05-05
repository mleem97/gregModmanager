using Steamworks;
using Steamworks.Ugc;
using GregModmanager.Steam;

namespace GregModmanager.Services;

/// <summary>
/// Periodically polls Steam for subscription changes and emits diffs
/// (newly subscribed / unsubscribed item IDs).
/// </summary>
public sealed class SubscriptionPoller : IDisposable
{
	private readonly SteamWorkshopService _steam;
	private readonly TimeSpan _interval;
	private HashSet<ulong> _knownIds = new();
	private CancellationTokenSource? _cts;
	private Task? _pollLoop;

	public event Action<IReadOnlyList<ulong>>? NewSubscriptionsDetected;
	public event Action<IReadOnlyList<ulong>>? UnsubscriptionsDetected;

	public SubscriptionPoller(SteamWorkshopService steam, TimeSpan? interval = null)
	{
		_steam = steam;
		_interval = interval ?? TimeSpan.FromSeconds(30);
	}

	public void Start()
	{
		if (_pollLoop is not null) return;
		_cts = new CancellationTokenSource();
		_pollLoop = PollLoopAsync(_cts.Token);
	}

	public void Stop()
	{
		_cts?.Cancel();
		_pollLoop = null;
	}

	private async Task PollLoopAsync(CancellationToken ct)
	{
		while (!ct.IsCancellationRequested)
		{
			try
			{
				var currentIds = await FetchSubscribedIdsAsync(ct).ConfigureAwait(false);
				if (currentIds is not null)
				{
					var added = currentIds.Except(_knownIds).ToList();
					var removed = _knownIds.Except(currentIds).ToList();

					if (added.Count > 0)
						NewSubscriptionsDetected?.Invoke(added);
					if (removed.Count > 0)
						UnsubscriptionsDetected?.Invoke(removed);

					_knownIds = currentIds;
				}
			}
			catch (OperationCanceledException) { break; }
			catch
			{
				// Swallow transient failures; retry next cycle
			}

			try { await Task.Delay(_interval, ct).ConfigureAwait(false); }
			catch (OperationCanceledException) { break; }
		}
	}

	private async Task<HashSet<ulong>?> FetchSubscribedIdsAsync(CancellationToken ct)
	{
		if (!_steam.EnsureInitialized(null)) return null;

		ct.ThrowIfCancellationRequested();

		var result = await Query.Items
			.WhereUserSubscribed()
			.GetPageAsync(1)
			.ConfigureAwait(false);

		if (!result.HasValue) return null;

		using var page = result.Value;
		var ids = new HashSet<ulong>();
		foreach (Item item in page.Entries)
		{
			ids.Add(item.Id.Value);
		}

		return ids;
	}

	public void Dispose()
	{
		Stop();
		_cts?.Dispose();
	}
}

