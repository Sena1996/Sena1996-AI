use std::collections::{HashMap, HashSet};

use super::aggregator::AggregatedResponses;
use super::config::SynthesisMethod;
use super::consensus::ConsensusResult;
use super::error::{DevilError, DevilResult};

#[derive(Debug, Clone)]
pub struct SynthesizedResponse {
    pub content: String,
    pub method: SynthesisMethod,
    pub confidence: f64,
    pub verification_rounds: Option<usize>,
    pub facts_verified: Option<usize>,
    pub facts_rejected: Option<usize>,
}

pub struct ResponseSynthesizer {
    method: SynthesisMethod,
    max_facts: usize,
}

impl ResponseSynthesizer {
    pub fn new(method: SynthesisMethod) -> Self {
        Self {
            method,
            max_facts: 20,
        }
    }

    pub fn with_max_facts(mut self, max_facts: usize) -> Self {
        self.max_facts = max_facts;
        self
    }

    pub fn synthesize(
        &self,
        aggregated: &AggregatedResponses,
        consensus: &ConsensusResult,
    ) -> DevilResult<SynthesizedResponse> {
        match self.method {
            SynthesisMethod::MajorityVoting => self.majority_vote(aggregated, consensus),
            SynthesisMethod::WeightedMerge => self.weighted_merge(aggregated, consensus),
            SynthesisMethod::BestOfN => self.best_of_n(aggregated, consensus),
            SynthesisMethod::LongestCommonSubsequence => self.lcs_merge(aggregated),
            SynthesisMethod::MetaLLM => self.meta_llm_placeholder(aggregated),
            SynthesisMethod::CrossVerification => {
                self.cross_verification_sync(aggregated, consensus)
            }
        }
    }

    fn majority_vote(
        &self,
        _aggregated: &AggregatedResponses,
        consensus: &ConsensusResult,
    ) -> DevilResult<SynthesizedResponse> {
        let largest_cluster = consensus
            .clusters
            .iter()
            .max_by_key(|c| c.provider_ids.len())
            .ok_or_else(|| DevilError::SynthesisError("No clusters found".to_string()))?;

        Ok(SynthesizedResponse {
            content: largest_cluster.representative_content.clone(),
            method: SynthesisMethod::MajorityVoting,
            confidence: largest_cluster.similarity_score,
            verification_rounds: None,
            facts_verified: None,
            facts_rejected: None,
        })
    }

    fn weighted_merge(
        &self,
        aggregated: &AggregatedResponses,
        consensus: &ConsensusResult,
    ) -> DevilResult<SynthesizedResponse> {
        let mut facts: HashMap<String, f64> = HashMap::new();

        for response in &aggregated.responses {
            if let Some(content) = &response.content {
                for sentence in content.split('.') {
                    let sentence = sentence.trim();
                    if sentence.len() > 15 {
                        let latency_weight = 1.0 / (response.latency_ms as f64 / 1000.0 + 0.1);
                        let consensus_weight = consensus.get_fact_agreement(sentence).max(0.1);
                        let weight = latency_weight * consensus_weight;
                        *facts.entry(sentence.to_string()).or_default() += weight;
                    }
                }
            }
        }

        let mut sorted_facts: Vec<_> = facts.into_iter().collect();
        sorted_facts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let content = sorted_facts
            .into_iter()
            .take(self.max_facts)
            .map(|(fact, _)| fact)
            .collect::<Vec<_>>()
            .join(". ")
            + ".";

        Ok(SynthesizedResponse {
            content,
            method: SynthesisMethod::WeightedMerge,
            confidence: consensus.agreement_score,
            verification_rounds: None,
            facts_verified: None,
            facts_rejected: None,
        })
    }

    fn best_of_n(
        &self,
        aggregated: &AggregatedResponses,
        consensus: &ConsensusResult,
    ) -> DevilResult<SynthesizedResponse> {
        let best = aggregated
            .responses
            .iter()
            .filter(|r| r.content.is_some())
            .min_by_key(|r| r.latency_ms)
            .ok_or_else(|| DevilError::SynthesisError("No successful responses".to_string()))?;

        Ok(SynthesizedResponse {
            content: best.content.clone().unwrap_or_default(),
            method: SynthesisMethod::BestOfN,
            confidence: consensus.agreement_score,
            verification_rounds: None,
            facts_verified: None,
            facts_rejected: None,
        })
    }

    fn lcs_merge(&self, aggregated: &AggregatedResponses) -> DevilResult<SynthesizedResponse> {
        let contents: Vec<&str> = aggregated
            .responses
            .iter()
            .filter_map(|r| r.content.as_deref())
            .collect();

        if contents.is_empty() {
            return Err(DevilError::SynthesisError("No content to merge".to_string()));
        }

        let all_sentences: Vec<HashSet<&str>> = contents
            .iter()
            .map(|c| {
                c.split('.')
                    .map(|s| s.trim())
                    .filter(|s| s.len() > 10)
                    .collect()
            })
            .collect();

        let common_sentences: HashSet<&str> = if all_sentences.len() > 1 {
            all_sentences[0]
                .iter()
                .filter(|s| all_sentences[1..].iter().all(|set| set.contains(*s)))
                .copied()
                .collect()
        } else {
            all_sentences.first().cloned().unwrap_or_default()
        };

        let content = if common_sentences.is_empty() {
            contents.first().copied().unwrap_or("").to_string()
        } else {
            common_sentences
                .into_iter()
                .collect::<Vec<_>>()
                .join(". ")
                + "."
        };

        Ok(SynthesizedResponse {
            content,
            method: SynthesisMethod::LongestCommonSubsequence,
            confidence: 0.8,
            verification_rounds: None,
            facts_verified: None,
            facts_rejected: None,
        })
    }

    fn meta_llm_placeholder(
        &self,
        aggregated: &AggregatedResponses,
    ) -> DevilResult<SynthesizedResponse> {
        let contents: Vec<&str> = aggregated
            .responses
            .iter()
            .filter_map(|r| r.content.as_deref())
            .collect();

        let content = format!(
            "[Meta-LLM synthesis would process {} responses here]\n\n{}",
            contents.len(),
            contents.first().copied().unwrap_or("")
        );

        Ok(SynthesizedResponse {
            content,
            method: SynthesisMethod::MetaLLM,
            confidence: 0.7,
            verification_rounds: None,
            facts_verified: None,
            facts_rejected: None,
        })
    }

    fn cross_verification_sync(
        &self,
        aggregated: &AggregatedResponses,
        consensus: &ConsensusResult,
    ) -> DevilResult<SynthesizedResponse> {
        let mut all_facts: Vec<String> = Vec::new();
        for response in &aggregated.responses {
            if let Some(content) = &response.content {
                let facts = self.extract_facts(content);
                all_facts.extend(facts);
            }
        }

        let unique_facts_set: HashSet<String> = all_facts.into_iter().collect();
        let unique_facts: Vec<String> = unique_facts_set.into_iter().collect();
        let total_unique_facts = unique_facts.len();

        let provider_count = aggregated.successful_count;
        let min_votes = (provider_count / 2) + 1;

        let mut verified_facts: Vec<String> = Vec::new();
        let mut rejected_count = 0;

        for fact in &unique_facts {
            let agreement = consensus.get_fact_agreement(fact);
            let supporting_count = (agreement * provider_count as f64).ceil() as usize;

            if supporting_count >= min_votes {
                verified_facts.push(fact.clone());
            } else {
                rejected_count += 1;
            }
        }

        if verified_facts.is_empty() && !unique_facts.is_empty() {
            verified_facts = unique_facts
                .into_iter()
                .take(self.max_facts / 2)
                .collect();
        }

        let content = if verified_facts.is_empty() {
            "Cross-verification could not confirm any facts across all AI models.".to_string()
        } else {
            verified_facts
                .iter()
                .take(self.max_facts)
                .cloned()
                .collect::<Vec<_>>()
                .join(". ")
                + "."
        };

        let facts_verified = verified_facts.len();

        Ok(SynthesizedResponse {
            content,
            method: SynthesisMethod::CrossVerification,
            confidence: if total_unique_facts == 0 {
                0.0
            } else {
                facts_verified as f64 / (facts_verified + rejected_count).max(1) as f64
            },
            verification_rounds: Some(1),
            facts_verified: Some(facts_verified),
            facts_rejected: Some(rejected_count),
        })
    }

    fn extract_facts(&self, text: &str) -> Vec<String> {
        text.split('.')
            .map(|s| s.trim().to_string())
            .filter(|s| {
                s.len() > 15
                    && !s.to_lowercase().starts_with("i ")
                    && !s.to_lowercase().starts_with("we ")
                    && !s.contains('?')
            })
            .collect()
    }
}

impl Default for ResponseSynthesizer {
    fn default() -> Self {
        Self::new(SynthesisMethod::CrossVerification)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devil::aggregator::{ProviderResponse, ResponseAggregator};
    use crate::devil::consensus::ConsensusEngine;
    use std::time::Duration;

    fn create_test_aggregated() -> AggregatedResponses {
        let responses = vec![
            ProviderResponse::success(
                "claude".to_string(),
                "m".to_string(),
                "The Moon is 384,000 km from Earth. It has no atmosphere. The Moon is tidally locked."
                    .to_string(),
                Duration::from_millis(1500),
            ),
            ProviderResponse::success(
                "openai".to_string(),
                "m".to_string(),
                "The Moon is approximately 384,000 km away. It lacks an atmosphere. It is tidally locked to Earth."
                    .to_string(),
                Duration::from_millis(1200),
            ),
        ];

        ResponseAggregator::new().aggregate(responses)
    }

    #[test]
    fn test_majority_voting() {
        let aggregated = create_test_aggregated();
        let consensus = ConsensusEngine::new().analyze(&aggregated).unwrap();
        let synthesizer = ResponseSynthesizer::new(SynthesisMethod::MajorityVoting);

        let result = synthesizer.synthesize(&aggregated, &consensus).unwrap();
        assert!(!result.content.is_empty());
        assert_eq!(result.method, SynthesisMethod::MajorityVoting);
    }

    #[test]
    fn test_weighted_merge() {
        let aggregated = create_test_aggregated();
        let consensus = ConsensusEngine::new().analyze(&aggregated).unwrap();
        let synthesizer = ResponseSynthesizer::new(SynthesisMethod::WeightedMerge);

        let result = synthesizer.synthesize(&aggregated, &consensus).unwrap();
        assert!(!result.content.is_empty());
    }

    #[test]
    fn test_cross_verification() {
        let aggregated = create_test_aggregated();
        let consensus = ConsensusEngine::new().analyze(&aggregated).unwrap();
        let synthesizer = ResponseSynthesizer::new(SynthesisMethod::CrossVerification);

        let result = synthesizer.synthesize(&aggregated, &consensus).unwrap();
        assert!(result.verification_rounds.is_some());
        assert!(result.facts_verified.is_some());
    }

    #[test]
    fn test_extract_facts() {
        let synthesizer = ResponseSynthesizer::default();
        let facts =
            synthesizer.extract_facts("The Moon is large. I think it is beautiful. Is it far?");

        assert_eq!(facts.len(), 1);
        assert!(facts[0].contains("Moon"));
    }
}
