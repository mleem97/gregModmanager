#if HAS_MELONLOADER
using MelonLoader;

[assembly: MelonInfo(typeof(SubDirectoryFixer.SubDirectoryFixerMod), "SubDirectoryFixer", "1.5.0", "gregModmanager")]

namespace SubDirectoryFixer;

public class SubDirectoryFixerMod : MelonMod
{
    public override void OnInitializeMelon()
    {
        MelonLogger.Msg("SubDirectoryFixer plugin loaded by MelonLoader.");
    }
}
#else
namespace SubDirectoryFixer;

// Build-friendly shim when MelonLoader is not available at compile time.
public static class SubDirectoryFixerPluginShim
{
    public static string DescribePlugin() => "SubDirectoryFixer (no MelonLoader available at build time).";
}
#endif
