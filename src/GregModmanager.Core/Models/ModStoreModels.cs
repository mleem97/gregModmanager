namespace GregModmanager.Models;

public sealed record ModStoreRelease(
    Guid Id,
    string Version,
    string Changelog,
    string? ChecksumSha256,
    long? FileSize,
    string FileName,
    DateTimeOffset? PublishedAt,
    string DownloadUrl);

public sealed record ModStoreCatalogItem(
    Guid Id,
    string Slug,
    string Title,
    string Description,
    string Author,
    string Category,
    string[] Tags,
    string? ImageUrl,
    ModStoreRelease Release);

public sealed record ModStoreCatalogResponse(int SchemaVersion, List<ModStoreCatalogItem> Mods);

public sealed record ModStoreInstalledEntry(
    Guid ModId,
    string Title,
    string Version,
    Guid ReleaseId,
    string InstallPath,
    DateTimeOffset InstalledAt);

public sealed record ModStoreInstallManifest(int SchemaVersion, List<ModStoreInstalledEntry> Installed);

public sealed record ModStoreUpdate(
    Guid ModId,
    string Title,
    string InstalledVersion,
    ModStoreRelease Release);

public sealed record ModStoreUpdateResponse(int SchemaVersion, List<ModStoreUpdate> Updates);

public sealed record ModStoreUpdateCheckRequest(List<ModStoreInstalledVersion> Installed);
public sealed record ModStoreInstalledVersion(Guid ModId, string Version);

public sealed record ModStoreInstallResult(bool Success, string Message, string? InstallPath = null);
