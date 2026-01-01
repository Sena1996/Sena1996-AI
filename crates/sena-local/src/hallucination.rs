use regex::Regex;
use sena_core::{CompletionRequest, Provider, Result};
use std::collections::HashSet;
use std::sync::Arc;

use crate::consensus::{ConsensusConfig, ConsensusResult, MultiModelConsensus, VotingStrategy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HallucinationType {
    FabricatedFact,
    InconsistentClaim,
    NonExistentReference,
    InvalidCode,
    MadeUpApi,
    FalseAttribution,
    TemporalError,
    LogicalContradiction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct HallucinationDetector {
    fake_library_patterns: Vec<Regex>,
    suspicious_patterns: Vec<(Regex, HallucinationType, RiskLevel)>,
    known_fake_terms: HashSet<String>,
    confidence_phrases: Vec<Regex>,
}

impl HallucinationDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            fake_library_patterns: Vec::new(),
            suspicious_patterns: Vec::new(),
            known_fake_terms: HashSet::new(),
            confidence_phrases: Vec::new(),
        };
        detector.init_patterns();
        detector
    }

    fn init_patterns(&mut self) {
        self.fake_library_patterns = vec![
            Regex::new(r"(?i)import\s+[a-z]+_magic").ok(),
            Regex::new(r"(?i)from\s+super_[a-z]+\s+import").ok(),
            Regex::new(r#"(?i)require\(['""]@fake/"#).ok(),
            Regex::new(r"(?i)use\s+nonexistent::").ok(),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.suspicious_patterns = vec![
            (
                Regex::new(r"(?i)\b(always|never|definitely|certainly|absolutely)\b.*\b(will|must|should)\b").unwrap(),
                HallucinationType::FabricatedFact,
                RiskLevel::Medium,
            ),
            (
                Regex::new(r"(?i)according to (the latest|recent) (studies|research|data)").unwrap(),
                HallucinationType::FalseAttribution,
                RiskLevel::High,
            ),
            (
                Regex::new(r"(?i)as of (202[5-9]|20[3-9]\d)").unwrap(),
                HallucinationType::TemporalError,
                RiskLevel::High,
            ),
            (
                Regex::new(r"(?i)(version|v)\s*\d+\.\d+\.\d+.*released").unwrap(),
                HallucinationType::FabricatedFact,
                RiskLevel::Medium,
            ),
            (
                Regex::new(r"(?i)official documentation (states|says|mentions)").unwrap(),
                HallucinationType::NonExistentReference,
                RiskLevel::High,
            ),
            (
                Regex::new(r"(?i)the\s+(creator|author|founder)\s+(of|behind)\s+\w+\s+(said|stated|announced)").unwrap(),
                HallucinationType::FalseAttribution,
                RiskLevel::High,
            ),
            (
                Regex::new(r"(?i)built-in\s+(function|method)\s+called\s+\w+_\w+_\w+").unwrap(),
                HallucinationType::MadeUpApi,
                RiskLevel::High,
            ),
            (
                Regex::new(r"(?i)(this|it)\s+(is|was)\s+(not|n't).*but\s+(also|simultaneously)\s+(is|was)").unwrap(),
                HallucinationType::LogicalContradiction,
                RiskLevel::Critical,
            ),
        ];

        self.known_fake_terms = [
            "quantumjs",
            "hyperrust",
            "megapython",
            "superframework",
            "ultra-fast-db",
            "instantai",
            "magicorm",
            "wonderapi",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        self.confidence_phrases = vec![
            Regex::new(r"(?i)I('m| am)\s+(100%\s+)?(certain|sure|confident)").ok(),
            Regex::new(r"(?i)there('s| is)\s+no\s+(doubt|question)").ok(),
            Regex::new(r"(?i)it('s| is)\s+(a\s+)?well[- ]known\s+fact").ok(),
            Regex::new(r"(?i)everyone\s+knows\s+(that)?").ok(),
            Regex::new(r"(?i)obviously|clearly|undoubtedly").ok(),
        ]
        .into_iter()
        .flatten()
        .collect();
    }

    pub fn detect(&self, response: &str, context: Option<&str>) -> DetectionResult {
        let mut issues = Vec::new();
        let mut risk_level = RiskLevel::Low;

        for pattern in &self.fake_library_patterns {
            if pattern.is_match(response) {
                issues.push(HallucinationIssue {
                    issue_type: HallucinationType::MadeUpApi,
                    description: "Potentially fabricated library or import detected".to_string(),
                    risk: RiskLevel::High,
                    snippet: self.extract_snippet(response, pattern),
                });
                risk_level = risk_level.max(RiskLevel::High);
            }
        }

        for (pattern, issue_type, pattern_risk) in &self.suspicious_patterns {
            if pattern.is_match(response) {
                issues.push(HallucinationIssue {
                    issue_type: *issue_type,
                    description: format!("Suspicious pattern detected: {:?}", issue_type),
                    risk: *pattern_risk,
                    snippet: self.extract_snippet(response, pattern),
                });
                risk_level = risk_level.max(*pattern_risk);
            }
        }

        let response_lower = response.to_lowercase();
        for term in &self.known_fake_terms {
            if response_lower.contains(term) {
                issues.push(HallucinationIssue {
                    issue_type: HallucinationType::MadeUpApi,
                    description: format!("Known fake term detected: {}", term),
                    risk: RiskLevel::High,
                    snippet: Some(term.clone()),
                });
                risk_level = risk_level.max(RiskLevel::High);
            }
        }

        let overconfidence_count = self
            .confidence_phrases
            .iter()
            .filter(|p| p.is_match(response))
            .count();

        if overconfidence_count >= 2 {
            issues.push(HallucinationIssue {
                issue_type: HallucinationType::FabricatedFact,
                description: "Multiple overconfident assertions detected".to_string(),
                risk: RiskLevel::Medium,
                snippet: None,
            });
            risk_level = risk_level.max(RiskLevel::Medium);
        }

        if let Some(ctx) = context {
            if let Some(contradiction) = self.check_context_contradiction(response, ctx) {
                issues.push(contradiction);
                risk_level = risk_level.max(RiskLevel::High);
            }
        }

        let code_issues = self.check_code_validity(response);
        for issue in code_issues {
            risk_level = risk_level.max(issue.risk);
            issues.push(issue);
        }

        let is_suspicious = !issues.is_empty();
        let confidence = self.calculate_confidence(&issues);
        let recommendation = self.get_recommendation(risk_level);

        DetectionResult {
            is_suspicious,
            risk_level,
            issues,
            confidence,
            recommendation,
        }
    }

    fn extract_snippet(&self, text: &str, pattern: &Regex) -> Option<String> {
        pattern.find(text).map(|m| {
            let start = m.start().saturating_sub(20);
            let end = (m.end() + 20).min(text.len());
            format!("...{}...", &text[start..end])
        })
    }

    fn check_context_contradiction(
        &self,
        response: &str,
        context: &str,
    ) -> Option<HallucinationIssue> {
        let response_lower = response.to_lowercase();
        let context_lower = context.to_lowercase();

        let negation_pairs = [
            ("is not", "is"),
            ("does not", "does"),
            ("cannot", "can"),
            ("should not", "should"),
            ("will not", "will"),
        ];

        for (negative, positive) in negation_pairs {
            if (response_lower.contains(negative) && context_lower.contains(positive))
                || (response_lower.contains(positive) && context_lower.contains(negative))
            {
                return Some(HallucinationIssue {
                    issue_type: HallucinationType::InconsistentClaim,
                    description: "Response may contradict provided context".to_string(),
                    risk: RiskLevel::High,
                    snippet: None,
                });
            }
        }

        None
    }

    fn check_code_validity(&self, response: &str) -> Vec<HallucinationIssue> {
        let mut issues = Vec::new();

        let code_block_regex = Regex::new(r"```(\w+)?\n([\s\S]*?)```").unwrap();

        for cap in code_block_regex.captures_iter(response) {
            let lang = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let code = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            if lang == "python" || lang.is_empty() {
                if code.contains("import ") && code.contains("_magic") {
                    issues.push(HallucinationIssue {
                        issue_type: HallucinationType::MadeUpApi,
                        description: "Suspicious Python import detected".to_string(),
                        risk: RiskLevel::High,
                        snippet: Some(code.lines().next().unwrap_or("").to_string()),
                    });
                }
            }

            if lang == "rust" {
                if code.contains("use ") && code.contains("::nonexistent") {
                    issues.push(HallucinationIssue {
                        issue_type: HallucinationType::MadeUpApi,
                        description: "Suspicious Rust crate detected".to_string(),
                        risk: RiskLevel::High,
                        snippet: Some(code.lines().next().unwrap_or("").to_string()),
                    });
                }
            }

            let bracket_balance =
                code.matches('{').count() as i32 - code.matches('}').count() as i32;
            let paren_balance =
                code.matches('(').count() as i32 - code.matches(')').count() as i32;

            if bracket_balance.abs() > 2 || paren_balance.abs() > 2 {
                issues.push(HallucinationIssue {
                    issue_type: HallucinationType::InvalidCode,
                    description: "Code has unbalanced brackets or parentheses".to_string(),
                    risk: RiskLevel::Medium,
                    snippet: None,
                });
            }
        }

        issues
    }

    fn calculate_confidence(&self, issues: &[HallucinationIssue]) -> f32 {
        if issues.is_empty() {
            return 0.0;
        }

        let total_risk: f32 = issues
            .iter()
            .map(|i| match i.risk {
                RiskLevel::Low => 0.25,
                RiskLevel::Medium => 0.5,
                RiskLevel::High => 0.75,
                RiskLevel::Critical => 1.0,
            })
            .sum();

        (total_risk / issues.len() as f32).min(1.0)
    }

    fn get_recommendation(&self, risk: RiskLevel) -> String {
        match risk {
            RiskLevel::Low => "Response appears reliable".to_string(),
            RiskLevel::Medium => "Verify claims with external sources".to_string(),
            RiskLevel::High => "Cross-check all facts and code before using".to_string(),
            RiskLevel::Critical => {
                "Do not trust this response without thorough verification".to_string()
            }
        }
    }

    pub fn validate_response(&self, response: &str) -> bool {
        let result = self.detect(response, None);
        result.risk_level < RiskLevel::High
    }
}

impl Default for HallucinationDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HallucinationController {
    detector: HallucinationDetector,
    consensus: MultiModelConsensus,
    fact_checker: FactChecker,
    config: ControllerConfig,
}

#[derive(Debug, Clone)]
pub struct ControllerConfig {
    pub enable_consensus: bool,
    pub enable_fact_check: bool,
    pub consensus_threshold: f32,
    pub min_providers_for_consensus: usize,
    pub fact_check_keywords: Vec<String>,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            enable_consensus: true,
            enable_fact_check: true,
            consensus_threshold: 0.7,
            min_providers_for_consensus: 3,
            fact_check_keywords: vec![
                "according to".to_string(),
                "research shows".to_string(),
                "studies indicate".to_string(),
                "historically".to_string(),
                "founded in".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub basic_detection: DetectionResult,
    pub consensus_result: Option<ConsensusResult>,
    pub fact_check_results: Vec<FactCheckResult>,
    pub overall_confidence: f32,
    pub verified: bool,
}

impl HallucinationController {
    pub fn new(config: ControllerConfig) -> Self {
        let consensus_config = ConsensusConfig {
            min_providers: config.min_providers_for_consensus,
            similarity_threshold: config.consensus_threshold,
            timeout_ms: 30000,
            voting_strategy: VotingStrategy::Majority,
        };

        Self {
            detector: HallucinationDetector::new(),
            consensus: MultiModelConsensus::new(consensus_config),
            fact_checker: FactChecker::new(),
            config,
        }
    }

    pub fn detect(&self, response: &str, context: Option<&str>) -> DetectionResult {
        self.detector.detect(response, context)
    }

    pub async fn verify_with_consensus(
        &self,
        request: &CompletionRequest,
        providers: &[Arc<dyn Provider>],
    ) -> Result<VerificationResult> {
        let consensus_result = if self.config.enable_consensus && providers.len() >= self.config.min_providers_for_consensus {
            Some(self.consensus.verify(request, providers).await?)
        } else {
            None
        };

        let primary_response = consensus_result
            .as_ref()
            .and_then(|c| c.final_response.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");

        let basic_detection = self.detector.detect(primary_response, None);

        let fact_check_results = if self.config.enable_fact_check {
            self.check_facts(primary_response).await
        } else {
            vec![]
        };

        let overall_confidence = self.calculate_overall_confidence(
            &basic_detection,
            consensus_result.as_ref(),
            &fact_check_results,
        );

        let verified = overall_confidence >= self.config.consensus_threshold
            && basic_detection.risk_level < RiskLevel::High;

        Ok(VerificationResult {
            basic_detection,
            consensus_result,
            fact_check_results,
            overall_confidence,
            verified,
        })
    }

    async fn check_facts(&self, text: &str) -> Vec<FactCheckResult> {
        let mut results = Vec::new();

        for keyword in &self.config.fact_check_keywords {
            if text.to_lowercase().contains(&keyword.to_lowercase()) {
                let claims = self.extract_claims_near_keyword(text, keyword);
                for claim in claims {
                    if let Ok(result) = self.fact_checker.verify_claim(&claim).await {
                        results.push(result);
                    }
                }
            }
        }

        results
    }

    fn extract_claims_near_keyword(&self, text: &str, keyword: &str) -> Vec<String> {
        let mut claims = Vec::new();
        let text_lower = text.to_lowercase();

        for (idx, _) in text_lower.match_indices(&keyword.to_lowercase()) {
            let start = text[..idx].rfind(|c: char| c == '.' || c == '\n').map(|i| i + 1).unwrap_or(0);
            let end = text[idx..].find(|c: char| c == '.' || c == '\n').map(|i| idx + i + 1).unwrap_or(text.len());
            let sentence = text[start..end].trim().to_string();
            if sentence.len() > 10 {
                claims.push(sentence);
            }
        }

        claims
    }

    fn calculate_overall_confidence(
        &self,
        detection: &DetectionResult,
        consensus: Option<&ConsensusResult>,
        fact_checks: &[FactCheckResult],
    ) -> f32 {
        let mut confidence = 1.0 - detection.confidence;

        if let Some(c) = consensus {
            if c.consensus_reached {
                confidence = (confidence + c.confidence) / 2.0;
            } else {
                confidence *= 0.5;
            }
        }

        if !fact_checks.is_empty() {
            let verified_count = fact_checks.iter().filter(|f| f.verified).count();
            let fact_confidence = verified_count as f32 / fact_checks.len() as f32;
            confidence = (confidence * 0.7) + (fact_confidence * 0.3);
        }

        confidence.clamp(0.0, 1.0)
    }

    pub fn with_config(mut self, config: ControllerConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for HallucinationController {
    fn default() -> Self {
        Self::new(ControllerConfig::default())
    }
}

pub struct FactChecker {
    client: reqwest::Client,
    wikipedia_api: String,
}

#[derive(Debug, Clone)]
pub struct FactCheckResult {
    pub claim: String,
    pub source: String,
    pub verified: bool,
    pub confidence: f32,
    pub evidence: Option<String>,
}

impl FactChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            wikipedia_api: "https://en.wikipedia.org/api/rest_v1/page/summary".to_string(),
        }
    }

    pub async fn verify_claim(&self, claim: &str) -> Result<FactCheckResult> {
        let keywords = self.extract_verifiable_terms(claim);

        if keywords.is_empty() {
            return Ok(FactCheckResult {
                claim: claim.to_string(),
                source: "none".to_string(),
                verified: false,
                confidence: 0.0,
                evidence: None,
            });
        }

        for keyword in keywords.iter().take(3) {
            if let Ok(result) = self.query_wikipedia(keyword, claim).await {
                if result.verified {
                    return Ok(result);
                }
            }
        }

        Ok(FactCheckResult {
            claim: claim.to_string(),
            source: "wikipedia".to_string(),
            verified: false,
            confidence: 0.3,
            evidence: None,
        })
    }

    async fn query_wikipedia(&self, term: &str, original_claim: &str) -> Result<FactCheckResult> {
        let url = format!("{}/{}", self.wikipedia_api, urlencoding::encode(term));

        let response = self.client
            .get(&url)
            .header("User-Agent", "SENA/1.0 (Hallucination Fact Checker)")
            .send()
            .await
            .map_err(|e| sena_core::Error::network(e.to_string()))?;

        if !response.status().is_success() {
            return Ok(FactCheckResult {
                claim: original_claim.to_string(),
                source: "wikipedia".to_string(),
                verified: false,
                confidence: 0.0,
                evidence: None,
            });
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| sena_core::Error::provider(format!("failed to parse response: {}", e)))?;

        let extract = data["extract"].as_str().unwrap_or("");

        let (verified, confidence) = self.compare_claim_to_evidence(original_claim, extract);

        Ok(FactCheckResult {
            claim: original_claim.to_string(),
            source: format!("wikipedia:{}", term),
            verified,
            confidence,
            evidence: if verified { Some(extract.chars().take(500).collect()) } else { None },
        })
    }

    fn extract_verifiable_terms(&self, claim: &str) -> Vec<String> {
        let stopwords: HashSet<&str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will",
            "would", "could", "should", "may", "might", "must", "shall",
            "can", "need", "dare", "ought", "used", "to", "of", "in",
            "for", "on", "with", "at", "by", "from", "as", "into", "through",
            "that", "which", "who", "whom", "this", "these", "those", "it",
        ].iter().copied().collect();

        let words: Vec<String> = claim
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3 && !stopwords.contains(w.to_lowercase().as_str()))
            .map(String::from)
            .collect();

        let capitalized: Vec<String> = claim
            .split_whitespace()
            .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
            .filter(|w| w.len() > 2)
            .map(String::from)
            .collect();

        let mut terms = capitalized;
        terms.extend(words.into_iter().take(5));

        terms.into_iter().take(5).collect()
    }

    fn compare_claim_to_evidence(&self, claim: &str, evidence: &str) -> (bool, f32) {
        if evidence.is_empty() {
            return (false, 0.0);
        }

        let claim_lower = claim.to_lowercase();
        let evidence_lower = evidence.to_lowercase();

        let claim_words: HashSet<_> = claim_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3)
            .collect();

        let evidence_words: HashSet<_> = evidence_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3)
            .collect();

        let intersection = claim_words.intersection(&evidence_words).count();
        let union = claim_words.union(&evidence_words).count();

        if union == 0 {
            return (false, 0.0);
        }

        let similarity = intersection as f32 / union as f32;

        let verified = similarity > 0.15 && intersection >= 3;
        let confidence = (similarity * 2.0).clamp(0.0, 1.0);

        (verified, confidence)
    }
}

impl Default for FactChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub is_suspicious: bool,
    pub risk_level: RiskLevel,
    pub issues: Vec<HallucinationIssue>,
    pub confidence: f32,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub struct HallucinationIssue {
    pub issue_type: HallucinationType,
    pub description: String,
    pub risk: RiskLevel,
    pub snippet: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_response() {
        let detector = HallucinationDetector::new();
        let result = detector.detect("Here is a simple function that adds two numbers.", None);
        assert!(!result.is_suspicious || result.risk_level == RiskLevel::Low);
    }

    #[test]
    fn test_suspicious_import() {
        let detector = HallucinationDetector::new();
        let code = "```python\nimport super_magic_library\n```";
        let result = detector.detect(code, None);
        assert!(result.is_suspicious);
        assert!(result.risk_level >= RiskLevel::Medium);
    }

    #[test]
    fn test_overconfidence() {
        let detector = HallucinationDetector::new();
        let text = "I'm 100% certain this is correct. There's no doubt about it. Obviously this works.";
        let result = detector.detect(text, None);
        assert!(result.is_suspicious);
    }

    #[test]
    fn test_future_date() {
        let detector = HallucinationDetector::new();
        let result = detector.detect("As of 2028, this library was updated.", None);
        assert!(result.is_suspicious);
        assert!(result
            .issues
            .iter()
            .any(|i| i.issue_type == HallucinationType::TemporalError));
    }

    #[test]
    fn test_valid_response() {
        let detector = HallucinationDetector::new();
        assert!(detector.validate_response("The function returns the sum of two integers."));
    }
}
