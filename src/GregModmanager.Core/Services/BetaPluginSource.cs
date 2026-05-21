using System.Text.Json;
using GregModmanager.Models;
using GregModmanager.Localization;
using System.Collections.Generic;
using System.Net.Http;
using System;

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
		var url = S.Preferences.GetString(PrefKeyBetaServerUrl, string.Empty);
		if (string.IsNullOrWhiteSpace(url))
		{
			throw new InvalidOperationException(
				"Beta-Kanal: Server-URL ist noch nicht konfiguriert. " +
				"Setze die URL unter Einstellungen (Preferences-Key: greg_beta_server_url).");
		}

		var endpoint = url.TrimEnd('/') + "/api/plugins";

		try
		{
			using var request = new HttpRequestMessage(HttpMethod.Get, endpoint);
			using var response = _http.Send(request);
			response.EnsureSuccessStatusCode();

			using var stream = response.Content.ReadAsStream();

			var plugins = JsonSerializer.Deserialize(stream, AppJsonContext.Default.ListPluginPackageInfo);

			return plugins ?? new List<PluginPackageInfo>();
		}
		catch (Exception ex)
		{
			throw new InvalidOperationException($"Beta-Kanal: Fehler beim Abrufen der Plugins (URL: {endpoint}). Details: {ex.Message}", ex);
		}
	}
}
