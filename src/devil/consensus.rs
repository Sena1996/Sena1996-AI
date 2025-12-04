use std::collections::{HashMap, HashSet};

use super::aggregator::AggregatedResponses;
use super::error::{DevilError, DevilResult};

#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub agreement_score: f64,
    pub clusters: Vec<ResponseCluster>,
    pub agreed_facts: Vec<AgreedFact>,
    pub outliers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResponseCluster {
    pub provider_ids: Vec<String>,
    pub representative_content: String,
    pub similarity_score: f64,
}

#[derive(Debug, Clone)]
pub struct AgreedFact {
    pub fact: String,
    pub supporting_providers: Vec<String>,
    pub agreement_ratio: f64,
}

impl ConsensusResult {
    pub fn get_fact_agreement(&self, fact: &str) -> f64 {
        let fact_lower = fact.to_lowercase();
        for agreed in &self.agreed_facts {
            if agreed.fact.to_lowercase().contains(&fact_lower)
                || fact_lower.contains(&agreed.fact.to_lowercase())
            {
                return agreed.agreement_ratio;
            }
        }
        0.0
    }
}

pub struct ConsensusEngine {
    similarity_threshold: f64,
    minimum_agreement: f64,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.3,
            minimum_agreement: 0.5,
        }
    }

    pub fn with_thresholds(similarity_threshold: f64, minimum_agreement: f64) -> Self {
        Self {
            similarity_threshold,
            minimum_agreement,
        }
    }

    pub fn analyze(&self, aggregated: &AggregatedResponses) -> DevilResult<ConsensusResult> {
        let successful: Vec<(&str, &str)> = aggregated
            .responses
            .iter()
            .filter_map(|r| r.content.as_deref().map(|c| (r.provider_id.as_str(), c)))
            .collect();

        if successful.is_empty() {
            return Err(DevilError::AllProvidersFailed(
                "No successful responses to analyze".to_string(),
            ));
        }

        let similarity_matrix = self.calculate_similarity_matrix(&successful);
        let clusters = self.cluster_responses(&successful, &similarity_matrix);
        let agreed_facts = self.extract_agreed_facts(&successful);
        let outliers = self.identify_outliers(&successful, &similarity_matrix);
        let agreement_score = self.calculate_agreement_score(&clusters, successful.len());

        Ok(ConsensusResult {
            agreement_score,
            clusters,
            agreed_facts,
            outliers,
        })
    }

    fn calculate_similarity_matrix(&self, responses: &[(&str, &str)]) -> Vec<Vec<f64>> {
        let n = responses.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in i..n {
                let sim = self.text_similarity(responses[i].1, responses[j].1);
                matrix[i][j] = sim;
                matrix[j][i] = sim;
            }
        }
        matrix
    }

    fn text_similarity(&self, a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        let words_a: HashSet<&str> = a_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .collect();
        let words_b: HashSet<&str> = b_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .collect();

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn cluster_responses(
        &self,
        responses: &[(&str, &str)],
        similarity_matrix: &[Vec<f64>],
    ) -> Vec<ResponseCluster> {
        let n = responses.len();
        let mut visited = vec![false; n];
        let mut clusters = Vec::new();

        for i in 0..n {
            if visited[i] {
                continue;
            }

            let mut cluster_indices = vec![i];
            visited[i] = true;

            for j in (i + 1)..n {
                if !visited[j] && similarity_matrix[i][j] >= self.similarity_threshold {
                    cluster_indices.push(j);
                    visited[j] = true;
                }
            }

            let provider_ids: Vec<String> = cluster_indices
                .iter()
                .map(|&idx| responses[idx].0.to_string())
                .collect();

            let representative_idx = cluster_indices[0];
            let representative_content = responses[representative_idx].1.to_string();

            let avg_similarity: f64 = if cluster_indices.len() > 1 {
                let mut sum = 0.0;
                let mut count = 0;
                for &idx_a in &cluster_indices {
                    for &idx_b in &cluster_indices {
                        if idx_a < idx_b {
                            sum += similarity_matrix[idx_a][idx_b];
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    sum / count as f64
                } else {
                    1.0
                }
            } else {
                1.0
            };

            clusters.push(ResponseCluster {
                provider_ids,
                representative_content,
                similarity_score: avg_similarity,
            });
        }

        clusters
    }

    fn extract_agreed_facts(&self, responses: &[(&str, &str)]) -> Vec<AgreedFact> {
        let mut sentence_providers: HashMap<String, Vec<String>> = HashMap::new();

        for (provider_id, content) in responses {
            let sentences: Vec<&str> = content
                .split('.')
                .map(|s| s.trim())
                .filter(|s| s.len() > 15)
                .collect();

            for sentence in sentences {
                let normalized = sentence.to_lowercase();
                sentence_providers
                    .entry(normalized)
                    .or_default()
                    .push(provider_id.to_string());
            }
        }

        let total_providers = responses.len();
        let mut agreed_facts: Vec<AgreedFact> = sentence_providers
            .into_iter()
            .filter_map(|(fact, providers)| {
                let agreement_ratio = providers.len() as f64 / total_providers as f64;
                if agreement_ratio >= self.minimum_agreement {
                    Some(AgreedFact {
                        fact,
                        supporting_providers: providers,
                        agreement_ratio,
                    })
                } else {
                    None
                }
            })
            .collect();

        agreed_facts.sort_by(|a, b| {
            b.agreement_ratio
                .partial_cmp(&a.agreement_ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        agreed_facts
    }

    fn identify_outliers(
        &self,
        responses: &[(&str, &str)],
        similarity_matrix: &[Vec<f64>],
    ) -> Vec<String> {
        let n = responses.len();
        let mut outliers = Vec::new();

        for i in 0..n {
            let avg_similarity: f64 = similarity_matrix[i]
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &sim)| sim)
                .sum::<f64>()
                / (n - 1).max(1) as f64;

            if avg_similarity < self.similarity_threshold * 0.5 {
                outliers.push(responses[i].0.to_string());
            }
        }

        outliers
    }

    fn calculate_agreement_score(&self, clusters: &[ResponseCluster], total: usize) -> f64 {
        if clusters.is_empty() || total == 0 {
            return 0.0;
        }

        let largest_cluster_size = clusters
            .iter()
            .map(|c| c.provider_ids.len())
            .max()
            .unwrap_or(0);

        let avg_cluster_similarity: f64 = clusters.iter().map(|c| c.similarity_score).sum::<f64>()
            / clusters.len() as f64;

        let size_factor = largest_cluster_size as f64 / total as f64;
        let similarity_factor = avg_cluster_similarity;

        (size_factor * 0.6 + similarity_factor * 0.4).min(1.0)
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devil::aggregator::{ProviderResponse, ResponseAggregator};
    use std::time::Duration;

    #[test]
    fn test_text_similarity() {
        let engine = ConsensusEngine::new();

        let sim = engine.text_similarity(
            "The Moon is Earth's natural satellite",
            "The Moon is the natural satellite of Earth",
        );
        assert!(sim > 0.5);

        let diff_sim = engine.text_similarity(
            "The Moon is Earth's satellite",
            "Bananas are yellow fruits",
        );
        assert!(diff_sim < 0.2);
    }

    #[test]
    fn test_consensus_analysis() {
        let responses = vec![
            ProviderResponse::success(
                "claude".to_string(),
                "m".to_string(),
                "The Moon is about 384,000 km from Earth. It has no atmosphere.".to_string(),
                Duration::from_millis(100),
            ),
            ProviderResponse::success(
                "openai".to_string(),
                "m".to_string(),
                "The Moon is approximately 384,000 km away from Earth. It lacks an atmosphere."
                    .to_string(),
                Duration::from_millis(100),
            ),
            ProviderResponse::success(
                "gemini".to_string(),
                "m".to_string(),
                "Earth's Moon is 384,000 km distant. There is no atmosphere on the Moon."
                    .to_string(),
                Duration::from_millis(100),
            ),
        ];

        let aggregator = ResponseAggregator::new();
        let aggregated = aggregator.aggregate(responses);

        let engine = ConsensusEngine::new();
        let result = engine.analyze(&aggregated).unwrap();

        assert!(result.agreement_score > 0.5);
        assert!(result.clusters.len() <= 3);
    }

    #[test]
    fn test_outlier_detection() {
        let responses = vec![
            ProviderResponse::success(
                "a".to_string(),
                "m".to_string(),
                "The sky is blue due to Rayleigh scattering.".to_string(),
                Duration::from_millis(100),
            ),
            ProviderResponse::success(
                "b".to_string(),
                "m".to_string(),
                "The sky appears blue because of light scattering.".to_string(),
                Duration::from_millis(100),
            ),
            ProviderResponse::success(
                "outlier".to_string(),
                "m".to_string(),
                "Pizza is delicious with extra cheese and pepperoni.".to_string(),
                Duration::from_millis(100),
            ),
        ];

        let aggregator = ResponseAggregator::new();
        let aggregated = aggregator.aggregate(responses);

        let engine = ConsensusEngine::new();
        let result = engine.analyze(&aggregated).unwrap();

        assert!(result.outliers.contains(&"outlier".to_string()));
    }
}
