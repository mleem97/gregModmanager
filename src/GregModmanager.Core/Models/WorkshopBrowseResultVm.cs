namespace GregModmanager.Models;

public sealed class WorkshopBrowseResultVm
{
	public List<WorkshopItemDetailVm> Items { get; init; } = new();
	public int TotalResults { get; init; }
	public int CurrentPage { get; init; }
	public bool HasMorePages { get; init; }
}

