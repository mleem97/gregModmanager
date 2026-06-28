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
			// Use Task.Run to avoid deadlocks from sync-over-async on UI threads
			var plugins = Task.Run(async () =>
			{
				var response = await _http.GetAsync(endpoint).ConfigureAwait(false);
				response.EnsureSuccessStatusCode();

				var json = await response.Content.ReadAsStringAsync().ConfigureAwait(false);

				return JsonSerializer.Deserialize<List<PluginPackageInfo>>(json, AppJsonContext.SharedOptions);
			}).GetAwaiter().GetResult();

			return plugins ?? new List<PluginPackageInfo>();
		}
		catch (Exception ex)
		{
			throw new InvalidOperationException($"Beta-Kanal: Fehler beim Abrufen der Plugins (URL: {endpoint}). Details: {ex.Message}", ex);
		}
	}
}
