//! Publish rate limiter (`SteamPublishRateLimiter` port).
//!
//! Token bucket: at most `max_attempts` publishes per rolling window, with a
//! minimum interval between attempts. Thread-safe via a mutex.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Default: 30 s between attempts, max 5 per 10 minutes.
pub const DEFAULT_MIN_INTERVAL: Duration = Duration::from_secs(30);
/// Default rolling window.
pub const DEFAULT_ROLLING_WINDOW: Duration = Duration::from_secs(600);
/// Default attempts per window.
pub const DEFAULT_MAX_ATTEMPTS: usize = 5;

/// Shared limiter (mirrors the C# singleton).
#[derive(Debug, Clone)]
pub struct PublishRateLimiter {
    inner: Arc<Mutex<LimiterInner>>,
    min_interval: Duration,
    window: Duration,
    max_attempts: usize,
}

#[derive(Debug, Default)]
struct LimiterInner {
    attempts: VecDeque<Instant>,
}

impl PublishRateLimiter {
    /// Creates a limiter with custom bounds.
    pub fn new(min_interval: Duration, window: Duration, max_attempts: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LimiterInner::default())),
            min_interval,
            window,
            max_attempts,
        }
    }

    /// Shared instance with default bounds.
    pub fn shared() -> Self {
        Self::new(
            DEFAULT_MIN_INTERVAL,
            DEFAULT_ROLLING_WINDOW,
            DEFAULT_MAX_ATTEMPTS,
        )
    }

    /// Tries to acquire a publish slot.
    /// `Ok(())` on success, `Err(retry_after)` when cooling down.
    pub fn try_acquire(&self) -> std::result::Result<(), Duration> {
        let mut inner = self.inner.lock().expect("limiter lock");
        let now = Instant::now();
        while inner
            .attempts
            .front()
            .is_some_and(|t| now.duration_since(*t) >= self.window)
        {
            inner.attempts.pop_front();
        }
        if let Some(last) = inner.attempts.back() {
            let since = now.duration_since(*last);
            if since < self.min_interval {
                return Err(self.min_interval - since);
            }
        }
        if inner.attempts.len() >= self.max_attempts {
            if let Some(oldest) = inner.attempts.front() {
                let wait = self.window.saturating_sub(now.duration_since(*oldest));
                return Err(wait.max(Duration::from_secs(1)));
            }
        }
        inner.attempts.push_back(now);
        Ok(())
    }
}

impl Default for PublishRateLimiter {
    fn default() -> Self {
        Self::shared()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforces_interval_and_window() {
        let limiter =
            PublishRateLimiter::new(Duration::from_millis(50), Duration::from_millis(200), 2);
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_err());
        std::thread::sleep(Duration::from_millis(60));
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_err());
    }
}
