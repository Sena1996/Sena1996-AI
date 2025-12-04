mod aggregator;
mod config;
mod consensus;
mod error;
mod executor;
mod synthesizer;

pub use aggregator::{AggregatedResponses, ProviderResponse, ResponseAggregator};
pub use config::{DevilConfig, SynthesisMethod, WaitMode};
pub use consensus::{ConsensusEngine, ConsensusResult};
pub use error::{DevilError, DevilResult};
pub use executor::DevilExecutor;
pub use synthesizer::{ResponseSynthesizer, SynthesizedResponse};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevilResponse {
    pub content: String,
    pub provider_responses: Vec<ProviderResponseSummary>,
    pub consensus_score: f64,
    pub synthesis_method: SynthesisMethod,
    pub total_latency_ms: u64,
    pub facts_verified: Option<usize>,
    pub facts_rejected: Option<usize>,
    pub verification_rounds: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponseSummary {
    pub provider_id: String,
    pub model: String,
    pub status: ResponseStatus,
    pub latency_ms: u64,
    pub content_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    Success,
    Timeout,
    Error(String),
}

impl DevilResponse {
    pub fn format_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "Devil Mode Response (Consensus: {:.0}%)\n",
            self.consensus_score * 100.0
        ));
        summary.push_str(&format!(
            "Synthesis: {:?} | Latency: {}ms\n",
            self.synthesis_method, self.total_latency_ms
        ));

        if let (Some(verified), Some(rejected)) = (self.facts_verified, self.facts_rejected) {
            summary.push_str(&format!(
                "Facts: {} verified, {} rejected\n",
                verified, rejected
            ));
        }

        summary.push_str("\nProvider Results:\n");
        for response in &self.provider_responses {
            let status_str = match &response.status {
                ResponseStatus::Success => "OK",
                ResponseStatus::Timeout => "TIMEOUT",
                ResponseStatus::Error(e) => e.as_str(),
            };
            summary.push_str(&format!(
                "  {} ({}): {} - {}ms\n",
                response.provider_id, response.model, status_str, response.latency_ms
            ));
        }

        summary.push_str(&format!("\n{}\n", self.content));

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devil_response_summary() {
        let response = DevilResponse {
            content: "The Moon is Earth's natural satellite.".to_string(),
            provider_responses: vec![
                ProviderResponseSummary {
                    provider_id: "claude".to_string(),
                    model: "claude-3-opus".to_string(),
                    status: ResponseStatus::Success,
                    latency_ms: 1500,
                    content_preview: Some("The Moon...".to_string()),
                },
                ProviderResponseSummary {
                    provider_id: "openai".to_string(),
                    model: "gpt-4".to_string(),
                    status: ResponseStatus::Success,
                    latency_ms: 1200,
                    content_preview: Some("The Moon...".to_string()),
                },
            ],
            consensus_score: 0.85,
            synthesis_method: SynthesisMethod::CrossVerification,
            total_latency_ms: 3500,
            facts_verified: Some(5),
            facts_rejected: Some(1),
            verification_rounds: Some(2),
        };

        let summary = response.format_summary();
        assert!(summary.contains("85%"));
        assert!(summary.contains("claude"));
        assert!(summary.contains("openai"));
    }
}
