namespace GregModmanager.Models;

/// <summary>Row in "My uploads" — a Workshop item published by the current Steam user.</summary>
public sealed class PublishedWorkshopItemVm
{
	public required ulong PublishedFileId { get; init; }

	public required string Title { get; init; }

	public DateTime Updated { get; init; }
}

