//! SENA v5.0 - Layer 2: Negative Space Architecture (Rust)
//!
//! Inspired by Sushruta (600 BCE) - The Father of Surgery
//!
//! Ancient surgeons had detailed taxonomies of what NOT to cut.
//! The negative space (where NOT to operate) was as important as
//! the positive space (where to operate).
//!
//! Applied to AI: Define failure modes before success modes.
//! Know what the system must NEVER do.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

/// Errors for Negative Space operations
#[derive(Error, Debug)]
pub enum NegativeSpaceError {
    #[error("Violation detected: {0}")]
    ViolationDetected(String),
    #[error("Prohibition not found: {0}")]
    ProhibitionNotFound(String),
    #[error("Invalid boundary definition: {0}")]
    InvalidBoundary(String),
    #[error("Action blocked by prohibition: {0}")]
    ActionBlocked(String),
}

/// Severity levels for prohibitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProhibitionLevel {
    /// Absolute prohibition - NEVER violate
    Absolute,
    /// Strong prohibition - avoid unless extreme circumstances
    Strong,
    /// Standard prohibition - normal safety boundary
    Standard,
    /// Advisory prohibition - warning level
    Advisory,
}

impl ProhibitionLevel {
    pub fn severity_score(&self) -> f64 {
        match self {
            ProhibitionLevel::Absolute => 1.0,
            ProhibitionLevel::Strong => 0.8,
            ProhibitionLevel::Standard => 0.5,
            ProhibitionLevel::Advisory => 0.2,
        }
    }
}

/// Severity of a violation attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Critical - immediate halt required
    Critical,
    /// Major - significant concern
    Major,
    /// Minor - logged but may continue
    Minor,
    /// Negligible - informational only
    Negligible,
}

/// Categories of prohibitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProhibitionCategory {
    /// Safety-related prohibitions
    Safety,
    /// Privacy-related prohibitions
    Privacy,
    /// Ethics-related prohibitions
    Ethics,
    /// Legal-related prohibitions
    Legal,
    /// Technical-related prohibitions
    Technical,
    /// Operational-related prohibitions
    Operational,
}

/// A single prohibition rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prohibition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: ProhibitionLevel,
    pub category: ProhibitionCategory,
    pub patterns: Vec<String>,
    pub context_conditions: HashMap<String, String>,
    pub exceptions: Vec<String>,
    pub rationale: String,
    pub created_at: DateTime<Utc>,
    pub violation_count: u64,
    pub last_violation: Option<DateTime<Utc>>,
}

impl Prohibition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        level: ProhibitionLevel,
        category: ProhibitionCategory,
    ) -> Self {
        let name = name.into();
        let description = description.into();
        let id = Self::generate_id(&name, &description);

        Self {
            id,
            name,
            description,
            level,
            category,
            patterns: Vec::new(),
            context_conditions: HashMap::new(),
            exceptions: Vec::new(),
            rationale: String::new(),
            created_at: Utc::now(),
            violation_count: 0,
            last_violation: None,
        }
    }

    fn generate_id(name: &str, description: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(description.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        format!("proh_{}", hex::encode(&hasher.finalize()[..8]))
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns.extend(patterns);
        self
    }

    pub fn with_exception(mut self, exception: impl Into<String>) -> Self {
        self.exceptions.push(exception.into());
        self
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = rationale.into();
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context_conditions.insert(key.into(), value.into());
        self
    }

    /// Check if content matches this prohibition
    pub fn matches(&self, content: &str, context: &HashMap<String, String>) -> bool {
        // Check context conditions first
        for (key, expected_value) in &self.context_conditions {
            if let Some(actual_value) = context.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            }
        }

        // Check if content matches any pattern
        for pattern in &self.patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(content) {
                    // Check exceptions
                    for exception in &self.exceptions {
                        if let Ok(exc_regex) = regex::Regex::new(exception) {
                            if exc_regex.is_match(content) {
                                return false;
                            }
                        }
                    }
                    return true;
                }
            } else {
                // Fall back to simple contains check
                if content.to_lowercase().contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }

        false
    }

    pub fn record_violation(&mut self) {
        self.violation_count += 1;
        self.last_violation = Some(Utc::now());
    }
}

/// Defines a safe boundary for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeBoundary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub boundary_type: BoundaryType,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Vec<String>,
    pub forbidden_values: Vec<String>,
    pub validation_fn_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryType {
    Numeric,
    Enumerated,
    Pattern,
    Custom,
}

impl SafeBoundary {
    pub fn numeric(
        name: impl Into<String>,
        description: impl Into<String>,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        let name = name.into();
        let id = format!("bnd_{}", &name.to_lowercase().replace(' ', "_"));

        Self {
            id,
            name,
            description: description.into(),
            boundary_type: BoundaryType::Numeric,
            min_value: min,
            max_value: max,
            allowed_values: Vec::new(),
            forbidden_values: Vec::new(),
            validation_fn_name: None,
        }
    }

    pub fn enumerated(
        name: impl Into<String>,
        description: impl Into<String>,
        allowed: Vec<String>,
    ) -> Self {
        let name = name.into();
        let id = format!("bnd_{}", &name.to_lowercase().replace(' ', "_"));

        Self {
            id,
            name,
            description: description.into(),
            boundary_type: BoundaryType::Enumerated,
            min_value: None,
            max_value: None,
            allowed_values: allowed,
            forbidden_values: Vec::new(),
            validation_fn_name: None,
        }
    }

    pub fn with_forbidden(mut self, forbidden: Vec<String>) -> Self {
        self.forbidden_values = forbidden;
        self
    }

    /// Check if a value is within this boundary
    pub fn is_within(&self, value: &BoundaryValue) -> bool {
        match (&self.boundary_type, value) {
            (BoundaryType::Numeric, BoundaryValue::Number(n)) => {
                let above_min = self.min_value.map_or(true, |min| *n >= min);
                let below_max = self.max_value.map_or(true, |max| *n <= max);
                above_min && below_max
            }
            (BoundaryType::Enumerated, BoundaryValue::Text(s)) => {
                if self.forbidden_values.contains(s) {
                    return false;
                }
                if self.allowed_values.is_empty() {
                    return true;
                }
                self.allowed_values.contains(s)
            }
            (BoundaryType::Pattern, BoundaryValue::Text(s)) => {
                for forbidden in &self.forbidden_values {
                    if let Ok(regex) = regex::Regex::new(forbidden) {
                        if regex.is_match(s) {
                            return false;
                        }
                    }
                }
                true
            }
            _ => true,
        }
    }
}

/// Value types for boundary checking
#[derive(Debug, Clone)]
pub enum BoundaryValue {
    Number(f64),
    Text(String),
    Boolean(bool),
}

/// Record of a violation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationAttempt {
    pub id: String,
    pub prohibition_id: String,
    pub timestamp: DateTime<Utc>,
    pub content_hash: String,
    pub severity: ViolationSeverity,
    pub context: HashMap<String, String>,
    pub action_taken: String,
    pub blocked: bool,
}

impl ViolationAttempt {
    pub fn new(
        prohibition_id: impl Into<String>,
        content: &str,
        severity: ViolationSeverity,
        context: HashMap<String, String>,
        blocked: bool,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = hex::encode(&hasher.finalize()[..16]);

        let id = format!("viol_{}", &content_hash[..8]);

        Self {
            id,
            prohibition_id: prohibition_id.into(),
            timestamp: Utc::now(),
            content_hash,
            severity,
            context,
            action_taken: if blocked {
                "BLOCKED".to_string()
            } else {
                "LOGGED".to_string()
            },
            blocked,
        }
    }
}

/// Complete negative space definition for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativeSpaceDefinition {
    pub id: String,
    pub domain: String,
    pub description: String,
    pub prohibitions: Vec<Prohibition>,
    pub boundaries: Vec<SafeBoundary>,
    pub created_at: DateTime<Utc>,
    pub version: String,
}

impl NegativeSpaceDefinition {
    pub fn new(domain: impl Into<String>, description: impl Into<String>) -> Self {
        let domain = domain.into();
        let id = format!("nsd_{}", domain.to_lowercase().replace(' ', "_"));

        Self {
            id,
            domain,
            description: description.into(),
            prohibitions: Vec::new(),
            boundaries: Vec::new(),
            created_at: Utc::now(),
            version: "1.0.0".to_string(),
        }
    }

    pub fn add_prohibition(&mut self, prohibition: Prohibition) {
        self.prohibitions.push(prohibition);
    }

    pub fn add_boundary(&mut self, boundary: SafeBoundary) {
        self.boundaries.push(boundary);
    }
}

/// Result of checking an action against negative space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativeSpaceCheckResult {
    pub allowed: bool,
    pub violations: Vec<ViolationAttempt>,
    pub boundary_violations: Vec<String>,
    pub risk_score: f64,
    pub recommendations: Vec<String>,
}

impl NegativeSpaceCheckResult {
    pub fn default_allowed() -> Self {
        Self {
            allowed: true,
            violations: Vec::new(),
            boundary_violations: Vec::new(),
            risk_score: 0.0,
            recommendations: Vec::new(),
        }
    }
}

/// The main Negative Space Architecture engine
pub struct NegativeSpaceArchitecture {
    definitions: HashMap<String, NegativeSpaceDefinition>,
    prohibitions: HashMap<String, Prohibition>,
    boundaries: HashMap<String, SafeBoundary>,
    violation_history: Vec<ViolationAttempt>,
    strict_mode: bool,
}

impl Default for NegativeSpaceArchitecture {
    fn default() -> Self {
        Self::new()
    }
}

impl NegativeSpaceArchitecture {
    pub fn new() -> Self {
        let mut engine = Self {
            definitions: HashMap::new(),
            prohibitions: HashMap::new(),
            boundaries: HashMap::new(),
            violation_history: Vec::new(),
            strict_mode: true,
        };
        engine.initialize_core_prohibitions();
        engine
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Initialize core safety prohibitions
    fn initialize_core_prohibitions(&mut self) {
        // Absolute prohibitions - NEVER violate
        let harmful_content = Prohibition::new(
            "no_harmful_content",
            "Never generate content designed to harm individuals or groups",
            ProhibitionLevel::Absolute,
            ProhibitionCategory::Safety,
        )
        .with_patterns(vec![
            r"(?i)how\s+to\s+(kill|harm|hurt|injure)".to_string(),
            r"(?i)instructions\s+for\s+(weapons|explosives|poison)".to_string(),
            r"(?i)ways\s+to\s+(attack|assault|murder)".to_string(),
        ])
        .with_rationale("Fundamental safety requirement to prevent physical harm");

        let privacy_violation = Prohibition::new(
            "no_privacy_violation",
            "Never expose or seek to expose private personal information",
            ProhibitionLevel::Absolute,
            ProhibitionCategory::Privacy,
        )
        .with_patterns(vec![
            r"(?i)(social\s+security|ssn)\s*:?\s*\d".to_string(),
            r"(?i)credit\s+card\s*:?\s*\d{4}".to_string(),
            r"\b\d{3}-\d{2}-\d{4}\b".to_string(),
        ])
        .with_rationale("Privacy is a fundamental right that must be protected");

        let deception = Prohibition::new(
            "no_deception",
            "Never deliberately deceive users about AI identity or capabilities",
            ProhibitionLevel::Absolute,
            ProhibitionCategory::Ethics,
        )
        .with_patterns(vec![
            r"(?i)i\s+am\s+(a\s+)?human".to_string(),
            r"(?i)i\s+have\s+(real\s+)?feelings".to_string(),
            r"(?i)i\s+am\s+not\s+(a\s+)?(robot|ai|artificial)".to_string(),
        ])
        .with_exception(r"(?i)(roleplay|fiction|story|character)".to_string())
        .with_rationale("Trust requires honesty about nature and capabilities");

        // Strong prohibitions
        let misinformation = Prohibition::new(
            "no_misinformation",
            "Avoid generating known false information as fact",
            ProhibitionLevel::Strong,
            ProhibitionCategory::Ethics,
        )
        .with_rationale("Information integrity is essential for trust");

        let unauthorized_access = Prohibition::new(
            "no_unauthorized_access",
            "Never assist in unauthorized system access",
            ProhibitionLevel::Strong,
            ProhibitionCategory::Legal,
        )
        .with_patterns(vec![
            r"(?i)hack\s+(into|password)".to_string(),
            r"(?i)bypass\s+(security|authentication)".to_string(),
            r"(?i)exploit\s+(vulnerability|bug)".to_string(),
        ])
        .with_exception(r"(?i)(educational|ctf|authorized|pentesting)".to_string())
        .with_rationale("Unauthorized access is illegal and harmful");

        // Add to engine
        self.add_prohibition(harmful_content);
        self.add_prohibition(privacy_violation);
        self.add_prohibition(deception);
        self.add_prohibition(misinformation);
        self.add_prohibition(unauthorized_access);

        // Add core boundaries
        let response_length = SafeBoundary::numeric(
            "response_length",
            "Maximum response length in tokens",
            Some(1.0),
            Some(100000.0),
        );

        let confidence_threshold = SafeBoundary::numeric(
            "confidence_threshold",
            "Minimum confidence for assertions",
            Some(0.0),
            Some(1.0),
        );

        self.add_boundary(response_length);
        self.add_boundary(confidence_threshold);
    }

    /// Add a prohibition to the engine
    pub fn add_prohibition(&mut self, prohibition: Prohibition) {
        self.prohibitions
            .insert(prohibition.id.clone(), prohibition);
    }

    /// Add a boundary to the engine
    pub fn add_boundary(&mut self, boundary: SafeBoundary) {
        self.boundaries.insert(boundary.id.clone(), boundary);
    }

    /// Add a complete negative space definition
    pub fn add_definition(&mut self, definition: NegativeSpaceDefinition) {
        // Also add individual prohibitions and boundaries
        for prohibition in &definition.prohibitions {
            self.prohibitions
                .insert(prohibition.id.clone(), prohibition.clone());
        }
        for boundary in &definition.boundaries {
            self.boundaries
                .insert(boundary.id.clone(), boundary.clone());
        }
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Define a new prohibition
    pub fn define_prohibition(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        level: ProhibitionLevel,
        category: ProhibitionCategory,
        patterns: Vec<String>,
    ) -> String {
        let prohibition =
            Prohibition::new(name, description, level, category).with_patterns(patterns);

        let id = prohibition.id.clone();
        self.add_prohibition(prohibition);
        id
    }

    /// Check if an action violates any prohibitions
    pub fn check_action(
        &mut self,
        content: &str,
        context: &HashMap<String, String>,
    ) -> NegativeSpaceCheckResult {
        let mut violations = Vec::new();
        let mut risk_score = 0.0;
        let mut recommendations = Vec::new();

        // Check against all prohibitions
        for prohibition in self.prohibitions.values_mut() {
            if prohibition.matches(content, context) {
                let severity = match prohibition.level {
                    ProhibitionLevel::Absolute => ViolationSeverity::Critical,
                    ProhibitionLevel::Strong => ViolationSeverity::Major,
                    ProhibitionLevel::Standard => ViolationSeverity::Minor,
                    ProhibitionLevel::Advisory => ViolationSeverity::Negligible,
                };

                let blocked = matches!(
                    prohibition.level,
                    ProhibitionLevel::Absolute | ProhibitionLevel::Strong
                ) && self.strict_mode;

                let violation = ViolationAttempt::new(
                    &prohibition.id,
                    content,
                    severity,
                    context.clone(),
                    blocked,
                );

                risk_score += prohibition.level.severity_score();
                prohibition.record_violation();

                recommendations.push(format!(
                    "Violation of '{}': {}",
                    prohibition.name, prohibition.description
                ));

                violations.push(violation);
            }
        }

        // Normalize risk score
        if !self.prohibitions.is_empty() {
            risk_score /= self.prohibitions.len() as f64;
        }

        // Store violations in history
        self.violation_history.extend(violations.clone());

        let has_blocking_violation = violations.iter().any(|v| v.blocked);

        NegativeSpaceCheckResult {
            allowed: !has_blocking_violation,
            violations,
            boundary_violations: Vec::new(),
            risk_score,
            recommendations,
        }
    }

    /// Check a value against boundaries
    pub fn check_boundary(
        &self,
        boundary_id: &str,
        value: &BoundaryValue,
    ) -> Result<bool, NegativeSpaceError> {
        let boundary = self
            .boundaries
            .get(boundary_id)
            .ok_or_else(|| NegativeSpaceError::InvalidBoundary(boundary_id.to_string()))?;

        Ok(boundary.is_within(value))
    }

    /// Validate a complete response
    pub fn validate_response(
        &mut self,
        content: &str,
        context: &HashMap<String, String>,
    ) -> Result<NegativeSpaceCheckResult, NegativeSpaceError> {
        let result = self.check_action(content, context);

        if !result.allowed && self.strict_mode {
            return Err(NegativeSpaceError::ActionBlocked(
                result
                    .violations
                    .iter()
                    .map(|v| v.prohibition_id.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        Ok(result)
    }

    /// Get violation history
    pub fn get_violation_history(&self) -> &[ViolationAttempt] {
        &self.violation_history
    }

    /// Get all prohibitions
    pub fn get_prohibitions(&self) -> Vec<&Prohibition> {
        self.prohibitions.values().collect()
    }

    /// Get prohibition by ID
    pub fn get_prohibition(&self, id: &str) -> Option<&Prohibition> {
        self.prohibitions.get(id)
    }

    /// Get all boundaries
    pub fn get_boundaries(&self) -> Vec<&SafeBoundary> {
        self.boundaries.values().collect()
    }

    /// Enforce boundaries on multiple values
    pub fn enforce_boundaries(
        &self,
        checks: Vec<(&str, BoundaryValue)>,
    ) -> Result<(), NegativeSpaceError> {
        for (boundary_id, value) in checks {
            if !self.check_boundary(boundary_id, &value)? {
                return Err(NegativeSpaceError::InvalidBoundary(format!(
                    "Value violates boundary: {}",
                    boundary_id
                )));
            }
        }
        Ok(())
    }

    /// Generate a safety report
    pub fn generate_safety_report(&self) -> SafetyReport {
        let total_violations = self.violation_history.len();
        let critical_violations = self
            .violation_history
            .iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Critical))
            .count();
        let blocked_count = self.violation_history.iter().filter(|v| v.blocked).count();

        let most_violated: Vec<(String, u64)> = self
            .prohibitions
            .values()
            .filter(|p| p.violation_count > 0)
            .map(|p| (p.name.clone(), p.violation_count))
            .collect();

        SafetyReport {
            total_prohibitions: self.prohibitions.len(),
            total_boundaries: self.boundaries.len(),
            total_violations,
            critical_violations,
            blocked_count,
            most_violated,
            strict_mode: self.strict_mode,
        }
    }
}

/// Safety report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    pub total_prohibitions: usize,
    pub total_boundaries: usize,
    pub total_violations: usize,
    pub critical_violations: usize,
    pub blocked_count: usize,
    pub most_violated: Vec<(String, u64)>,
    pub strict_mode: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prohibition_creation() {
        let prohibition = Prohibition::new(
            "test_prohibition",
            "Test description",
            ProhibitionLevel::Standard,
            ProhibitionCategory::Safety,
        )
        .with_pattern(r"(?i)test\s+pattern")
        .with_rationale("Test rationale");

        assert!(prohibition.id.starts_with("proh_"));
        assert_eq!(prohibition.name, "test_prohibition");
        assert_eq!(prohibition.level, ProhibitionLevel::Standard);
        assert!(!prohibition.patterns.is_empty());
    }

    #[test]
    fn test_prohibition_matching() {
        let prohibition = Prohibition::new(
            "test",
            "Test",
            ProhibitionLevel::Standard,
            ProhibitionCategory::Safety,
        )
        .with_pattern(r"(?i)forbidden\s+word");

        let context = HashMap::new();

        assert!(prohibition.matches("This contains forbidden word", &context));
        assert!(!prohibition.matches("This is safe", &context));
    }

    #[test]
    fn test_safe_boundary_numeric() {
        let boundary = SafeBoundary::numeric("test", "Test boundary", Some(0.0), Some(100.0));

        assert!(boundary.is_within(&BoundaryValue::Number(50.0)));
        assert!(boundary.is_within(&BoundaryValue::Number(0.0)));
        assert!(boundary.is_within(&BoundaryValue::Number(100.0)));
        assert!(!boundary.is_within(&BoundaryValue::Number(-1.0)));
        assert!(!boundary.is_within(&BoundaryValue::Number(101.0)));
    }

    #[test]
    fn test_safe_boundary_enumerated() {
        let boundary = SafeBoundary::enumerated(
            "test",
            "Test boundary",
            vec!["allowed1".to_string(), "allowed2".to_string()],
        );

        assert!(boundary.is_within(&BoundaryValue::Text("allowed1".to_string())));
        assert!(boundary.is_within(&BoundaryValue::Text("allowed2".to_string())));
        assert!(!boundary.is_within(&BoundaryValue::Text("forbidden".to_string())));
    }

    #[test]
    fn test_negative_space_architecture_creation() {
        let engine = NegativeSpaceArchitecture::new();

        // Should have core prohibitions initialized
        assert!(!engine.prohibitions.is_empty());
        assert!(!engine.boundaries.is_empty());
    }

    #[test]
    fn test_check_action_safe() {
        let mut engine = NegativeSpaceArchitecture::new().with_strict_mode(false);
        let context = HashMap::new();

        let result = engine.check_action("Hello, how are you?", &context);

        assert!(result.allowed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_check_action_violation() {
        let mut engine = NegativeSpaceArchitecture::new().with_strict_mode(true);
        let context = HashMap::new();

        let result = engine.check_action("I am a human, not an AI", &context);

        // Should detect deception prohibition
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_safety_report() {
        let engine = NegativeSpaceArchitecture::new();
        let report = engine.generate_safety_report();

        assert!(report.total_prohibitions > 0);
        assert!(report.total_boundaries > 0);
        assert!(report.strict_mode);
    }
}
