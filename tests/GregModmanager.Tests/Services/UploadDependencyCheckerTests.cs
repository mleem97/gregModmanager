using GregModmanager.Models;
using GregModmanager.Services;
using GregModmanager.Steam;

namespace GregModmanager.Tests.Services;

public class UploadDependencyCheckerTests : IDisposable
{
    private readonly string _tempPath;

    public UploadDependencyCheckerTests()
    {
        _tempPath = Path.Combine(Path.GetTempPath(), Path.GetRandomFileName());
        Directory.CreateDirectory(_tempPath);
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempPath))
        {
            try
            {
                Directory.Delete(_tempPath, true);
            }
            catch
            {
                // Ignore cleanup errors in tests
            }
        }
    }

    private WorkshopMetadata CreateValidMetadata()
    {
        return new WorkshopMetadata
        {
            Title = "Valid Title",
            Description = "Valid Description",
            Visibility = "Public",
            Tags = ["Tag1"],
            PreviewImageRelativePath = "preview.png"
        };
    }

    [Fact]
    public void Check_MissingContentFolder_ReturnsError()
    {
        // Arrange
        var metadata = CreateValidMetadata();

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Content folder" && r.Severity == UploadCheckSeverity.Error);
        Assert.False(UploadDependencyChecker.IsReadyToUpload(results));
    }

    [Fact]
    public void Check_EmptyContentFolder_ReturnsError()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Content folder" && r.Severity == UploadCheckSeverity.Error);
        Assert.False(UploadDependencyChecker.IsReadyToUpload(results));
    }

    [Fact]
    public void Check_MissingNativeConfigJson_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "config.json" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_PresentNativeConfigJson_ReturnsOk()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "config.json"), "{}");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "config.json" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("   ")]
    public void Check_EmptyTitle_ReturnsError(string? emptyTitle)
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Title = emptyTitle!;

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Title" && r.Severity == UploadCheckSeverity.Error);
        Assert.False(UploadDependencyChecker.IsReadyToUpload(results));
    }

    [Fact]
    public void Check_LongTitle_ReturnsError()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Title = new string('A', SteamConstants.MaxTitleLength + 1);

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Title" && r.Severity == UploadCheckSeverity.Error);
        Assert.False(UploadDependencyChecker.IsReadyToUpload(results));
    }

    [Fact]
    public void Check_ValidTitle_ReturnsOk()
    {
        // Arrange
        var metadata = CreateValidMetadata();

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Title" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Fact]
    public void Check_EmptyDescription_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Description = "";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Description" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_LongDescription_ReturnsError()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Description = new string('A', SteamConstants.MaxDescriptionLength + 1);

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Description" && r.Severity == UploadCheckSeverity.Error);
    }

    [Theory]
    [InlineData("Public")]
    [InlineData("FriendsOnly")]
    [InlineData("Private")]
    public void Check_ValidVisibility_ReturnsOk(string visibility)
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Visibility = visibility;

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Visibility" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Fact]
    public void Check_InvalidVisibility_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Visibility = "InvalidVisibility";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Visibility" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_MissingPreviewImage_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.PreviewImageRelativePath = "preview.png";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Preview image" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_LargePreviewImage_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.PreviewImageRelativePath = "preview.png";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        var previewPath = Path.Combine(_tempPath, "preview.png");
        // Create a fake large file using sparse file if possible, or just write some data.
        using (var fs = new FileStream(previewPath, FileMode.Create, FileAccess.Write, FileShare.None))
        {
            fs.SetLength(1_048_576 + 10);
        }

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Preview image" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_ValidPreviewImage_ReturnsOk()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.PreviewImageRelativePath = "preview.png";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        var previewPath = Path.Combine(_tempPath, "preview.png");
        File.WriteAllText(previewPath, "fake image content");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Preview image" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Fact]
    public void Check_EmptyTags_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Tags = new List<string>();

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Tags" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_ValidTags_ReturnsOk()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Tags = ["Tag1", "Tag2"];

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "Tags" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Fact]
    public void Check_FirstPublishEmptyChangelog_ReturnsError()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.PublishedFileId = 0;

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata, changeLog: "");

        // Assert
        Assert.Contains(results, r => r.Label == "Changelog" && r.Severity == UploadCheckSeverity.Error);
    }

    [Fact]
    public void Check_UpdateEmptyChangelog_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.PublishedFileId = 12345;

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata, changeLog: "");

        // Assert
        Assert.Contains(results, r => r.Label == "Changelog" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_ValidChangelog_ReturnsOk()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.PublishedFileId = 12345;

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata, changeLog: "Initial release");

        // Assert
        Assert.Contains(results, r => r.Label == "Changelog" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Fact]
    public void Check_GregFrameworkDependency_NeedsGregWithoutDescription_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Needsgreg = true;
        metadata.Description = "Some standard mod without mentioning the framework";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "GregFramework (greg)" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_GregFrameworkDependency_NeedsGregWithDescription_ReturnsOk()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Needsgreg = true;
        metadata.Description = "This mod requires gregCoreModFramework to work.";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "GregFramework (greg)" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Fact]
    public void Check_GregFrameworkDependency_DoesNotNeedGregButMentionsIt_ReturnsWarning()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Needsgreg = false;
        metadata.Description = "This mod is similar to gregCoreModFramework.";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "GregFramework (greg)" && r.Severity == UploadCheckSeverity.Warning);
    }

    [Fact]
    public void Check_GregFrameworkDependency_DoesNotNeedGregAndNoMentions_ReturnsOk()
    {
        // Arrange
        var metadata = CreateValidMetadata();
        metadata.Needsgreg = false;
        metadata.Description = "Standard mod without any mentions.";

        var contentDir = Path.Combine(_tempPath, "content");
        Directory.CreateDirectory(contentDir);
        File.WriteAllText(Path.Combine(contentDir, "somefile.txt"), "test");

        // Act
        var results = UploadDependencyChecker.Check(_tempPath, metadata);

        // Assert
        Assert.Contains(results, r => r.Label == "GregFramework (greg)" && r.Severity == UploadCheckSeverity.Ok);
    }

    [Fact]
    public void IsReadyToUpload_NoErrors_ReturnsTrue()
    {
        // Arrange
        var results = new List<UploadCheckResult>
        {
            new UploadCheckResult { Label = "Test", Severity = UploadCheckSeverity.Ok, Detail = "" },
            new UploadCheckResult { Label = "Test", Severity = UploadCheckSeverity.Warning, Detail = "" }
        };

        // Act
        var isReady = UploadDependencyChecker.IsReadyToUpload(results);

        // Assert
        Assert.True(isReady);
    }

    [Fact]
    public void IsReadyToUpload_WithErrors_ReturnsFalse()
    {
        // Arrange
        var results = new List<UploadCheckResult>
        {
            new UploadCheckResult { Label = "Test", Severity = UploadCheckSeverity.Ok, Detail = "" },
            new UploadCheckResult { Label = "Test", Severity = UploadCheckSeverity.Error, Detail = "" }
        };

        // Act
        var isReady = UploadDependencyChecker.IsReadyToUpload(results);

        // Assert
        Assert.False(isReady);
    }
}
