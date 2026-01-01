use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cached_input_per_million: Option<f64>,
}

impl ModelPricing {
    pub fn new(input: f64, output: f64) -> Self {
        Self {
            input_per_million: input,
            output_per_million: output,
            cached_input_per_million: None,
        }
    }

    pub fn with_cached(mut self, cached: f64) -> Self {
        self.cached_input_per_million = Some(cached);
        self
    }

    pub fn calculate(&self, input_tokens: u32, output_tokens: u32, cached_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_per_million;
        let cached_cost = if let Some(cached_rate) = self.cached_input_per_million {
            (cached_tokens as f64 / 1_000_000.0) * cached_rate
        } else {
            0.0
        };

        input_cost + output_cost + cached_cost
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub provider: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub request_count: u64,
    pub total_cost_usd: f64,
    pub last_request_at: u64,
}

impl Default for ProviderUsage {
    fn default() -> Self {
        Self {
            provider: String::new(),
            model: String::new(),
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            request_count: 0,
            total_cost_usd: 0.0,
            last_request_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSnapshot {
    pub timestamp: u64,
    pub total_cost_usd: f64,
    pub by_provider: HashMap<String, ProviderUsage>,
    pub total_tokens: u64,
    pub total_requests: u64,
}

pub struct CostTracker {
    pricing: Arc<RwLock<HashMap<String, ModelPricing>>>,
    usage: Arc<RwLock<HashMap<String, ProviderUsage>>>,
    budget_limit_usd: Option<f64>,
    alert_threshold: Option<f64>,
    total_cost_cents: AtomicU64,
}

impl CostTracker {
    pub fn new() -> Self {
        let mut pricing = HashMap::new();

        pricing.insert("claude-3-5-sonnet-20241022".to_string(), ModelPricing::new(3.0, 15.0));
        pricing.insert("claude-3-5-haiku-20241022".to_string(), ModelPricing::new(0.80, 4.0));
        pricing.insert("claude-3-opus-20240229".to_string(), ModelPricing::new(15.0, 75.0));
        pricing.insert("gpt-4o".to_string(), ModelPricing::new(2.5, 10.0));
        pricing.insert("gpt-4o-mini".to_string(), ModelPricing::new(0.15, 0.6));
        pricing.insert("gpt-4-turbo".to_string(), ModelPricing::new(10.0, 30.0));
        pricing.insert("gemini-1.5-pro".to_string(), ModelPricing::new(1.25, 5.0));
        pricing.insert("gemini-1.5-flash".to_string(), ModelPricing::new(0.075, 0.3));
        pricing.insert("gemini-2.0-flash-exp".to_string(), ModelPricing::new(0.0, 0.0));
        pricing.insert("deepseek-chat".to_string(), ModelPricing::new(0.14, 0.28).with_cached(0.014));
        pricing.insert("deepseek-reasoner".to_string(), ModelPricing::new(0.55, 2.19));
        pricing.insert("mistral-large-latest".to_string(), ModelPricing::new(2.0, 6.0));
        pricing.insert("mistral-small-latest".to_string(), ModelPricing::new(0.2, 0.6));
        pricing.insert("codestral-latest".to_string(), ModelPricing::new(0.2, 0.6));
        pricing.insert("command-r-plus".to_string(), ModelPricing::new(2.5, 10.0));
        pricing.insert("command-r".to_string(), ModelPricing::new(0.15, 0.6));
        pricing.insert("grok-2".to_string(), ModelPricing::new(2.0, 10.0));
        pricing.insert("grok-2-mini".to_string(), ModelPricing::new(0.2, 1.0));
        pricing.insert("llama-3.1-70b-versatile".to_string(), ModelPricing::new(0.59, 0.79));
        pricing.insert("llama-3.1-8b-instant".to_string(), ModelPricing::new(0.05, 0.08));
        pricing.insert("mixtral-8x7b-32768".to_string(), ModelPricing::new(0.24, 0.24));

        Self {
            pricing: Arc::new(RwLock::new(pricing)),
            usage: Arc::new(RwLock::new(HashMap::new())),
            budget_limit_usd: None,
            alert_threshold: None,
            total_cost_cents: AtomicU64::new(0),
        }
    }

    pub fn with_budget(mut self, limit_usd: f64) -> Self {
        self.budget_limit_usd = Some(limit_usd);
        self
    }

    pub fn with_alert_threshold(mut self, threshold_percent: f64) -> Self {
        self.alert_threshold = Some(threshold_percent);
        self
    }

    pub async fn set_pricing(&self, model: &str, pricing: ModelPricing) {
        self.pricing.write().await.insert(model.to_string(), pricing);
    }

    pub async fn record(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        cached_tokens: u32,
    ) -> f64 {
        let pricing = self.pricing.read().await;
        let cost = pricing
            .get(model)
            .map(|p| p.calculate(input_tokens, output_tokens, cached_tokens))
            .unwrap_or(0.0);

        drop(pricing);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let key = format!("{}:{}", provider, model);
        let mut usage = self.usage.write().await;
        let entry = usage.entry(key).or_insert_with(|| ProviderUsage {
            provider: provider.to_string(),
            model: model.to_string(),
            ..Default::default()
        });

        entry.input_tokens += input_tokens as u64;
        entry.output_tokens += output_tokens as u64;
        entry.cached_tokens += cached_tokens as u64;
        entry.request_count += 1;
        entry.total_cost_usd += cost;
        entry.last_request_at = now;

        self.total_cost_cents.fetch_add((cost * 100.0) as u64, Ordering::Relaxed);

        cost
    }

    pub fn total_cost(&self) -> f64 {
        self.total_cost_cents.load(Ordering::Relaxed) as f64 / 100.0
    }

    pub async fn snapshot(&self) -> CostSnapshot {
        let usage = self.usage.read().await;
        let by_provider: HashMap<String, ProviderUsage> = usage
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let total_tokens: u64 = by_provider
            .values()
            .map(|u| u.input_tokens + u.output_tokens)
            .sum();

        let total_requests: u64 = by_provider.values().map(|u| u.request_count).sum();

        CostSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_cost_usd: self.total_cost(),
            by_provider,
            total_tokens,
            total_requests,
        }
    }

    pub async fn provider_cost(&self, provider: &str) -> f64 {
        let usage = self.usage.read().await;
        usage
            .iter()
            .filter(|(k, _)| k.starts_with(provider))
            .map(|(_, v)| v.total_cost_usd)
            .sum()
    }

    pub fn check_budget(&self) -> BudgetStatus {
        let total = self.total_cost();

        match (self.budget_limit_usd, self.alert_threshold) {
            (Some(limit), Some(threshold)) => {
                let percent_used = (total / limit) * 100.0;
                if total >= limit {
                    BudgetStatus::Exceeded { total, limit }
                } else if percent_used >= threshold {
                    BudgetStatus::Warning {
                        total,
                        limit,
                        percent_used,
                    }
                } else {
                    BudgetStatus::Ok { total, remaining: limit - total }
                }
            }
            (Some(limit), None) => {
                if total >= limit {
                    BudgetStatus::Exceeded { total, limit }
                } else {
                    BudgetStatus::Ok { total, remaining: limit - total }
                }
            }
            _ => BudgetStatus::NoBudget { total },
        }
    }

    pub async fn reset(&self) {
        self.usage.write().await.clear();
        self.total_cost_cents.store(0, Ordering::Relaxed);
    }

    pub async fn cost_by_model(&self) -> Vec<(String, f64)> {
        let usage = self.usage.read().await;
        let mut costs: Vec<_> = usage
            .iter()
            .map(|(_, v)| (v.model.clone(), v.total_cost_usd))
            .collect();
        costs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        costs
    }

    pub async fn estimate_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        let pricing = self.pricing.read().await;
        pricing
            .get(model)
            .map(|p| p.calculate(input_tokens, output_tokens, 0))
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum BudgetStatus {
    Ok { total: f64, remaining: f64 },
    Warning { total: f64, limit: f64, percent_used: f64 },
    Exceeded { total: f64, limit: f64 },
    NoBudget { total: f64 },
}

impl BudgetStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, BudgetStatus::Ok { .. } | BudgetStatus::NoBudget { .. })
    }

    pub fn can_proceed(&self) -> bool {
        !matches!(self, BudgetStatus::Exceeded { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_pricing() {
        let pricing = ModelPricing::new(3.0, 15.0);
        let cost = pricing.calculate(1000, 500, 0);

        let expected = (1000.0 / 1_000_000.0) * 3.0 + (500.0 / 1_000_000.0) * 15.0;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_cached_pricing() {
        let pricing = ModelPricing::new(0.14, 0.28).with_cached(0.014);
        let cost = pricing.calculate(500, 500, 500);

        let expected = (500.0 / 1_000_000.0) * 0.14
            + (500.0 / 1_000_000.0) * 0.28
            + (500.0 / 1_000_000.0) * 0.014;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_cost_tracker() {
        let tracker = CostTracker::new();

        tracker.record("anthropic", "claude-3-5-sonnet-20241022", 1000, 500, 0).await;

        let snapshot = tracker.snapshot().await;
        assert!(snapshot.total_cost_usd > 0.0);
        assert_eq!(snapshot.total_requests, 1);
    }

    #[tokio::test]
    async fn test_budget_check() {
        let tracker = CostTracker::new()
            .with_budget(1.0)
            .with_alert_threshold(80.0);

        tracker.record("anthropic", "claude-3-opus-20240229", 10_000, 5_000, 0).await;

        let status = tracker.check_budget();
        assert!(status.can_proceed());
    }

    #[tokio::test]
    async fn test_estimate_cost() {
        let tracker = CostTracker::new();

        let estimate = tracker.estimate_cost("gpt-4o", 1_000_000, 500_000).await;
        assert!(estimate.is_some());
        let cost = estimate.unwrap();
        assert!((cost - 7.5).abs() < 0.01);
    }
}
