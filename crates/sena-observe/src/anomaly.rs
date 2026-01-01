use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    Latency,
    TokenUsage,
    Cost,
    ErrorCount,
    RequestCount,
    CacheHitRate,
    ProviderResponseTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: uuid::Uuid,
    pub metric_type: MetricType,
    pub value: f64,
    pub expected_mean: f64,
    pub expected_std: f64,
    pub z_score: f64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct MetricWindow {
    values: VecDeque<(Instant, f64)>,
    sum: f64,
    sum_sq: f64,
    window_seconds: u64,
}

impl MetricWindow {
    fn new(window_seconds: u64) -> Self {
        Self {
            values: VecDeque::new(),
            sum: 0.0,
            sum_sq: 0.0,
            window_seconds,
        }
    }

    fn add(&mut self, value: f64) {
        let now = Instant::now();
        self.cleanup(now);

        self.values.push_back((now, value));
        self.sum += value;
        self.sum_sq += value * value;
    }

    fn cleanup(&mut self, now: Instant) {
        let cutoff = std::time::Duration::from_secs(self.window_seconds);

        while let Some(&(time, value)) = self.values.front() {
            if now.duration_since(time) > cutoff {
                self.values.pop_front();
                self.sum -= value;
                self.sum_sq -= value * value;
            } else {
                break;
            }
        }
    }

    fn count(&self) -> usize {
        self.values.len()
    }

    fn mean(&self) -> Option<f64> {
        let n = self.count();
        if n == 0 {
            None
        } else {
            Some(self.sum / n as f64)
        }
    }

    fn std_dev(&self) -> Option<f64> {
        let n = self.count();
        if n < 2 {
            return None;
        }

        let mean = self.sum / n as f64;
        let variance = (self.sum_sq / n as f64) - (mean * mean);

        if variance < 0.0 {
            Some(0.0)
        } else {
            Some(variance.sqrt())
        }
    }

    fn z_score(&self, value: f64) -> Option<f64> {
        let mean = self.mean()?;
        let std = self.std_dev()?;

        if std < 1e-10 {
            if (value - mean).abs() < 1e-10 {
                Some(0.0)
            } else {
                Some(if value > mean { 10.0 } else { -10.0 })
            }
        } else {
            Some((value - mean) / std)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    pub z_score_threshold: f64,
    pub min_samples: usize,
    pub window_seconds: u64,
    pub enabled: bool,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            z_score_threshold: 3.0,
            min_samples: 30,
            window_seconds: 3600,
            enabled: true,
        }
    }
}

pub trait AnomalyHandler: Send + Sync {
    fn handle(&self, anomaly: &Anomaly);
}

pub struct LogAnomalyHandler;

impl AnomalyHandler for LogAnomalyHandler {
    fn handle(&self, anomaly: &Anomaly) {
        tracing::warn!(
            "[ANOMALY] {:?}: {} (z-score: {:.2}, value: {:.4}, expected: {:.4} ± {:.4})",
            anomaly.metric_type,
            anomaly.description,
            anomaly.z_score,
            anomaly.value,
            anomaly.expected_mean,
            anomaly.expected_std
        );
    }
}

pub struct AnomalyDetector {
    windows: RwLock<HashMap<MetricType, MetricWindow>>,
    configs: RwLock<HashMap<MetricType, AnomalyConfig>>,
    anomalies: RwLock<Vec<Anomaly>>,
    handlers: RwLock<Vec<std::sync::Arc<dyn AnomalyHandler>>>,
    max_anomalies: usize,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        let default = AnomalyConfig::default();

        configs.insert(MetricType::Latency, AnomalyConfig {
            z_score_threshold: 2.5,
            ..default.clone()
        });
        configs.insert(MetricType::TokenUsage, default.clone());
        configs.insert(MetricType::Cost, AnomalyConfig {
            z_score_threshold: 2.0,
            ..default.clone()
        });
        configs.insert(MetricType::ErrorCount, AnomalyConfig {
            z_score_threshold: 2.0,
            min_samples: 20,
            ..default.clone()
        });
        configs.insert(MetricType::RequestCount, default.clone());
        configs.insert(MetricType::CacheHitRate, default.clone());
        configs.insert(MetricType::ProviderResponseTime, AnomalyConfig {
            z_score_threshold: 2.5,
            ..default
        });

        Self {
            windows: RwLock::new(HashMap::new()),
            configs: RwLock::new(configs),
            anomalies: RwLock::new(Vec::new()),
            handlers: RwLock::new(vec![std::sync::Arc::new(LogAnomalyHandler)]),
            max_anomalies: 500,
        }
    }

    pub fn add_handler(&self, handler: std::sync::Arc<dyn AnomalyHandler>) {
        self.handlers.write().push(handler);
    }

    pub fn set_config(&self, metric_type: MetricType, config: AnomalyConfig) {
        self.configs.write().insert(metric_type, config);
    }

    pub fn get_config(&self, metric_type: MetricType) -> Option<AnomalyConfig> {
        self.configs.read().get(&metric_type).cloned()
    }

    pub fn record(&self, metric_type: MetricType, value: f64) -> Option<Anomaly> {
        self.record_with_metadata(metric_type, value, HashMap::new())
    }

    pub fn record_with_metadata(
        &self,
        metric_type: MetricType,
        value: f64,
        metadata: HashMap<String, String>,
    ) -> Option<Anomaly> {
        let config = self.configs.read().get(&metric_type)?.clone();

        if !config.enabled {
            return None;
        }

        let mut windows = self.windows.write();
        let window = windows
            .entry(metric_type)
            .or_insert_with(|| MetricWindow::new(config.window_seconds));

        let anomaly = if window.count() >= config.min_samples {
            if let (Some(z_score), Some(mean), Some(std)) =
                (window.z_score(value), window.mean(), window.std_dev())
            {
                if z_score.abs() > config.z_score_threshold {
                    let description = self.format_anomaly_description(metric_type, value, mean, z_score);

                    Some(Anomaly {
                        id: uuid::Uuid::new_v4(),
                        metric_type,
                        value,
                        expected_mean: mean,
                        expected_std: std,
                        z_score,
                        detected_at: chrono::Utc::now(),
                        description,
                        metadata,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        window.add(value);
        drop(windows);

        if let Some(ref a) = anomaly {
            self.trigger_anomaly(a.clone());
        }

        anomaly
    }

    fn format_anomaly_description(
        &self,
        metric_type: MetricType,
        value: f64,
        mean: f64,
        z_score: f64,
    ) -> String {
        let direction = if z_score > 0.0 { "higher" } else { "lower" };
        let magnitude = z_score.abs();

        match metric_type {
            MetricType::Latency => {
                format!(
                    "Latency {:.0}ms is {:.1}σ {} than usual ({:.0}ms)",
                    value, magnitude, direction, mean
                )
            }
            MetricType::TokenUsage => {
                format!(
                    "Token usage {} is {:.1}σ {} than usual ({:.0})",
                    value as u64, magnitude, direction, mean
                )
            }
            MetricType::Cost => {
                format!(
                    "Cost ${:.4} is {:.1}σ {} than usual (${:.4})",
                    value, magnitude, direction, mean
                )
            }
            MetricType::ErrorCount => {
                format!(
                    "Error count {} is {:.1}σ {} than usual ({:.1})",
                    value as u64, magnitude, direction, mean
                )
            }
            MetricType::RequestCount => {
                format!(
                    "Request count {} is {:.1}σ {} than usual ({:.1})",
                    value as u64, magnitude, direction, mean
                )
            }
            MetricType::CacheHitRate => {
                format!(
                    "Cache hit rate {:.1}% is {:.1}σ {} than usual ({:.1}%)",
                    value * 100.0, magnitude, direction, mean * 100.0
                )
            }
            MetricType::ProviderResponseTime => {
                format!(
                    "Provider response time {:.0}ms is {:.1}σ {} than usual ({:.0}ms)",
                    value, magnitude, direction, mean
                )
            }
        }
    }

    fn trigger_anomaly(&self, anomaly: Anomaly) {
        let handlers = self.handlers.read();
        for handler in handlers.iter() {
            handler.handle(&anomaly);
        }
        drop(handlers);

        let mut anomalies = self.anomalies.write();
        anomalies.push(anomaly);

        if anomalies.len() > self.max_anomalies {
            let excess = anomalies.len() - self.max_anomalies;
            anomalies.drain(0..excess);
        }
    }

    pub fn get_anomalies(&self, limit: Option<usize>) -> Vec<Anomaly> {
        let anomalies = self.anomalies.read();
        let limit = limit.unwrap_or(anomalies.len());
        anomalies.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_anomalies_by_type(&self, metric_type: MetricType) -> Vec<Anomaly> {
        self.anomalies
            .read()
            .iter()
            .filter(|a| a.metric_type == metric_type)
            .cloned()
            .collect()
    }

    pub fn get_stats(&self, metric_type: MetricType) -> Option<(f64, f64, usize)> {
        let windows = self.windows.read();
        let window = windows.get(&metric_type)?;
        Some((window.mean()?, window.std_dev()?, window.count()))
    }

    pub fn clear_anomalies(&self) {
        self.anomalies.write().clear();
    }

    pub fn record_latency(&self, latency_ms: f64) -> Option<Anomaly> {
        self.record(MetricType::Latency, latency_ms)
    }

    pub fn record_tokens(&self, tokens: u64) -> Option<Anomaly> {
        self.record(MetricType::TokenUsage, tokens as f64)
    }

    pub fn record_cost(&self, cost_usd: f64) -> Option<Anomaly> {
        self.record(MetricType::Cost, cost_usd)
    }

    pub fn record_errors(&self, count: u64) -> Option<Anomaly> {
        self.record(MetricType::ErrorCount, count as f64)
    }

    pub fn record_requests(&self, count: u64) -> Option<Anomaly> {
        self.record(MetricType::RequestCount, count as f64)
    }

    pub fn record_cache_hit_rate(&self, rate: f64) -> Option<Anomaly> {
        self.record(MetricType::CacheHitRate, rate)
    }

    pub fn record_provider_response(&self, provider: &str, latency_ms: f64) -> Option<Anomaly> {
        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), provider.to_string());
        self.record_with_metadata(MetricType::ProviderResponseTime, latency_ms, metadata)
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_values_no_anomaly() {
        let detector = AnomalyDetector::new();
        detector.set_config(MetricType::Latency, AnomalyConfig {
            z_score_threshold: 3.0,
            min_samples: 10,
            window_seconds: 3600,
            enabled: true,
        });

        for _ in 0..20 {
            detector.record_latency(100.0);
        }

        let result = detector.record_latency(100.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_anomaly_detected() {
        let detector = AnomalyDetector::new();
        detector.set_config(MetricType::Latency, AnomalyConfig {
            z_score_threshold: 2.0,
            min_samples: 10,
            window_seconds: 3600,
            enabled: true,
        });

        for _ in 0..20 {
            detector.record_latency(100.0);
        }

        let result = detector.record_latency(500.0);
        assert!(result.is_some());

        let anomaly = result.unwrap();
        assert!(anomaly.z_score > 2.0);
    }

    #[test]
    fn test_stats() {
        let detector = AnomalyDetector::new();

        for i in 0..10 {
            detector.record_latency(100.0 + i as f64);
        }

        let stats = detector.get_stats(MetricType::Latency);
        assert!(stats.is_some());

        let (mean, std, count) = stats.unwrap();
        assert_eq!(count, 10);
        assert!((mean - 104.5).abs() < 0.1);
        assert!(std > 0.0);
    }

    #[test]
    fn test_disabled_metric() {
        let detector = AnomalyDetector::new();
        detector.set_config(MetricType::Latency, AnomalyConfig {
            enabled: false,
            ..Default::default()
        });

        for _ in 0..50 {
            detector.record_latency(100.0);
        }

        let result = detector.record_latency(10000.0);
        assert!(result.is_none());
    }
}
