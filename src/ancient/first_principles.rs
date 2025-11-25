//! SENA v5.0 - Layer 0: First Principles Engine (Rust)
//!
//! Ancient Source: Eratosthenes (240 BCE)
//! - Measured Earth's circumference with 98% accuracy
//! - Used only: observation + geometry + logic
//! - Method: Observe first, theorize after
//!
//! Core Principle:
//!     Before building anything, understand WHY it works.
//!     Start from observable reality, not assumptions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Status of a principle or observation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TruthStatus {
    /// Raw observation, not yet analyzed
    Observed,
    /// Assumption being challenged
    Questioned,
    /// Confirmed through testing
    Verified,
    /// Fundamental truth, cannot be questioned
    Bedrock,
    /// Proven false
    Invalidated,
}

/// A raw observation - never process, just record.
/// Like Eratosthenes observing the shadow in Alexandria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, serde_json::Value>,
    pub source: String,
    pub anomaly: bool,
    pub questions_raised: Vec<String>,
}

impl Observation {
    pub fn new(content: String, context: HashMap<String, serde_json::Value>, source: String) -> Self {
        let timestamp = Utc::now();
        let id = generate_id(&format!("{}{}", content, timestamp));

        Self {
            id,
            content,
            timestamp,
            context,
            source,
            anomaly: false,
            questions_raised: Vec::new(),
        }
    }
}

/// A verified fundamental truth.
/// Like: "Light travels in straight lines" or "Gravity pulls downward"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstPrinciple {
    pub id: String,
    pub statement: String,
    pub verification_method: String,
    pub evidence: Vec<String>,
    pub status: TruthStatus,
    pub domain: String,
    pub dependencies: Vec<String>,
    pub created: DateTime<Utc>,
    pub last_verified: DateTime<Utc>,
}

impl FirstPrinciple {
    pub fn new(
        statement: String,
        verification_method: String,
        evidence: Vec<String>,
        status: TruthStatus,
        domain: String,
    ) -> Self {
        let now = Utc::now();
        let id = generate_id(&format!("{}{}", statement, domain));

        Self {
            id,
            statement,
            verification_method,
            evidence,
            status,
            domain,
            dependencies: Vec::new(),
            created: now,
            last_verified: now,
        }
    }

    /// Is this a bedrock truth that cannot be further reduced?
    pub fn is_bedrock(&self) -> bool {
        self.status == TruthStatus::Bedrock && self.dependencies.is_empty()
    }
}

/// Something we believe but haven't verified.
/// Must be tracked and eventually converted to FirstPrinciple or invalidated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub id: String,
    pub belief: String,
    pub why_believed: String,
    pub evidence_for: Vec<String>,
    pub evidence_against: Vec<String>,
    pub risk_if_wrong: String,
    pub created: DateTime<Utc>,
}

impl Assumption {
    pub fn new(belief: String, why_believed: String, risk_if_wrong: String) -> Self {
        Self {
            id: generate_id(&belief),
            belief,
            why_believed,
            evidence_for: Vec::new(),
            evidence_against: Vec::new(),
            risk_if_wrong,
            created: Utc::now(),
        }
    }
}

/// Result of building from first principles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub goal: String,
    pub applicable_principles: Vec<PrincipleRef>,
    pub unverified_assumptions: Vec<AssumptionRef>,
    pub safe_to_build: bool,
    pub warnings: Vec<String>,
}

/// Reference to a principle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipleRef {
    pub id: String,
    pub statement: String,
    pub status: String,
}

/// Reference to an assumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionRef {
    pub id: String,
    pub belief: String,
    pub risk: String,
}

/// Engine for first principles thinking.
///
/// Method (from Eratosthenes):
/// 1. OBSERVE - Record raw observations without interpretation
/// 2. QUESTION - Challenge all assumptions
/// 3. REDUCE - Break down to fundamental truths
/// 4. VERIFY - Test each principle
/// 5. BUILD - Reconstruct from verified foundations
pub struct FirstPrinciplesEngine {
    observations: HashMap<String, Observation>,
    principles: HashMap<String, FirstPrinciple>,
    assumptions: HashMap<String, Assumption>,
    question_queue: Vec<String>,
}

impl Default for FirstPrinciplesEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FirstPrinciplesEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            observations: HashMap::new(),
            principles: HashMap::new(),
            assumptions: HashMap::new(),
            question_queue: Vec::new(),
        };

        engine.init_bedrock_principles();
        engine
    }

    fn init_bedrock_principles(&mut self) {
        let bedrock_principles = vec![
            FirstPrinciple {
                id: "bedrock_001".to_string(),
                statement: "Users deserve accurate, truthful responses".to_string(),
                verification_method: "Axiom - ethical foundation".to_string(),
                evidence: vec![
                    "Universal human expectation".to_string(),
                    "Basis of trust".to_string(),
                ],
                status: TruthStatus::Bedrock,
                domain: "ethics".to_string(),
                dependencies: Vec::new(),
                created: Utc::now(),
                last_verified: Utc::now(),
            },
            FirstPrinciple {
                id: "bedrock_002".to_string(),
                statement: "Systems should fail safely, never catastrophically".to_string(),
                verification_method: "Engineering consensus".to_string(),
                evidence: vec![
                    "Roman concrete lasted 2000 years".to_string(),
                    "Qanat systems self-maintain".to_string(),
                ],
                status: TruthStatus::Bedrock,
                domain: "reliability".to_string(),
                dependencies: Vec::new(),
                created: Utc::now(),
                last_verified: Utc::now(),
            },
            FirstPrinciple {
                id: "bedrock_003".to_string(),
                statement: "Constraints provide free resources if understood correctly".to_string(),
                verification_method: "Ancient engineering evidence".to_string(),
                evidence: vec![
                    "Qanat used gravity (constraint) as energy".to_string(),
                    "Pyramids used simple tools".to_string(),
                ],
                status: TruthStatus::Bedrock,
                domain: "design".to_string(),
                dependencies: Vec::new(),
                created: Utc::now(),
                last_verified: Utc::now(),
            },
            FirstPrinciple {
                id: "bedrock_004".to_string(),
                statement: "Knowledge stored in structure is faster than knowledge retrieved".to_string(),
                verification_method: "Polynesian navigation proof".to_string(),
                evidence: vec![
                    "Star compass encoded in memory, not tools".to_string(),
                    "Zhang Heng seismoscope".to_string(),
                ],
                status: TruthStatus::Bedrock,
                domain: "architecture".to_string(),
                dependencies: Vec::new(),
                created: Utc::now(),
                last_verified: Utc::now(),
            },
            FirstPrinciple {
                id: "bedrock_005".to_string(),
                statement: "Define what must NEVER happen before defining what should happen".to_string(),
                verification_method: "Sushruta surgical methodology".to_string(),
                evidence: vec!["2600 years of successful surgical practice".to_string()],
                status: TruthStatus::Bedrock,
                domain: "safety".to_string(),
                dependencies: Vec::new(),
                created: Utc::now(),
                last_verified: Utc::now(),
            },
        ];

        for principle in bedrock_principles {
            self.principles.insert(principle.id.clone(), principle);
        }
    }

    // =========================================================================
    // STEP 1: OBSERVE
    // =========================================================================

    /// Record a raw observation without processing.
    pub fn observe(
        &mut self,
        content: String,
        context: HashMap<String, serde_json::Value>,
        source: String,
    ) -> Observation {
        let mut obs = Observation::new(content, context, source);

        // Check if anomalous
        obs.anomaly = self.is_anomalous(&obs);

        // Generate questions
        obs.questions_raised = self.generate_questions(&obs);

        // Add questions to queue
        self.question_queue.extend(obs.questions_raised.clone());

        self.observations.insert(obs.id.clone(), obs.clone());
        obs
    }

    fn is_anomalous(&self, obs: &Observation) -> bool {
        for principle in self.principles.values() {
            if self.contradicts(&obs.content, &principle.statement) {
                return true;
            }
        }
        false
    }

    fn contradicts(&self, observation: &str, principle: &str) -> bool {
        let negatives = ["not", "never", "cannot", "impossible", "false"];
        let obs_lower = observation.to_lowercase();
        let prin_lower = principle.to_lowercase();

        for word in obs_lower.split_whitespace() {
            if negatives.contains(&word) {
                continue;
            }
            if prin_lower.contains(word) && negatives.iter().any(|neg| obs_lower.contains(neg)) {
                return true;
            }
        }
        false
    }

    fn generate_questions(&self, obs: &Observation) -> Vec<String> {
        let mut questions = Vec::new();

        // Always ask WHY
        questions.push(format!("WHY: {}?", obs.content));

        // If anomalous, ask what's different
        if obs.anomaly {
            questions.push("ANOMALY: Why does this differ from expected?".to_string());
        }

        // Ask about conditions
        if !obs.context.is_empty() {
            questions.push("CONDITIONS: Under what conditions is this true?".to_string());
        }

        questions
    }

    // =========================================================================
    // STEP 2: QUESTION
    // =========================================================================

    /// Register an assumption for questioning.
    pub fn question_assumption(&mut self, belief: String, why_believed: String) -> Assumption {
        let risk = self.assess_risk(&belief);
        let assumption = Assumption::new(belief.clone(), why_believed, risk);

        self.question_queue.push(format!("VERIFY: {}", belief));
        self.assumptions.insert(assumption.id.clone(), assumption.clone());

        assumption
    }

    fn assess_risk(&self, belief: &str) -> String {
        let high_risk = ["always", "never", "all", "none", "must", "critical"];
        let belief_lower = belief.to_lowercase();

        if high_risk.iter().any(|word| belief_lower.contains(word)) {
            "HIGH - Absolute statement, failure could be catastrophic".to_string()
        } else {
            "MEDIUM - Should verify before relying on".to_string()
        }
    }

    pub fn add_evidence(&mut self, assumption_id: &str, evidence: String, supports: bool) {
        if let Some(assumption) = self.assumptions.get_mut(assumption_id) {
            if supports {
                assumption.evidence_for.push(evidence);
            } else {
                assumption.evidence_against.push(evidence);
            }
        }
    }

    // =========================================================================
    // STEP 3: REDUCE (Five Whys)
    // =========================================================================

    /// Apply Five Whys to reduce to fundamental principle.
    pub fn reduce_to_principle(&self, statement: &str, depth: usize) -> Vec<String> {
        let mut chain = vec![statement.to_string()];
        let mut current = statement.to_string();

        for _ in 0..depth {
            if let Some(why) = self.ask_why(&current) {
                if why == current {
                    break;
                }
                chain.push(why.clone());
                current = why;
            } else {
                break;
            }
        }

        chain
    }

    fn ask_why(&self, statement: &str) -> Option<String> {
        // Check if this matches a known bedrock principle
        for principle in self.principles.values() {
            if principle.is_bedrock() && self.relates_to(statement, &principle.statement) {
                return Some(principle.statement.clone());
            }
        }
        None
    }

    fn relates_to(&self, statement: &str, principle: &str) -> bool {
        let stmt_words: std::collections::HashSet<_> = statement
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();
        let prin_words: std::collections::HashSet<_> = principle
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();

        stmt_words.intersection(&prin_words).count() >= 2
    }

    // =========================================================================
    // STEP 4: VERIFY
    // =========================================================================

    /// Verify and register a first principle.
    pub fn verify_principle(
        &mut self,
        statement: String,
        method: String,
        evidence: Vec<String>,
        domain: String,
    ) -> FirstPrinciple {
        let mut principle = FirstPrinciple::new(
            statement.clone(),
            method,
            evidence,
            TruthStatus::Verified,
            domain,
        );

        principle.dependencies = self.find_dependencies(&statement);

        // Remove from assumptions if present
        self.assumptions.retain(|_, a| a.belief != statement);

        self.principles.insert(principle.id.clone(), principle.clone());
        principle
    }

    fn find_dependencies(&self, statement: &str) -> Vec<String> {
        self.principles
            .iter()
            .filter(|(_, p)| self.relates_to(statement, &p.statement))
            .map(|(id, _)| id.clone())
            .collect()
    }

    // =========================================================================
    // STEP 5: BUILD
    // =========================================================================

    pub fn build_from_principles(&self, goal: &str) -> BuildResult {
        let mut result = BuildResult {
            goal: goal.to_string(),
            applicable_principles: Vec::new(),
            unverified_assumptions: Vec::new(),
            safe_to_build: true,
            warnings: Vec::new(),
        };

        // Find applicable principles
        for principle in self.principles.values() {
            if self.relates_to(goal, &principle.statement) {
                result.applicable_principles.push(PrincipleRef {
                    id: principle.id.clone(),
                    statement: principle.statement.clone(),
                    status: format!("{:?}", principle.status),
                });
            }
        }

        // Check for unverified assumptions
        for assumption in self.assumptions.values() {
            if self.relates_to(goal, &assumption.belief) {
                result.unverified_assumptions.push(AssumptionRef {
                    id: assumption.id.clone(),
                    belief: assumption.belief.clone(),
                    risk: assumption.risk_if_wrong.clone(),
                });
                result.safe_to_build = false;
                result.warnings.push(format!("Unverified assumption: {}", assumption.belief));
            }
        }

        result
    }

    // =========================================================================
    // QUERY METHODS
    // =========================================================================

    pub fn get_bedrock_principles(&self) -> Vec<&FirstPrinciple> {
        self.principles.values().filter(|p| p.is_bedrock()).collect()
    }

    pub fn get_unverified_assumptions(&self) -> Vec<&Assumption> {
        self.assumptions.values().collect()
    }

    pub fn get_pending_questions(&self) -> &[String] {
        &self.question_queue
    }

    /// Validate an action against all verified principles.
    pub fn validate_against_principles(&self, action: &str) -> ValidationResult {
        let mut result = ValidationResult {
            action: action.to_string(),
            valid: true,
            conflicts: Vec::new(),
            supports: Vec::new(),
        };

        for principle in self.principles.values() {
            if principle.status != TruthStatus::Verified && principle.status != TruthStatus::Bedrock {
                continue;
            }

            if self.contradicts(action, &principle.statement) {
                result.valid = false;
                result.conflicts.push(ConflictInfo {
                    principle: principle.statement.clone(),
                    reason: "Action contradicts verified principle".to_string(),
                });
            } else if self.relates_to(action, &principle.statement) {
                result.supports.push(SupportInfo {
                    principle: principle.statement.clone(),
                    reason: "Action aligns with verified principle".to_string(),
                });
            }
        }

        result
    }
}

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub action: String,
    pub valid: bool,
    pub conflicts: Vec<ConflictInfo>,
    pub supports: Vec<SupportInfo>,
}

#[derive(Debug, Serialize)]
pub struct ConflictInfo {
    pub principle: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct SupportInfo {
    pub principle: String,
    pub reason: String,
}


/// Generate a deterministic ID from content
fn generate_id(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = FirstPrinciplesEngine::new();
        assert!(!engine.principles.is_empty());
    }

    #[test]
    fn test_bedrock_principles() {
        let engine = FirstPrinciplesEngine::new();
        let bedrock = engine.get_bedrock_principles();
        assert_eq!(bedrock.len(), 5);
    }

    #[test]
    fn test_observation() {
        let mut engine = FirstPrinciplesEngine::new();
        let obs = engine.observe(
            "Test observation".to_string(),
            HashMap::new(),
            "test".to_string(),
        );
        assert!(!obs.id.is_empty());
    }
}
