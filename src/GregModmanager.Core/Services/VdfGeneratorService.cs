using GregModmanager.Steam;

namespace GregModmanager.Services;

/// <summary>
/// Builds a minimal SteamCMD <c>workshop_build_item.vdf</c> for manual or scripted uploads.
/// </summary>
public sealed class VdfGeneratorService
{
	public string BuildWorkshopItemVdf(
		string contentFolder,
		string? previewFile,
		ulong? publishedFileId)
	{
		var id = publishedFileId is null or 0
			? "\"0\""
			: $"\"{publishedFileId.Value}\"";

		var preview = string.IsNullOrWhiteSpace(previewFile) ? "\"\"" : $"\"{Escape(previewFile)}\"";

		return
			$"\"workshopitem\"\n{{\n" +
			$"\t\"appid\" \"{SteamConstants.DataCenterAppId}\"\n" +
			$"\t\"publishedfileid\" {id}\n" +
			$"\t\"content\" \"{Escape(contentFolder)}\"\n" +
			$"\t\"preview\" {preview}\n" +
			"}";
	}

	private static string Escape(string path)
	{
		return path.Replace('\\', '/').Replace("\"", "\\\"", StringComparison.Ordinal);
	}
}

