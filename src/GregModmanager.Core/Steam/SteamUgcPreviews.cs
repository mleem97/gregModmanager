using System.Reflection;
using System.Runtime.CompilerServices;
using Steamworks;
using Steamworks.Data;
using Steamworks.Ugc;

[assembly: InternalsVisibleTo("GregModmanager")]

namespace GregModmanager.Steam;

/// <summary>
/// Provides access to Steamworks UGC additional preview file functions
/// via reflection on <c>SteamUGC.Internal</c>, since Facepunch.Steamworks 2.3.x
/// does not expose these methods publicly.
/// </summary>
internal static class SteamUgcPreviews
{
	private static object? _ugcInternal;
	private static MethodInfo? _startItemUpdate;
	private static MethodInfo? _addItemPreviewFile;
	private static MethodInfo? _submitItemUpdate;
	private static MethodInfo? _getNumAdditionalPreviews;
	private static MethodInfo? _getAdditionalPreview;
	private static MethodInfo? _setReturnAdditionalPreviews;
	private static Type? _itemPreviewType;
	private static FieldInfo? _resultPageHandleField;
	private static bool _resolved;

	[System.Diagnostics.CodeAnalysis.SuppressMessage("SonarLint", "S3011", Justification = "Reflection on internal Steamworks APIs is required for additional preview support.")]
	[System.Diagnostics.CodeAnalysis.UnconditionalSuppressMessage("Trimming", "IL2026:RequiresUnreferencedCode", Justification = "Steamworks internal types are preserved via library settings or trimmer roots.")]
	[System.Diagnostics.CodeAnalysis.UnconditionalSuppressMessage("Trimming", "IL2075:DynamicallyAccessedMembers", Justification = "Reflection on Steamworks.UGC.Internal is required for full feature set.")]
	private static bool Resolve()
	{
		if (_resolved) return _ugcInternal is not null;
		_resolved = true;

		try
		{
			var ugcType = typeof(SteamUGC);
			const BindingFlags bf = BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public;

			var internalProp = ugcType.GetProperty("Internal", BindingFlags.Static | BindingFlags.NonPublic | BindingFlags.Public);
			if (internalProp is null) return false;

			_ugcInternal = internalProp.GetValue(null);
			if (_ugcInternal is null) return false;

			var internalType = _ugcInternal.GetType();

			void Resolve()
			{
				_startItemUpdate = internalType.GetMethod("StartItemUpdate", bf);
				_addItemPreviewFile = internalType.GetMethod("AddItemPreviewFile", bf);
				_submitItemUpdate = internalType.GetMethod("SubmitItemUpdate", bf);
				_getNumAdditionalPreviews = internalType.GetMethod("GetQueryUGCNumAdditionalPreviews", bf);
				_getAdditionalPreview = internalType.GetMethod("GetQueryUGCAdditionalPreview", bf);
				_setReturnAdditionalPreviews = internalType.GetMethod("SetReturnAdditionalPreviews", bf);
			}
			Resolve();

			_itemPreviewType = internalType.Assembly.GetType("Steamworks.ItemPreviewType");

			var handleFld = typeof(ResultPage).GetField("Handle", bf);
			_resultPageHandleField = handleFld;

			return _startItemUpdate is not null && _addItemPreviewFile is not null;
		}
		catch
		{
			return false;
		}
	}

	public static bool IsAvailable => Resolve();

	public static bool CanQueryPreviews =>
		Resolve()
		&& _getNumAdditionalPreviews is not null
		&& _getAdditionalPreview is not null
		&& _setReturnAdditionalPreviews is not null
		&& _resultPageHandleField is not null;

	public static async Task<List<string>> QueryAdditionalPreviewUrlsAsync(ulong publishedFileId)
	{
		var urls = new List<string>();
		if (!CanQueryPreviews) return urls;

		var page = await Query.All
			.WithFileId((PublishedFileId)publishedFileId)
			.WithAdditionalPreviews(true)
			.GetPageAsync(1)
			.ConfigureAwait(false);

		if (!page.HasValue) return urls;

		var resultPage = page.Value;
		try
		{
			var handleObj = _resultPageHandleField!.GetValue(resultPage);
			if (handleObj is null) return urls;

			var numPreviews = _getNumAdditionalPreviews!.Invoke(_ugcInternal, [handleObj, (uint)0]);
			if (numPreviews is uint count && count > 0)
			{
				ExtractPreviewUrls(handleObj, count, urls);
			}
		}
		finally
		{
			resultPage.Dispose();
		}

		return urls;
	}

	[System.Diagnostics.CodeAnalysis.UnconditionalSuppressMessage("Trimming", "IL2077:DynamicallyAccessedMembers", Justification = "ItemPreviewType is an enum and safe to activate.")]
	private static void ExtractPreviewUrls(object handleObj, uint count, List<string> urls)
	{
		var imagePreviewTypeValue = _itemPreviewType is not null
			? Activator.CreateInstance(_itemPreviewType)
			: null;

		for (uint j = 0; j < count; j++)
		{
			var args = new object?[] { handleObj, (uint)0, j, null, null, imagePreviewTypeValue };
			var ok = _getAdditionalPreview!.Invoke(_ugcInternal, args);
			if (ok is not true) continue;

			var url = args[3] as string;
			var previewTypeVal = args[5];

			if (!string.IsNullOrEmpty(url) && IsImageType(previewTypeVal))
			{
				urls.Add(url);
			}
		}
	}

	private static bool IsImageType(object? previewTypeVal)
	{
		if (previewTypeVal is null || _itemPreviewType is null) return true;
		try
		{
			return Convert.ToInt32(previewTypeVal) == 0; // 0 = k_EItemPreviewType_Image
		}
		catch { return false; }
	}

	public static async Task<bool> UploadAdditionalPreviewsAsync(
		ulong publishedFileId,
		IReadOnlyList<string> imagePaths,
		IProgress<string>? log,
		CancellationToken ct)
	{
		if (!Resolve() || imagePaths.Count == 0)
			return false;

		try
		{
			var handle = _startItemUpdate!.Invoke(_ugcInternal, [(AppId)SteamConstants.DataCenterAppId, (PublishedFileId)publishedFileId]);
			if (handle is null) return false;

			var added = await ProcessPreviewFilesAsync(handle, imagePaths, log, ct);
			if (added == 0) return false;

			var callResult = _submitItemUpdate!.Invoke(_ugcInternal, [handle, (string?)null]);
			if (callResult is null) return false;

			await WaitWithCallbacksAsync(120, ct);

			log?.Report($"Uploaded {added} additional preview image(s).");
			return true;
		}
		catch (Exception ex) when (ex is not OperationCanceledException)
		{
			log?.Report($"Additional previews upload error: {ex.Message}");
			return false;
		}
	}

	private static Task<int> ProcessPreviewFilesAsync(object handle, IReadOnlyList<string> imagePaths, IProgress<string>? log, CancellationToken ct)
	{
		var imagePreviewValue = _itemPreviewType is not null ? Enum.ToObject(_itemPreviewType, 0) : (object)0;
		var added = 0;

		foreach (var path in imagePaths)
		{
			ct.ThrowIfCancellationRequested();

			if (!File.Exists(path))
			{
				log?.Report($"Screenshot not found, skipping: {Path.GetFileName(path)}");
				continue;
			}

			if (new FileInfo(path).Length > 1_048_576)
			{
				log?.Report($"Screenshot too large (>1 MB), skipping: {Path.GetFileName(path)}");
				continue;
			}

			if (_addItemPreviewFile!.Invoke(_ugcInternal, [handle, path, imagePreviewValue]) is true)
			{
				added++;
				log?.Report($"Added preview: {Path.GetFileName(path)}");
			}
			else
			{
				log?.Report($"Failed to add preview: {Path.GetFileName(path)}");
			}
		}
		return Task.FromResult(added);
	}

	private static async Task WaitWithCallbacksAsync(int intervals, CancellationToken ct)
	{
		for (var i = 0; i < intervals; i++)
		{
			ct.ThrowIfCancellationRequested();
			await Task.Delay(500, ct).ConfigureAwait(false);
			SteamClient.RunCallbacks();
		}
	}
}

