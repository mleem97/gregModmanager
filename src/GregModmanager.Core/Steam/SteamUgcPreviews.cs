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
	private static MethodInfo? _removeItemPreview;
	private static MethodInfo? _submitItemUpdate;
	private static MethodInfo? _getNumAdditionalPreviews;
	private static MethodInfo? _getAdditionalPreview;
	private static MethodInfo? _setReturnAdditionalPreviews;
	private static Type? _itemPreviewType;
	private static FieldInfo? _resultPageHandleField;
	private static bool _resolved;

	private static bool Resolve()
	{
		if (_resolved) return _ugcInternal is not null;
		_resolved = true;

		try
		{
			var ugcType = typeof(SteamUGC);
			var internalProp = ugcType.GetProperty("Internal",
				BindingFlags.Static | BindingFlags.NonPublic);
			if (internalProp is null) return false;

			_ugcInternal = internalProp.GetValue(null);
			if (_ugcInternal is null) return false;

			var internalType = _ugcInternal.GetType();
			const BindingFlags bf = BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public;

			_startItemUpdate = internalType.GetMethod("StartItemUpdate", bf);
			_addItemPreviewFile = internalType.GetMethod("AddItemPreviewFile", bf);
			_removeItemPreview = internalType.GetMethod("RemoveItemPreview", bf);
			_submitItemUpdate = internalType.GetMethod("SubmitItemUpdate", bf);
			_getNumAdditionalPreviews = internalType.GetMethod("GetQueryUGCNumAdditionalPreviews", bf);
			_getAdditionalPreview = internalType.GetMethod("GetQueryUGCAdditionalPreview", bf);
			_setReturnAdditionalPreviews = internalType.GetMethod("SetReturnAdditionalPreviews", bf);

			_itemPreviewType = internalType.Assembly.GetType("Steamworks.ItemPreviewType");

			_resultPageHandleField = typeof(ResultPage).GetField("Handle",
				BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public);

			return _startItemUpdate is not null && _addItemPreviewFile is not null;
		}
		catch
		{
			return false;
		}
	}

	public static bool IsAvailable => Resolve();

	/// <summary>
	/// Whether the reflection-based query for additional preview URLs is functional.
	/// </summary>
	public static bool CanQueryPreviews =>
		Resolve()
		&& _getNumAdditionalPreviews is not null
		&& _getAdditionalPreview is not null
		&& _setReturnAdditionalPreviews is not null
		&& _resultPageHandleField is not null;

	/// <summary>
	/// Queries Steam for additional preview image URLs of a published workshop item.
	/// Uses a details query with <c>SetReturnAdditionalPreviews</c> enabled, then
	/// reads results via reflection on <c>SteamUGC.Internal</c>.
	/// </summary>
	/// <returns>List of image URLs, or empty if none or unavailable.</returns>
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
			if (numPreviews is not uint count || count == 0) return urls;

			var imagePreviewTypeValue = _itemPreviewType is not null
				? Activator.CreateInstance(_itemPreviewType)
				: null;

			for (uint j = 0; j < count; j++)
			{
				var args = new object?[] { handleObj, (uint)0, j, null, null, imagePreviewTypeValue };
				var ok = _getAdditionalPreview!.Invoke(_ugcInternal, args);
				if (ok is not true) continue;

				var url = args[3] as string;
				if (string.IsNullOrEmpty(url)) continue;

				var previewTypeVal = args[5];
				if (previewTypeVal is not null && _itemPreviewType is not null)
				{
					var typeInt = Convert.ToInt32(previewTypeVal);
					if (typeInt != 0) continue; // 0 = k_EItemPreviewType_Image
				}

				urls.Add(url);
			}
		}
		finally
		{
			resultPage.Dispose();
		}

		return urls;
	}

	/// <summary>
	/// Uploads additional preview images for a published workshop item.
	/// This creates a separate update pass specifically for preview files.
	/// </summary>
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
			var appId = (AppId)SteamConstants.DataCenterAppId;
			var fileId = (PublishedFileId)publishedFileId;

			var handle = _startItemUpdate!.Invoke(_ugcInternal, [appId, fileId]);
			if (handle is null) return false;

			var imagePreviewValue = _itemPreviewType is not null
				? Enum.ToObject(_itemPreviewType, 0)  // k_EItemPreviewType_Image = 0
				: (object)0;

			var added = 0;
			foreach (var path in imagePaths)
			{
				ct.ThrowIfCancellationRequested();

				if (!File.Exists(path))
				{
					log?.Report($"Screenshot not found, skipping: {Path.GetFileName(path)}");
					continue;
				}

				var fi = new FileInfo(path);
				if (fi.Length > 1_048_576)
				{
					log?.Report($"Screenshot too large (>1 MB), skipping: {Path.GetFileName(path)}");
					continue;
				}

				var result = _addItemPreviewFile!.Invoke(_ugcInternal, [handle, path, imagePreviewValue]);
				if (result is true)
				{
					added++;
					log?.Report($"Added preview: {Path.GetFileName(path)}");
				}
				else
				{
					log?.Report($"Failed to add preview: {Path.GetFileName(path)}");
				}
			}

			if (added == 0)
			{
				log?.Report("No preview images were added.");
				return false;
			}

			var callResult = _submitItemUpdate!.Invoke(_ugcInternal, [handle, (string?)null]);
			if (callResult is null) return false;

			for (var i = 0; i < 120; i++)
			{
				ct.ThrowIfCancellationRequested();
				await Task.Delay(500, ct).ConfigureAwait(false);
				SteamClient.RunCallbacks();
			}

			log?.Report($"Uploaded {added} additional preview image(s).");
			return true;
		}
		catch (Exception ex) when (ex is not OperationCanceledException)
		{
			log?.Report($"Additional previews upload error: {ex.Message}");
			return false;
		}
	}
}

