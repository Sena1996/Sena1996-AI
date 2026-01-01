use sena_core::{HealthCheck, HealthStatus};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::Utc;

pub struct HealthRegistry {
    checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn update(&self, component: &str, check: HealthCheck) {
        self.checks.write().insert(component.to_string(), check);
    }

    pub fn get(&self, component: &str) -> Option<HealthCheck> {
        self.checks.read().get(component).cloned()
    }

    pub fn all(&self) -> HashMap<String, HealthCheck> {
        self.checks.read().clone()
    }

    pub fn overall_status(&self) -> HealthStatus {
        let checks = self.checks.read();

        if checks.is_empty() {
            return HealthStatus::Healthy;
        }

        let mut has_degraded = false;

        for check in checks.values() {
            match check.status {
                HealthStatus::Unhealthy => return HealthStatus::Unhealthy,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => {}
            }
        }

        if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let checks = self.checks.read();
        let overall = self.overall_status();

        serde_json::json!({
            "status": format!("{:?}", overall).to_lowercase(),
            "timestamp": Utc::now().to_rfc3339(),
            "components": checks.iter().map(|(name, check)| {
                (name.clone(), serde_json::json!({
                    "status": format!("{:?}", check.status).to_lowercase(),
                    "latency_ms": check.latency_ms,
                    "message": check.message,
                    "last_check": check.last_check.to_rfc3339()
                }))
            }).collect::<HashMap<_, _>>()
        })
    }

    pub fn mark_healthy(&self, component: &str) {
        self.update(component, HealthCheck::healthy());
    }

    pub fn mark_degraded(&self, component: &str, message: &str) {
        self.update(component, HealthCheck::degraded(message));
    }

    pub fn mark_unhealthy(&self, component: &str, message: &str) {
        self.update(component, HealthCheck::unhealthy(message));
    }

    pub fn mark_with_latency(&self, component: &str, latency_ms: u64) {
        self.update(component, HealthCheck::healthy().with_latency(latency_ms));
    }
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HealthRegistry {
    fn clone(&self) -> Self {
        Self {
            checks: self.checks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_registry() {
        let registry = HealthRegistry::new();

        registry.mark_healthy("provider1");
        registry.mark_healthy("provider2");

        assert_eq!(registry.overall_status(), HealthStatus::Healthy);

        registry.mark_degraded("provider1", "slow response");
        assert_eq!(registry.overall_status(), HealthStatus::Degraded);

        registry.mark_unhealthy("provider2", "connection failed");
        assert_eq!(registry.overall_status(), HealthStatus::Unhealthy);
    }
}
