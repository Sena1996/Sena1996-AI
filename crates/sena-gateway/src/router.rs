use dashmap::DashMap;
use parking_lot::RwLock;
use sena_core::{
    CompletionRequest, CompletionResponse, Error, HealthCheck, Provider, Result,
};
use sena_observe::{Metrics, Timer};
use std::sync::Arc;

use crate::circuit::{CircuitBreaker, CircuitConfig};
use crate::cost::{BudgetStatus, CostSnapshot, CostTracker};
use crate::retry::{RetryConfig, RetryPolicy};

pub struct Gateway {
    providers: DashMap<String, ProviderEntry>,
    failover_chain: RwLock<Vec<String>>,
    retry_config: RetryConfig,
    cost_tracker: Arc<CostTracker>,
}

struct ProviderEntry {
    provider: Arc<dyn Provider>,
    circuit: CircuitBreaker,
    priority: u8,
}

impl Gateway {
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
            failover_chain: RwLock::new(Vec::new()),
            retry_config: RetryConfig::default(),
            cost_tracker: Arc::new(CostTracker::new()),
        }
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub fn with_cost_tracker(mut self, tracker: Arc<CostTracker>) -> Self {
        self.cost_tracker = tracker;
        self
    }

    pub fn with_budget(mut self, budget_usd: f64) -> Self {
        self.cost_tracker = Arc::new(CostTracker::new().with_budget(budget_usd));
        self
    }

    pub fn cost_tracker(&self) -> &Arc<CostTracker> {
        &self.cost_tracker
    }

    pub fn check_budget(&self) -> BudgetStatus {
        self.cost_tracker.check_budget()
    }

    pub async fn cost_snapshot(&self) -> CostSnapshot {
        self.cost_tracker.snapshot().await
    }

    pub fn register<P: Provider + 'static>(&self, provider: P) {
        self.register_with_config(provider, CircuitConfig::default());
    }

    pub fn register_with_config<P: Provider + 'static>(
        &self,
        provider: P,
        circuit_config: CircuitConfig,
    ) {
        let name = provider.name().to_string();
        let priority = provider.priority();

        self.providers.insert(
            name.clone(),
            ProviderEntry {
                provider: Arc::new(provider),
                circuit: CircuitBreaker::new(circuit_config),
                priority,
            },
        );

        self.update_failover_chain();
    }

    fn update_failover_chain(&self) {
        let mut entries: Vec<_> = self
            .providers
            .iter()
            .map(|e| (e.key().clone(), e.priority))
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));

        let chain: Vec<String> = entries.into_iter().map(|(name, _)| name).collect();
        *self.failover_chain.write() = chain;
    }

    pub fn set_failover_chain(&self, chain: Vec<String>) {
        *self.failover_chain.write() = chain;
    }

    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let budget_status = self.cost_tracker.check_budget();
        if !budget_status.can_proceed() {
            let msg = match &budget_status {
                BudgetStatus::Exceeded { total, limit } => {
                    format!("budget exceeded: ${:.2} of ${:.2} used", total, limit)
                }
                _ => "budget check failed".to_string(),
            };
            return Err(Error::rate_limit(msg));
        }

        let mut last_error = None;
        let chain = self.failover_chain.read().clone();

        for provider_name in &chain {
            if let Some(entry) = self.providers.get(provider_name) {
                if !entry.circuit.can_execute() {
                    tracing::debug!(provider = provider_name, "skipping due to open circuit");
                    continue;
                }

                match self
                    .try_provider(&entry.provider, &entry.circuit, request.clone())
                    .await
                {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        tracing::warn!(
                            provider = provider_name,
                            error = %e,
                            "provider failed, trying next"
                        );
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::provider("no providers available")))
    }

    async fn try_provider(
        &self,
        provider: &Arc<dyn Provider>,
        circuit: &CircuitBreaker,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        let mut policy = RetryPolicy::new(self.retry_config.clone());

        loop {
            let timer = Timer::start();

            match provider.complete(request.clone()).await {
                Ok(response) => {
                    circuit.record_success();
                    timer.record_request(provider.name());
                    Metrics::record_request(provider.name(), "success");
                    Metrics::record_tokens(
                        provider.name(),
                        response.usage.prompt_tokens,
                        response.usage.completion_tokens,
                    );

                    self.cost_tracker.record(
                        provider.name(),
                        &response.model,
                        response.usage.prompt_tokens,
                        response.usage.completion_tokens,
                        0,
                    ).await;

                    return Ok(response);
                }
                Err(e) => {
                    timer.record_request(provider.name());

                    if !e.is_retryable() {
                        circuit.record_failure();
                        Metrics::record_request(provider.name(), "error");
                        return Err(e);
                    }

                    if let Some(delay) = policy.next_attempt() {
                        tracing::debug!(
                            provider = provider.name(),
                            attempt = policy.current_attempt(),
                            delay_ms = delay.as_millis() as u64,
                            "retrying after transient error"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        circuit.record_failure();
                        Metrics::record_request(provider.name(), "error");
                        return Err(e);
                    }
                }
            }
        }
    }

    pub async fn complete_with_provider(
        &self,
        provider_name: &str,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        let entry = self
            .providers
            .get(provider_name)
            .ok_or_else(|| Error::not_found(format!("provider {} not found", provider_name)))?;

        entry.circuit.check()?;

        self.try_provider(&entry.provider, &entry.circuit, request)
            .await
    }

    pub async fn health(&self, provider_name: &str) -> Option<HealthCheck> {
        self.providers
            .get(provider_name)
            .map(|e| futures::executor::block_on(e.provider.health()))
    }

    pub async fn all_health(&self) -> Vec<(String, HealthCheck)> {
        let mut results = Vec::new();

        for entry in self.providers.iter() {
            let check = entry.provider.health().await;
            results.push((entry.key().clone(), check));
        }

        results
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.iter().map(|e| e.key().clone()).collect()
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(name).map(|e| e.provider.clone())
    }

    pub fn reset_circuit(&self, provider_name: &str) {
        if let Some(entry) = self.providers.get(provider_name) {
            entry.circuit.reset();
        }
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}
