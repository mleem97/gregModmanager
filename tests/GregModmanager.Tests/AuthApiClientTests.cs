using System.Net;
using System.Text;
using GregModmanager.Services.Auth;

namespace GregModmanager.Tests;

public class AuthApiClientTests
{
    [Fact]
    public async Task GetLoginUrlAsync_UsesHomeFallback_WhenPublicRouteIsUnavailable()
    {
        var handler = new RecordingHandler(request =>
            request.RequestUri?.Host == "datacentermods.com"
                ? new HttpResponseMessage(HttpStatusCode.NotFound)
                : new HttpResponseMessage(HttpStatusCode.Redirect));
        var client = CreateClient(handler);

        var result = await client.GetLoginUrlAsync("request-123");

        Assert.StartsWith("https://datacentermods.home/auth/login?", result, StringComparison.Ordinal);
        Assert.Equal(2, handler.Requests.Count);
    }

    [Fact]
    public async Task ExchangeCallbackCodeAsync_FallsBackAfterTransportFailure()
    {
        var handler = new RecordingHandler(request =>
        {
            if (request.RequestUri?.Host == "datacentermods.com")
                throw new HttpRequestException("DNS unavailable");

            return JsonResponse(HttpStatusCode.OK,
                """{"accessToken":"token","sessionId":"session","user":{"subjectId":"user","email":"user@example.com","displayName":"User","roles":["user"],"tenant":"gregframework"}}""");
        });
        var client = CreateClient(handler);

        var result = await client.ExchangeCallbackCodeAsync("request", "code", "state", "nonce", "signature");

        Assert.NotNull(result);
        Assert.Equal("token", result.AccessToken);
        Assert.Equal(2, handler.Requests.Count);
    }

    [Fact]
    public async Task ExchangeCallbackCodeAsync_DoesNotFailOverOnUnauthorized()
    {
        var handler = new RecordingHandler(_ => new HttpResponseMessage(HttpStatusCode.Unauthorized));
        var client = CreateClient(handler);

        var result = await client.ExchangeCallbackCodeAsync("request", "code", "state", "nonce", "signature");

        Assert.Null(result);
        Assert.Single(handler.Requests);
        Assert.Equal("datacentermods.com", handler.Requests[0].Host);
    }

    private static AuthApiClient CreateClient(RecordingHandler handler)
    {
        return new AuthApiClient(
            new HttpClient(handler) { Timeout = TimeSpan.FromSeconds(2) },
            new[]
            {
                "https://datacentermods.com/auth/login?redirect_uri={0}&requestId={1}",
                "https://datacentermods.home/auth/login?redirect_uri={0}&requestId={1}"
            },
            new[]
            {
                "https://datacentermods.com/auth",
                "https://datacentermods.home/auth"
            });
    }

    private static HttpResponseMessage JsonResponse(HttpStatusCode status, string json) => new(status)
    {
        Content = new StringContent(json, Encoding.UTF8, "application/json")
    };

    private sealed class RecordingHandler(Func<HttpRequestMessage, HttpResponseMessage> respond) : HttpMessageHandler
    {
        public List<Uri> Requests { get; } = [];

        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            Requests.Add(request.RequestUri!);
            return Task.FromResult(respond(request));
        }
    }
}
