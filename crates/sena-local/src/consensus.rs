use sena_core::{CompletionRequest, Provider, Result};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub min_providers: usize,
    pub similarity_threshold: f32,
    pub timeout_ms: u64,
    pub voting_strategy: VotingStrategy,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_providers: 3,
            similarity_threshold: 0.7,
            timeout_ms: 30000,
            voting_strategy: VotingStrategy::Majority,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VotingStrategy {
    Majority,
    Unanimous,
    WeightedByConfidence,
}

#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub consensus_reached: bool,
    pub confidence: f32,
    pub responses: Vec<ProviderResponse>,
    pub agreement_matrix: Vec<Vec<f32>>,
    pub final_response: Option<String>,
    pub discrepancies: Vec<Discrepancy>,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub provider_name: String,
    pub content: String,
    pub latency_ms: u64,
    pub tokens_used: u32,
}

#[derive(Debug, Clone)]
pub struct Discrepancy {
    pub topic: String,
    pub claims: Vec<(String, String)>,
    pub severity: DiscrepancySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscrepancySeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

pub struct MultiModelConsensus {
    config: ConsensusConfig,
}

impl MultiModelConsensus {
    pub fn new(config: ConsensusConfig) -> Self {
        Self { config }
    }

    pub async fn verify(
        &self,
        request: &CompletionRequest,
        providers: &[Arc<dyn Provider>],
    ) -> Result<ConsensusResult> {
        if providers.len() < self.config.min_providers {
            return Err(sena_core::Error::validation(format!(
                "need at least {} providers for consensus, got {}",
                self.config.min_providers,
                providers.len()
            )));
        }

        let responses = self.query_providers(request, providers).await?;
        let agreement_matrix = self.compute_agreement_matrix(&responses);
        let discrepancies = self.find_discrepancies(&responses);
        let confidence = self.calculate_consensus_confidence(&agreement_matrix);
        let consensus_reached = self.check_consensus(&agreement_matrix, &discrepancies);
        let final_response = if consensus_reached {
            self.merge_responses(&responses)
        } else {
            None
        };

        Ok(ConsensusResult {
            consensus_reached,
            confidence,
            responses,
            agreement_matrix,
            final_response,
            discrepancies,
        })
    }

    async fn query_providers(
        &self,
        request: &CompletionRequest,
        providers: &[Arc<dyn Provider>],
    ) -> Result<Vec<ProviderResponse>> {
        let mut responses = Vec::with_capacity(providers.len());
        let timeout = tokio::time::Duration::from_millis(self.config.timeout_ms);

        for provider in providers {
            let start = std::time::Instant::now();
            let result = tokio::time::timeout(timeout, provider.complete(request.clone())).await;

            match result {
                Ok(Ok(response)) => {
                    responses.push(ProviderResponse {
                        provider_name: provider.name().to_string(),
                        content: response.content,
                        latency_ms: start.elapsed().as_millis() as u64,
                        tokens_used: response.usage.total_tokens,
                    });
                }
                Ok(Err(e)) => {
                    tracing::warn!(provider = provider.name(), error = %e, "provider failed");
                }
                Err(_) => {
                    tracing::warn!(provider = provider.name(), "provider timed out");
                }
            }
        }

        if responses.len() < self.config.min_providers {
            return Err(sena_core::Error::provider(format!(
                "only {} providers responded, need {}",
                responses.len(),
                self.config.min_providers
            )));
        }

        Ok(responses)
    }

    fn compute_agreement_matrix(&self, responses: &[ProviderResponse]) -> Vec<Vec<f32>> {
        let n = responses.len();
        let mut matrix = vec![vec![1.0f32; n]; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let similarity = self.text_similarity(&responses[i].content, &responses[j].content);
                matrix[i][j] = similarity;
                matrix[j][i] = similarity;
            }
        }

        matrix
    }

    fn text_similarity(&self, a: &str, b: &str) -> f32 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        let a_words: std::collections::HashSet<_> = a_lower.split_whitespace().collect();
        let b_words: std::collections::HashSet<_> = b_lower.split_whitespace().collect();

        if a_words.is_empty() && b_words.is_empty() {
            return 1.0;
        }

        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    fn find_discrepancies(&self, responses: &[ProviderResponse]) -> Vec<Discrepancy> {
        let mut discrepancies = Vec::new();
        let claims = self.extract_claims(responses);

        for (topic, topic_claims) in claims {
            let unique_claims: std::collections::HashSet<_> =
                topic_claims.iter().map(|(_, c)| c.as_str()).collect();

            if unique_claims.len() > 1 {
                let severity = self.assess_discrepancy_severity(&unique_claims);
                discrepancies.push(Discrepancy {
                    topic,
                    claims: topic_claims,
                    severity,
                });
            }
        }

        discrepancies
    }

    fn extract_claims(&self, responses: &[ProviderResponse]) -> HashMap<String, Vec<(String, String)>> {
        let mut claims: HashMap<String, Vec<(String, String)>> = HashMap::new();

        let patterns = [
            (regex::Regex::new(r"(?i)(\w+)\s+(is|are|was|were)\s+(\w+)").ok(), "definition"),
            (regex::Regex::new(r"(?i)(\w+)\s+(should|must|can|cannot)\s+").ok(), "requirement"),
            (regex::Regex::new(r"(?i)(version|v)\s*(\d+\.\d+)").ok(), "version"),
        ];

        for response in responses {
            for (pattern_opt, topic_type) in &patterns {
                if let Some(pattern) = pattern_opt {
                    for cap in pattern.captures_iter(&response.content) {
                        let claim = cap.get(0).map(|m| m.as_str().to_string()).unwrap_or_default();
                        let topic = format!("{}_{}", topic_type, cap.get(1).map(|m| m.as_str()).unwrap_or(""));

                        claims
                            .entry(topic)
                            .or_default()
                            .push((response.provider_name.clone(), claim));
                    }
                }
            }
        }

        claims
    }

    fn assess_discrepancy_severity(&self, claims: &std::collections::HashSet<&str>) -> DiscrepancySeverity {
        let has_negation = claims.iter().any(|c| {
            c.contains("not") || c.contains("cannot") || c.contains("shouldn't")
        });
        let has_positive = claims.iter().any(|c| {
            c.contains("can") || c.contains("should") || c.contains("must")
        }) && !has_negation;

        if has_negation && has_positive {
            DiscrepancySeverity::Critical
        } else if claims.len() > 3 {
            DiscrepancySeverity::Major
        } else if claims.len() > 2 {
            DiscrepancySeverity::Moderate
        } else {
            DiscrepancySeverity::Minor
        }
    }

    fn calculate_consensus_confidence(&self, matrix: &[Vec<f32>]) -> f32 {
        if matrix.is_empty() {
            return 0.0;
        }

        let n = matrix.len();
        let mut total = 0.0;
        let mut count = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                total += matrix[i][j];
                count += 1;
            }
        }

        if count == 0 {
            1.0
        } else {
            total / count as f32
        }
    }

    fn check_consensus(&self, matrix: &[Vec<f32>], discrepancies: &[Discrepancy]) -> bool {
        let confidence = self.calculate_consensus_confidence(matrix);

        if confidence < self.config.similarity_threshold {
            return false;
        }

        let has_critical = discrepancies.iter().any(|d| d.severity == DiscrepancySeverity::Critical);
        let major_count = discrepancies.iter().filter(|d| d.severity == DiscrepancySeverity::Major).count();

        match self.config.voting_strategy {
            VotingStrategy::Unanimous => !has_critical && major_count == 0 && confidence >= 0.9,
            VotingStrategy::Majority => !has_critical && confidence >= self.config.similarity_threshold,
            VotingStrategy::WeightedByConfidence => {
                let penalty = major_count as f32 * 0.1 + if has_critical { 0.5 } else { 0.0 };
                (confidence - penalty) >= self.config.similarity_threshold
            }
        }
    }

    fn merge_responses(&self, responses: &[ProviderResponse]) -> Option<String> {
        if responses.is_empty() {
            return None;
        }

        let mut content_lengths: Vec<_> = responses
            .iter()
            .map(|r| (r.content.len(), &r.content))
            .collect();
        content_lengths.sort_by(|a, b| b.0.cmp(&a.0));

        Some(content_lengths[content_lengths.len() / 2].1.clone())
    }
}

impl Default for MultiModelConsensus {
    fn default() -> Self {
        Self::new(ConsensusConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_similarity() {
        let consensus = MultiModelConsensus::default();

        let sim = consensus.text_similarity(
            "The function returns a value",
            "The function returns a result"
        );
        assert!(sim > 0.5);

        let sim_same = consensus.text_similarity("hello world", "hello world");
        assert!((sim_same - 1.0).abs() < 0.001);

        let sim_diff = consensus.text_similarity("hello", "goodbye");
        assert!(sim_diff < 0.5);
    }

    #[test]
    fn test_agreement_matrix() {
        let consensus = MultiModelConsensus::default();
        let responses = vec![
            ProviderResponse {
                provider_name: "a".to_string(),
                content: "The answer is 42".to_string(),
                latency_ms: 100,
                tokens_used: 10,
            },
            ProviderResponse {
                provider_name: "b".to_string(),
                content: "The answer is 42".to_string(),
                latency_ms: 150,
                tokens_used: 10,
            },
        ];

        let matrix = consensus.compute_agreement_matrix(&responses);
        assert_eq!(matrix.len(), 2);
        assert!((matrix[0][1] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_consensus_confidence() {
        let consensus = MultiModelConsensus::default();
        let matrix = vec![
            vec![1.0, 0.9, 0.85],
            vec![0.9, 1.0, 0.88],
            vec![0.85, 0.88, 1.0],
        ];

        let confidence = consensus.calculate_consensus_confidence(&matrix);
        assert!(confidence > 0.8);
    }
}
