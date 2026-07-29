using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using GregModmanager.Models;

namespace GregModmanager.Services;

public sealed class TelemetryService
{
    private static readonly string LokiUrl = AppSettings.DefaultLokiUrl;
    
    private static readonly string LokiUser = Environment.GetEnvironmentVariable("TELEMETRY_USER") ?? TelemetrySecrets.LokiUser;
    private static readonly string LokiPass = Environment.GetEnvironmentVariable("TELEMETRY_PASS") ?? TelemetrySecrets.LokiPass;
    private static readonly string LokiTenant = Environment.GetEnvironmentVariable("TELEMETRY_TENANT") ?? TelemetrySecrets.LokiTenant;

    private readonly HttpClient _http = new();
    private readonly string _appVersion;

    public TelemetryService()
    {
        _appVersion = System.Reflection.Assembly.GetExecutingAssembly().GetName().Version?.ToString() ?? "0.0.0.0";
        
        // Tenant ID for Loki (X-Scope-OrgID)
        var tenant = LokiTenant == "__LOKI_TENANT__" ? "managerclient" : LokiTenant;
        _http.DefaultRequestHeaders.Add("X-Scope-OrgID", tenant);

        // Basic Auth
        if (LokiUser != "__LOKI_USER__" && LokiPass != "__LOKI_PASS__")
        {
            var authBytes = System.Text.Encoding.ASCII.GetBytes($"{LokiUser}:{LokiPass}");
            _http.DefaultRequestHeaders.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Basic", Convert.ToBase64String(authBytes));
        }
    }

    public async Task ReportCrashesAsync()
    {
        if (!AppSettings.IsTelemetryEnabled()) return;
        try
        {
            var reportsDir = AppFileLog.ReportsDir;
            if (!Directory.Exists(reportsDir)) return;

            var files = Directory.GetFiles(reportsDir, "crash-*.json");
            foreach (var file in files)
            {
                try
                {
                    var content = await File.ReadAllTextAsync(file);
                    var report = JsonSerializer.Deserialize(content, AppJsonContext.Default.CrashReport);
                    if (report != null)
                    {
                        var success = await PushToLokiAsync("crash", content, new Dictionary<string, string>
                        {
                            { "level", "critical" },
                            { "exception", report.ExceptionType ?? "unknown" }
                        });

                        if (success)
                        {
                            File.Delete(file);
                            AppFileLog.Info($"Telemetry: Uploaded and deleted crash report {Path.GetFileName(file)}");
                        }
                    }
                }
                catch (Exception ex)
                {
                    AppFileLog.Error($"Telemetry: Failed to process crash report {file}", ex);
                }
            }
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Telemetry: Error in ReportCrashesAsync", ex);
        }
    }

    public Task TrackEventAsync(string eventName, object payload)
    {
        return TrackEventAsync(eventName, payload, extraLabels: null);
    }

    public async Task TrackEventAsync(string eventName, object payload, Dictionary<string, string>? extraLabels)
    {
        if (!AppSettings.IsTelemetryEnabled()) return;
        
        var labels = new Dictionary<string, string>
        {
            { "event", eventName },
            { "level", "info" },
            { "machine_id", GetMachineId() }
        };

        if (extraLabels != null)
        {
            foreach (var kvp in extraLabels) labels[kvp.Key] = kvp.Value;
        }

        string message;
        try
        {
            message = payload switch
            {
                SyncCollectionEvent sync => JsonSerializer.Serialize(sync, AppJsonContext.Default.SyncCollectionEvent),
                TelemetryStartupEvent startup => JsonSerializer.Serialize(startup, AppJsonContext.Default.TelemetryStartupEvent),
                // Unknown telemetry payloads must not require reflection metadata in a trimmed build.
                // Typed payloads above carry the structured data; unknown events remain diagnostic text.
                _ => payload?.ToString() ?? "{}"
            };
        }
        catch (Exception ex)
        {
            AppFileLog.Warn($"Telemetry: Could not serialize event '{eventName}': {ex.Message}");
            return;
        }

        await PushToLokiAsync(eventName, message, labels);
    }

    private static string GetMachineId()
    {
        // Einfache anonyme ID basierend auf dem Maschinennamen/User (gehasht)
        var raw = $"{Environment.MachineName}-{Environment.UserName}";
        using var sha = System.Security.Cryptography.SHA256.Create();
        var hash = sha.ComputeHash(System.Text.Encoding.UTF8.GetBytes(raw));
        return BitConverter.ToString(hash).Replace("-", string.Empty, StringComparison.Ordinal).Substring(0, 12).ToLower(CultureInfo.InvariantCulture);
    }

    private async Task<bool> PushToLokiAsync(string job, string line, Dictionary<string, string> labels)
    {
        if (!AppSettings.IsTelemetryEnabled()) return false;
        if (!Uri.TryCreate(LokiUrl, UriKind.Absolute, out var lokiUri) ||
            (lokiUri.Scheme != Uri.UriSchemeHttp && lokiUri.Scheme != Uri.UriSchemeHttps))
        {
            return false;
        }
        try
        {
            // Loki labels
            var streamLabels = new Dictionary<string, string>
            {
                { "app", "gregModmanager" },
                { "job", job },
                { "version", _appVersion },
                { "os", Environment.OSVersion.Platform.ToString() }
            };

            foreach (var kvp in labels) streamLabels[kvp.Key] = kvp.Value;

            // Loki values: [ "nanoseconds", "line" ]
            var timestampNs = (DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() * 1000000).ToString(CultureInfo.InvariantCulture);
            
            var request = new LokiPushRequest
            {
                Streams = new List<LokiStream>
                {
                    new LokiStream
                    {
                        Stream = streamLabels,
                        Values = new List<List<string>>
                        {
                            new List<string> { timestampNs, line }
                        }
                    }
                }
            };

            var json = JsonSerializer.Serialize(request, AppJsonContext.Default.LokiPushRequest);
            var response = await _http.PostAsync(lokiUri, new StringContent(json, System.Text.Encoding.UTF8, "application/json"));
            if (!response.IsSuccessStatusCode)
            {
                var error = await response.Content.ReadAsStringAsync();
                AppFileLog.Warn($"Telemetry: Loki push failed with {response.StatusCode}: {error}");
                return false;
            }

            return true;
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Telemetry: Exception pushing to Loki", ex);
            return false;
        }
    }

}
