using System;

namespace GregModmanager.Models.Install;

public class InstallIntentContext
{
    public string IntentId { get; set; } = string.Empty;
    public string PackageId { get; set; } = string.Empty;
    public string SourceUrl { get; set; } = string.Empty;
    public string SubjectId { get; set; } = string.Empty;
    public long ExpiresAt { get; set; }
    public string[] RequiredRoles { get; set; } = Array.Empty<string>();
    public string Signature { get; set; } = string.Empty;
}
