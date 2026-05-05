using Steamworks;
using Steamworks.Data;
using Steamworks.Ugc;

namespace GregModmanager.Services;

/// <summary>
/// Downloads Steam Workshop items by their published file ID and reports progress.
/// Wraps the Facepunch <see cref="Item.DownloadAsync"/> API.
/// </summary>
public sealed class WorkshopDownloadService
{
	private readonly SteamWorkshopService _steam;

	public WorkshopDownloadService(SteamWorkshopService steam)
	{
		_steam = steam;
	}

	/// <summary>
	/// Downloads a single Workshop item. Returns the local install directory on success, null on failure.
	/// </summary>
	public async Task<DownloadResult> DownloadItemAsync(
		ulong publishedFileId,
		IProgress<float>? progress = null,
		IProgress<string>? log = null,
		CancellationToken ct = default)
	{
		if (!_steam.EnsureInitialized(log))
			return DownloadResult.Fail("Steam is not available.");

		ct.ThrowIfCancellationRequested();

		var item = await Item.GetAsync((PublishedFileId)publishedFileId).ConfigureAwait(false);
		if (!item.HasValue)
			return DownloadResult.Fail($"Item {publishedFileId} not found on Steam.");

		var ugc = item.Value;
		log?.Report($"Downloading {ugc.Title ?? publishedFileId.ToString()}…");

		try
		{
			await ugc.DownloadAsync(
				p => progress?.Report(p),
				60,
				ct).ConfigureAwait(false);
		}
		catch (Exception ex)
		{
			return DownloadResult.Fail($"Download failed: {ex.Message}");
		}

		var dir = ugc.Directory;
		if (string.IsNullOrEmpty(dir) || !Directory.Exists(dir))
			return DownloadResult.Fail("Steam did not provide a local folder after download.");

		log?.Report($"Downloaded to {dir}");
		return DownloadResult.Ok(dir, ugc.Title ?? publishedFileId.ToString());
	}

	/// <summary>
	/// Downloads multiple items sequentially, reporting aggregate progress.
	/// </summary>
	public async Task<IReadOnlyList<DownloadResult>> DownloadItemsAsync(
		IReadOnlyList<ulong> ids,
		IProgress<string>? log = null,
		CancellationToken ct = default)
	{
		var results = new List<DownloadResult>(ids.Count);
		for (var i = 0; i < ids.Count; i++)
		{
			ct.ThrowIfCancellationRequested();
			var itemProgress = new Progress<float>(p =>
				log?.Report($"[{i + 1}/{ids.Count}] {p:P0}"));
			var result = await DownloadItemAsync(ids[i], itemProgress, log, ct).ConfigureAwait(false);
			results.Add(result);
		}

		return results;
	}
}

public readonly record struct DownloadResult(bool Success, string? LocalDirectory, string Title, string? ErrorMessage)
{
	public static DownloadResult Ok(string directory, string title) => new(true, directory, title, null);
	public static DownloadResult Fail(string error) => new(false, null, "", error);
}

