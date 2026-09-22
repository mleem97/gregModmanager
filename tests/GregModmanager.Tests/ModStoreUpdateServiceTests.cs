using System.Net;
using System.Text;
using GregModmanager.Models;
using GregModmanager.Services;

namespace GregModmanager.Tests;

public sealed class ModStoreUpdateServiceTests
{
    [Fact]
    public async Task CatalogFallsBackWhenPrimaryIsUnavailable()
    {
        var handler = new StubHandler(request => request.RequestUri!.Host == "datacentermods.com"
            ? new HttpResponseMessage(HttpStatusCode.ServiceUnavailable)
            : new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{\"schemaVersion\":1,\"mods\":[]}", Encoding.UTF8, "application/json")
            });
        var service = new ModStoreUpdateService(new HttpClient(handler), ["https://datacentermods.com", "https://datacentermods.home"]);

        var catalog = await service.GetCatalogAsync();

        Assert.Empty(catalog.Mods);
        Assert.Equal(2, handler.Requests.Count);
        Assert.Equal("datacentermods.home", handler.Requests[1].Host);
    }

    [Fact]
    public async Task InstallRejectsReleaseWithoutChecksumBeforeDownloading()
    {
        var handler = new StubHandler(_ => throw new InvalidOperationException("Network must not be used"));
        var service = new ModStoreUpdateService(new HttpClient(handler), ["https://datacentermods.com"]);
        var release = new ModStoreRelease(Guid.NewGuid(), "1.1.0", "Changes", null, 12, "mod.zip", DateTimeOffset.UtcNow, "https://datacentermods.com/api/v1/releases/1/download");
        var root = Path.Combine(Path.GetTempPath(), $"greg-modstore-test-{Guid.NewGuid():N}");
        Directory.CreateDirectory(root);
        try
        {
            var result = await service.InstallUpdateAsync(new ModStoreUpdate(Guid.NewGuid(), "Example", "1.0.0", release), root);
            Assert.False(result.Success);
            Assert.Contains("SHA-256", result.Message);
            Assert.Empty(handler.Requests);
        }
        finally { Directory.Delete(root, recursive: true); }
    }

    private sealed class StubHandler(Func<HttpRequestMessage, HttpResponseMessage> send) : HttpMessageHandler
    {
        public List<Uri> Requests { get; } = [];
        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            Requests.Add(request.RequestUri!);
            return Task.FromResult(send(request));
        }
    }
}
