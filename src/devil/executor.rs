use std::time::{Duration, Instant};

use super::aggregator::{ProviderResponse, ResponseAggregator};
use super::config::DevilConfig;
use super::consensus::ConsensusEngine;
use super::error::{DevilError, DevilResult};
use super::synthesizer::ResponseSynthesizer;
use super::{DevilResponse, ProviderResponseSummary, ResponseStatus};

pub struct DevilExecutor {
    config: DevilConfig,
    aggregator: ResponseAggregator,
    consensus: ConsensusEngine,
}

impl DevilExecutor {
    pub fn new(config: DevilConfig) -> Self {
        let consensus_threshold = config.consensus_threshold;
        Self {
            config,
            aggregator: ResponseAggregator::new(),
            consensus: ConsensusEngine::with_thresholds(0.3, consensus_threshold),
        }
    }

    pub fn execute_sync(&self, _prompt: &str, responses: Vec<ProviderResponse>) -> DevilResult<DevilResponse> {
        if responses.is_empty() {
            return Err(DevilError::NoProviders);
        }

        let aggregated = self.aggregator.aggregate(responses);

        if aggregated.successful_count == 0 {
            return Err(DevilError::AllProvidersFailed(
                "All provider requests failed".to_string(),
            ));
        }

        let consensus = self.consensus.analyze(&aggregated)?;

        let synthesizer = ResponseSynthesizer::new(self.config.synthesis_method)
            .with_max_facts(self.config.max_facts_per_response);
        let synthesized = synthesizer.synthesize(&aggregated, &consensus)?;

        let provider_responses: Vec<ProviderResponseSummary> = aggregated
            .responses
            .iter()
            .map(|r| ProviderResponseSummary {
                provider_id: r.provider_id.clone(),
                model: r.model.clone(),
                status: if r.content.is_some() {
                    ResponseStatus::Success
                } else if r.error.as_deref() == Some("Timeout") {
                    ResponseStatus::Timeout
                } else {
                    ResponseStatus::Error(r.error.clone().unwrap_or_else(|| "Unknown error".to_string()))
                },
                latency_ms: r.latency_ms,
                content_preview: r.content.as_ref().map(|c| {
                    if c.len() > 100 {
                        format!("{}...", &c[..100])
                    } else {
                        c.clone()
                    }
                }),
            })
            .collect();

        Ok(DevilResponse {
            content: synthesized.content,
            provider_responses,
            consensus_score: consensus.agreement_score,
            synthesis_method: synthesized.method,
            total_latency_ms: aggregated.total_latency_ms,
            facts_verified: synthesized.facts_verified,
            facts_rejected: synthesized.facts_rejected,
            verification_rounds: synthesized.verification_rounds,
        })
    }

    pub fn config(&self) -> &DevilConfig {
        &self.config
    }
}

impl Default for DevilExecutor {
    fn default() -> Self {
        Self::new(DevilConfig::default())
    }
}

#[allow(dead_code)]
pub struct MockProviderPool {
    providers: Vec<MockProvider>,
}

pub type ResponseGenerator = Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

#[allow(dead_code)]
pub struct MockProvider {
    pub id: String,
    pub model: String,
    pub response_generator: ResponseGenerator,
    pub latency: Duration,
}

#[allow(dead_code)]
impl MockProviderPool {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: MockProvider) {
        self.providers.push(provider);
    }

    pub fn execute_all(&self, prompt: &str) -> Vec<ProviderResponse> {
        self.providers
            .iter()
            .map(|provider| {
                let start = Instant::now();
                std::thread::sleep(provider.latency);
                let result = (provider.response_generator)(prompt);
                let latency = start.elapsed();

                match result {
                    Ok(content) => ProviderResponse::success(
                        provider.id.clone(),
                        provider.model.clone(),
                        content,
                        latency,
                    ),
                    Err(error) => ProviderResponse::failure(
                        provider.id.clone(),
                        provider.model.clone(),
                        error,
                        latency,
                    ),
                }
            })
            .collect()
    }
}

impl Default for MockProviderPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devil::SynthesisMethod;

    fn create_mock_responses() -> Vec<ProviderResponse> {
        vec![
            ProviderResponse::success(
                "claude".to_string(),
                "claude-3".to_string(),
                "The Moon is about 384,000 km from Earth. It is tidally locked. It has no atmosphere.".to_string(),
                Duration::from_millis(1500),
            ),
            ProviderResponse::success(
                "openai".to_string(),
                "gpt-4".to_string(),
                "The Moon is approximately 384,000 km away from Earth. The Moon is tidally locked to Earth. There is no atmosphere on the Moon.".to_string(),
                Duration::from_millis(1200),
            ),
            ProviderResponse::success(
                "gemini".to_string(),
                "gemini-pro".to_string(),
                "Earth's Moon is 384,000 km distant. It's tidally locked and has no atmosphere.".to_string(),
                Duration::from_millis(1800),
            ),
        ]
    }

    #[test]
    fn test_devil_executor_basic() {
        let executor = DevilExecutor::default();
        let responses = create_mock_responses();

        let result = executor.execute_sync("Tell me about the Moon", responses).unwrap();

        assert!(!result.content.is_empty());
        assert!(result.consensus_score > 0.0);
        assert_eq!(result.provider_responses.len(), 3);
    }

    #[test]
    fn test_devil_executor_with_failures() {
        let executor = DevilExecutor::default();
        let responses = vec![
            ProviderResponse::success(
                "claude".to_string(),
                "m".to_string(),
                "Valid response about the Moon.".to_string(),
                Duration::from_millis(100),
            ),
            ProviderResponse::failure(
                "openai".to_string(),
                "m".to_string(),
                "Connection timeout".to_string(),
                Duration::from_millis(5000),
            ),
        ];

        let result = executor.execute_sync("Test", responses).unwrap();
        assert!(!result.content.is_empty());

        let failed_count = result
            .provider_responses
            .iter()
            .filter(|r| !matches!(r.status, ResponseStatus::Success))
            .count();
        assert_eq!(failed_count, 1);
    }

    #[test]
    fn test_devil_executor_all_fail() {
        let executor = DevilExecutor::default();
        let responses = vec![
            ProviderResponse::failure(
                "a".to_string(),
                "m".to_string(),
                "Error 1".to_string(),
                Duration::from_millis(100),
            ),
            ProviderResponse::failure(
                "b".to_string(),
                "m".to_string(),
                "Error 2".to_string(),
                Duration::from_millis(100),
            ),
        ];

        let result = executor.execute_sync("Test", responses);
        assert!(matches!(result, Err(DevilError::AllProvidersFailed(_))));
    }

    #[test]
    fn test_devil_executor_empty() {
        let executor = DevilExecutor::default();
        let result = executor.execute_sync("Test", vec![]);
        assert!(matches!(result, Err(DevilError::NoProviders)));
    }

    #[test]
    fn test_different_synthesis_methods() {
        let responses = create_mock_responses();

        for method in [
            SynthesisMethod::MajorityVoting,
            SynthesisMethod::WeightedMerge,
            SynthesisMethod::BestOfN,
            SynthesisMethod::CrossVerification,
        ] {
            let config = DevilConfig::default().with_synthesis(method);
            let executor = DevilExecutor::new(config);

            let result = executor.execute_sync("Moon info", responses.clone()).unwrap();
            assert!(!result.content.is_empty(), "Failed for {:?}", method);
        }
    }
}
