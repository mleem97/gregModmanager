namespace GregModmanager.Models;

public readonly record struct ImportOutcome(bool Success, string ProjectRoot, string Message)
{
	public static ImportOutcome Ok(string projectRoot) => new(true, projectRoot, "Imported.");

	public static ImportOutcome Fail(string message) => new(false, string.Empty, message);
}

