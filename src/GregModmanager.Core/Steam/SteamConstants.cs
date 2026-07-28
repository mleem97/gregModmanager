namespace GregModmanager.Steam;

public static class SteamConstants
{
	/// <summary>Steam App ID for Data Center (gregCoreMF).</summary>
	public static uint DataCenterAppId => 4170200;

	public static int MaxTitleLength => 128;

	public static int MaxDescriptionLength => 8000;

	/// <summary>Steam Workshop's maximum main preview image size: 1 MiB.</summary>
	public static long MaxPreviewImageBytes => 1_048_576;
}
