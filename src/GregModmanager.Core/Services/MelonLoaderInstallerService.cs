using System.IO.Compression;
using System.Net.Http.Headers;
using System.Reflection;
using System.Text.Json;

namespace GregModmanager.Services;

public sealed record MelonLoaderInstallState(bool Installed, Version? Version, string? AssemblyPath);

public sealed record MelonLoaderInstallResult(
	bool Success,
	bool Changed,
	string Message,
	Version? InstalledVersion);

/// <summary>Installs or updates the official MelonLoader archive into a selected game root.</summary>
public sealed class MelonLoaderInstallerService
{
	private readonly HttpClient _http;

	public MelonLoaderInstallerService()
	{
		_http = new HttpClient { Timeout = TimeSpan.FromMinutes(10) };
		_http.DefaultRequestHeaders.UserAgent.Add(new ProductInfoHeaderValue("gregModmanager", "1.6"));
		_http.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));
	}

	public MelonLoaderInstallState Detect(string gameRoot)
	{
		var path = Path.Combine(gameRoot, "MelonLoader", "net6", "MelonLoader.dll");
		if (!File.Exists(path)) return new(false, null, null);

		try
		{
			return new(true, AssemblyName.GetAssemblyName(path).Version, path);
		}
		catch
		{
			return new(true, null, path);
		}
	}

	public async Task<MelonLoaderInstallResult> EnsureCurrentAsync(
		string gameRoot,
		IProgress<string>? progress = null,
		CancellationToken cancellationToken = default)
	{
		if (string.IsNullOrWhiteSpace(gameRoot) || !Directory.Exists(gameRoot))
			return new(false, false, "Der Spielordner wurde nicht gefunden.", null);

		if (OperatingSystem.IsMacOS())
			return new(false, false, "Für macOS ist in diesem Installer kein MelonLoader-Archiv hinterlegt.", null);

		try
		{
			progress?.Report("Suche aktuelle MelonLoader-Version…");
			var release = await GetLatestReleaseAsync(cancellationToken);
			var installed = Detect(gameRoot);
			if (installed.Version is not null && installed.Version >= release.Version)
				return new(true, false, $"MelonLoader {installed.Version} ist bereits aktuell.", installed.Version);

			progress?.Report($"Lade MelonLoader {release.Version} herunter…");
			var archivePath = Path.Combine(Path.GetTempPath(), $"greg-melonloader-{Guid.NewGuid():N}.zip");
			try
			{
				await using (var source = await _http.GetStreamAsync(release.AssetUrl, cancellationToken))
				await using (var target = File.Create(archivePath))
					await source.CopyToAsync(target, cancellationToken);

				var staging = Path.Combine(Path.GetTempPath(), $"greg-melonloader-stage-{Guid.NewGuid():N}");
				try
				{
					Directory.CreateDirectory(staging);
					ExtractSafely(archivePath, staging);
					var sourceRoot = Directory.Exists(Path.Combine(staging, "MelonLoader"))
						? staging
						: throw new InvalidDataException("Das MelonLoader-Archiv enthält keinen MelonLoader-Ordner.");

					progress?.Report("Installiere MelonLoader in den Spielordner…");
					CopyDirectory(sourceRoot, gameRoot);
					var result = Detect(gameRoot);
					return new(result.Installed, true,
						result.Installed ? $"MelonLoader {release.Version} wurde installiert." : "MelonLoader wurde entpackt, aber die DLL fehlt.",
						result.Version ?? release.Version);
				}
				finally
				{
					TryDeleteDirectory(staging);
				}
			}
			finally
			{
				try { File.Delete(archivePath); } catch { /* temporary cleanup is best effort */ }
			}
		}
		catch (OperationCanceledException)
		{
			return new(false, false, "Die MelonLoader-Installation wurde abgebrochen.", null);
		}
		catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or JsonException)
		{
			return new(false, false, $"MelonLoader konnte nicht installiert werden: {ex.Message}", null);
		}
	}

	private async Task<(Version Version, string AssetUrl)> GetLatestReleaseAsync(CancellationToken cancellationToken)
	{
		var apiUrl = AppSettings.MelonLoaderLatestReleaseApiUrl;
		using var response = await _http.GetAsync(apiUrl, cancellationToken);
		response.EnsureSuccessStatusCode();
		await using var stream = await response.Content.ReadAsStreamAsync(cancellationToken);
		using var json = await JsonDocument.ParseAsync(stream, cancellationToken: cancellationToken);
		var root = json.RootElement;
		var tag = root.GetProperty("tag_name").GetString() ?? throw new InvalidDataException("GitHub-Release ohne tag_name.");
		if (!Version.TryParse(tag.TrimStart('v', 'V'), out var version))
			throw new InvalidDataException($"Ungültige MelonLoader-Version: {tag}");

		var assetName = OperatingSystem.IsWindows() ? "MelonLoader.x64.zip" : "MelonLoader.Linux.x64.zip";
		foreach (var asset in root.GetProperty("assets").EnumerateArray())
		{
			if (string.Equals(asset.GetProperty("name").GetString(), assetName, StringComparison.Ordinal))
				return (version, asset.GetProperty("browser_download_url").GetString() ?? throw new InvalidDataException("Release-Asset ohne URL."));
		}
		throw new InvalidDataException($"Kein passendes MelonLoader-Asset ({assetName}) gefunden.");
	}

	private static void ExtractSafely(string archivePath, string destination)
	{
		using var archive = ZipFile.OpenRead(archivePath);
		var root = Path.GetFullPath(destination) + Path.DirectorySeparatorChar;
		foreach (var entry in archive.Entries)
		{
			var target = Path.GetFullPath(Path.Combine(destination, entry.FullName));
			if (!target.StartsWith(root, StringComparison.Ordinal))
				throw new InvalidDataException($"Unsicherer Archivpfad: {entry.FullName}");
			if (string.IsNullOrEmpty(entry.Name))
			{
				Directory.CreateDirectory(target);
				continue;
			}
			Directory.CreateDirectory(Path.GetDirectoryName(target)!);
			entry.ExtractToFile(target, overwrite: true);
		}
	}

	private static void CopyDirectory(string source, string destination)
	{
		foreach (var directory in Directory.EnumerateDirectories(source, "*", SearchOption.AllDirectories))
			Directory.CreateDirectory(Path.Combine(destination, Path.GetRelativePath(source, directory)));
		foreach (var file in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
		{
			var target = Path.Combine(destination, Path.GetRelativePath(source, file));
			Directory.CreateDirectory(Path.GetDirectoryName(target)!);
			File.Copy(file, target, overwrite: true);
		}
	}

	private static void TryDeleteDirectory(string path)
	{
		try { if (Directory.Exists(path)) Directory.Delete(path, recursive: true); } catch { }
	}
}
