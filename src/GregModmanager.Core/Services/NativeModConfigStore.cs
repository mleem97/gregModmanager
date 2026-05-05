using System.Text.Json;
using System.Text.Json.Serialization;
using GregModmanager.Models;

namespace GregModmanager.Services;

public static class NativeModConfigStore
{
	private static readonly JsonSerializerOptions JsonOptions = new()
	{
		WriteIndented = true,
		DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
		PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
		ReadCommentHandling = JsonCommentHandling.Skip,
		AllowTrailingCommas = true,
	};

	public static string ConfigJsonPath(string projectRoot) =>
		Path.Combine(projectRoot, "content", "config.json");

	public static string ModOptionsJsonPath(string projectRoot) =>
		Path.Combine(projectRoot, "content", "modconfig.json");

	public static NativeModConfig LoadConfig(string projectRoot)
	{
		var path = ConfigJsonPath(projectRoot);
		if (!File.Exists(path))
		{
			return new NativeModConfig();
		}

		var json = File.ReadAllText(path);
		return JsonSerializer.Deserialize<NativeModConfig>(json, JsonOptions) ?? new NativeModConfig();
	}

	public static void SaveConfig(string projectRoot, NativeModConfig config)
	{
		var path = ConfigJsonPath(projectRoot);
		var dir = Path.GetDirectoryName(path);
		if (!string.IsNullOrEmpty(dir))
		{
			Directory.CreateDirectory(dir);
		}

		var json = JsonSerializer.Serialize(config, JsonOptions);
		File.WriteAllText(path, json);
	}

	public static ModOptionsConfigFile LoadModOptions(string projectRoot)
	{
		var path = ModOptionsJsonPath(projectRoot);
		if (!File.Exists(path))
		{
			return new ModOptionsConfigFile();
		}

		var json = File.ReadAllText(path);
		return JsonSerializer.Deserialize<ModOptionsConfigFile>(json, JsonOptions) ?? new ModOptionsConfigFile();
	}

	public static void SaveModOptions(string projectRoot, ModOptionsConfigFile config)
	{
		var path = ModOptionsJsonPath(projectRoot);
		var dir = Path.GetDirectoryName(path);
		if (!string.IsNullOrEmpty(dir))
		{
			Directory.CreateDirectory(dir);
		}

		var json = JsonSerializer.Serialize(config, JsonOptions);
		File.WriteAllText(path, json);
	}
}

