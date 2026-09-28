using System;

namespace GregModmanager.Models;

public record CrashReport
{
    public DateTime Timestamp { get; init; }
    public string? AppVersion { get; init; }
    public string? OsVersion { get; init; }
    public string? ExceptionType { get; init; }
    public string? Message { get; init; }
    public string? StackTrace { get; init; }
    public string? Source { get; init; }
    public int ProcessId { get; init; }
}
