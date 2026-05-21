using System.Diagnostics;
using Steamworks;
using Steamworks.Data;
using Steamworks.Ugc;
using GregModmanager.Models;
using GregModmanager.Steam;

namespace GregModmanager.Services;

public sealed class SteamWorkshopService
{
	private static readonly object InitLock = new();

	private bool _initialized;

	/// <summary>Last reason the connection banner is red (init exception, not logged on, etc.). Empty when unknown.</summary>
	public string LastSteamConnectionHint { get; private set; } = string.Empty;

	public bool EnsureInitialized(IProgress<string>? log)
	{
		lock (InitLock)
		{
			if (_initialized)
			{
				return true;
			}

			try
			{
				var preloaded = SteamApiNativeLoader.TryPreload();
				if (!preloaded)
				{
					var paths = string.Join("; ", SteamApiNativeLoader.GetAttemptedPaths());
					LastSteamConnectionHint = $"steam_api64.dll nicht gefunden. Gesucht: {paths}";
					log?.Report(LastSteamConnectionHint);
					return false;
				}

				SteamClient.Init(SteamConstants.DataCenterAppId, true);
				_initialized = true;
				LastSteamConnectionHint = string.Empty;
				log?.Report("Steam API initialized.");
				return true;
			}
			catch (Exception ex)
			{
				LastSteamConnectionHint = ex.Message;
				log?.Report($"Steam init failed: {ex.Message}");
				return false;
			}
		}
	}

	/// <summary>True when Steam API is up and the user is logged on (see <see cref="SteamClient.IsLoggedOn"/>).</summary>
	public bool TryGetSteamReady(out string? userName)
	{
		userName = null;
		lock (InitLock)
		{
			if (!_initialized && !EnsureInitialized(null))
			{
				return false;
			}

			if (!SteamClient.IsValid)
			{
				LastSteamConnectionHint = "Steam API ungültig (IsValid=false).";
				return false;
			}

			if (!SteamClient.IsLoggedOn)
			{
				LastSteamConnectionHint = "Steam-Client: nicht eingeloggt oder kein Benutzer aktiv.";
				return false;
			}

			LastSteamConnectionHint = string.Empty;
			userName = SteamClient.Name;
			return true;
		}
	}

	public void Shutdown()
	{
		lock (InitLock)
		{
			if (!_initialized)
			{
				return;
			}

			try
			{
				SteamClient.Shutdown();
			}
			catch
			{
				// ignored
			}

			_initialized = false;
			LastSteamConnectionHint = string.Empty;
		}
	}

	#region Publish / Update

	public async Task<PublishOutcome> PublishAsync(
		string projectRoot,
		WorkshopMetadata metadata,
		string contentFolder,
		string? changeLog,
		IProgress<float>? uploadProgress,
		IProgress<string>? log,
		CancellationToken cancellationToken)
	{
		if (!EnsureInitialized(log))
		{
			return PublishOutcome.Fail("Steam is not available (is Steam running?)");
		}

		cancellationToken.ThrowIfCancellationRequested();

		if (!Directory.Exists(contentFolder))
		{
			return PublishOutcome.Fail($"Content folder not found: {contentFolder}");
		}

		var title = (metadata.Title ?? string.Empty).Trim();
		var description = (metadata.Description ?? string.Empty).Trim();
		if (string.IsNullOrEmpty(title))
		{
			return PublishOutcome.Fail("Title is required.");
		}

		if (title.Length > SteamConstants.MaxTitleLength || description.Length > SteamConstants.MaxDescriptionLength)
		{
			return PublishOutcome.Fail("Title or description exceeds Steam limits.");
		}

		var previewPath = string.IsNullOrWhiteSpace(metadata.PreviewImageRelativePath)
			? null
			: Path.Combine(projectRoot, metadata.PreviewImageRelativePath);

		Steamworks.Ugc.Editor editor = metadata.PublishedFileId == 0
			? Steamworks.Ugc.Editor.NewCommunityFile
			: new Steamworks.Ugc.Editor((PublishedFileId)metadata.PublishedFileId);

		// Embed mod-type marker so subscribers know where to install this item.
		try
		{
			var markerPath = Path.Combine(contentFolder, "greg-modmanager.meta.json");
			var markerJson = System.Text.Json.JsonSerializer.Serialize(new ModStoreMarker { modType = metadata.ModType }, AppJsonContext.Default.ModStoreMarker);
			await File.WriteAllTextAsync(markerPath, markerJson, cancellationToken);
		}
		catch
		{
			// non-critical
		}

		editor = editor
			.WithTitle(title)
			.WithDescription(description)
			.WithContent(contentFolder);

		if (!string.IsNullOrEmpty(previewPath) && File.Exists(previewPath))
		{
			editor = editor.WithPreviewFile(previewPath);
		}

		editor = metadata.Tags
			.Where(t => !string.IsNullOrWhiteSpace(t))
			.Aggregate(editor, (current, tag) => current.WithTag(tag.Trim()));

		if (!string.IsNullOrWhiteSpace(changeLog))
		{
			editor = editor.WithChangeLog(changeLog);
		}

		if (!SteamPublishRateLimiter.Shared.TryAcquire(out var retryAfter))
		{
			var seconds = Math.Max(1, (int)Math.Ceiling(retryAfter.TotalSeconds));
			return PublishOutcome.Fail($"Steam publish cooldown active. Please wait {seconds}s before trying again.");
		}

		editor = ApplyVisibility(editor, metadata.Visibility);

		log?.Report(metadata.PublishedFileId == 0
			? "Creating new workshop item..."
			: $"Updating workshop item {metadata.PublishedFileId}...");

		Steamworks.Ugc.PublishResult result;
		try
		{
			result = await editor.SubmitAsync(uploadProgress).ConfigureAwait(false);
		}
		catch (Exception ex)
		{
			return PublishOutcome.Fail(ex.Message);
		}

		if (!result.Success)
		{
			return PublishOutcome.Fail($"Steam publish failed: {result.Result}");
		}

		if (result.NeedsWorkshopAgreement)
		{
			return PublishOutcome.Fail("Workshop legal agreement must be accepted in the Steam client.");
		}

		var id = result.FileId.Value;
		if (id != 0)
		{
			metadata.PublishedFileId = id;
		}

		return PublishOutcome.Ok(metadata.PublishedFileId);
	}

	/// <summary>After a successful publish, re-download the item from Steam and replace local <c>content/</c> so the folder is in sync with Steam's version (like a git pull after push). Also syncs the preview image and additional gallery screenshots.</summary>
	public async Task<bool> SyncAfterPublishAsync(
		ulong publishedFileId,
		string projectRoot,
		WorkshopMetadata metadata,
		WorkspaceService workspace,
		IProgress<string>? log,
		CancellationToken ct)
	{
		if (!EnsureInitialized(log))
		{
			return false;
		}

		log?.Report("Syncing with Steam after publish...");

		var item = await Item.GetAsync((PublishedFileId)publishedFileId).ConfigureAwait(false);
		if (!item.HasValue)
		{
			log?.Report("Sync: could not load item from Steam.");
			return false;
		}

		var ugc = item.Value;
		await ugc.DownloadAsync(null, 60, ct).ConfigureAwait(false);

		var steamDir = ugc.Directory;
		if (string.IsNullOrEmpty(steamDir) || !Directory.Exists(steamDir))
		{
			log?.Report("Sync: Steam did not provide a local folder.");
			return false;
		}

		var contentDir = Path.Combine(projectRoot, "content");

		if (Directory.Exists(contentDir))
		{
			Directory.Delete(contentDir, recursive: true);
		}

		Directory.CreateDirectory(contentDir);
		WorkspaceService.CopyDirectoryRecursive(steamDir, contentDir);
		log?.Report("Sync complete: local content/ now matches Steam.");

		await SyncPreviewImagesAsync(publishedFileId, ugc.PreviewImageUrl, projectRoot, metadata, log, ct).ConfigureAwait(false);
		WorkspaceService.SaveMetadata(projectRoot, metadata);

		return true;
	}

	#endregion

	#region Browse / Search / Query

	public async Task<WorkshopBrowseResultVm> BrowseAsync(
		int page,
		WorkshopSortMode sort,
		string? tag,
		CancellationToken ct)
	{
		if (!EnsureInitialized(null))
		{
			return EmptyBrowseResult(page);
		}

		ct.ThrowIfCancellationRequested();

		var query = ApplySort(Query.Items, sort);

		if (!string.IsNullOrWhiteSpace(tag))
		{
			query = query.WithTag(tag);
		}

		query = query.WithLongDescription(true);

		return await ExecuteQueryAsync(query, page, ct).ConfigureAwait(false);
	}

	public async Task<WorkshopBrowseResultVm> SearchAsync(
		string searchText,
		int page,
		CancellationToken ct)
	{
		if (!EnsureInitialized(null))
		{
			return EmptyBrowseResult(page);
		}

		ct.ThrowIfCancellationRequested();

		var query = Query.Items
			.WhereSearchText(searchText)
			.RankedByTextSearch()
			.WithLongDescription(true);

		return await ExecuteQueryAsync(query, page, ct).ConfigureAwait(false);
	}

	public async Task<WorkshopBrowseResultVm> ListSubscribedAsync(int page, CancellationToken ct)
	{
		if (!EnsureInitialized(null))
		{
			return EmptyBrowseResult(page);
		}

		ct.ThrowIfCancellationRequested();

		var query = Query.Items
			.WhereUserSubscribed()
			.SortByUpdateDate()
			.WithLongDescription(true);

		return await ExecuteQueryAsync(query, page, ct).ConfigureAwait(false);
	}

	public async Task<WorkshopBrowseResultVm> ListFavoritedAsync(int page, CancellationToken ct)
	{
		if (!EnsureInitialized(null))
		{
			return EmptyBrowseResult(page);
		}

		ct.ThrowIfCancellationRequested();

		var query = Query.Items
			.WhereUserFavorited()
			.SortByUpdateDate()
			.WithLongDescription(true);

		return await ExecuteQueryAsync(query, page, ct).ConfigureAwait(false);
	}

	public async Task<WorkshopBrowseResultVm> ListMyPublishedPagedAsync(int page, CancellationToken ct)
	{
		if (!EnsureInitialized(null))
		{
			return EmptyBrowseResult(page);
		}

		ct.ThrowIfCancellationRequested();

		var query = Query.Items
			.WhereUserPublished()
			.SortByUpdateDate()
			.WithLongDescription(true);

		return await ExecuteQueryAsync(query, page, ct).ConfigureAwait(false);
	}

	/// <summary>Workshop items you published (first page only, legacy compat).</summary>
	public async Task<IReadOnlyList<PublishedWorkshopItemVm>> ListMyPublishedAsync(
		CancellationToken cancellationToken = default)
	{
		if (!EnsureInitialized(null))
		{
			return Array.Empty<PublishedWorkshopItemVm>();
		}

		cancellationToken.ThrowIfCancellationRequested();

		var pageResult = await Query.Items
			.WhereUserPublished()
			.SortByUpdateDate()
			.GetPageAsync(1)
			.ConfigureAwait(false);

		if (!pageResult.HasValue)
		{
			return Array.Empty<PublishedWorkshopItemVm>();
		}

		using var page = pageResult.Value;
		var list = new List<PublishedWorkshopItemVm>();
		list.AddRange(page.Entries.Select(item => new PublishedWorkshopItemVm
		{
			PublishedFileId = item.Id.Value,
			Title = string.IsNullOrWhiteSpace(item.Title) ? $"Item {item.Id.Value}" : item.Title,
			Updated = item.Updated,
		}));

		return list;
	}

	#endregion

	#region Item Detail

	public async Task<WorkshopItemDetailVm?> GetItemDetailsAsync(ulong publishedFileId, CancellationToken ct)
	{
		if (!EnsureInitialized(null))
		{
			return null;
		}

		ct.ThrowIfCancellationRequested();

		var item = await Item.GetAsync((PublishedFileId)publishedFileId).ConfigureAwait(false);
		if (!item.HasValue)
		{
			return null;
		}

		var vm = MapItemToVm(item.Value);

		if (SteamUgcPreviews.CanQueryPreviews)
		{
			var galleryUrls = await SteamUgcPreviews.QueryAdditionalPreviewUrlsAsync(publishedFileId).ConfigureAwait(false);
			vm.AdditionalPreviewUrls = galleryUrls.ToArray();
		}

		return vm;
	}

	/// <summary>
	/// Copies title, description, visibility, and tags from a live Steam Workshop item into
	/// <paramref name="target"/>. Preserves app-local fields from <paramref name="localSnapshot"/>
	/// (<see cref="WorkshopMetadata.Needsgreg"/>, preview path, additional screenshot paths).
	/// </summary>
	public static void ApplySteamWorkshopToMetadata(
		WorkshopItemDetailVm steam,
		WorkshopMetadata target,
		WorkshopMetadata localSnapshot,
		int maxTags = 20)
	{
		target.PublishedFileId = steam.PublishedFileId;
		target.Title = steam.Title ?? "";
		target.Description = steam.Description ?? "";
		target.Visibility = steam.Visibility is "Private" or "FriendsOnly" or "Public"
			? steam.Visibility
			: "Public";
		target.Tags = steam.Tags
			.Where(t => !string.IsNullOrWhiteSpace(t))
			.Select(t => t.Trim())
			.Take(maxTags)
			.ToList();

		target.Needsgreg = localSnapshot.Needsgreg;
		target.NeedsMelonLoader = localSnapshot.NeedsMelonLoader;
		target.NativeConfigProfile = string.IsNullOrWhiteSpace(localSnapshot.NativeConfigProfile)
			? "decoration"
			: localSnapshot.NativeConfigProfile;
		target.ModType = string.IsNullOrWhiteSpace(localSnapshot.ModType)
			? "PlacableObject"
			: localSnapshot.ModType;
		target.PreviewImageRelativePath = localSnapshot.PreviewImageRelativePath ?? "preview.png";
		target.AdditionalPreviews = new List<string>(localSnapshot.AdditionalPreviews);
		target.WorkshopDependencyIds = new List<ulong>(localSnapshot.WorkshopDependencyIds ?? new List<ulong>());
	}

	#endregion

	#region User Actions (Subscribe, Favorite, Vote)

	public async Task<bool> SubscribeAsync(ulong fileId)
	{
		if (!EnsureInitialized(null))
		{
			return false;
		}

		try
		{
			var item = new Item((PublishedFileId)fileId);
			await item.Subscribe();
			return true;
		}
		catch
		{
			return false;
		}
	}

	public async Task<bool> UnsubscribeAsync(ulong fileId)
	{
		if (!EnsureInitialized(null))
		{
			return false;
		}

		try
		{
			var item = new Item((PublishedFileId)fileId);
			await item.Unsubscribe();
			return true;
		}
		catch
		{
			return false;
		}
	}

	public async Task<bool> AddFavoriteAsync(ulong fileId)
	{
		if (!EnsureInitialized(null))
		{
			return false;
		}

		try
		{
			var item = new Item((PublishedFileId)fileId);
			await item.AddFavorite();
			return true;
		}
		catch
		{
			return false;
		}
	}

	public async Task<bool> RemoveFavoriteAsync(ulong fileId)
	{
		if (!EnsureInitialized(null))
		{
			return false;
		}

		try
		{
			var item = new Item((PublishedFileId)fileId);
			await item.RemoveFavorite();
			return true;
		}
		catch
		{
			return false;
		}
	}

	public async Task<bool> VoteAsync(ulong fileId, bool up)
	{
		if (!EnsureInitialized(null))
		{
			return false;
		}

		try
		{
			var item = new Item((PublishedFileId)fileId);
			var result = await item.Vote(up);
			return result == Result.OK;
		}
		catch
		{
			return false;
		}
	}

	#endregion

	#region Steam Page / Overlay

	public void OpenItemInBrowser(ulong fileId)
	{
		var url = $"https://steamcommunity.com/sharedfiles/filedetails/?id={fileId}";
		_ = SafeProcess.OpenUrlAsync(url);
	}

	#endregion

	#region Import

	/// <summary>Download a published item from Steam and copy into <c>&lt;game&gt;/workshop/&lt;folder&gt;/content</c> for editing.</summary>
	public async Task<ImportOutcome> ImportPublishedToWorkspaceAsync(
		ulong publishedFileId,
		string? folderName,
		WorkspaceService workspace,
		IProgress<string>? log,
		IProgress<float>? progress,
		CancellationToken cancellationToken = default)
	{
		if (!EnsureInitialized(log))
		{
			return ImportOutcome.Fail("Steam is not available (is Steam running?)");
		}

		workspace.EnsureWorkspaceStructure();

		var item = await Item.GetAsync((PublishedFileId)publishedFileId).ConfigureAwait(false);
		if (!item.HasValue)
		{
			return ImportOutcome.Fail("Could not load workshop item from Steam.");
		}

		var ugc = item.Value;
		log?.Report($"Downloading workshop file {publishedFileId}...");

		await ugc.DownloadAsync(
				p => progress?.Report(p),
				60,
				cancellationToken)
			.ConfigureAwait(false);

		var srcDir = ugc.Directory;
		if (string.IsNullOrEmpty(srcDir) || !Directory.Exists(srcDir))
		{
			return ImportOutcome.Fail("Steam did not provide a local folder after download. Try again in a few seconds.");
		}

		var dirName = string.IsNullOrWhiteSpace(folderName)
			? DefaultImportFolderName(ugc.Title, publishedFileId)
			: WorkspaceService.SanitizeFolderName(folderName);

		if (string.IsNullOrEmpty(dirName))
		{
			return ImportOutcome.Fail("Folder name is invalid.");
		}

		var destRoot = Path.Combine(workspace.WorkspaceRoot, dirName);
		if (Directory.Exists(destRoot))
		{
			return ImportOutcome.Fail($"A folder already exists: {destRoot}");
		}

		Directory.CreateDirectory(destRoot);
		var contentDir = Path.Combine(destRoot, "content");
		Directory.CreateDirectory(contentDir);

		try
		{
			WorkspaceService.CopyDirectoryRecursive(srcDir, contentDir);
		}
		catch (Exception ex)
		{
			try
			{
				Directory.Delete(destRoot, recursive: true);
			}
			catch
			{
				// ignored
			}

			return ImportOutcome.Fail($"Copy failed: {ex.Message}");
		}

		var previewRelPath = "preview.png";
		try
		{
			var previewUrl = ugc.PreviewImageUrl;
			if (!string.IsNullOrEmpty(previewUrl))
			{
				using var http = new HttpClient();
				var bytes = await http.GetByteArrayAsync(previewUrl).ConfigureAwait(false);
				var ext = DetectImageExtension(previewUrl, bytes);
				previewRelPath = $"preview{ext}";
				await File.WriteAllBytesAsync(Path.Combine(destRoot, previewRelPath), bytes).ConfigureAwait(false);
				log?.Report($"Downloaded preview image as {previewRelPath}");
			}
		}
		catch
		{
			// non-critical — proceed without preview
		}

		var meta = new WorkshopMetadata
		{
			PublishedFileId = publishedFileId,
			Title = string.IsNullOrWhiteSpace(ugc.Title) ? dirName : ugc.Title.Trim(),
			Description = ugc.Description ?? string.Empty,
			PreviewImageRelativePath = previewRelPath,
			Visibility = ugc.IsPrivate ? "Private" : ugc.IsFriendsOnly ? "FriendsOnly" : "Public",
		};

		if (ugc.Tags is { Length: > 0 })
		{
			meta.Tags.AddRange(ugc.Tags);
		}

		if (SteamUgcPreviews.CanQueryPreviews)
		{
			await DownloadGalleryImagesAsync(publishedFileId, destRoot, meta, log, cancellationToken).ConfigureAwait(false);
		}

		WorkspaceService.SaveMetadata(destRoot, meta);
		log?.Report($"Imported to {destRoot}");
		return ImportOutcome.Ok(destRoot);
	}

	#endregion

	#region Private Helpers

	/// <summary>Downloads the main preview image and additional gallery screenshots from Steam and updates <paramref name="metadata"/>.</summary>
	private static async Task SyncPreviewImagesAsync(
		ulong publishedFileId,
		string? previewImageUrl,
		string projectRoot,
		WorkshopMetadata metadata,
		IProgress<string>? log,
		CancellationToken ct)
	{
		using var http = new HttpClient();

		if (!string.IsNullOrEmpty(previewImageUrl))
		{
			try
			{
				ct.ThrowIfCancellationRequested();
				var bytes = await http.GetByteArrayAsync(previewImageUrl, ct).ConfigureAwait(false);
				var ext = DetectImageExtension(previewImageUrl, bytes);
				var previewName = $"preview{ext}";
				await File.WriteAllBytesAsync(Path.Combine(projectRoot, previewName), bytes, ct).ConfigureAwait(false);
				metadata.PreviewImageRelativePath = previewName;
				log?.Report($"Synced preview image: {previewName}");
			}
			catch (Exception ex) when (ex is not OperationCanceledException)
			{
				log?.Report($"Could not sync preview image: {ex.Message}");
			}
		}

		if (SteamUgcPreviews.CanQueryPreviews)
		{
			await DownloadGalleryImagesAsync(publishedFileId, projectRoot, metadata, log, ct).ConfigureAwait(false);
		}
	}

	/// <summary>Queries additional preview URLs from Steam via reflection and downloads them to the screenshots/ folder.</summary>
	private static async Task DownloadGalleryImagesAsync(
		ulong publishedFileId,
		string projectRoot,
		WorkshopMetadata metadata,
		IProgress<string>? log,
		CancellationToken ct)
	{
		List<string> urls;
		try
		{
			urls = await SteamUgcPreviews.QueryAdditionalPreviewUrlsAsync(publishedFileId).ConfigureAwait(false);
		}
		catch
		{
			return;
		}

		if (urls.Count == 0) return;

		var screenshotsDir = Path.Combine(projectRoot, "screenshots");
		Directory.CreateDirectory(screenshotsDir);
		metadata.AdditionalPreviews.Clear();

		using var http = new HttpClient();
		var index = 0;
		foreach (var url in urls)
		{
			ct.ThrowIfCancellationRequested();

			try
			{
				var bytes = await http.GetByteArrayAsync(url, ct).ConfigureAwait(false);
				var ext = DetectImageExtension(url, bytes);
				var fileName = $"screenshot_{++index}{ext}";
				await File.WriteAllBytesAsync(Path.Combine(screenshotsDir, fileName), bytes, ct).ConfigureAwait(false);
				metadata.AdditionalPreviews.Add(Path.Combine("screenshots", fileName));
				log?.Report($"Synced gallery image: {fileName}");
			}
			catch (Exception ex) when (ex is not OperationCanceledException)
			{
				log?.Report($"Could not download gallery image: {ex.Message}");
			}
		}

		if (index > 0)
			log?.Report($"Synced {index} gallery image(s) from Steam.");
	}

	private static Steamworks.Ugc.Editor ApplyVisibility(Steamworks.Ugc.Editor editor, string visibility)
	{
		return visibility switch
		{
			"Private" => editor.WithPrivateVisibility(),
			"FriendsOnly" => editor.WithFriendsOnlyVisibility(),
			_ => editor.WithPublicVisibility(),
		};
	}

	private static Query ApplySort(Query query, WorkshopSortMode sort)
	{
		return sort switch
		{
			WorkshopSortMode.CreationDate => query.SortByCreationDate(),
			WorkshopSortMode.VoteScore => query.SortByVoteScore(),
			WorkshopSortMode.Trending => query.RankedByTrend(),
			WorkshopSortMode.Subscriptions => query.RankedByTotalUniqueSubscriptions(),
			WorkshopSortMode.TitleAsc => query.SortByTitleAsc(),
			_ => query.SortByUpdateDate(),
		};
	}

	private static async Task<WorkshopBrowseResultVm> ExecuteQueryAsync(Query query, int page, CancellationToken ct)
	{
		ct.ThrowIfCancellationRequested();

		var pageResult = await query.GetPageAsync(page).ConfigureAwait(false);
		if (!pageResult.HasValue)
		{
			return EmptyBrowseResult(page);
		}

		using var resultPage = pageResult.Value;
		var items = resultPage.Entries.Select(MapItemToVm).ToList();

		return new WorkshopBrowseResultVm
		{
			Items = items,
			TotalResults = resultPage.TotalCount,
			CurrentPage = page,
			HasMorePages = resultPage.TotalCount > page * 50,
		};
	}

	private static WorkshopItemDetailVm MapItemToVm(Item item)
	{
		var tags = item.Tags ?? [];
		var description = item.Description ?? "";
		var dep = WorkshopDependencyInference.Infer(tags, description);

		return new WorkshopItemDetailVm
		{
			PublishedFileId = item.Id.Value,
			Title = string.IsNullOrWhiteSpace(item.Title) ? $"Item {item.Id.Value}" : item.Title,
			Description = description,
			Tags = tags,
			OwnerName = item.Owner.Name ?? "",
			OwnerSteamId = item.Owner.Id.Value,
			PreviewImageUrl = item.PreviewImageUrl ?? "",
			Created = item.Created,
			Updated = item.Updated,
			Score = item.Score,
			VotesUp = item.VotesUp,
			VotesDown = item.VotesDown,
			NumSubscriptions = item.NumSubscriptions,
			NumFavorites = item.NumFavorites,
			NumComments = item.NumComments,
			SizeBytes = item.SizeBytes,
			IsSubscribed = item.IsSubscribed,
			IsInstalled = item.IsInstalled,
			NeedsUpdate = item.NeedsUpdate,
			IsDownloading = item.IsDownloading,
			IsBanned = item.IsBanned,
			Visibility = item.IsPrivate ? "Private" : item.IsFriendsOnly ? "FriendsOnly" : "Public",
			Url = item.Url ?? "",
			ChangelogUrl = item.ChangelogUrl ?? "",
			StatsUrl = item.StatsUrl ?? "",
			CommentsUrl = item.CommentsUrl ?? "",
			DependencyHintCompact = dep.CompactLine,
			DependencyHintBlock = dep.BulletBlock,
			HasDependencyHints = dep.HasAny,
		};
	}

	private static WorkshopBrowseResultVm EmptyBrowseResult(int page) => new()
	{
		Items = new List<WorkshopItemDetailVm>(),
		TotalResults = 0,
		CurrentPage = page,
		HasMorePages = false,
	};

	private static string DetectImageExtension(string url, byte[] data)
	{
		if (data.Length >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF)
			return ".jpg";
		if (data.Length >= 8 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47)
			return ".png";
		if (data.Length >= 4 && data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46)
			return ".gif";
		if (data.Length >= 4 && data[0] == 0x52 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x46)
			return ".webp";

		var ext = Path.GetExtension(new Uri(url).AbsolutePath)?.ToLowerInvariant();
		if (ext is ".png" or ".jpg" or ".jpeg" or ".gif" or ".webp")
			return ext == ".jpeg" ? ".jpg" : ext;

		return ".png";
	}

	private static string DefaultImportFolderName(string? title, ulong publishedFileId)
	{
		var baseName = string.IsNullOrWhiteSpace(title) ? "workshop" : title.Trim();
		var invalid = Path.GetInvalidFileNameChars();
		var chars = baseName.Select(c => invalid.Contains(c) ? '_' : c).ToArray();
		var s = new string(chars).Trim('.', ' ');
		if (s.Length > 48)
		{
			s = s[..48].TrimEnd();
		}

		return string.IsNullOrEmpty(s) ? $"ws_{publishedFileId}" : $"{s}_{publishedFileId}";
	}

	#endregion
}

public readonly record struct PublishOutcome(bool Success, ulong PublishedFileId, string Message)
{
	public static PublishOutcome Ok(ulong id) => new(true, id, "Published.");

	public static PublishOutcome Fail(string message) => new(false, 0, message);
}

