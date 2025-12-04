use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use crate::ancient::{HarmonyStatus, HarmonyValidationEngine, NegativeSpaceArchitecture};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HallucinationResponse {
    Block,
    Warn,
    Log,
    Pass,
}

#[derive(Debug, Clone)]
pub struct HallucinationResult {
    pub is_hallucination: bool,
    pub risk_score: f64,
    pub response: HallucinationResponse,
    pub harmony_status: HarmonyStatus,
    pub warnings: Vec<String>,
    pub details: HallucinationDetails,
}

#[derive(Debug, Clone, Default)]
pub struct HallucinationDetails {
    pub consistency_score: f64,
    pub semantic_entropy: f64,
    pub fact_validation_score: f64,
    pub suspicious_patterns: Vec<String>,
}

pub struct HallucinationDetector {
    negative_space: Arc<RwLock<NegativeSpaceArchitecture>>,
    harmony_engine: Arc<RwLock<HarmonyValidationEngine>>,
    confidence_threshold: f64,
    block_threshold: f64,
    warn_threshold: f64,
    log_threshold: f64,
}

impl HallucinationDetector {
    pub fn new(
        negative_space: Arc<RwLock<NegativeSpaceArchitecture>>,
        harmony_engine: Arc<RwLock<HarmonyValidationEngine>>,
    ) -> Self {
        Self {
            negative_space,
            harmony_engine,
            confidence_threshold: 0.7,
            block_threshold: 0.85,
            warn_threshold: 0.70,
            log_threshold: 0.50,
        }
    }

    pub fn with_threshold(
        negative_space: Arc<RwLock<NegativeSpaceArchitecture>>,
        harmony_engine: Arc<RwLock<HarmonyValidationEngine>>,
        threshold: f64,
    ) -> Self {
        Self {
            negative_space,
            harmony_engine,
            confidence_threshold: threshold,
            block_threshold: 0.85,
            warn_threshold: 0.70,
            log_threshold: 0.50,
        }
    }

    pub fn check(&self, content: &str) -> HallucinationResult {
        let ns_check = self
            .negative_space
            .write()
            .map(|mut ns| ns.check_action(content, &std::collections::HashMap::new()))
            .unwrap_or_else(|_| crate::ancient::NegativeSpaceCheckResult::default_allowed());

        let harmony_result = self
            .harmony_engine
            .write()
            .map(|mut he| he.validate(content))
            .unwrap_or_else(|_| crate::ancient::harmony_validation::ValidationResult::default());

        let semantic_entropy = self.calculate_semantic_entropy(content);
        let suspicious_patterns = self.detect_suspicious_patterns(content);

        let risk_components = [
            ns_check.risk_score * 0.3,
            (1.0 - harmony_result.overall_confidence) * 0.3,
            semantic_entropy * 0.2,
            (suspicious_patterns.len() as f64 / 10.0).min(1.0) * 0.2,
        ];

        let risk_score: f64 = risk_components.iter().sum();

        let response = if risk_score > self.block_threshold {
            HallucinationResponse::Block
        } else if risk_score > self.warn_threshold {
            HallucinationResponse::Warn
        } else if risk_score > self.log_threshold {
            HallucinationResponse::Log
        } else {
            HallucinationResponse::Pass
        };

        let is_hallucination = risk_score > self.confidence_threshold;

        let mut warnings = Vec::new();
        if !ns_check.allowed {
            warnings.push("Content violates negative space constraints".to_string());
        }
        if harmony_result.overall_confidence < 0.5 {
            warnings.push("Low harmony validation confidence".to_string());
        }
        if semantic_entropy > 0.7 {
            warnings.push("High semantic entropy detected".to_string());
        }
        for pattern in &suspicious_patterns {
            warnings.push(format!("Suspicious pattern: {}", pattern));
        }

        HallucinationResult {
            is_hallucination,
            risk_score,
            response,
            harmony_status: harmony_result.overall_status,
            warnings,
            details: HallucinationDetails {
                consistency_score: 1.0 - ns_check.risk_score,
                semantic_entropy,
                fact_validation_score: harmony_result.overall_confidence,
                suspicious_patterns,
            },
        }
    }

    fn calculate_semantic_entropy(&self, text: &str) -> f64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }

        let unique_words: HashSet<&str> = words.iter().copied().collect();
        let uniqueness_ratio = unique_words.len() as f64 / words.len() as f64;

        let avg_word_length: f64 = words.iter().map(|w| w.len() as f64).sum::<f64>() / words.len() as f64;

        let length_variance = if avg_word_length > 8.0 {
            (avg_word_length - 8.0) / 20.0
        } else {
            0.0
        };

        ((1.0 - uniqueness_ratio) * 0.5 + length_variance * 0.5).min(1.0)
    }

    fn detect_suspicious_patterns(&self, text: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let text_lower = text.to_lowercase();

        let certainty_phrases = [
            "i am absolutely certain",
            "this is definitely true",
            "there is no doubt",
            "100% accurate",
            "guaranteed to be",
            "without any possibility of error",
        ];

        for phrase in certainty_phrases {
            if text_lower.contains(phrase) {
                patterns.push(format!("Overconfident assertion: '{}'", phrase));
            }
        }

        let contradiction_pairs = [
            ("always", "never"),
            ("all", "none"),
            ("everyone", "no one"),
            ("everything", "nothing"),
        ];

        for (a, b) in contradiction_pairs {
            if text_lower.contains(a) && text_lower.contains(b) {
                patterns.push(format!("Potential contradiction: '{}' and '{}'", a, b));
            }
        }

        let vague_quantifiers = [
            "many experts say",
            "studies show",
            "research indicates",
            "it is known that",
            "sources confirm",
        ];

        for phrase in vague_quantifiers {
            if text_lower.contains(phrase) {
                patterns.push(format!("Vague attribution: '{}'", phrase));
            }
        }

        patterns
    }

    pub fn check_consistency(&self, responses: &[String]) -> f64 {
        if responses.len() < 2 {
            return 1.0;
        }

        let mut total_similarity: f64 = 0.0;
        let mut comparisons: u32 = 0;

        for i in 0..responses.len() {
            for j in (i + 1)..responses.len() {
                total_similarity += self.text_similarity(&responses[i], &responses[j]);
                comparisons += 1;
            }
        }

        if comparisons == 0 {
            1.0
        } else {
            total_similarity / comparisons as f64
        }
    }

    fn text_similarity(&self, a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        let words_a: HashSet<&str> = a_lower.split_whitespace().collect();
        let words_b: HashSet<&str> = b_lower.split_whitespace().collect();

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_detector() -> HallucinationDetector {
        HallucinationDetector::new(
            Arc::new(RwLock::new(NegativeSpaceArchitecture::new())),
            Arc::new(RwLock::new(HarmonyValidationEngine::new())),
        )
    }

    #[test]
    fn test_normal_content() {
        let detector = create_detector();
        let result = detector.check("Rust is a systems programming language focused on safety.");

        assert!(!result.is_hallucination);
        assert!(result.risk_score < 0.7);
    }

    #[test]
    fn test_overconfident_content() {
        let detector = create_detector();
        let result = detector.check(
            "I am absolutely certain this is 100% accurate and there is no doubt about it.",
        );

        assert!(result.details.suspicious_patterns.len() >= 2);
    }

    #[test]
    fn test_consistency_check() {
        let detector = create_detector();

        let similar_responses = vec![
            "The sky is blue due to Rayleigh scattering.".to_string(),
            "The sky appears blue because of Rayleigh scattering of light.".to_string(),
        ];
        let consistency = detector.check_consistency(&similar_responses);
        assert!(consistency > 0.3);

        let different_responses = vec![
            "The sky is blue.".to_string(),
            "Bananas are yellow fruits.".to_string(),
        ];
        let inconsistency = detector.check_consistency(&different_responses);
        assert!(inconsistency < 0.3);
    }

    #[test]
    fn test_semantic_entropy() {
        let detector = create_detector();

        let low_entropy = "the the the the the";
        let entropy = detector.calculate_semantic_entropy(low_entropy);
        assert!(entropy > 0.3);

        let normal_text = "This is a normal sentence with varied vocabulary.";
        let normal_entropy = detector.calculate_semantic_entropy(normal_text);
        assert!(normal_entropy < 0.5);
    }
}
