namespace GregModmanager.Models;

public enum DependencyStatus
{
	Ok,
	Warning,
	Missing,
}

/// <summary>Single dependency check (e.g. "MelonLoader installed").</summary>
public sealed class DependencyCheckResult
{
	public required string Label { get; init; }
	public required DependencyStatus Status { get; init; }
	public required string Detail { get; init; }

	/// <summary>Optional filesystem path the user can open for context.</summary>
	public string? Path { get; init; }
}

