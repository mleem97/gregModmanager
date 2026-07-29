using System.Globalization;
using System.Resources;
using GregModmanager.Services;

namespace GregModmanager.Localization;

/// <summary>
/// Short-named static accessor for localized strings.
/// Reads from <c>Resources/Strings/AppStrings.resx</c> satellite assemblies.
/// Culture is set once at startup; language changes require a restart.
/// </summary>
public static class S
{
	private const string LanguagePreferenceKey = "AppLanguage";

	private static readonly ResourceManager Rm =
		new("GregModmanager.Resources.Strings.AppStrings", typeof(S).Assembly);

	public static IPreferences Preferences { get; set; } = new JsonFilePreferences();

	public static string Get(string key)
		=> Rm.GetString(key, CultureInfo.CurrentUICulture) ?? key;

	public static string Format(string key, params object[] args)
		=> string.Format(CultureInfo.CurrentCulture, Get(key), args);

	/// <summary>
	/// Call once at app startup (before any UI) to apply the saved language preference.
	/// Falls back to system language if nothing is saved.
	/// </summary>
	public static void ApplySavedCulture()
	{
		var saved = Preferences.GetString(LanguagePreferenceKey, "");
		if (string.IsNullOrWhiteSpace(saved)) return;

		try
		{
			var culture = new CultureInfo(saved);
			CultureInfo.DefaultThreadCurrentCulture = culture;
			CultureInfo.DefaultThreadCurrentUICulture = culture;
			CultureInfo.CurrentCulture = culture;
			CultureInfo.CurrentUICulture = culture;
		}
		catch
		{
			// invalid culture string — ignore, keep system default
		}
	}

	public static void SetLanguage(string cultureCode)
	{
		Preferences.SetString(LanguagePreferenceKey, cultureCode);
	}

	public static string GetSavedLanguage()
		=> Preferences.GetString(LanguagePreferenceKey, "");

	public static readonly (string Code, string DisplayName)[] SupportedLanguages =
	[
		("", "System default"),
        ("en", "English"),
        ("de", "Deutsch"),
        ("fr", "Français"),
        ("es", "Español"),
		("it", "Italiano"),
		("ja", "日本語"),
		("pl", "Polski"),
		("ru", "Русский"),
		("zh", "中文"),
	];
}
