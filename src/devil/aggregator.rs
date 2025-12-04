use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub provider_id: String,
    pub model: String,
    pub result: Result<String, String>,
    pub latency: Duration,
}

impl ProviderResponse {
    pub fn success(provider_id: String, model: String, content: String, latency: Duration) -> Self {
        Self {
            provider_id,
            model,
            result: Ok(content),
            latency,
        }
    }

    pub fn failure(provider_id: String, model: String, error: String, latency: Duration) -> Self {
        Self {
            provider_id,
            model,
            result: Err(error),
            latency,
        }
    }

    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    pub fn content(&self) -> Option<&str> {
        self.result.as_ref().ok().map(|s| s.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResponses {
    pub responses: Vec<ProviderResponseData>,
    pub successful_count: usize,
    pub failed_count: usize,
    pub total_latency_ms: u64,
    pub fastest_provider: Option<String>,
    pub slowest_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponseData {
    pub provider_id: String,
    pub model: String,
    pub content: Option<String>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

pub struct ResponseAggregator;

impl ResponseAggregator {
    pub fn new() -> Self {
        Self
    }

    pub fn aggregate(&self, responses: Vec<ProviderResponse>) -> AggregatedResponses {
        let mut successful_count = 0;
        let mut failed_count = 0;
        let mut total_latency_ms: u64 = 0;
        let mut fastest: Option<(&str, u64)> = None;
        let mut slowest: Option<(&str, u64)> = None;

        let response_data: Vec<ProviderResponseData> = responses
            .iter()
            .map(|r| {
                let latency_ms = r.latency.as_millis() as u64;
                total_latency_ms = total_latency_ms.max(latency_ms);

                match &r.result {
                    Ok(_) => {
                        successful_count += 1;

                        match fastest {
                            None => fastest = Some((&r.provider_id, latency_ms)),
                            Some((_, f)) if latency_ms < f => {
                                fastest = Some((&r.provider_id, latency_ms))
                            }
                            _ => {}
                        }

                        match slowest {
                            None => slowest = Some((&r.provider_id, latency_ms)),
                            Some((_, s)) if latency_ms > s => {
                                slowest = Some((&r.provider_id, latency_ms))
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        failed_count += 1;
                    }
                }

                ProviderResponseData {
                    provider_id: r.provider_id.clone(),
                    model: r.model.clone(),
                    content: r.result.as_ref().ok().cloned(),
                    error: r.result.as_ref().err().cloned(),
                    latency_ms,
                }
            })
            .collect();

        AggregatedResponses {
            responses: response_data,
            successful_count,
            failed_count,
            total_latency_ms,
            fastest_provider: fastest.map(|(id, _)| id.to_string()),
            slowest_provider: slowest.map(|(id, _)| id.to_string()),
        }
    }

    pub fn get_successful_contents(aggregated: &AggregatedResponses) -> Vec<&str> {
        aggregated
            .responses
            .iter()
            .filter_map(|r| r.content.as_deref())
            .collect()
    }
}

impl Default for ResponseAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator() {
        let responses = vec![
            ProviderResponse::success(
                "claude".to_string(),
                "claude-3".to_string(),
                "Response from Claude".to_string(),
                Duration::from_millis(1500),
            ),
            ProviderResponse::success(
                "openai".to_string(),
                "gpt-4".to_string(),
                "Response from OpenAI".to_string(),
                Duration::from_millis(1200),
            ),
            ProviderResponse::failure(
                "ollama".to_string(),
                "llama2".to_string(),
                "Connection timeout".to_string(),
                Duration::from_millis(5000),
            ),
        ];

        let aggregator = ResponseAggregator::new();
        let aggregated = aggregator.aggregate(responses);

        assert_eq!(aggregated.successful_count, 2);
        assert_eq!(aggregated.failed_count, 1);
        assert_eq!(aggregated.fastest_provider, Some("openai".to_string()));
        assert_eq!(aggregated.slowest_provider, Some("claude".to_string()));
    }

    #[test]
    fn test_get_successful_contents() {
        let responses = vec![
            ProviderResponse::success(
                "a".to_string(),
                "m".to_string(),
                "Content A".to_string(),
                Duration::from_millis(100),
            ),
            ProviderResponse::failure(
                "b".to_string(),
                "m".to_string(),
                "Error".to_string(),
                Duration::from_millis(100),
            ),
        ];

        let aggregator = ResponseAggregator::new();
        let aggregated = aggregator.aggregate(responses);
        let contents = ResponseAggregator::get_successful_contents(&aggregated);

        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0], "Content A");
    }
}
