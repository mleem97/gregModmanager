using System.Net;

namespace GregModmanager.Services.Auth;

/// <summary>
/// Retries authentication requests against an explicitly trusted fallback host
/// when the primary endpoint is unavailable. Authentication and validation
/// failures are returned unchanged and never trigger host failover.
/// </summary>
internal static class AuthEndpointFallback
{
    public static async Task<HttpResponseMessage> SendAsync(
        HttpClient http,
        IReadOnlyList<string> endpoints,
        Func<string, HttpRequestMessage> requestFactory)
    {
        if (endpoints.Count == 0)
            throw new InvalidOperationException("No authentication endpoint is configured.");

        Exception? lastTransportError = null;
        for (var index = 0; index < endpoints.Count; index++)
        {
            var isLast = index == endpoints.Count - 1;
            try
            {
                using var request = requestFactory(endpoints[index]);
                var response = await http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead);
                if (!isLast && IsEndpointUnavailable(response.StatusCode))
                {
                    response.Dispose();
                    continue;
                }

                return response;
            }
            catch (Exception ex) when (!isLast && IsTransportFailure(ex))
            {
                lastTransportError = ex;
            }
            catch (Exception ex) when (isLast && IsTransportFailure(ex))
            {
                lastTransportError = ex;
            }
        }

        throw lastTransportError ?? new HttpRequestException("All configured authentication endpoints are unavailable.");
    }

    public static async Task<string> ResolveReachableUrlAsync(HttpClient http, IReadOnlyList<string> urls)
    {
        if (urls.Count == 0)
            throw new InvalidOperationException("No authentication login URL is configured.");

        Exception? lastTransportError = null;
        for (var index = 0; index < urls.Count; index++)
        {
            var isLast = index == urls.Count - 1;
            try
            {
                using var request = new HttpRequestMessage(HttpMethod.Get, urls[index]);
                using var response = await http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead);
                if (!isLast && IsEndpointUnavailable(response.StatusCode))
                    continue;

                // Return the handoff URL, not a redirect target followed by the
                // probe client. The user's browser must execute the auth flow.
                return urls[index];
            }
            catch (Exception ex) when (IsTransportFailure(ex))
            {
                lastTransportError = ex;
                if (isLast)
                    break;
            }
        }

        throw lastTransportError ?? new HttpRequestException("All configured login endpoints are unavailable.");
    }

    private static bool IsTransportFailure(Exception exception) =>
        exception is HttpRequestException or TaskCanceledException;

    private static bool IsEndpointUnavailable(HttpStatusCode statusCode) => statusCode is
        HttpStatusCode.NotFound or
        HttpStatusCode.RequestTimeout or
        HttpStatusCode.InternalServerError or
        HttpStatusCode.BadGateway or
        HttpStatusCode.ServiceUnavailable or
        HttpStatusCode.GatewayTimeout;
}
