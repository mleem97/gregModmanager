using System.Xml.Linq;

namespace GregModmanager.Tests;

public class ProjectSanityTests
{
    private static readonly string ProjectFile = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "GregModmanager.Core.csproj"));

    [Fact]
    public void ProjectFile_Exists()
    {
        Assert.True(File.Exists(ProjectFile), "Expected project file at: {ProjectFile}");
    }

    [Fact]
    public void ProjectFile_Defines_TargetFramework()
    {
        var document = XDocument.Load(ProjectFile);
        var hasTargetFramework = document.Descendants().Any(node => node.Name.LocalName is "TargetFramework" or "TargetFrameworks");
        Assert.True(hasTargetFramework, "Expected TargetFramework or TargetFrameworks in project file.");
    }
}


