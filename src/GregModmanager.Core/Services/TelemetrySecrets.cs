namespace GregModmanager.Services;

internal static class TelemetrySecrets
{
    // Diese Werte werden während des CI/CD-Builds durch echte Daten ersetzt.
    // Lokal werden sie einfach als Platzhalter belassen (Telemetrie ist dann inaktiv).
    public const string LokiUrl = "http://telemetry.datacentermods.com/loki/api/v1/push";
    public const string LokiUser = "managerclient";
    public const string LokiPass = "99Feuerwehrauto!";
    public const string LokiTenant = "managerclient";
}






