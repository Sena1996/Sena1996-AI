//! Base Component
//! All controller components inherit from this trait

use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Component metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub name: String,
    pub initialized: bool,
    pub uptime_seconds: Option<f64>,
    pub calls: u64,
    pub errors: u64,
    pub total_time_ms: f64,
    pub average_call_time_ms: f64,
}

/// Component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub name: String,
    pub initialized: bool,
    pub healthy: bool,
    pub details: HashMap<String, serde_json::Value>,
}

/// Base trait for all controller components
/// Provides common functionality and lifecycle management
pub trait BaseComponent: Send + Sync {
    /// Get component name
    fn name(&self) -> &str;

    /// Initialize component
    fn initialize(&mut self) -> Result<(), String>;

    /// Cleanup resources
    fn cleanup(&mut self) -> Result<(), String>;

    /// Get component status
    fn get_status(&self) -> ComponentStatus;

    /// Check if component is initialized
    fn is_initialized(&self) -> bool;

    /// Get component metrics
    fn get_metrics(&self) -> ComponentMetrics;
}

/// Common state for components
#[derive(Debug, Clone)]
pub struct ComponentState {
    pub name: String,
    pub initialized: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub calls: u64,
    pub errors: u64,
    pub total_time_ms: f64,
}

impl ComponentState {
    /// Create new component state
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: false,
            start_time: None,
            calls: 0,
            errors: 0,
            total_time_ms: 0.0,
        }
    }

    /// Mark as initialized
    pub fn mark_initialized(&mut self) {
        self.initialized = true;
        self.start_time = Some(Utc::now());
    }

    /// Record a call
    pub fn record_call(&mut self, duration_ms: f64, is_error: bool) {
        self.calls += 1;
        self.total_time_ms += duration_ms;
        if is_error {
            self.errors += 1;
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> Option<f64> {
        self.start_time.map(|start| {
            let now = Utc::now();
            (now - start).num_milliseconds() as f64 / 1000.0
        })
    }

    /// Get metrics
    pub fn metrics(&self) -> ComponentMetrics {
        let avg_time = if self.calls > 0 {
            self.total_time_ms / self.calls as f64
        } else {
            0.0
        };

        ComponentMetrics {
            name: self.name.clone(),
            initialized: self.initialized,
            uptime_seconds: self.uptime_seconds(),
            calls: self.calls,
            errors: self.errors,
            total_time_ms: self.total_time_ms,
            average_call_time_ms: avg_time,
        }
    }
}

/// Macro to implement common BaseComponent methods
#[macro_export]
macro_rules! impl_base_component {
    ($type:ty, $state_field:ident) => {
        impl BaseComponent for $type {
            fn name(&self) -> &str {
                &self.$state_field.name
            }

            fn is_initialized(&self) -> bool {
                self.$state_field.initialized
            }

            fn get_metrics(&self) -> ComponentMetrics {
                self.$state_field.metrics()
            }
        }
    };
}

/// Timer guard for measuring operation duration
pub struct CallTimer {
    start: Instant,
}

impl CallTimer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_state_creation() {
        let state = ComponentState::new("test");
        assert_eq!(state.name, "test");
        assert!(!state.initialized);
        assert!(state.start_time.is_none());
    }

    #[test]
    fn test_component_state_initialization() {
        let mut state = ComponentState::new("test");
        state.mark_initialized();
        assert!(state.initialized);
        assert!(state.start_time.is_some());
    }

    #[test]
    fn test_record_call() {
        let mut state = ComponentState::new("test");
        state.record_call(100.0, false);
        state.record_call(50.0, true);

        assert_eq!(state.calls, 2);
        assert_eq!(state.errors, 1);
        assert_eq!(state.total_time_ms, 150.0);
    }

    #[test]
    fn test_metrics() {
        let mut state = ComponentState::new("test");
        state.record_call(100.0, false);
        state.record_call(200.0, false);

        let metrics = state.metrics();
        assert_eq!(metrics.name, "test");
        assert_eq!(metrics.calls, 2);
        assert_eq!(metrics.total_time_ms, 300.0);
        assert_eq!(metrics.average_call_time_ms, 150.0);
    }

    #[test]
    fn test_call_timer() {
        let timer = CallTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10.0);
    }
}
