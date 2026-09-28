using GregModmanager.Models;
using GregModmanager.Services;

namespace GregModmanager.Tests;

public class PublishSyncTests : IDisposable
{
    private readonly string _root = Path.Combine(Path.GetTempPath(), "gm-synctest-" + Guid.NewGuid().ToString("N"));
    private readonly WorkspaceService _workspace = new();

    public PublishSyncTests()
    {
        Directory.CreateDirectory(Path.Combine(_root, "content"));
        File.WriteAllText(Path.Combine(_root, "content", "mod.dll"), "v1-bytes");
        File.WriteAllText(Path.Combine(_root, "preview.png"), "preview-bytes");
    }

    public void Dispose()
    {
        try { Directory.Delete(_root, true); } catch { /* best effort */ }
    }

    private WorkshopMetadata BaseMeta() => new()
    {
        PublishedFileId = 1234567890,
        Title = "Sync Test Mod",
        Description = "Plain description without keywords.",
        Visibility = "Public",
        Version = "1.0.0",
        PreviewImageRelativePath = "preview.png",
        Tags = new List<string> { "mod" },
        NeedsMelonLoader = false,
        Needsgreg = false,
    };

    [Fact]
    public void ComputePublishHash_Deterministic()
    {
        var meta = BaseMeta();
        var h1 = WorkspaceService.ComputePublishHash(_root, meta, meta.Description);
        var h2 = WorkspaceService.ComputePublishHash(_root, meta, meta.Description);
        Assert.NotEqual("", h1);
        Assert.Equal(h1, h2);
    }

    [Fact]
    public void ComputePublishHash_ChangesWhenContentChanges()
    {
        var meta = BaseMeta();
        var before = WorkspaceService.ComputePublishHash(_root, meta, meta.Description);
        File.WriteAllText(Path.Combine(_root, "content", "mod.dll"), "v2-bytes-changed");
        var after = WorkspaceService.ComputePublishHash(_root, meta, meta.Description);
        Assert.NotEqual(before, after);
    }

    [Fact]
    public void ComputePublishHash_ChangesWhenMetadataChanges()
    {
        var meta = BaseMeta();
        var before = WorkspaceService.ComputePublishHash(_root, meta, meta.Description);
        meta.Title = "Sync Test Mod Renamed";
        var after = WorkspaceService.ComputePublishHash(_root, meta, meta.Description);
        Assert.NotEqual(before, after);
    }

    [Fact]
    public void ComputePublishHash_MissingContent_ReturnsEmpty()
    {
        var meta = BaseMeta();
        Assert.Equal("", WorkspaceService.ComputePublishHash(Path.Combine(_root, "nope"), meta, meta.Description));
    }

    [Fact]
    public void GetSyncState_Unpublished_WhenIdZero()
    {
        var meta = BaseMeta();
        meta.PublishedFileId = 0;
        WorkspaceService.SaveMetadata(_root, meta);
        Assert.Equal(WorkspaceService.ProjectSyncState.Unpublished, _workspace.GetSyncState(_root));
    }

    [Fact]
    public void GetSyncState_FullRoundtrip()
    {
        var meta = BaseMeta();
        WorkspaceService.SaveMetadata(_root, meta);
        Assert.Equal(WorkspaceService.ProjectSyncState.Unknown, _workspace.GetSyncState(_root));

        // Simulate a successful publish: record the hash.
        meta.LastPublishedHash = WorkspaceService.ComputePublishHash(_root, meta, meta.Description);
        Assert.NotEqual("", meta.LastPublishedHash);
        WorkspaceService.SaveMetadata(_root, meta);
        Assert.Equal(WorkspaceService.ProjectSyncState.Synced, _workspace.GetSyncState(_root));

        // Touch content -> Modified.
        File.WriteAllText(Path.Combine(_root, "content", "mod.dll"), "v3-bytes");
        Assert.Equal(WorkspaceService.ProjectSyncState.Modified, _workspace.GetSyncState(_root));
    }
}
