use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertType {
    CostThreshold,
    ErrorRate,
    LatencySpike,
    ProviderDown,
    TokenBudgetExceeded,
    HallucinationRate,
    CacheHitLow,
    RateLimitApproaching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: uuid::Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    pub alert_type: AlertType,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub window_seconds: u64,
    pub cooldown_seconds: u64,
    pub enabled: bool,
}

impl Default for AlertThreshold {
    fn default() -> Self {
        Self {
            alert_type: AlertType::ErrorRate,
            warning_threshold: 0.05,
            critical_threshold: 0.10,
            window_seconds: 300,
            cooldown_seconds: 600,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
struct AlertState {
    last_triggered: Option<Instant>,
    current_value: f64,
    sample_count: u64,
    window_start: Instant,
}

impl Default for AlertState {
    fn default() -> Self {
        Self {
            last_triggered: None,
            current_value: 0.0,
            sample_count: 0,
            window_start: Instant::now(),
        }
    }
}

pub trait AlertHandler: Send + Sync {
    fn handle(&self, alert: &Alert);
}

pub struct LogAlertHandler;

impl AlertHandler for LogAlertHandler {
    fn handle(&self, alert: &Alert) {
        let severity_str = match alert.severity {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARN",
            AlertSeverity::Critical => "CRIT",
        };
        tracing::warn!(
            "[ALERT][{}] {:?}: {} (value: {:.4}, threshold: {:.4})",
            severity_str,
            alert.alert_type,
            alert.message,
            alert.value,
            alert.threshold
        );
    }
}

pub struct CallbackAlertHandler<F>
where
    F: Fn(&Alert) + Send + Sync,
{
    callback: F,
}

impl<F> CallbackAlertHandler<F>
where
    F: Fn(&Alert) + Send + Sync,
{
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> AlertHandler for CallbackAlertHandler<F>
where
    F: Fn(&Alert) + Send + Sync,
{
    fn handle(&self, alert: &Alert) {
        (self.callback)(alert);
    }
}

pub struct AlertManager {
    thresholds: RwLock<HashMap<AlertType, AlertThreshold>>,
    state: RwLock<HashMap<AlertType, AlertState>>,
    alerts: RwLock<Vec<Alert>>,
    handlers: RwLock<Vec<Arc<dyn AlertHandler>>>,
    max_alerts: usize,
}

impl AlertManager {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();

        thresholds.insert(
            AlertType::CostThreshold,
            AlertThreshold {
                alert_type: AlertType::CostThreshold,
                warning_threshold: 5.0,
                critical_threshold: 10.0,
                window_seconds: 3600,
                cooldown_seconds: 1800,
                enabled: true,
            },
        );

        thresholds.insert(
            AlertType::ErrorRate,
            AlertThreshold {
                alert_type: AlertType::ErrorRate,
                warning_threshold: 0.05,
                critical_threshold: 0.15,
                window_seconds: 300,
                cooldown_seconds: 600,
                enabled: true,
            },
        );

        thresholds.insert(
            AlertType::LatencySpike,
            AlertThreshold {
                alert_type: AlertType::LatencySpike,
                warning_threshold: 2000.0,
                critical_threshold: 5000.0,
                window_seconds: 60,
                cooldown_seconds: 300,
                enabled: true,
            },
        );

        thresholds.insert(
            AlertType::ProviderDown,
            AlertThreshold {
                alert_type: AlertType::ProviderDown,
                warning_threshold: 1.0,
                critical_threshold: 1.0,
                window_seconds: 60,
                cooldown_seconds: 300,
                enabled: true,
            },
        );

        thresholds.insert(
            AlertType::TokenBudgetExceeded,
            AlertThreshold {
                alert_type: AlertType::TokenBudgetExceeded,
                warning_threshold: 0.8,
                critical_threshold: 0.95,
                window_seconds: 86400,
                cooldown_seconds: 3600,
                enabled: true,
            },
        );

        thresholds.insert(
            AlertType::HallucinationRate,
            AlertThreshold {
                alert_type: AlertType::HallucinationRate,
                warning_threshold: 0.05,
                critical_threshold: 0.10,
                window_seconds: 3600,
                cooldown_seconds: 1800,
                enabled: true,
            },
        );

        thresholds.insert(
            AlertType::CacheHitLow,
            AlertThreshold {
                alert_type: AlertType::CacheHitLow,
                warning_threshold: 0.3,
                critical_threshold: 0.1,
                window_seconds: 3600,
                cooldown_seconds: 1800,
                enabled: true,
            },
        );

        thresholds.insert(
            AlertType::RateLimitApproaching,
            AlertThreshold {
                alert_type: AlertType::RateLimitApproaching,
                warning_threshold: 0.8,
                critical_threshold: 0.95,
                window_seconds: 60,
                cooldown_seconds: 120,
                enabled: true,
            },
        );

        Self {
            thresholds: RwLock::new(thresholds),
            state: RwLock::new(HashMap::new()),
            alerts: RwLock::new(Vec::new()),
            handlers: RwLock::new(vec![Arc::new(LogAlertHandler)]),
            max_alerts: 1000,
        }
    }

    pub fn add_handler(&self, handler: Arc<dyn AlertHandler>) {
        self.handlers.write().push(handler);
    }

    pub fn set_threshold(&self, threshold: AlertThreshold) {
        self.thresholds.write().insert(threshold.alert_type, threshold);
    }

    pub fn get_threshold(&self, alert_type: AlertType) -> Option<AlertThreshold> {
        self.thresholds.read().get(&alert_type).cloned()
    }

    pub fn enable_alert(&self, alert_type: AlertType, enabled: bool) {
        if let Some(threshold) = self.thresholds.write().get_mut(&alert_type) {
            threshold.enabled = enabled;
        }
    }

    pub fn record_value(&self, alert_type: AlertType, value: f64) {
        self.record_value_with_metadata(alert_type, value, HashMap::new());
    }

    pub fn record_value_with_metadata(
        &self,
        alert_type: AlertType,
        value: f64,
        metadata: HashMap<String, String>,
    ) {
        let threshold = match self.thresholds.read().get(&alert_type) {
            Some(t) if t.enabled => t.clone(),
            _ => return,
        };

        let mut state = self.state.write();
        let alert_state = state.entry(alert_type).or_default();

        let window = Duration::from_secs(threshold.window_seconds);
        if alert_state.window_start.elapsed() > window {
            alert_state.current_value = value;
            alert_state.sample_count = 1;
            alert_state.window_start = Instant::now();
        } else {
            let total = alert_state.current_value * alert_state.sample_count as f64 + value;
            alert_state.sample_count += 1;
            alert_state.current_value = total / alert_state.sample_count as f64;
        }

        let cooldown = Duration::from_secs(threshold.cooldown_seconds);
        if let Some(last) = alert_state.last_triggered {
            if last.elapsed() < cooldown {
                return;
            }
        }

        let (should_alert, severity, threshold_value) = if alert_type == AlertType::CacheHitLow {
            if value < threshold.critical_threshold {
                (true, AlertSeverity::Critical, threshold.critical_threshold)
            } else if value < threshold.warning_threshold {
                (true, AlertSeverity::Warning, threshold.warning_threshold)
            } else {
                (false, AlertSeverity::Info, 0.0)
            }
        } else if value >= threshold.critical_threshold {
            (true, AlertSeverity::Critical, threshold.critical_threshold)
        } else if value >= threshold.warning_threshold {
            (true, AlertSeverity::Warning, threshold.warning_threshold)
        } else {
            (false, AlertSeverity::Info, 0.0)
        };

        if should_alert {
            alert_state.last_triggered = Some(Instant::now());
            drop(state);

            let message = self.format_alert_message(alert_type, value, threshold_value, severity);

            let alert = Alert {
                id: uuid::Uuid::new_v4(),
                alert_type,
                severity,
                message,
                value,
                threshold: threshold_value,
                triggered_at: chrono::Utc::now(),
                acknowledged: false,
                metadata,
            };

            self.trigger_alert(alert);
        }
    }

    fn format_alert_message(
        &self,
        alert_type: AlertType,
        value: f64,
        threshold: f64,
        severity: AlertSeverity,
    ) -> String {
        let severity_prefix = match severity {
            AlertSeverity::Info => "",
            AlertSeverity::Warning => "Warning: ",
            AlertSeverity::Critical => "CRITICAL: ",
        };

        match alert_type {
            AlertType::CostThreshold => {
                format!("{}Cost threshold exceeded: ${:.2} (limit: ${:.2})", severity_prefix, value, threshold)
            }
            AlertType::ErrorRate => {
                format!("{}Error rate elevated: {:.1}% (threshold: {:.1}%)", severity_prefix, value * 100.0, threshold * 100.0)
            }
            AlertType::LatencySpike => {
                format!("{}Latency spike detected: {:.0}ms (threshold: {:.0}ms)", severity_prefix, value, threshold)
            }
            AlertType::ProviderDown => {
                format!("{}Provider appears to be down", severity_prefix)
            }
            AlertType::TokenBudgetExceeded => {
                format!("{}Token budget usage: {:.1}% (threshold: {:.1}%)", severity_prefix, value * 100.0, threshold * 100.0)
            }
            AlertType::HallucinationRate => {
                format!("{}Hallucination rate elevated: {:.1}% (threshold: {:.1}%)", severity_prefix, value * 100.0, threshold * 100.0)
            }
            AlertType::CacheHitLow => {
                format!("{}Cache hit rate low: {:.1}% (threshold: {:.1}%)", severity_prefix, value * 100.0, threshold * 100.0)
            }
            AlertType::RateLimitApproaching => {
                format!("{}Rate limit approaching: {:.1}% used (threshold: {:.1}%)", severity_prefix, value * 100.0, threshold * 100.0)
            }
        }
    }

    fn trigger_alert(&self, alert: Alert) {
        let handlers = self.handlers.read();
        for handler in handlers.iter() {
            handler.handle(&alert);
        }
        drop(handlers);

        let mut alerts = self.alerts.write();
        alerts.push(alert);

        if alerts.len() > self.max_alerts {
            let excess = alerts.len() - self.max_alerts;
            alerts.drain(0..excess);
        }
    }

    pub fn get_alerts(&self, limit: Option<usize>) -> Vec<Alert> {
        let alerts = self.alerts.read();
        let limit = limit.unwrap_or(alerts.len());
        alerts.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_unacknowledged(&self) -> Vec<Alert> {
        self.alerts
            .read()
            .iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    pub fn acknowledge(&self, alert_id: uuid::Uuid) -> bool {
        let mut alerts = self.alerts.write();
        for alert in alerts.iter_mut() {
            if alert.id == alert_id {
                alert.acknowledged = true;
                return true;
            }
        }
        false
    }

    pub fn acknowledge_all(&self) -> usize {
        let mut alerts = self.alerts.write();
        let mut count = 0;
        for alert in alerts.iter_mut() {
            if !alert.acknowledged {
                alert.acknowledged = true;
                count += 1;
            }
        }
        count
    }

    pub fn clear_alerts(&self) {
        self.alerts.write().clear();
    }

    pub fn check_cost(&self, cost_usd: f64) {
        self.record_value(AlertType::CostThreshold, cost_usd);
    }

    pub fn check_error_rate(&self, errors: u64, total: u64) {
        if total > 0 {
            let rate = errors as f64 / total as f64;
            self.record_value(AlertType::ErrorRate, rate);
        }
    }

    pub fn check_latency(&self, latency_ms: f64) {
        self.record_value(AlertType::LatencySpike, latency_ms);
    }

    pub fn check_provider_status(&self, provider: &str, is_down: bool) {
        if is_down {
            let mut metadata = HashMap::new();
            metadata.insert("provider".to_string(), provider.to_string());
            self.record_value_with_metadata(AlertType::ProviderDown, 1.0, metadata);
        }
    }

    pub fn check_token_budget(&self, used: u64, budget: u64) {
        if budget > 0 {
            let ratio = used as f64 / budget as f64;
            self.record_value(AlertType::TokenBudgetExceeded, ratio);
        }
    }

    pub fn check_hallucination_rate(&self, detected: u64, total: u64) {
        if total > 0 {
            let rate = detected as f64 / total as f64;
            self.record_value(AlertType::HallucinationRate, rate);
        }
    }

    pub fn check_cache_hit_rate(&self, hits: u64, total: u64) {
        if total > 0 {
            let rate = hits as f64 / total as f64;
            self.record_value(AlertType::CacheHitLow, rate);
        }
    }

    pub fn check_rate_limit(&self, used: u64, limit: u64) {
        if limit > 0 {
            let ratio = used as f64 / limit as f64;
            self.record_value(AlertType::RateLimitApproaching, ratio);
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_cost_alert() {
        let manager = AlertManager::new();
        manager.set_threshold(AlertThreshold {
            alert_type: AlertType::CostThreshold,
            warning_threshold: 1.0,
            critical_threshold: 5.0,
            window_seconds: 60,
            cooldown_seconds: 0,
            enabled: true,
        });

        manager.check_cost(0.5);
        assert_eq!(manager.get_alerts(None).len(), 0);

        manager.check_cost(2.0);
        let alerts = manager.get_alerts(None);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);

        manager.check_cost(10.0);
        let alerts = manager.get_alerts(None);
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_error_rate_alert() {
        let manager = AlertManager::new();
        manager.set_threshold(AlertThreshold {
            alert_type: AlertType::ErrorRate,
            warning_threshold: 0.05,
            critical_threshold: 0.10,
            window_seconds: 60,
            cooldown_seconds: 0,
            enabled: true,
        });

        manager.check_error_rate(1, 100);
        assert_eq!(manager.get_alerts(None).len(), 0);

        manager.check_error_rate(8, 100);
        assert_eq!(manager.get_alerts(None).len(), 1);
    }

    #[test]
    fn test_custom_handler() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let manager = AlertManager::new();
        manager.add_handler(Arc::new(CallbackAlertHandler::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })));

        manager.set_threshold(AlertThreshold {
            alert_type: AlertType::CostThreshold,
            warning_threshold: 1.0,
            critical_threshold: 5.0,
            window_seconds: 60,
            cooldown_seconds: 0,
            enabled: true,
        });

        manager.check_cost(10.0);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_acknowledge() {
        let manager = AlertManager::new();
        manager.set_threshold(AlertThreshold {
            alert_type: AlertType::CostThreshold,
            warning_threshold: 1.0,
            critical_threshold: 5.0,
            window_seconds: 60,
            cooldown_seconds: 0,
            enabled: true,
        });

        manager.check_cost(10.0);

        let unack = manager.get_unacknowledged();
        assert_eq!(unack.len(), 1);

        manager.acknowledge(unack[0].id);
        assert_eq!(manager.get_unacknowledged().len(), 0);
    }
}
