use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use sena_core::{CircuitState, Error, Result};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: RwLock<Option<DateTime<Utc>>>,
    config: CircuitConfig,
}

#[derive(Debug, Clone)]
pub struct CircuitConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub half_open_timeout: Duration,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            half_open_timeout: Duration::from_secs(30),
        }
    }
}

impl From<&sena_core::config::CircuitBreakerConfig> for CircuitConfig {
    fn from(config: &sena_core::config::CircuitBreakerConfig) -> Self {
        Self {
            failure_threshold: config.failure_threshold,
            success_threshold: config.success_threshold,
            half_open_timeout: Duration::from_secs(config.half_open_timeout_secs),
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            config,
        }
    }

    pub fn state(&self) -> CircuitState {
        let mut state = self.state.write();

        if *state == CircuitState::Open {
            if let Some(last_failure) = *self.last_failure_time.read() {
                let elapsed = Utc::now().signed_duration_since(last_failure);
                if elapsed.to_std().unwrap_or(Duration::ZERO) >= self.config.half_open_timeout {
                    *state = CircuitState::HalfOpen;
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
        }

        *state
    }

    pub fn can_execute(&self) -> bool {
        self.state() != CircuitState::Open
    }

    pub fn check(&self) -> Result<()> {
        if self.can_execute() {
            Ok(())
        } else {
            Err(Error::provider("circuit breaker is open"))
        }
    }

    pub fn record_success(&self) {
        let mut state = self.state.write();

        match *state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                    tracing::info!("circuit breaker closed after successful requests");
                }
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&self) {
        let mut state = self.state.write();

        match *state {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.failure_threshold {
                    *state = CircuitState::Open;
                    *self.last_failure_time.write() = Some(Utc::now());
                    tracing::warn!(
                        failures = count,
                        "circuit breaker opened after {} failures",
                        count
                    );
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                *self.last_failure_time.write() = Some(Utc::now());
                self.success_count.store(0, Ordering::SeqCst);
                tracing::warn!("circuit breaker reopened after failure in half-open state");
            }
            CircuitState::Open => {}
        }
    }

    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.last_failure_time.write() = None;
    }

    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::SeqCst)
    }

    pub fn success_count(&self) -> u32 {
        self.success_count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_transitions() {
        let config = CircuitConfig {
            failure_threshold: 3,
            success_threshold: 2,
            half_open_timeout: Duration::from_millis(100),
        };

        let breaker = CircuitBreaker::new(config);

        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);

        assert!(!breaker.can_execute());
    }

    #[test]
    fn test_success_resets_failures() {
        let config = CircuitConfig {
            failure_threshold: 3,
            ..Default::default()
        };

        let breaker = CircuitBreaker::new(config);

        breaker.record_failure();
        breaker.record_failure();
        breaker.record_success();

        assert_eq!(breaker.failure_count(), 0);
        assert_eq!(breaker.state(), CircuitState::Closed);
    }
}
