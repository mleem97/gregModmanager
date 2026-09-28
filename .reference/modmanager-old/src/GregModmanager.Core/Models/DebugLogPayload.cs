namespace GregModmanager.Models;

public class DebugLogPayload
{
    public string? BaseDir { get; set; }
    public string? TempLog { get; set; }
    public string[]? Args { get; set; }
    public int? ExitCode { get; set; }
    public string? Message { get; set; }
    public string? ExType { get; set; }
    public string? StackTrace { get; set; }
}
