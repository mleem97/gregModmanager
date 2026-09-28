namespace GregModmanager.Services;

public interface IPreferences
{
    string GetString(string key, string defaultValue);
    bool GetBool(string key, bool defaultValue);
    int GetInt(string key, int defaultValue);
    void SetString(string key, string value);
    void SetBool(string key, bool value);
    void SetInt(string key, int value);
    void Remove(string key);
}
