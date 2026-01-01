use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl RetryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let base_delay = self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32 - 1);
        let max_delay = self.max_delay.as_secs_f64();
        let capped_delay = base_delay.min(max_delay);

        let jitter_range = capped_delay * self.jitter;
        let jitter = if jitter_range > 0.0 {
            rand::thread_rng().gen_range(-jitter_range..jitter_range)
        } else {
            0.0
        };

        Duration::from_secs_f64((capped_delay + jitter).max(0.0))
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

pub struct RetryPolicy {
    config: RetryConfig,
    attempt: u32,
}

impl RetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self { config, attempt: 0 }
    }

    pub fn next_attempt(&mut self) -> Option<Duration> {
        if self.config.should_retry(self.attempt) {
            let delay = self.config.delay_for_attempt(self.attempt);
            self.attempt += 1;
            Some(delay)
        } else {
            None
        }
    }

    pub fn current_attempt(&self) -> u32 {
        self.attempt
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: 0.0,
        };

        assert_eq!(config.delay_for_attempt(0), Duration::ZERO);
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            max_retries: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: 0.0,
        };

        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(5));
    }

    #[test]
    fn test_retry_policy() {
        let config = RetryConfig {
            max_retries: 3,
            ..Default::default()
        };

        let mut policy = RetryPolicy::new(config);

        assert!(policy.next_attempt().is_some());
        assert!(policy.next_attempt().is_some());
        assert!(policy.next_attempt().is_some());
        assert!(policy.next_attempt().is_none());
    }
}
