use chrono::{DateTime, Utc};
use dashmap::DashMap;
use sena_core::{Error, Result};
use std::time::Duration;

pub struct RateLimiter {
    requests_per_minute: u32,
    burst_size: u32,
    buckets: DashMap<String, TokenBucket>,
}

struct TokenBucket {
    tokens: f64,
    last_update: DateTime<Utc>,
    capacity: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            tokens: capacity as f64,
            last_update: Utc::now(),
            capacity: capacity as f64,
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();

        let tokens_needed = tokens as f64;
        if self.tokens >= tokens_needed {
            self.tokens -= tokens_needed;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Utc::now();
        let elapsed = (now - self.last_update).num_milliseconds() as f64 / 1000.0;
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_update = now;
    }

    fn time_until_available(&self, tokens: u32) -> Duration {
        let tokens_needed = tokens as f64;
        if self.tokens >= tokens_needed {
            Duration::ZERO
        } else {
            let deficit = tokens_needed - self.tokens;
            Duration::from_secs_f64(deficit / self.refill_rate)
        }
    }
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        Self {
            requests_per_minute,
            burst_size,
            buckets: DashMap::new(),
        }
    }

    pub fn from_config(config: &sena_core::config::RateLimitConfig) -> Self {
        Self::new(config.requests_per_minute, config.burst_size)
    }

    pub fn try_acquire(&self, key: &str) -> Result<()> {
        self.try_acquire_n(key, 1)
    }

    pub fn try_acquire_n(&self, key: &str, tokens: u32) -> Result<()> {
        let refill_rate = self.requests_per_minute as f64 / 60.0;

        let mut bucket = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.burst_size, refill_rate));

        if bucket.try_consume(tokens) {
            Ok(())
        } else {
            let wait_time = bucket.time_until_available(tokens);
            Err(Error::rate_limit(format!(
                "rate limit exceeded, retry after {:.1}s",
                wait_time.as_secs_f64()
            )))
        }
    }

    pub fn check(&self, key: &str) -> bool {
        self.check_n(key, 1)
    }

    pub fn check_n(&self, key: &str, tokens: u32) -> bool {
        let refill_rate = self.requests_per_minute as f64 / 60.0;

        let mut bucket = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.burst_size, refill_rate));

        bucket.refill();
        bucket.tokens >= tokens as f64
    }

    pub fn remaining(&self, key: &str) -> u32 {
        let refill_rate = self.requests_per_minute as f64 / 60.0;

        let mut bucket = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.burst_size, refill_rate));

        bucket.refill();
        bucket.tokens as u32
    }

    pub fn reset(&self, key: &str) {
        self.buckets.remove(key);
    }

    pub fn clear(&self) {
        self.buckets.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(60, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(60, 5);

        for _ in 0..5 {
            assert!(limiter.try_acquire("user1").is_ok());
        }

        assert!(limiter.try_acquire("user1").is_err());

        assert!(limiter.try_acquire("user2").is_ok());
    }

    #[test]
    fn test_remaining() {
        let limiter = RateLimiter::new(60, 10);
        assert_eq!(limiter.remaining("user1"), 10);
        limiter.try_acquire("user1").unwrap();
        assert_eq!(limiter.remaining("user1"), 9);
    }
}
