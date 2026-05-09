using System.Text.Json;
using GregModmanager.Models;

namespace GregModmanager.Services;

public sealed class JsonFilePreferences : IPreferences
{
    private readonly string _filePath;
    private readonly Dictionary<string, JsonElement> _data = new();
    private readonly object _lock = new();

    public JsonFilePreferences()
    {
        var dir = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "gregModmanager");
        Directory.CreateDirectory(dir);
        _filePath = Path.Combine(dir, "preferences.json");
        Load();
    }

    private void Load()
    {
        if (!File.Exists(_filePath)) return;
        try
        {
            var json = File.ReadAllText(_filePath);
            var dict = JsonSerializer.Deserialize(json, AppJsonContext.Default.DictionaryStringJsonElement);
            if (dict != null)
            {
                foreach (var kv in dict)
                    _data[kv.Key] = kv.Value;
            }
        }
        catch { /* ignore corrupt prefs */ }
    }

    private void Save()
    {
        lock (_lock)
        {
            File.WriteAllText(_filePath, JsonSerializer.Serialize(_data, AppJsonContext.Default.DictionaryStringJsonElement));
        }
    }

    public string GetString(string key, string defaultValue)
    {
        lock (_lock)
        {
            if (_data.TryGetValue(key, out var el) && el.ValueKind == JsonValueKind.String)
                return el.GetString() ?? defaultValue;
            return defaultValue;
        }
    }

    public bool GetBool(string key, bool defaultValue)
    {
        lock (_lock)
        {
            if (_data.TryGetValue(key, out var el) && (el.ValueKind == JsonValueKind.True || el.ValueKind == JsonValueKind.False))
                return el.GetBoolean();
            return defaultValue;
        }
    }

    public int GetInt(string key, int defaultValue)
    {
        lock (_lock)
        {
            if (_data.TryGetValue(key, out var el) && el.ValueKind == JsonValueKind.Number)
                return el.GetInt32();
            return defaultValue;
        }
    }

    public void SetString(string key, string value)
    {
        lock (_lock) { _data[key] = JsonSerializer.SerializeToElement(value); Save(); }
    }

    public void SetBool(string key, bool value)
    {
        lock (_lock) { _data[key] = JsonSerializer.SerializeToElement(value); Save(); }
    }

    public void SetInt(string key, int value)
    {
        lock (_lock) { _data[key] = JsonSerializer.SerializeToElement(value); Save(); }
    }

    public void Remove(string key)
    {
        lock (_lock) { _data.Remove(key); Save(); }
    }
}
