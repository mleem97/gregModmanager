using System.Diagnostics;
using System.Reflection;
using System.Runtime.InteropServices;
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
				return HasUsableSteamClient(log);
			}

			try
			{
				var preloaded = SteamApiNativeLoader.TryPreload();
				if (!preloaded)
				{
					var paths = string.Join("; ", SteamApiNativeLoader.GetAttemptedPaths());
					var nativeLibrary = RuntimeInformation.IsOSPlatform(OSPlatform.Windows)
						? "steam_api64.dll"
						: "libsteam_api64.so / libsteam_api.so";
					LastSteamConnectionHint = $"{nativeLibrary} nicht gefunden. Gesucht: {paths}";
					AppFileLog.Warn($"[Steam] {LastSteamConnectionHint}");
					log?.Report(LastSteamConnectionHint);
					return false;
				}

				// Check if SteamClient is already initialized
				try
				{
					if (SteamClient.IsValid)
					{
						if (HasUsableSteamClient(log))
						{
							_initialized = true;
							LastSteamConnectionHint = string.Empty;
							AppFileLog.Info($"[Steam] SteamClient already initialized.");
							return true;
						}

						return false;
					}
				}
				catch
				{
					// IsValid can throw during initialization
				}

				AppFileLog.Info($"[Steam] Calling SteamClient.Init({SteamConstants.DataCenterAppId}, true)...");
				try
				{
					SteamClient.Init(SteamConstants.DataCenterAppId, true);
				}
				catch (Exception initEx) when (initEx.Message.Contains("SteamApps") || initEx.Message.Contains("v008") || initEx.Message.Contains("v009"))
				{
					// SteamApps interface version mismatch is non-fatal - Steam is still connected
					AppFileLog.Warn($"[Steam] SteamApps interface version mismatch (non-fatal): {initEx.Message}");
				}

				// Give async callbacks a moment to complete
				Thread.Sleep(500);
				
				if (!HasUsableSteamClient(log))
				{
					return false;
				}

				_initialized = true;
				LastSteamConnectionHint = string.Empty;
				AppFileLog.Info($"[Steam] SteamClient.Init succeeded.");
				log?.Report("Steam API initialized.");
				return true;
			}
			catch (Exception ex)
			{
				// If already initialized, treat as success
				if (ex.Message.Contains("already initialized"))
				{
					if (HasUsableSteamClient(log))
					{
						_initialized = true;
						LastSteamConnectionHint = string.Empty;
						AppFileLog.Info($"[Steam] SteamClient was already initialized.");
						return true;
					}

					return false;
				}
				
				LastSteamConnectionHint = ex.Message;
				AppFileLog.Error($"[Steam] SteamClient.Init failed: {ex.Message}", ex);
				log?.Report($"Steam init failed: {ex.Message}");
				return false;
			}
		}
	}

	private bool HasUsableSteamClient(IProgress<string>? log)
	{
		try
		{
			if (!SteamClient.IsValid)
			{
				LastSteamConnectionHint = "Steam API ungültig (IsValid=false).";
				log?.Report(LastSteamConnectionHint);
				return false;
			}

			// Query construction requires a valid Steam ID. Some incompatible native
			// libraries report IsValid=true but throw here, which must not be treated
			// as a usable Workshop connection.
			_ = SteamClient.SteamId;
			return true;
		}
		catch (Exception ex)
		{
			_initialized = false;
			LastSteamConnectionHint = $"Steam API is not usable: {ex.Message}";
			AppFileLog.Warn($"[Steam] {LastSteamConnectionHint}");
			log?.Report(LastSteamConnectionHint);
			return false;
		}
	}

	/// <summary>True when Steam API is up and the user is logged on (see <see cref="SteamClient.IsLoggedOn"/>).</summary>
	public bool TryGetSteamReady(out string? userName)
	{
		userName = null;
		lock (InitLock)
		{
			if (!EnsureInitialized(null))
			{
				return false;
			}

			try
			{
				if (!SteamClient.IsValid)
				{
					LastSteamConnectionHint = "Steam API ungültig (IsValid=false).";
					return false;
				}

				// Try to get username - IsLoggedOn can throw in Cherry.Facepunch 2.5.0
				// when SteamUser interface failed to init (SteamApps_v008 mismatch).
				// Fall back to just checking IsValid which means the API is connected.
				string? name = null;
				try
				{
					if (SteamClient.IsLoggedOn)
					{
						name = SteamClient.Name;
					}
					else
					{
						// IsValid=true but not logged on - still show as connected
						name = "Steam User";
					}
				}
				catch
				{
					// IsLoggedOn threw - but IsValid is true, so Steam IS connected
					name = "Steam User";
					AppFileLog.Info("[Steam] IsLoggedOn unavailable, but IsValid=true - showing as connected.");
				}

				LastSteamConnectionHint = string.Empty;
				userName = name;
				return true;
			}
			catch (Exception ex)
			{
				LastSteamConnectionHint = $"Steam status check failed: {ex.Message}";
				return false;
			}
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
		if (previewPath is null || !File.Exists(previewPath))
		{
			return PublishOutcome.Fail("A preview image is required for every Workshop upload and update.");
		}

		var previewSize = new FileInfo(previewPath).Length;
		if (previewSize > SteamConstants.MaxPreviewImageBytes)
		{
			return PublishOutcome.Fail($"Preview image exceeds Steam's {WorkspaceService.FormatBytes(SteamConstants.MaxPreviewImageBytes)} limit.");
		}

		// Resolve to absolute paths – the native Steamworks API may not resolve
		// relative or partially-qualified paths correctly on Linux.
		var absContent = Path.GetFullPath(contentFolder);
		var absPreview = Path.GetFullPath(previewPath);

		// Embed mod-type marker so subscribers know where to install this item.
		try
		{
			var markerPath = Path.Combine(absContent, "greg-modmanager.meta.json");
			var markerJson = System.Text.Json.JsonSerializer.Serialize(new ModStoreMarker { modType = metadata.ModType }, AppJsonContext.Default.ModStoreMarker);
			File.WriteAllText(markerPath, markerJson);
		}
		catch
		{
			// non-critical
		}

		// Collect tags
		var tagList = metadata.Tags
			.Where(t => !string.IsNullOrWhiteSpace(t))
			.Select(t => t.Trim())
			.ToList();

		if (!SteamPublishRateLimiter.Shared.TryAcquire(out var retryAfter))
		{
			var seconds = Math.Max(1, (int)Math.Ceiling(retryAfter.TotalSeconds));
			return PublishOutcome.Fail($"Steam publish cooldown active. Please wait {seconds}s before trying again.");
		}

		// Use native ISteamUGC calls — the Facepunch Editor wrapper is broken
		// on Linux (SetItemContent/SetItemPreview return FileNotFound).
		return await NativePublishAsync(
			projectRoot, metadata, absContent, absPreview,
			changeLog, tagList, uploadProgress, log, cancellationToken
		).ConfigureAwait(false);
	}

	/// <summary>Completes local post-publish bookkeeping without replacing the editable local project from Steam.</summary>
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

		ct.ThrowIfCancellationRequested();
		if (publishedFileId == 0 || string.IsNullOrWhiteSpace(projectRoot) || !Directory.Exists(projectRoot))
		{
			log?.Report("Post-publish bookkeeping skipped: project or Workshop ID is invalid.");
			return false;
		}

		// The local project is the editable source of truth. Steam's downloaded UGC
		// cache must never overwrite content/ after a successful upload.
		log?.Report("Publish complete: local project content preserved for further editing.");
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
			.WhereUserSubscribed(SteamClient.SteamId)
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
			.WhereUserFavorited(SteamClient.SteamId)
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
			.WhereUserPublished(SteamClient.SteamId)
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
			.WhereUserPublished(SteamClient.SteamId)
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

		try
		{
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
		catch (Exception ex)
		{
			LastSteamConnectionHint = $"Workshop item {publishedFileId} could not be loaded: {ex.Message}";
			AppFileLog.Warn($"[Steam] {LastSteamConnectionHint}");
			return null;
		}
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
		// Keep local description when Steam returns empty — the user may have
		// edited it locally and the empty value means Steam was never updated.
		if (!string.IsNullOrWhiteSpace(steam.Description))
		{
			target.Description = steam.Description;
		}
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

		// Import is intentionally idempotent. Editing or importing the same Workshop item
		// again must reuse its existing local project instead of failing or creating duplicates.
		var existingProject = workspace.FindProjectByPublishedFileId(publishedFileId);
		if (existingProject is not null)
		{
			log?.Report($"Existing project reused: {existingProject.RootPath}");
			return ImportOutcome.Ok(existingProject.RootPath);
		}

		var destRoot = GetAvailableProjectRoot(workspace.WorkspaceRoot, dirName);
		if (!string.Equals(Path.GetFileName(destRoot), dirName, StringComparison.Ordinal))
			log?.Report($"Project folder already existed; importing into {destRoot}.");

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

	private static string GetAvailableProjectRoot(string workspaceRoot, string baseName)
	{
		var candidate = Path.Combine(workspaceRoot, baseName);
		if (!Directory.Exists(candidate) && !File.Exists(candidate)) return candidate;

		for (var suffix = 2; suffix <= 10_000; suffix++)
		{
			candidate = Path.Combine(workspaceRoot, $"{baseName}_{suffix}");
			if (!Directory.Exists(candidate) && !File.Exists(candidate)) return candidate;
		}

		throw new IOException($"Kein freier Projektordner für '{baseName}' gefunden.");
	}

	#endregion

	/// <summary>
	/// Native Steam publish that bypasses the Facepunch.Steamworks Editor wrapper.
	/// The Editor's WithContent/WithPreviewFile calls are broken on Linux (return
	/// FileNotFound even when files exist). This method calls ISteamUGC directly.
	/// </summary>
	private async Task<PublishOutcome> NativePublishAsync(
		string projectRoot,
		WorkshopMetadata metadata,
		string absContent,
		string absPreview,
		string? changeLog,
		List<string> tags,
		IProgress<float>? uploadProgress,
		IProgress<string>? log,
		CancellationToken ct)
	{
		// Resolve ISteamUGC internal object via reflection
		var ugcType = typeof(SteamUGC);
		var internalProp = ugcType.GetProperty("Internal", BindingFlags.Static | BindingFlags.NonPublic | BindingFlags.Public);
		var ugcInternal = internalProp?.GetValue(null);
		if (ugcInternal is null)
			return PublishOutcome.Fail("SteamUGC.Internal is null — Steam not initialized?");

		var internalType = ugcInternal.GetType();
		const BindingFlags bf = BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public | BindingFlags.FlattenHierarchy;

		// Use GetMethods + LINQ — the .NET trimmer strips method metadata that
		// GetMethod relies on, but GetMethods preserves them.
		MethodInfo? FindMethod(string name) =>
			internalType.GetMethods(bf).FirstOrDefault(m => m.Name == name);

		var startItemUpdate = FindMethod("StartItemUpdate");
		if (startItemUpdate is null)
		{
			var methods = string.Join(", ", internalType.GetMethods(bf).Select(m => m.Name).Distinct().Order());
			return PublishOutcome.Fail($"StartItemUpdate not found on {internalType.FullName}. Methods: {methods}");
		}
		var setItemTitle = FindMethod("SetItemTitle");
		var setItemDescription = FindMethod("SetItemDescription");
		var setItemContent = FindMethod("SetItemContent");
		var setItemPreview = FindMethod("SetItemPreview");
		var setItemVisibility = FindMethod("SetItemVisibility");
		var setItemTags = FindMethod("SetItemTags");
		var submitItemUpdate = FindMethod("SubmitItemUpdate");
		if (submitItemUpdate is null)
			return PublishOutcome.Fail("SubmitItemUpdate not found");
		var getUpdateProgress = FindMethod("GetItemUpdateProgress");

		// StartItemUpdate(appId, publishedFileId) -> UGCUpdateHandle_t
		var handle = startItemUpdate.Invoke(ugcInternal, [(AppId)SteamConstants.DataCenterAppId, (PublishedFileId)metadata.PublishedFileId]);
		if (handle is null)
			return PublishOutcome.Fail("StartItemUpdate returned null handle");

		// Set fields
		setItemTitle?.Invoke(ugcInternal, [handle, metadata.Title ?? ""]);
		setItemDescription?.Invoke(ugcInternal, [handle, metadata.Description ?? ""]);
		setItemContent?.Invoke(ugcInternal, [handle, absContent]);

		// Copy preview to temp file — the original may be locked by our own app
		// (Image controls, metadata scan, etc.) or by external processes.
		// Read bytes first to avoid any file-lock contention.
		string? tempPreview = null;
		try
		{
			var previewBytes = await File.ReadAllBytesAsync(absPreview, ct).ConfigureAwait(false);
			tempPreview = Path.Combine(Path.GetTempPath(), $"gm_preview_{Guid.NewGuid():N}{Path.GetExtension(absPreview)}");
			await File.WriteAllBytesAsync(tempPreview, previewBytes, ct).ConfigureAwait(false);
			setItemPreview?.Invoke(ugcInternal, [handle, tempPreview]);
		}
		catch (Exception ex)
		{
			log?.Report($"Preview copy failed, trying original: {ex.Message}");
			setItemPreview?.Invoke(ugcInternal, [handle, absPreview]);
		}

		// Visibility
		var visibilityValue = metadata.Visibility switch
		{
			"Private" => 2,    // RemoteStoragePublishedFileVisibility.Private
			"FriendsOnly" => 1, // RemoteStoragePublishedFileVisibility.FriendsOnly
			"Unlisted" => 3,    // RemoteStoragePublishedFileVisibility.Unlisted
			_ => 0              // RemoteStoragePublishedFileVisibility.Public
		};
		setItemVisibility?.Invoke(ugcInternal, [handle, visibilityValue]);

		// Tags via SteamParamStringArray_t
		if (tags.Count > 0 && setItemTags is not null)
		{
			try
			{
				var ssaType = typeof(SteamUGC).Assembly.GetType("Steamworks.Data.SteamParamStringArray_t");
				var ssaManagedType = typeof(SteamUGC).Assembly.GetType("Steamworks.Ugc.SteamParamStringArray");
				if (ssaType is not null && ssaManagedType is not null)
				{
					var nativeStrings = new IntPtr[tags.Count];
					var gcHandles = new GCHandle[tags.Count];
					for (int i = 0; i < tags.Count; i++)
					{
						gcHandles[i] = GCHandle.Alloc(System.Text.Encoding.UTF8.GetBytes(tags[i] + '\0'), GCHandleType.Pinned);
						nativeStrings[i] = gcHandles[i].AddrOfPinnedObject();
					}

					var nativeArray = Marshal.AllocHGlobal(tags.Count * IntPtr.Size);
					Marshal.Copy(nativeStrings, 0, nativeArray, tags.Count);

					var ssa = Activator.CreateInstance(ssaType)!;
					ssaType.GetField("Strings")!.SetValue(ssa, nativeArray);
					ssaType.GetField("NumStrings")!.SetValue(ssa, tags.Count);

					setItemTags.Invoke(ugcInternal, [handle, ssa, false]);

					Marshal.FreeHGlobal(nativeArray);
					foreach (var h in gcHandles) h.Free();
				}
			}
			catch (Exception ex)
			{
				log?.Report($"Tag upload failed (non-fatal): {ex.Message}");
			}
		}

		// SubmitItemUpdate(handle, changeNote)
		log?.Report(metadata.PublishedFileId == 0
			? "Creating new workshop item (native)..."
			: $"Updating workshop item {metadata.PublishedFileId} (native)...");

		var callResult = submitItemUpdate.Invoke(ugcInternal, [handle, changeLog ?? ""]);

		// Wait for completion by pumping callbacks — same pattern as SteamUgcPreviews.
		// GetItemUpdateProgress returns a boxed ItemUpdateStatus enum (NOT int —
		// `is int` never matches). Use Convert.ToInt32 on the boxed enum.
		// Status: 0=Invalid, 1=PreparingConfig, 2=PreparingContent,
		// 3=UploadingContent, 4=UploadingPreviewFile, 5=CommittingChanges.
		const int maxWaitIntervals = 600; // 5 minutes max
		var finished = false;
		for (int i = 0; i < maxWaitIntervals && !finished; i++)
		{
			ct.ThrowIfCancellationRequested();
			await Task.Delay(500, ct).ConfigureAwait(false);
			SteamClient.RunCallbacks();

			if (getUpdateProgress is not null)
			{
				try
				{
					var progressArgs = new object?[] { handle, (ulong)0, (ulong)0 };
					var statusObj = getUpdateProgress.Invoke(ugcInternal, progressArgs);
					if (statusObj is not null)
					{
						int updateStatus = Convert.ToInt32(statusObj);
						var bytesProcessed = (ulong)(progressArgs[1] ?? 0UL);
						var bytesTotal = (ulong)(progressArgs[2] ?? 0UL);
						if (bytesTotal > 0)
						{
							float pct = (float)bytesProcessed / bytesTotal;
							uploadProgress?.Report(pct);
							if (i % 4 == 0) // log every 2s, not every tick
								log?.Report($"Uploading... {pct:P0} ({bytesProcessed}/{bytesTotal} bytes)");
						}
						else if (i % 2 == 0) // every 1s when no byte counts yet
						{
							log?.Report($"Uploading... ({(i / 2) + 1}s elapsed, status={updateStatus})");
						}

						if (updateStatus >= 5) // CommittingChanges — done, finalize
						{
							finished = true;
							await Task.Delay(1000, ct).ConfigureAwait(false);
							SteamClient.RunCallbacks();
						}
					}
				}
				catch { /* best-effort progress */ }
			}
			else if (i % 2 == 0) // no progress API at all
			{
				log?.Report($"Uploading... ({(i / 2) + 1}s elapsed)");
			}
		}

		// Give Steam a moment to finalize
		await Task.Delay(2000, ct).ConfigureAwait(false);
		SteamClient.RunCallbacks();

		// Clean up temp preview file
		try { if (tempPreview is not null && File.Exists(tempPreview)) File.Delete(tempPreview); } catch { }

		log?.Report("Publish completed via native API.");
		return PublishOutcome.Ok(metadata.PublishedFileId);
	}
}

public readonly record struct PublishOutcome(bool Success, ulong PublishedFileId, string Message)
{
	public static PublishOutcome Ok(ulong id) => new(true, id, "Published.");

	public static PublishOutcome Fail(string message) => new(false, 0, message);
}

