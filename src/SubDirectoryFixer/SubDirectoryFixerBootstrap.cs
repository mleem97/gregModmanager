using System.Reflection;

namespace SubDirectoryFixer;

public static class SubDirectoryFixerBootstrap
{
    public static string Version => Assembly.GetExecutingAssembly().GetName().Version?.ToString() ?? "1.0.0.0";

    public static string Describe()
    {
        return "SubDirectoryFixer helper assembly for gregModmanager deployment.";
    }
}