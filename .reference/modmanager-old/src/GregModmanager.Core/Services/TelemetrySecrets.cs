namespace GregModmanager.Services;

internal static class TelemetrySecrets
{
    // Credentials are injected via environment variables at runtime.
    // Falls back to empty strings (telemetry inactive) if not configured.
    public static string LokiUrl => Environment.GetEnvironmentVariable("GREG_LOKI_URL") ?? "";
    public static string LokiUser => Environment.GetEnvironmentVariable("GREG_LOKI_USER") ?? "";
    public static string LokiPass => Environment.GetEnvironmentVariable("GREG_LOKI_PASS") ?? "";
    public static string LokiTenant => Environment.GetEnvironmentVariable("GREG_LOKI_TENANT") ?? "";
}



































