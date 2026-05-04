namespace GregModmanager.Services;

public sealed class SteamPublishRateLimiter
{
    private readonly object _gate = new();
    private readonly Queue<DateTimeOffset> _attempts = new();
    private DateTimeOffset _lastAttempt = DateTimeOffset.MinValue;

    public static SteamPublishRateLimiter Shared { get; } = new();

    public TimeSpan MinInterval { get; init; } = TimeSpan.FromSeconds(30);

    public TimeSpan RollingWindow { get; init; } = TimeSpan.FromMinutes(10);

    public int MaxAttemptsPerWindow { get; init; } = 5;

    public bool TryAcquire(out TimeSpan retryAfter)
    {
        lock (_gate)
        {
            var now = DateTimeOffset.UtcNow;
            var windowStart = now - RollingWindow;
            while (_attempts.Count > 0 && _attempts.Peek() < windowStart)
            {
                _attempts.Dequeue();
            }

            var untilIntervalReady = (_lastAttempt + MinInterval) - now;
            if (untilIntervalReady > TimeSpan.Zero)
            {
                retryAfter = untilIntervalReady;
                return false;
            }

            if (_attempts.Count >= MaxAttemptsPerWindow)
            {
                var oldestInWindow = _attempts.Peek();
                var untilWindowReady = (oldestInWindow + RollingWindow) - now;
                retryAfter = untilWindowReady > TimeSpan.Zero ? untilWindowReady : TimeSpan.FromSeconds(1);
                return false;
            }

            _lastAttempt = now;
            _attempts.Enqueue(now);
            retryAfter = TimeSpan.Zero;
            return true;
        }
    }
}
