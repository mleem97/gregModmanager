using System.IO.Compression;
using System.Net;
using System.Security.Cryptography;
using System.Text.Json;
using GregModmanager.Models;

namespace GregModmanager.Services;

/// <summary>Downloads and atomically installs versioned native Mod Store releases.</summary>
public sealed class ModStoreUpdateService
{
    private readonly HttpClient _http;
    private readonly IReadOnlyList<string> _baseUrls;

    public ModStoreUpdateService() : this(new HttpClient { Timeout = TimeSpan.FromMinutes(10) }, AppSettings.ModStoreApiBaseUrls) { }

    public ModStoreUpdateService(HttpClient http, IReadOnlyList<string> baseUrls)
    {
        _http = http;
        _baseUrls = baseUrls.Select(value => value.TrimEnd('/')).Distinct(StringComparer.OrdinalIgnoreCase).ToArray();
        if (_baseUrls.Count == 0) throw new ArgumentException("At least one Mod Store endpoint is required.", nameof(baseUrls));
    }

    public Task<ModStoreCatalogResponse> GetCatalogAsync(CancellationToken cancellationToken = default) =>
        GetJsonWithFallbackAsync<ModStoreCatalogResponse>("/api/v1/mods", cancellationToken);

    public async Task<ModStoreUpdateResponse> CheckForUpdatesAsync(string gameRoot, CancellationToken cancellationToken = default)
    {
        var manifest = LoadManifest(gameRoot);
        if (manifest.Installed.Count == 0) return new(1, []);
        var request = new ModStoreUpdateCheckRequest(manifest.Installed.Select(item => new ModStoreInstalledVersion(item.ModId, item.Version)).ToList());
        return await PostJsonWithFallbackAsync<ModStoreUpdateCheckRequest, ModStoreUpdateResponse>("/api/v1/mods/updates/check", request, cancellationToken);
    }

    public Task<ModStoreInstallResult> InstallAsync(ModStoreCatalogItem item, string gameRoot, IProgress<int>? progress = null, CancellationToken cancellationToken = default) =>
        InstallReleaseAsync(item.Id, item.Title, item.Release, gameRoot, progress, cancellationToken);

    public Task<ModStoreInstallResult> InstallUpdateAsync(ModStoreUpdate item, string gameRoot, IProgress<int>? progress = null, CancellationToken cancellationToken = default) =>
        InstallReleaseAsync(item.ModId, item.Title, item.Release, gameRoot, progress, cancellationToken);

    public ModStoreInstallManifest LoadManifest(string gameRoot)
    {
        var path = ManifestPath(gameRoot);
        if (!File.Exists(path)) return new(1, []);
        try { return JsonSerializer.Deserialize(File.ReadAllText(path), AppJsonContext.Default.ModStoreInstallManifest) ?? new(1, []); }
        catch (JsonException) { return new(1, []); }
    }

    private async Task<ModStoreInstallResult> InstallReleaseAsync(Guid modId, string title, ModStoreRelease release, string gameRoot, IProgress<int>? progress, CancellationToken cancellationToken)
    {
        if (string.IsNullOrWhiteSpace(gameRoot) || !Directory.Exists(gameRoot)) return new(false, "The Data Center game folder is not configured.");
        if (string.IsNullOrWhiteSpace(release.ChecksumSha256) || release.ChecksumSha256.Length != 64 || !release.ChecksumSha256.All(Uri.IsHexDigit)) return new(false, "The release has no valid SHA-256 checksum and was not installed.");
        if (!IsTrustedDownloadUrl(release.DownloadUrl)) return new(false, "The release download URL does not belong to a configured Mod Store host.");

        var modsRoot = Path.Combine(Path.GetFullPath(gameRoot), "Mods", "GregStore");
        var destination = Path.Combine(modsRoot, modId.ToString("D"));
        var staging = Path.Combine(modsRoot, $".{modId:D}-{Guid.NewGuid():N}.staging");
        var backup = Path.Combine(modsRoot, $".{modId:D}-{Guid.NewGuid():N}.backup");
        var artifact = Path.Combine(Path.GetTempPath(), $"gregmod-{release.Id:D}.download");
        Directory.CreateDirectory(modsRoot);

        try
        {
            using var response = await _http.GetAsync(release.DownloadUrl, HttpCompletionOption.ResponseHeadersRead, cancellationToken);
            response.EnsureSuccessStatusCode();
            var expectedLength = response.Content.Headers.ContentLength ?? release.FileSize;
            await using (var source = await response.Content.ReadAsStreamAsync(cancellationToken))
            await using (var target = File.Create(artifact))
            {
                var buffer = new byte[81920];
                long copied = 0;
                int read;
                while ((read = await source.ReadAsync(buffer, cancellationToken)) > 0)
                {
                    await target.WriteAsync(buffer.AsMemory(0, read), cancellationToken);
                    copied += read;
                    if (expectedLength > 0) progress?.Report((int)Math.Min(90, copied * 90 / expectedLength.Value));
                }
            }
            if (release.FileSize is > 0 && new FileInfo(artifact).Length != release.FileSize.Value) throw new InvalidDataException("Downloaded file size does not match release metadata.");
            await using var checksumStream = File.OpenRead(artifact);
            var checksum = Convert.ToHexString(await SHA256.HashDataAsync(checksumStream, cancellationToken)).ToLowerInvariant();
            if (!CryptographicOperations.FixedTimeEquals(Convert.FromHexString(checksum), Convert.FromHexString(release.ChecksumSha256))) throw new InvalidDataException("SHA-256 checksum verification failed.");

            Directory.CreateDirectory(staging);
            if (LooksLikeZip(artifact)) ExtractZipSafely(artifact, staging);
            else File.Copy(artifact, Path.Combine(staging, SanitizeFileName(release.FileName)), overwrite: true);
            progress?.Report(95);

            if (Directory.Exists(destination)) Directory.Move(destination, backup);
            try { Directory.Move(staging, destination); }
            catch
            {
                if (Directory.Exists(backup) && !Directory.Exists(destination)) Directory.Move(backup, destination);
                throw;
            }
            if (Directory.Exists(backup)) Directory.Delete(backup, recursive: true);

            SaveManifest(gameRoot, new(modId, title, release.Version, release.Id, destination, DateTimeOffset.UtcNow));
            progress?.Report(100);
            return new(true, $"{title} was installed at version {release.Version}.", destination);
        }
        catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or CryptographicException or UnauthorizedAccessException)
        {
            return new(false, $"Update installation failed: {ex.Message}");
        }
        finally
        {
            TryDeleteFile(artifact);
            TryDeleteDirectory(staging);
            if (Directory.Exists(backup) && Directory.Exists(destination)) TryDeleteDirectory(backup);
        }
    }

    private async Task<T> GetJsonWithFallbackAsync<T>(string path, CancellationToken cancellationToken) where T : class
    {
        Exception? last = null;
        foreach (var baseUrl in _baseUrls)
        {
            try
            {
                using var response = await _http.GetAsync(baseUrl + path, cancellationToken);
                if (IsUnavailable(response.StatusCode)) continue;
                response.EnsureSuccessStatusCode();
                var typeInfo = AppJsonContext.Default.GetTypeInfo(typeof(T)) ?? throw new JsonException($"No JSON metadata for {typeof(T).Name}.");
                return (await JsonSerializer.DeserializeAsync(await response.Content.ReadAsStreamAsync(cancellationToken), typeInfo, cancellationToken) as T) ?? throw new JsonException("The Mod Store returned an empty response.");
            }
            catch (Exception ex) when (ex is HttpRequestException or TaskCanceledException) { last = ex; }
        }
        throw last ?? new HttpRequestException("All configured Mod Store endpoints are unavailable.");
    }

    private async Task<TResponse> PostJsonWithFallbackAsync<TRequest, TResponse>(string path, TRequest body, CancellationToken cancellationToken)
        where TRequest : class
        where TResponse : class
    {
        var requestTypeInfo = AppJsonContext.Default.GetTypeInfo(typeof(TRequest)) ?? throw new JsonException($"No JSON metadata for {typeof(TRequest).Name}.");
        var payload = JsonSerializer.Serialize(body, requestTypeInfo);
        Exception? last = null;
        foreach (var baseUrl in _baseUrls)
        {
            try
            {
                using var content = new StringContent(payload, System.Text.Encoding.UTF8, "application/json");
                using var response = await _http.PostAsync(baseUrl + path, content, cancellationToken);
                if (IsUnavailable(response.StatusCode)) continue;
                response.EnsureSuccessStatusCode();
                var responseTypeInfo = AppJsonContext.Default.GetTypeInfo(typeof(TResponse)) ?? throw new JsonException($"No JSON metadata for {typeof(TResponse).Name}.");
                return (await JsonSerializer.DeserializeAsync(await response.Content.ReadAsStreamAsync(cancellationToken), responseTypeInfo, cancellationToken) as TResponse) ?? throw new JsonException("The Mod Store returned an empty response.");
            }
            catch (Exception ex) when (ex is HttpRequestException or TaskCanceledException) { last = ex; }
        }
        throw last ?? new HttpRequestException("All configured Mod Store endpoints are unavailable.");
    }

    private bool IsTrustedDownloadUrl(string value)
    {
        if (!Uri.TryCreate(value, UriKind.Absolute, out var uri) || uri.Scheme != Uri.UriSchemeHttps) return false;
        return _baseUrls.Select(url => new Uri(url)).Any(baseUri => string.Equals(baseUri.Host, uri.Host, StringComparison.OrdinalIgnoreCase));
    }

    private static bool IsUnavailable(HttpStatusCode status) => status is HttpStatusCode.NotFound or HttpStatusCode.RequestTimeout or HttpStatusCode.InternalServerError or HttpStatusCode.BadGateway or HttpStatusCode.ServiceUnavailable or HttpStatusCode.GatewayTimeout;
    private static string ManifestPath(string root) => Path.Combine(Path.GetFullPath(root), ".greg-modmanager", "modstore-installed.json");

    private static void SaveManifest(string gameRoot, ModStoreInstalledEntry installed)
    {
        var servicePath = ManifestPath(gameRoot);
        Directory.CreateDirectory(Path.GetDirectoryName(servicePath)!);
        ModStoreInstallManifest manifest;
        try { manifest = File.Exists(servicePath) ? JsonSerializer.Deserialize(File.ReadAllText(servicePath), AppJsonContext.Default.ModStoreInstallManifest) ?? new(1, []) : new(1, []); }
        catch (JsonException) { manifest = new(1, []); }
        manifest.Installed.RemoveAll(item => item.ModId == installed.ModId);
        manifest.Installed.Add(installed);
        var temporary = servicePath + ".tmp";
        File.WriteAllText(temporary, JsonSerializer.Serialize(manifest, AppJsonContext.Default.ModStoreInstallManifest));
        File.Move(temporary, servicePath, overwrite: true);
    }

    private static bool LooksLikeZip(string path)
    {
        using var stream = File.OpenRead(path);
        Span<byte> signature = stackalloc byte[4];
        return stream.Read(signature) == 4 && signature[0] == 0x50 && signature[1] == 0x4b;
    }

    private static void ExtractZipSafely(string archivePath, string destination)
    {
        using var archive = ZipFile.OpenRead(archivePath);
        var root = Path.GetFullPath(destination) + Path.DirectorySeparatorChar;
        foreach (var entry in archive.Entries)
        {
            var target = Path.GetFullPath(Path.Combine(destination, entry.FullName));
            if (!target.StartsWith(root, StringComparison.Ordinal)) throw new InvalidDataException($"Unsafe archive path: {entry.FullName}");
            if (string.IsNullOrEmpty(entry.Name)) { Directory.CreateDirectory(target); continue; }
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            entry.ExtractToFile(target, overwrite: true);
        }
    }

    private static string SanitizeFileName(string value) => string.Concat(value.Select(character => Path.GetInvalidFileNameChars().Contains(character) ? '_' : character));
    private static void TryDeleteFile(string path) { try { if (File.Exists(path)) File.Delete(path); } catch { } }
    private static void TryDeleteDirectory(string path) { try { if (Directory.Exists(path)) Directory.Delete(path, recursive: true); } catch { } }
}
