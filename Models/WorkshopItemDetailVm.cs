namespace GregModmanager.Models;

public sealed class WorkshopItemDetailVm
{
	public required ulong PublishedFileId { get; init; }
	public required string Title { get; init; }
	public string Description { get; init; } = "";
	public string[] Tags { get; init; } = [];
	public string OwnerName { get; init; } = "";
	public ulong OwnerSteamId { get; init; }
        public string PreviewImageUrl { get; init; } = "";

        public DateTime Created { get; init; }
	public DateTime Updated { get; init; }

	public float Score { get; init; }
	public uint VotesUp { get; init; }
	public uint VotesDown { get; init; }
	public ulong NumSubscriptions { get; init; }
	public ulong NumFavorites { get; init; }
	public ulong NumComments { get; init; }
	public long SizeBytes { get; init; }

	public bool IsSubscribed { get; init; }
	public bool IsInstalled { get; init; }
	public bool NeedsUpdate { get; init; }
	public bool IsDownloading { get; init; }
	public bool IsBanned { get; init; }

	public string Visibility { get; init; } = "Public";
	public string Url { get; init; } = "";
	public string ChangelogUrl { get; init; } = "";
	public string StatsUrl { get; init; } = "";
	public string CommentsUrl { get; init; } = "";

	/// <summary>URLs of additional preview images (gallery screenshots) from Steam.</summary>
	public string[] AdditionalPreviewUrls { get; set; } = [];

	/// <summary>Display label indicating the source ecosystem.</summary>
	public string SourceLabel => IsGregFramework ? "GregFramework" : "Steam Workshop";

	/// <summary>Badge color for source indicator.</summary>
	public string SourceColor => IsGregFramework ? "#61F4D8" : "#5A9E96";

	/// <summary>True when the item appears to belong to the GregFramework ecosystem.</summary>
	public bool IsGregFramework =>
		Tags.Any(t => GregFrameworkTags.Contains(t, StringComparer.OrdinalIgnoreCase));

	private static readonly string[] GregFrameworkTags = ["greg", "framework", "gregCore-mod-framework", "melonloader"];

	/// <summary>Inferred from Steam tags and description text.</summary>
	public string DependencyHintCompact { get; init; } = "";

	/// <summary>Multi-line bullet text for the detail page.</summary>
	public string DependencyHintBlock { get; init; } = "";

	public bool HasDependencyHints { get; init; }
}

