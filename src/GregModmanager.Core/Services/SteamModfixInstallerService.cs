using System.Net.Http.Headers;
using System.Reflection;
using System.Text.Json;

namespace GregModmanager.Services;

public sealed record SteamModfixInstallResult(bool Success, bool Changed, string Message, Version? Version);

/// <summary>Installs the published SteamModfix runtime plugin used to register external Workshop sources.</summary>
public sealed class SteamModfixInstallerService
{
	private const string ApiUrl = "https://api.github.com/repos/mleem97/gregPlugin.SteamModfix/releases/latest";
	private const string AssetName = "gregPlugin.SteamModfix.dll";
	private readonly HttpClient _http = new() { Timeout = TimeSpan.FromMinutes(5) };

	public SteamModfixInstallerService()
	{
		_http.DefaultRequestHeaders.UserAgent.Add(new ProductInfoHeaderValue("gregModmanager", "1.6"));
		_http.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));
	}

	public async Task<SteamModfixInstallResult> EnsureCurrentAsync(
		string gameRoot,
		IProgress<string>? progress = null,
		CancellationToken cancellationToken = default)
	{
		if (string.IsNullOrWhiteSpace(gameRoot) || !Directory.Exists(gameRoot))
			return new(false, false, "Der Spielordner wurde nicht gefunden.", null);

		var target = Path.Combine(gameRoot, "Plugins", AssetName);
		try
		{
			progress?.Report("Suche aktuelle SteamModfix-Version…");
			using var response = await _http.GetAsync(ApiUrl, cancellationToken);
			response.EnsureSuccessStatusCode();
			using var json = JsonDocument.Parse(await response.Content.ReadAsStreamAsync(cancellationToken));
			var tag = json.RootElement.GetProperty("tag_name").GetString() ?? "";
			Version.TryParse(tag.TrimStart('v', 'V'), out var latest);
			var current = ReadVersion(target);
			if (latest is not null && current is not null && current >= latest)
				return new(true, false, $"SteamModfix {current} ist bereits aktuell.", current);

			var assetNames = OperatingSystem.IsWindows()
				? new[] { AssetName, "gregPlugin.SteamModfix.Windows.dll" }
				: OperatingSystem.IsMacOS()
					? new[] { AssetName, "gregPlugin.SteamModfix.MacOS.dll", "gregPlugin.SteamModfix.macOS.dll" }
					: new[] { AssetName, "gregPlugin.SteamModfix.Linux.dll" };
			var asset = json.RootElement.GetProperty("assets").EnumerateArray()
				.FirstOrDefault(x => assetNames.Any(name => string.Equals(x.GetProperty("name").GetString(), name, StringComparison.OrdinalIgnoreCase)));
			if (asset.ValueKind == JsonValueKind.Undefined)
				return new(false, false, $"Das SteamModfix-Release enthält keine kompatible DLL für {Environment.OSVersion.Platform}.", latest);

			var url = asset.GetProperty("browser_download_url").GetString();
			if (string.IsNullOrWhiteSpace(url)) return new(false, false, "SteamModfix-Asset ohne Download-URL.", latest);
			Directory.CreateDirectory(Path.GetDirectoryName(target)!);
			progress?.Report("Installiere SteamModfix in den Spielordner…");
			var temp = target + $".{Guid.NewGuid():N}.tmp";
			try
			{
				await using var source = await _http.GetStreamAsync(url, cancellationToken);
				await using var destination = File.Create(temp);
				await source.CopyToAsync(destination, cancellationToken);
				File.Move(temp, target, overwrite: true);
			}
			finally
			{
				try { File.Delete(temp); } catch { }
			}
			var installed = ReadVersion(target) ?? latest;
			return new(File.Exists(target), true, $"SteamModfix {installed?.ToString() ?? tag} wurde installiert.", installed);
		}
		catch (OperationCanceledException)
		{
			return new(false, false, "Die SteamModfix-Installation wurde abgebrochen.", null);
		}
		catch (Exception ex) when (ex is HttpRequestException or IOException or JsonException)
		{
			return new(false, false, $"SteamModfix konnte nicht installiert werden: {ex.Message}", null);
		}
	}

	private static Version? ReadVersion(string path)
	{
		if (!File.Exists(path)) return null;
		try { return AssemblyName.GetAssemblyName(path).Version; } catch { return null; }
	}
}
