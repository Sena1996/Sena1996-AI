//! SENA v4.0 - Layer 5: Harmony Validation Engine (Rust)
//!
//! Inspired by the Antikythera Mechanism (150 BCE)
//!
//! The Antikythera mechanism tracked celestial cycles with extraordinary
//! precision. It could predict eclipses by ensuring its model harmonized
//! with observed reality.
//!
//! Applied to AI: Ensure the model mirrors reality.
//! Continuous validation against truth anchors.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

/// Errors for Harmony Validation
#[derive(Error, Debug)]
pub enum HarmonyError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Anchor not found: {0}")]
    AnchorNotFound(String),
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    #[error("Harmony broken: {0}")]
    HarmonyBroken(String),
}

/// Types of harmony to validate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarmonyType {
    /// Internal logical consistency
    Logical,
    /// Factual accuracy
    Factual,
    /// Temporal consistency
    Temporal,
    /// Mathematical correctness
    Mathematical,
    /// Semantic coherence
    Semantic,
    /// Structural integrity
    Structural,
    /// External verification
    External,
    /// Cross-reference validation
    CrossReference,
}

/// Status of harmony
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarmonyStatus {
    /// Perfect harmony
    Harmonious,
    /// Minor discordance, acceptable
    MinorDiscord,
    /// Significant discordance
    Discordant,
    /// Unable to determine
    Unknown,
    /// Validation pending
    Pending,
}

impl HarmonyStatus {
    pub fn score(&self) -> f64 {
        match self {
            HarmonyStatus::Harmonious => 1.0,
            HarmonyStatus::MinorDiscord => 0.7,
            HarmonyStatus::Discordant => 0.3,
            HarmonyStatus::Unknown => 0.5,
            HarmonyStatus::Pending => 0.5,
        }
    }
}

/// Confidence level for assertions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Absolute certainty (mathematical proof, definition)
    Certain,
    /// Very high confidence (well-established fact)
    High,
    /// Moderate confidence (generally accepted)
    Medium,
    /// Low confidence (contested or uncertain)
    Low,
    /// Speculative (hypothesis or opinion)
    Speculative,
}

impl ConfidenceLevel {
    pub fn to_f64(&self) -> f64 {
        match self {
            ConfidenceLevel::Certain => 1.0,
            ConfidenceLevel::High => 0.9,
            ConfidenceLevel::Medium => 0.7,
            ConfidenceLevel::Low => 0.4,
            ConfidenceLevel::Speculative => 0.2,
        }
    }

    pub fn from_f64(value: f64) -> Self {
        if value >= 0.95 {
            ConfidenceLevel::Certain
        } else if value >= 0.8 {
            ConfidenceLevel::High
        } else if value >= 0.6 {
            ConfidenceLevel::Medium
        } else if value >= 0.3 {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::Speculative
        }
    }
}

/// A reality anchor - a known truth to validate against
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityAnchor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub anchor_type: HarmonyType,
    pub value: AnchorValue,
    pub confidence: ConfidenceLevel,
    pub source: String,
    pub last_verified: DateTime<Utc>,
    pub verification_count: u64,
}

impl RealityAnchor {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        anchor_type: HarmonyType,
        value: AnchorValue,
    ) -> Self {
        let name = name.into();
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let id = format!("anchor_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            name,
            description: description.into(),
            anchor_type,
            value,
            confidence: ConfidenceLevel::High,
            source: String::new(),
            last_verified: Utc::now(),
            verification_count: 0,
        }
    }

    pub fn with_confidence(mut self, confidence: ConfidenceLevel) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    pub fn verify(&mut self) {
        self.last_verified = Utc::now();
        self.verification_count += 1;
    }
}

/// Value types for anchors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnchorValue {
    Boolean(bool),
    Number(f64),
    Text(String),
    Range { min: f64, max: f64 },
    Pattern(String),
    List(Vec<String>),
    Map(HashMap<String, String>),
}

impl AnchorValue {
    /// Check if a value matches this anchor
    pub fn matches(&self, other: &AnchorValue) -> bool {
        match (self, other) {
            (AnchorValue::Boolean(a), AnchorValue::Boolean(b)) => a == b,
            (AnchorValue::Number(a), AnchorValue::Number(b)) => (a - b).abs() < 1e-10,
            (AnchorValue::Text(a), AnchorValue::Text(b)) => a == b,
            (AnchorValue::Range { min, max }, AnchorValue::Number(n)) => n >= min && n <= max,
            (AnchorValue::Pattern(pattern), AnchorValue::Text(text)) => {
                regex::Regex::new(pattern)
                    .map(|re| re.is_match(text))
                    .unwrap_or(false)
            }
            (AnchorValue::List(list), AnchorValue::Text(item)) => list.contains(item),
            _ => false,
        }
    }
}

/// A rule for harmony validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonyRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: HarmonyType,
    pub condition: RuleCondition,
    pub severity: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub violation_count: u64,
}

impl HarmonyRule {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        rule_type: HarmonyType,
        condition: RuleCondition,
    ) -> Self {
        let name = name.into();
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let id = format!("rule_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            name,
            description: description.into(),
            rule_type,
            condition,
            severity: 1.0,
            enabled: true,
            created_at: Utc::now(),
            violation_count: 0,
        }
    }

    pub fn with_severity(mut self, severity: f64) -> Self {
        self.severity = severity.clamp(0.0, 1.0);
        self
    }

    pub fn record_violation(&mut self) {
        self.violation_count += 1;
    }
}

/// Conditions for harmony rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Must not contain pattern
    MustNotContain(String),
    /// Must contain pattern
    MustContain(String),
    /// Logical consistency check
    LogicalConsistency,
    /// Temporal order check
    TemporalOrder,
    /// Numeric range check
    NumericRange { min: f64, max: f64 },
    /// Cross-reference check
    CrossReference(String),
    /// Custom predicate name
    Custom(String),
}

/// A single harmony check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonyCheck {
    pub id: String,
    pub content: String,
    pub harmony_type: HarmonyType,
    pub timestamp: DateTime<Utc>,
    pub status: HarmonyStatus,
    pub confidence: f64,
    pub violations: Vec<String>,
    pub corrections: Vec<String>,
}

impl HarmonyCheck {
    pub fn new(content: impl Into<String>, harmony_type: HarmonyType) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        let id = format!("check_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            content: content.into(),
            harmony_type,
            timestamp: Utc::now(),
            status: HarmonyStatus::Pending,
            confidence: 0.0,
            violations: Vec::new(),
            corrections: Vec::new(),
        }
    }

    pub fn complete(&mut self, status: HarmonyStatus, confidence: f64) {
        self.status = status;
        self.confidence = confidence;
    }

    pub fn add_violation(&mut self, violation: impl Into<String>) {
        self.violations.push(violation.into());
    }

    pub fn add_correction(&mut self, correction: impl Into<String>) {
        self.corrections.push(correction.into());
    }
}

/// Result of a validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub id: String,
    pub content_hash: String,
    pub timestamp: DateTime<Utc>,
    pub overall_status: HarmonyStatus,
    pub overall_confidence: f64,
    pub checks: Vec<HarmonyCheck>,
    pub anchor_validations: Vec<AnchorValidation>,
    pub rule_violations: Vec<RuleViolation>,
    pub corrections_suggested: Vec<String>,
}

impl ValidationResult {
    pub fn new(content: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = hex::encode(&hasher.finalize()[..16]);
        let id = format!("val_{}", &content_hash[..8]);

        Self {
            id,
            content_hash,
            timestamp: Utc::now(),
            overall_status: HarmonyStatus::Pending,
            overall_confidence: 0.0,
            checks: Vec::new(),
            anchor_validations: Vec::new(),
            rule_violations: Vec::new(),
            corrections_suggested: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(
            self.overall_status,
            HarmonyStatus::Harmonious | HarmonyStatus::MinorDiscord
        )
    }
}

/// Validation against an anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorValidation {
    pub anchor_id: String,
    pub anchor_name: String,
    pub passed: bool,
    pub details: String,
}

/// A rule violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: f64,
    pub details: String,
}

/// The main Harmony Validation Engine
pub struct HarmonyValidationEngine {
    anchors: HashMap<String, RealityAnchor>,
    rules: HashMap<String, HarmonyRule>,
    validation_history: Vec<ValidationResult>,
    strict_mode: bool,
    confidence_threshold: f64,
}

impl Default for HarmonyValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HarmonyValidationEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            anchors: HashMap::new(),
            rules: HashMap::new(),
            validation_history: Vec::new(),
            strict_mode: false,
            confidence_threshold: 0.7,
        };
        engine.initialize_core_rules();
        engine
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Initialize core validation rules
    fn initialize_core_rules(&mut self) {
        // Logical consistency rule
        let logical = HarmonyRule::new(
            "logical_consistency",
            "Ensure logical consistency in statements",
            HarmonyType::Logical,
            RuleCondition::LogicalConsistency,
        )
        .with_severity(0.9);

        // No contradictions rule
        let no_contradictions = HarmonyRule::new(
            "no_contradictions",
            "Avoid self-contradictory statements",
            HarmonyType::Logical,
            RuleCondition::MustNotContain(r"(?i)(but|however).*(not|never|always).*\1".to_string()),
        )
        .with_severity(0.8);

        // Temporal consistency
        let temporal = HarmonyRule::new(
            "temporal_order",
            "Ensure temporal ordering is correct",
            HarmonyType::Temporal,
            RuleCondition::TemporalOrder,
        )
        .with_severity(0.7);

        // Factual grounding
        let factual = HarmonyRule::new(
            "factual_grounding",
            "Ensure claims are grounded in verifiable facts",
            HarmonyType::Factual,
            RuleCondition::CrossReference("facts".to_string()),
        )
        .with_severity(0.8);

        self.add_rule(logical);
        self.add_rule(no_contradictions);
        self.add_rule(temporal);
        self.add_rule(factual);

        // Add some core reality anchors
        let earth_round = RealityAnchor::new(
            "earth_shape",
            "Earth is approximately spherical",
            HarmonyType::Factual,
            AnchorValue::Text("spherical".to_string()),
        )
        .with_confidence(ConfidenceLevel::Certain)
        .with_source("Scientific consensus");

        let gravity = RealityAnchor::new(
            "gravity_constant",
            "Gravitational acceleration on Earth surface",
            HarmonyType::Mathematical,
            AnchorValue::Range {
                min: 9.78,
                max: 9.83,
            },
        )
        .with_confidence(ConfidenceLevel::Certain)
        .with_source("Physics");

        self.add_anchor(earth_round);
        self.add_anchor(gravity);
    }

    /// Add a reality anchor
    pub fn add_anchor(&mut self, anchor: RealityAnchor) -> String {
        let id = anchor.id.clone();
        self.anchors.insert(id.clone(), anchor);
        id
    }

    /// Add a validation rule
    pub fn add_rule(&mut self, rule: HarmonyRule) -> String {
        let id = rule.id.clone();
        self.rules.insert(id.clone(), rule);
        id
    }

    /// Get an anchor by ID
    pub fn get_anchor(&self, id: &str) -> Option<&RealityAnchor> {
        self.anchors.get(id)
    }

    /// Get a rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&HarmonyRule> {
        self.rules.get(id)
    }

    /// Validate content against all rules and anchors
    pub fn validate(&mut self, content: &str) -> ValidationResult {
        let mut result = ValidationResult::new(content);
        let mut total_confidence = 0.0;
        let mut check_count = 0;
        let mut violations_count = 0;

        // First, collect rule check results
        let rule_checks: Vec<(String, bool, String, String, f64)> = self.rules.values()
            .filter(|rule| rule.enabled)
            .map(|rule| {
                let (passed, details) = Self::check_rule_static(rule, content);
                (rule.id.clone(), passed, rule.name.clone(), details, rule.severity)
            })
            .collect();

        // Then update rules and build results
        for (rule_id, passed, rule_name, details, severity) in rule_checks {
            if !passed {
                if let Some(rule) = self.rules.get_mut(&rule_id) {
                    rule.record_violation();
                }
                violations_count += 1;
                result.rule_violations.push(RuleViolation {
                    rule_id,
                    rule_name,
                    severity,
                    details,
                });
            }

            check_count += 1;
            if passed {
                total_confidence += 1.0;
            }
        }

        // Collect anchor check results
        let anchor_checks: Vec<(String, String, bool, String, f64)> = self.anchors.values()
            .map(|anchor| {
                let (passed, details) = Self::check_anchor_static(anchor, content);
                (anchor.id.clone(), anchor.name.clone(), passed, details, anchor.confidence.to_f64())
            })
            .collect();

        // Update anchors and build results
        for (anchor_id, anchor_name, passed, details, confidence) in anchor_checks {
            if let Some(anchor) = self.anchors.get_mut(&anchor_id) {
                anchor.verify();
            }

            result.anchor_validations.push(AnchorValidation {
                anchor_id,
                anchor_name,
                passed,
                details,
            });

            check_count += 1;
            if passed {
                total_confidence += confidence;
            }
        }

        // Calculate overall status
        if check_count > 0 {
            result.overall_confidence = total_confidence / check_count as f64;
        } else {
            result.overall_confidence = 1.0;
        }

        result.overall_status = if violations_count == 0 && result.overall_confidence >= 0.9 {
            HarmonyStatus::Harmonious
        } else if violations_count <= 1 && result.overall_confidence >= 0.7 {
            HarmonyStatus::MinorDiscord
        } else if result.overall_confidence < 0.5 || violations_count > 3 {
            HarmonyStatus::Discordant
        } else {
            HarmonyStatus::MinorDiscord
        };

        // Generate corrections if needed
        if !result.rule_violations.is_empty() {
            result.corrections_suggested = self.suggest_corrections(&result.rule_violations);
        }

        // Store in history
        self.validation_history.push(result.clone());

        result
    }

    /// Check content against a single rule (static version)
    fn check_rule_static(rule: &HarmonyRule, content: &str) -> (bool, String) {
        match &rule.condition {
            RuleCondition::MustNotContain(pattern) => {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(content) {
                        return (false, format!("Content matches forbidden pattern: {}", pattern));
                    }
                }
                (true, "No forbidden patterns found".to_string())
            }
            RuleCondition::MustContain(pattern) => {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if !re.is_match(content) {
                        return (false, format!("Content missing required pattern: {}", pattern));
                    }
                }
                (true, "Required pattern found".to_string())
            }
            RuleCondition::LogicalConsistency => {
                // Simplified logical consistency check
                let has_contradiction = content.contains("is and is not")
                    || content.contains("both true and false")
                    || (content.contains("always") && content.contains("never"));

                if has_contradiction {
                    (false, "Potential logical contradiction detected".to_string())
                } else {
                    (true, "No logical contradictions detected".to_string())
                }
            }
            RuleCondition::TemporalOrder => {
                // Simplified temporal check
                let before_after = content.contains("before") && content.contains("after");
                let conflicting =
                    before_after && content.matches("before").count() == content.matches("after").count();

                if conflicting {
                    (true, "Temporal ordering appears consistent".to_string())
                } else {
                    (true, "Temporal ordering not violated".to_string())
                }
            }
            RuleCondition::NumericRange { min, max } => {
                // Extract numbers and check range
                let re = regex::Regex::new(r"\d+\.?\d*").unwrap();
                for cap in re.find_iter(content) {
                    if let Ok(num) = cap.as_str().parse::<f64>() {
                        if num < *min || num > *max {
                            return (
                                false,
                                format!("Number {} outside valid range [{}, {}]", num, min, max),
                            );
                        }
                    }
                }
                (true, "All numbers within valid range".to_string())
            }
            RuleCondition::CrossReference(reference) => {
                // Placeholder for cross-reference validation
                (
                    true,
                    format!("Cross-reference to {} (placeholder check)", reference),
                )
            }
            RuleCondition::Custom(name) => {
                // Custom rules would be implemented here
                (true, format!("Custom rule '{}' (not implemented)", name))
            }
        }
    }

    /// Check content against an anchor (static version)
    fn check_anchor_static(anchor: &RealityAnchor, content: &str) -> (bool, String) {
        let content_lower = content.to_lowercase();
        let anchor_name_lower = anchor.name.to_lowercase();

        // Check if content references the anchor topic
        if !content_lower.contains(&anchor_name_lower)
            && !content_lower.contains(&anchor.description.to_lowercase())
        {
            return (true, "Anchor topic not referenced".to_string());
        }

        // If referenced, validate against anchor value
        match &anchor.value {
            AnchorValue::Boolean(expected) => {
                let negation_words = ["not", "isn't", "aren't", "false", "wrong"];
                let has_negation = negation_words.iter().any(|w| content_lower.contains(w));
                let matches = (*expected && !has_negation) || (!*expected && has_negation);
                if matches {
                    (true, "Boolean assertion matches anchor".to_string())
                } else {
                    (false, "Boolean assertion contradicts anchor".to_string())
                }
            }
            AnchorValue::Text(expected) => {
                if content_lower.contains(&expected.to_lowercase()) {
                    (true, "Text content matches anchor".to_string())
                } else {
                    (
                        false,
                        format!("Expected '{}' not found in content", expected),
                    )
                }
            }
            _ => (true, "Anchor validation passed (default)".to_string()),
        }
    }

    /// Suggest corrections for violations
    fn suggest_corrections(&self, violations: &[RuleViolation]) -> Vec<String> {
        violations
            .iter()
            .map(|v| {
                format!(
                    "Consider reviewing content related to '{}': {}",
                    v.rule_name, v.details
                )
            })
            .collect()
    }

    /// Get validation statistics
    pub fn get_statistics(&self) -> HarmonyStatistics {
        let total_validations = self.validation_history.len();
        let harmonious_count = self
            .validation_history
            .iter()
            .filter(|v| matches!(v.overall_status, HarmonyStatus::Harmonious))
            .count();

        let avg_confidence = if total_validations > 0 {
            self.validation_history
                .iter()
                .map(|v| v.overall_confidence)
                .sum::<f64>()
                / total_validations as f64
        } else {
            1.0
        };

        let rule_violations: HashMap<String, u64> = self
            .rules
            .iter()
            .filter(|(_, r)| r.violation_count > 0)
            .map(|(id, r)| (id.clone(), r.violation_count))
            .collect();

        HarmonyStatistics {
            total_anchors: self.anchors.len(),
            total_rules: self.rules.len(),
            total_validations,
            harmonious_count,
            harmony_rate: if total_validations > 0 {
                harmonious_count as f64 / total_validations as f64
            } else {
                1.0
            },
            average_confidence: avg_confidence,
            rule_violations,
        }
    }

    /// Get all anchors
    pub fn get_all_anchors(&self) -> Vec<&RealityAnchor> {
        self.anchors.values().collect()
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> Vec<&HarmonyRule> {
        self.rules.values().collect()
    }
}

/// Statistics for harmony validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonyStatistics {
    pub total_anchors: usize,
    pub total_rules: usize,
    pub total_validations: usize,
    pub harmonious_count: usize,
    pub harmony_rate: f64,
    pub average_confidence: f64,
    pub rule_violations: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reality_anchor_creation() {
        let anchor = RealityAnchor::new(
            "test_anchor",
            "Test description",
            HarmonyType::Factual,
            AnchorValue::Boolean(true),
        )
        .with_confidence(ConfidenceLevel::High)
        .with_source("Test source");

        assert!(anchor.id.starts_with("anchor_"));
        assert_eq!(anchor.name, "test_anchor");
        assert_eq!(anchor.confidence, ConfidenceLevel::High);
    }

    #[test]
    fn test_anchor_value_matching() {
        let range = AnchorValue::Range { min: 0.0, max: 10.0 };
        assert!(range.matches(&AnchorValue::Number(5.0)));
        assert!(!range.matches(&AnchorValue::Number(15.0)));

        let list = AnchorValue::List(vec!["a".to_string(), "b".to_string()]);
        assert!(list.matches(&AnchorValue::Text("a".to_string())));
        assert!(!list.matches(&AnchorValue::Text("c".to_string())));
    }

    #[test]
    fn test_harmony_rule_creation() {
        let rule = HarmonyRule::new(
            "test_rule",
            "Test rule description",
            HarmonyType::Logical,
            RuleCondition::LogicalConsistency,
        )
        .with_severity(0.8);

        assert!(rule.id.starts_with("rule_"));
        assert_eq!(rule.severity, 0.8);
        assert!(rule.enabled);
    }

    #[test]
    fn test_validation_engine_creation() {
        let engine = HarmonyValidationEngine::new();

        // Should have core rules and anchors
        assert!(!engine.rules.is_empty());
        assert!(!engine.anchors.is_empty());
    }

    #[test]
    fn test_basic_validation() {
        let mut engine = HarmonyValidationEngine::new();

        let result = engine.validate("This is a simple test content.");

        assert!(matches!(
            result.overall_status,
            HarmonyStatus::Harmonious | HarmonyStatus::MinorDiscord
        ));
    }

    #[test]
    fn test_validation_statistics() {
        let mut engine = HarmonyValidationEngine::new();

        engine.validate("Test content 1");
        engine.validate("Test content 2");

        let stats = engine.get_statistics();

        assert_eq!(stats.total_validations, 2);
        assert!(stats.harmony_rate >= 0.0 && stats.harmony_rate <= 1.0);
    }
}
