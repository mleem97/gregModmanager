using System.Text.Json;
using GregModmanager.Models;

namespace GregModmanager.Services;

/// <summary>
/// Beta distribution channel served from a custom backend.
/// Configure base URL via Preferences.
/// </summary>
public sealed class BetaPluginSource : IgregPluginChannelSource
{
	/// <summary>Preferences key for the beta server base URL.</summary>
	public const string PrefKeyBetaServerUrl = "greg_beta_server_url";

	private static readonly HttpClient _http = new();

	public string ChannelName => "beta";

	public IReadOnlyList<PluginPackageInfo> ListPlugins()
	{
#if WINDOWS || ANDROID || IOS || MACCATALYST
		var url = Preferences.Default.Get(PrefKeyBetaServerUrl, string.Empty);
#else
		var url = "";
#endif
		if (string.IsNullOrWhiteSpace(url))
		{
			throw new InvalidOperationException(
				"Beta-Kanal: Server-URL ist noch nicht konfiguriert. " +
				"Setze die URL unter Einstellungen (Preferences-Key: greg_beta_server_url).");
		}

		var endpoint = url.TrimEnd('/') + "/api/plugins";

		try
		{
			var response = _http.GetAsync(endpoint).GetAwaiter().GetResult();
			response.EnsureSuccessStatusCode();

			var json = response.Content.ReadAsStringAsync().GetAwaiter().GetResult();

			var options = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
			var plugins = JsonSerializer.Deserialize<List<PluginPackageInfo>>(json, options);

			return plugins ?? new List<PluginPackageInfo>();
		}
		catch (Exception ex)
		{
			throw new InvalidOperationException($"Beta-Kanal: Fehler beim Abrufen der Plugins (URL: {endpoint}). Details: {ex.Message}", ex);
		}
	}
}
