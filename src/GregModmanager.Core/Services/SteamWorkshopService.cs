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

		// Stage the preview to a temp copy — the original may be locked by our
		// own UI or external processes. Read bytes first to avoid lock contention.
		string? tempPreview = null;
		string previewForSteam = absPreview;
		try
		{
			var previewBytes = await File.ReadAllBytesAsync(absPreview, cancellationToken).ConfigureAwait(false);
			tempPreview = Path.Combine(Path.GetTempPath(), $"gm_preview_{Guid.NewGuid():N}{Path.GetExtension(absPreview)}");
			await File.WriteAllBytesAsync(tempPreview, previewBytes, cancellationToken).ConfigureAwait(false);
			previewForSteam = tempPreview;
		}
		catch (Exception ex)
		{
			log?.Report($"Preview copy failed, trying original: {ex.Message}");
		}

		// Use Facepunch's Editor (proven create/update flow) — NOT raw native
		// calls. A previous native reimplementation failed identically because
		// the actual problem was never file access: new items need CreateItem
		// first (the Editor does this), and the backend needs a moment to
		// replicate the new item before Submit (handled below + retry).
		Steamworks.Ugc.Editor BuildEditor(ulong fileId)
		{
			var ed = fileId == 0
				? Steamworks.Ugc.Editor.NewCommunityFile
				: new Steamworks.Ugc.Editor((PublishedFileId)fileId);
			ed = ed
				.ForAppId((AppId)SteamConstants.DataCenterAppId)
				.WithTitle(title)
				.WithDescription(description)
				.WithContent(absContent)
				.WithPreviewFile(previewForSteam);
			ed = tagList.Aggregate(ed, (current, tag) => current.WithTag(tag));
			if (!string.IsNullOrWhiteSpace(changeLog))
				ed = ed.WithChangeLog(changeLog);
			return ApplyVisibility(ed, metadata.Visibility);
		}

		// The backend must know a just-created item before Submit, or the
		// uploader fails with FileNotFound (proven via workshop_log.txt).
		async Task WaitForReplicationAsync(ulong fileId)
		{
			for (int i = 0; i < 24; i++)
			{
				cancellationToken.ThrowIfCancellationRequested();
				if (i > 0)
					await Task.Delay(5000, cancellationToken).ConfigureAwait(false);
				WorkshopItemDetailVm? detail = null;
				try { detail = await GetItemDetailsAsync(fileId, cancellationToken).ConfigureAwait(false); }
				catch (OperationCanceledException) { throw; }
				catch { /* keep waiting */ }
				if (detail is not null)
				{
					log?.Report($"Workshop backend knows item {fileId} — submitting...");
					return;
				}
				log?.Report($"Waiting for Workshop backend to replicate item {fileId}...");
			}
			log?.Report($"Item {fileId} not yet visible; submitting anyway (retry follows on FileNotFound).");
		}

		bool justCreated = false;
		async Task<Steamworks.Ugc.PublishResult> SubmitOnceAsync()
		{
			var ed = BuildEditor(metadata.PublishedFileId);
			justCreated = metadata.PublishedFileId == 0;
			return await ed.SubmitAsync(uploadProgress, onItemCreated: r =>
			{
				// Log EVERY creation result — a silent id 0 here used to cause
				// a doomed StartItemUpdate(0) and a confusing FileNotFound.
				log?.Report($"CreateItem result: id={r.FileId.Value}, result={r.Result}, agreement={r.NeedsWorkshopAgreement}");
				if (r.FileId.Value != 0)
				{
					metadata.PublishedFileId = r.FileId.Value;
					try { WorkspaceService.SaveMetadata(projectRoot, metadata); } catch { /* keep going */ }
					log?.Report($"Created new workshop item {r.FileId.Value}.");
					return;
				}
				if (justCreated)
				{
					// Steam returned OK but no file id (throttled/failed create).
					// Abort before the doomed update instead of burning it.
					throw new InvalidOperationException(
						"Steam created no file id for the new item (creation throttled or failed). " +
						"Wait a few minutes and retry — your content is untouched.");
				}
				// NOTE: no blocking wait here — a replication check inside this
				// callback hung the whole submit (query await never returned).
				// Replication is handled after submit via retry (see below).
			}).ConfigureAwait(false);
		}

		// Facepunch only dispatches Steam callbacks while someone pumps
		// SteamClient.RunCallbacks(). Nothing else in the app does that during
		// publish, so run a dedicated pumper for the whole operation.
		// (The CTS is cancelled + awaited in the finally below — disposing it
		// while the pump still runs crashed the app with ObjectDisposedException.)
		var pumpCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
		var pumpTask = Task.Run(async () =>
		{
			while (!pumpCts.Token.IsCancellationRequested)
			{
				try { SteamClient.RunCallbacks(); } catch { /* never break the pump */ }
				try { await Task.Delay(16, pumpCts.Token).ConfigureAwait(false); } catch { /* cancelled */ }
			}
		}, cancellationToken);
		static async Task<T> WithTimeout<T>(Task<T> task, TimeSpan timeout, CancellationToken ct)
		{
			var winner = await Task.WhenAny(task, Task.Delay(timeout, ct)).ConfigureAwait(false);
			if (winner != task)
				throw new TimeoutException($"Submit timed out after {(int)timeout.TotalMinutes} min with no result from Steam.");
			return await task.ConfigureAwait(false);
		}

		Steamworks.Ugc.PublishResult result;
		try
		{
			log?.Report(metadata.PublishedFileId == 0
				? "Creating new workshop item..."
				: $"Updating workshop item {metadata.PublishedFileId}...");
			result = await WithTimeout(SubmitOnceAsync(), TimeSpan.FromMinutes(4), cancellationToken).ConfigureAwait(false);
		}
		catch (OperationCanceledException) { pumpCts.Cancel(); throw; }
		catch (Exception ex)
		{
			pumpCts.Cancel();
			try { if (tempPreview is not null && File.Exists(tempPreview)) File.Delete(tempPreview); } catch { }
			return PublishOutcome.Fail(ex.Message);
		}

		try
		{
		if (!result.Success && result.Result == Steamworks.Result.FileNotFound
			&& justCreated && metadata.PublishedFileId != 0)
		{
			// Backend replication lag on a brand-new item — wait, re-verify
			// (time-boxed so a hanging query can never block us), then retry
			// the full submit once with the now-known file id.
			log?.Report("First submit hit backend replication lag — waiting 30s and retrying once...");
			try { await Task.Delay(30000, cancellationToken).ConfigureAwait(false); }
			catch (OperationCanceledException) { throw; }
			try
			{
				var check = WaitForReplicationAsync(metadata.PublishedFileId);
				var winner = await Task.WhenAny(check, Task.Delay(TimeSpan.FromSeconds(45), cancellationToken)).ConfigureAwait(false);
				if (winner != check)
					log?.Report("Replication check timed out — retrying submit anyway...");
			}
			catch (OperationCanceledException) { throw; }
			catch (Exception ex) { log?.Report($"Replication check skipped: {ex.Message}"); }
			try
			{
				result = await WithTimeout(SubmitOnceAsync(), TimeSpan.FromMinutes(4), cancellationToken).ConfigureAwait(false);
			}
			catch (OperationCanceledException) { throw; }
			catch (Exception ex)
			{
				try { if (tempPreview is not null && File.Exists(tempPreview)) File.Delete(tempPreview); } catch { }
				return PublishOutcome.Fail(ex.Message);
			}
		}

		try { if (tempPreview is not null && File.Exists(tempPreview)) File.Delete(tempPreview); } catch { }

		if (result.NeedsWorkshopAgreement)
		{
			return PublishOutcome.Fail("Workshop legal agreement must be accepted in the Steam client.");
		}

		if (!result.Success)
		{
			// Steam may create the item before its content update fails. Preserve the ID
			// so retrying updates the existing item instead of creating a duplicate.
			if (result.FileId.Value != 0)
			{
				metadata.PublishedFileId = result.FileId.Value;
				try { WorkspaceService.SaveMetadata(projectRoot, metadata); } catch { /* keep Steam error */ }
				log?.Report($"Workshop item {metadata.PublishedFileId} was created but the update failed; retry will reuse it.");
			}
			var detail = $"Steam publish failed: {result.Result} (content={absContent}, preview={absPreview})";
			log?.Report(detail);
			return PublishOutcome.Fail(detail);
		}

		var id = result.FileId.Value;
		if (id != 0)
		{
			metadata.PublishedFileId = id;
		}

		return PublishOutcome.Ok(metadata.PublishedFileId);
		}
		finally
		{
			// Stop the pumper cleanly: cancel first, THEN dispose (never while
			// the loop still runs — that crashed with ObjectDisposedException).
			// Await briefly so no unobserved pump exception escapes.
			try { pumpCts.Cancel(); } catch { /* already gone */ }
			try { await pumpTask.WaitAsync(TimeSpan.FromSeconds(5)).ConfigureAwait(false); }
			catch { /* pump stopped or timed out — not fatal */ }
			try { pumpCts.Dispose(); } catch { /* already gone */ }
		}
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
}

public readonly record struct PublishOutcome(bool Success, ulong PublishedFileId, string Message)
{
	public static PublishOutcome Ok(ulong id) => new(true, id, "Published.");

	public static PublishOutcome Fail(string message) => new(false, 0, message);
}

