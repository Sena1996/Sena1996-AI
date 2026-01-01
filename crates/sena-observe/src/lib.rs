pub mod alerting;
pub mod anomaly;
pub mod audit;
pub mod events;
mod health;
mod logging;
mod metrics;

pub use alerting::{Alert, AlertHandler, AlertManager, AlertSeverity, AlertThreshold, AlertType};
pub use anomaly::{Anomaly, AnomalyConfig, AnomalyDetector, AnomalyHandler, MetricType};
pub use audit::{AuditAction, AuditEntry, AuditFilter, AuditLogger, AuditOutcome, AuditService};
pub use events::{Event, EventBus, EventHandler, EventLevel, EventType, LoggingHandler};
pub use health::HealthRegistry;
pub use logging::{init_logging, LogConfig};
pub use metrics::{CircuitStateMetric, Metrics, Timer};

use std::sync::Arc;

pub struct Observer {
    pub health: HealthRegistry,
    pub events: Arc<EventBus>,
    pub audit: Arc<AuditService>,
    pub alerts: Arc<AlertManager>,
    pub anomaly: Arc<AnomalyDetector>,
}

impl Observer {
    pub fn new() -> Self {
        Self {
            health: HealthRegistry::new(),
            events: Arc::new(EventBus::new()),
            audit: Arc::new(AuditService::in_memory(10000)),
            alerts: Arc::new(AlertManager::new()),
            anomaly: Arc::new(AnomalyDetector::new()),
        }
    }

    pub fn init(config: &sena_core::config::ObserveConfig) -> Self {
        init_logging(config);
        Self::new()
    }

    pub fn with_event_bus(mut self, bus: Arc<EventBus>) -> Self {
        self.events = bus;
        self
    }

    pub fn with_audit_service(mut self, audit: Arc<AuditService>) -> Self {
        self.audit = audit;
        self
    }

    pub fn with_alert_manager(mut self, alerts: Arc<AlertManager>) -> Self {
        self.alerts = alerts;
        self
    }

    pub fn with_anomaly_detector(mut self, anomaly: Arc<AnomalyDetector>) -> Self {
        self.anomaly = anomaly;
        self
    }

    pub fn metrics(&self) -> String {
        Metrics::gather()
    }

    pub fn health_check(&self) -> serde_json::Value {
        self.health.to_json()
    }

    pub fn emit_event(&self, event: Event) {
        self.events.emit(event);
    }

    pub async fn log_audit(&self, entry: AuditEntry) -> sena_core::Result<()> {
        self.audit.log(entry).await
    }

    pub fn check_alert(&self, alert_type: AlertType, value: f64) {
        self.alerts.record_value(alert_type, value);
    }

    pub fn record_metric(&self, metric_type: MetricType, value: f64) -> Option<Anomaly> {
        self.anomaly.record(metric_type, value)
    }

    pub fn get_alerts(&self, limit: Option<usize>) -> Vec<Alert> {
        self.alerts.get_alerts(limit)
    }

    pub fn get_anomalies(&self, limit: Option<usize>) -> Vec<Anomaly> {
        self.anomaly.get_anomalies(limit)
    }
}

impl Default for Observer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Observer {
    fn clone(&self) -> Self {
        Self {
            health: self.health.clone(),
            events: self.events.clone(),
            audit: self.audit.clone(),
            alerts: self.alerts.clone(),
            anomaly: self.anomaly.clone(),
        }
    }
}
