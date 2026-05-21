using System;
using Xunit;
using GregModmanager.Services;
using GregModmanager.Localization;
using System.Collections.Generic;

namespace GregModmanager.Tests;

public class BetaPluginSourceTests
{
    private class MockPreferences : IPreferences
    {
        private readonly Dictionary<string, object> _data = new();

        public string GetString(string key, string defaultValue) => _data.TryGetValue(key, out var val) && val is string s ? s : defaultValue;
        public bool GetBool(string key, bool defaultValue) => _data.TryGetValue(key, out var val) && val is bool b ? b : defaultValue;
        public int GetInt(string key, int defaultValue) => _data.TryGetValue(key, out var val) && val is int i ? i : defaultValue;

        public void SetString(string key, string value) => _data[key] = value;
        public void SetBool(string key, bool value) => _data[key] = value;
        public void SetInt(string key, int value) => _data[key] = value;
        public void Remove(string key) => _data.Remove(key);
    }

    [Fact]
    public void ListPlugins_ThrowsWhenUrlNotConfigured()
    {
        var mockPrefs = new MockPreferences();
        S.Preferences = mockPrefs;

        var source = new BetaPluginSource();

        var ex = Assert.Throws<InvalidOperationException>(() => source.ListPlugins());
        Assert.Contains("Server-URL ist noch nicht konfiguriert", ex.Message);
    }

    [Fact]
    public void ListPlugins_ThrowsWhenUrlIsInvalidOrEndpointFails()
    {
        var mockPrefs = new MockPreferences();
        mockPrefs.SetString(BetaPluginSource.PrefKeyBetaServerUrl, "http://localhost:12345/invalid");
        S.Preferences = mockPrefs;

        var source = new BetaPluginSource();

        var ex = Assert.Throws<InvalidOperationException>(() => source.ListPlugins());
        Assert.Contains("Fehler beim Abrufen der Plugins", ex.Message);
        Assert.Contains("http://localhost:12345/invalid/api/plugins", ex.Message);
    }
}
