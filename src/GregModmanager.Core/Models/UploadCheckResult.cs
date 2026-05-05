namespace GregModmanager.Models;

public sealed class UploadCheckResult
{
	public required string Label { get; init; }
	public required UploadCheckSeverity Severity { get; init; }
	public required string Detail { get; init; }
}

public enum UploadCheckSeverity
{
	Ok,
	Warning,
	Error,
}

