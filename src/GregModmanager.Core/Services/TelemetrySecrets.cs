namespace GregModmanager.Services;

internal static class TelemetrySecrets
{
    // Diese Werte werden während des CI/CD-Builds durch echte Daten ersetzt.
    // Lokal werden sie einfach als Platzhalter belassen (Telemetrie ist dann inaktiv).
    public const string LokiUrl = "__LOKI_URL__";
    public const string LokiUser = "__LOKI_USER__";
    public const string LokiPass = "__LOKI_PASS__";
    public const string LokiTenant = "__LOKI_TENANT__";
}
