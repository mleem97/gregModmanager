namespace GregModmanager.Models;

public sealed class WorkshopProject
{
	public required string Name { get; init; }

	public required string RootPath { get; init; }

	public required string ContentPath { get; init; }

	public required string MetadataPath { get; init; }

	public bool IsValidLayout { get; init; }
}

