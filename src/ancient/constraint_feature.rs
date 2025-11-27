//! SENA v5.0 - Layer 1: Constraint-as-Feature Engine (Rust)
//!
//! Ancient Source: Persian Qanat Engineers (3000+ years)
//! - Used gravity (constraint) as the POWER SOURCE for water delivery
//! - Used geology (constraint) as the FILTER for water purity
//! - Used distance (constraint) as the COOLING MECHANISM
//!
//! Core Principle:
//!     Every constraint is a potential feature in disguise.
//!     What you cannot do defines what you must do differently.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Types of constraints that can become features
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintType {
    Resource,   // Memory, CPU, storage
    Time,       // Latency, timeout, deadline
    Capacity,   // Token limits, batch size
    Structural, // Architecture, dependency
    External,   // API limits, rate limits
    Physical,   // Network, hardware
    Functional, // Functionality constraints
}

/// Types of features that can emerge from constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureType {
    Compression,    // Data compression
    Prioritization, // Smart ordering
    Caching,        // Strategic storage
    Batching,       // Efficient grouping
    Streaming,      // Incremental processing
    Fallback,       // Graceful alternatives
    Optimization,   // Performance enhancement
}

/// A constraint that limits the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub name: String,
    pub constraint_type: ConstraintType,
    pub description: String,
    pub limit_value: serde_json::Value,
    pub unit: String,
    pub is_hard: bool,
    pub context: HashMap<String, serde_json::Value>,
    pub discovered: DateTime<Utc>,
}

impl Constraint {
    pub fn new(
        name: String,
        constraint_type: ConstraintType,
        description: String,
        limit_value: serde_json::Value,
        unit: String,
        is_hard: bool,
    ) -> Self {
        let id = generate_id(&format!("{}{:?}", name, constraint_type));
        Self {
            id,
            name,
            constraint_type,
            description,
            limit_value,
            unit,
            is_hard,
            context: HashMap::new(),
            discovered: Utc::now(),
        }
    }
}

/// A feature that emerges FROM a constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub feature_type: FeatureType,
    pub description: String,
    pub source_constraint_id: String,
    pub implementation: String,
    pub benefit: String,
    pub created: DateTime<Utc>,
}

/// The transformation of a constraint into a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub constraint_id: String,
    pub feature_id: String,
    pub insight: String,
    pub method: String,
    pub confidence: f64,
    pub verified: bool,
    pub created: DateTime<Utc>,
}

/// Transformation pattern from ancient wisdom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationPattern {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub feature_type: FeatureType,
    pub insight: String,
    pub method: String,
}

/// Engine for transforming constraints into features.
pub struct ConstraintFeatureEngine {
    constraints: HashMap<String, Constraint>,
    features: HashMap<String, Feature>,
    transformations: HashMap<String, Transformation>,
    transformation_patterns: Vec<TransformationPattern>,
}

impl Default for ConstraintFeatureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintFeatureEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            constraints: HashMap::new(),
            features: HashMap::new(),
            transformations: HashMap::new(),
            transformation_patterns: Vec::new(),
        };

        engine.init_transformation_patterns();
        engine.init_sena_constraints();
        engine
    }

    fn init_transformation_patterns(&mut self) {
        self.transformation_patterns = vec![
            TransformationPattern {
                name: "gravity_to_energy".to_string(),
                constraint_type: ConstraintType::Physical,
                feature_type: FeatureType::Optimization,
                insight: "Gravity prevents uphill, but provides free downhill".to_string(),
                method: "Design flow to work WITH gravity, not against".to_string(),
            },
            TransformationPattern {
                name: "capacity_to_compression".to_string(),
                constraint_type: ConstraintType::Capacity,
                feature_type: FeatureType::Compression,
                insight: "Limited space forces efficient encoding".to_string(),
                method: "Compress semantically, not just syntactically".to_string(),
            },
            TransformationPattern {
                name: "time_to_streaming".to_string(),
                constraint_type: ConstraintType::Time,
                feature_type: FeatureType::Streaming,
                insight: "Can't wait for everything? Send incrementally".to_string(),
                method: "Stream results as computed".to_string(),
            },
            TransformationPattern {
                name: "memory_to_caching".to_string(),
                constraint_type: ConstraintType::Resource,
                feature_type: FeatureType::Caching,
                insight: "Can't store everything? Store what matters".to_string(),
                method: "Intelligent prioritized caching".to_string(),
            },
            TransformationPattern {
                name: "rate_to_batching".to_string(),
                constraint_type: ConstraintType::External,
                feature_type: FeatureType::Batching,
                insight: "Limited calls? Make each call count more".to_string(),
                method: "Batch requests for efficiency".to_string(),
            },
        ];
    }

    fn init_sena_constraints(&mut self) {
        let constraints = vec![
            Constraint::new(
                "Context Window Limit".to_string(),
                ConstraintType::Capacity,
                "Maximum tokens in conversation context".to_string(),
                serde_json::json!(200000),
                "tokens".to_string(),
                true,
            ),
            Constraint::new(
                "Response Latency Target".to_string(),
                ConstraintType::Time,
                "Target time for first response token".to_string(),
                serde_json::json!(500),
                "milliseconds".to_string(),
                false,
            ),
            Constraint::new(
                "Memory Budget".to_string(),
                ConstraintType::Resource,
                "Available working memory for session".to_string(),
                serde_json::json!(8),
                "GB".to_string(),
                true,
            ),
            Constraint::new(
                "API Rate Limit".to_string(),
                ConstraintType::External,
                "Maximum API calls per minute".to_string(),
                serde_json::json!(60),
                "calls/minute".to_string(),
                true,
            ),
        ];

        for constraint in constraints {
            self.constraints.insert(constraint.id.clone(), constraint);
        }
    }

    /// Identify and register a constraint.
    pub fn identify_constraint(
        &mut self,
        name: String,
        constraint_type: ConstraintType,
        description: String,
        limit_value: serde_json::Value,
        unit: String,
        is_hard: bool,
    ) -> Constraint {
        let constraint = Constraint::new(
            name,
            constraint_type,
            description,
            limit_value,
            unit,
            is_hard,
        );
        self.constraints
            .insert(constraint.id.clone(), constraint.clone());
        constraint
    }

    /// Analyze WHY a constraint exists and what it implies.
    pub fn analyze_constraint(&self, constraint_id: &str) -> Option<ConstraintAnalysis> {
        let constraint = self.constraints.get(constraint_id)?;

        Some(ConstraintAnalysis {
            constraint: constraint.clone(),
            why_exists: self.analyze_why(&constraint.constraint_type),
            what_prevents: self.analyze_prevents(&constraint.constraint_type),
            what_enables: self.analyze_enables(&constraint.constraint_type),
            transformation_potential: self.assess_transformation_potential(constraint),
        })
    }

    fn analyze_why(&self, ctype: &ConstraintType) -> String {
        match ctype {
            ConstraintType::Resource => "Physical/computational limits of the system".to_string(),
            ConstraintType::Time => {
                "User experience and system responsiveness requirements".to_string()
            }
            ConstraintType::Capacity => {
                "Architectural design decisions and memory management".to_string()
            }
            ConstraintType::Structural => {
                "System architecture and dependency relationships".to_string()
            }
            ConstraintType::External => "Third-party limitations and API contracts".to_string(),
            ConstraintType::Physical => "Laws of physics and hardware limitations".to_string(),
            ConstraintType::Functional => {
                "Functional requirements and feature specifications".to_string()
            }
        }
    }

    fn analyze_prevents(&self, ctype: &ConstraintType) -> Vec<String> {
        match ctype {
            ConstraintType::Resource => vec![
                "Unlimited computation".to_string(),
                "Unbounded memory usage".to_string(),
            ],
            ConstraintType::Time => vec![
                "Slow responses".to_string(),
                "Long processing times".to_string(),
            ],
            ConstraintType::Capacity => vec![
                "Large data storage".to_string(),
                "Unlimited context".to_string(),
            ],
            ConstraintType::Structural => vec![
                "Arbitrary connections".to_string(),
                "Circular dependencies".to_string(),
            ],
            ConstraintType::External => vec![
                "Unlimited API calls".to_string(),
                "Free resource usage".to_string(),
            ],
            ConstraintType::Physical => vec![
                "Instant communication".to_string(),
                "Infinite storage".to_string(),
            ],
            ConstraintType::Functional => vec![
                "Unlimited features".to_string(),
                "Unconstrained functionality".to_string(),
            ],
        }
    }

    fn analyze_enables(&self, ctype: &ConstraintType) -> Vec<String> {
        match ctype {
            ConstraintType::Resource => vec![
                "Forces efficient algorithms".to_string(),
                "Encourages smart resource allocation".to_string(),
                "Enables predictable performance".to_string(),
            ],
            ConstraintType::Time => vec![
                "Forces responsive design".to_string(),
                "Enables streaming responses".to_string(),
                "Encourages incremental processing".to_string(),
            ],
            ConstraintType::Capacity => vec![
                "Forces semantic compression".to_string(),
                "Enables focused context".to_string(),
                "Encourages relevance filtering".to_string(),
            ],
            ConstraintType::Structural => vec![
                "Forces clean architecture".to_string(),
                "Enables modular design".to_string(),
            ],
            ConstraintType::External => vec![
                "Forces caching strategies".to_string(),
                "Enables batch optimization".to_string(),
            ],
            ConstraintType::Physical => vec![
                "Forces distributed design".to_string(),
                "Enables edge computing".to_string(),
            ],
            ConstraintType::Functional => vec![
                "Forces focused functionality".to_string(),
                "Enables clear requirements".to_string(),
            ],
        }
    }

    fn assess_transformation_potential(&self, constraint: &Constraint) -> f64 {
        let mut potential: f64 = if constraint.is_hard { 0.7 } else { 0.5 };

        // Check for known patterns
        let has_pattern = self
            .transformation_patterns
            .iter()
            .any(|p| p.constraint_type == constraint.constraint_type);

        if has_pattern {
            potential += 0.2;
        }

        potential.min(1.0)
    }

    /// Invert the constraint - ask what it ENABLES, not what it PREVENTS.
    pub fn invert_constraint(&self, constraint_id: &str) -> Option<ConstraintInversion> {
        let constraint = self.constraints.get(constraint_id)?;

        let matching_patterns: Vec<_> = self
            .transformation_patterns
            .iter()
            .filter(|p| p.constraint_type == constraint.constraint_type)
            .cloned()
            .collect();

        Some(ConstraintInversion {
            constraint: constraint.clone(),
            original_view: format!(
                "This {:?} constraint limits us to {} {}",
                constraint.constraint_type, constraint.limit_value, constraint.unit
            ),
            inverted_view: self.generate_inverted_view(constraint),
            potential_features: self.generate_potential_features(&constraint.constraint_type),
            matching_patterns,
        })
    }

    fn generate_inverted_view(&self, constraint: &Constraint) -> String {
        match constraint.constraint_type {
            ConstraintType::Capacity => format!(
                "This {} {} limit FORCES us to prioritize what matters most",
                constraint.limit_value, constraint.unit
            ),
            ConstraintType::Time => format!(
                "This {} {} limit ENABLES responsive, streaming design",
                constraint.limit_value, constraint.unit
            ),
            ConstraintType::Resource => format!(
                "This {} {} limit DRIVES efficient, optimized code",
                constraint.limit_value, constraint.unit
            ),
            ConstraintType::External => format!(
                "This {} {} limit ENCOURAGES smart batching and caching",
                constraint.limit_value, constraint.unit
            ),
            _ => "This constraint can be transformed into an opportunity".to_string(),
        }
    }

    fn generate_potential_features(&self, ctype: &ConstraintType) -> Vec<PotentialFeature> {
        match ctype {
            ConstraintType::Capacity => vec![
                PotentialFeature {
                    feature_type: FeatureType::Compression,
                    name: "Semantic Compression".to_string(),
                    benefit: "Denser, more meaningful information".to_string(),
                },
                PotentialFeature {
                    feature_type: FeatureType::Prioritization,
                    name: "Relevance Ranking".to_string(),
                    benefit: "Most important information first".to_string(),
                },
            ],
            ConstraintType::Time => vec![
                PotentialFeature {
                    feature_type: FeatureType::Streaming,
                    name: "Incremental Response".to_string(),
                    benefit: "Immediate partial results".to_string(),
                },
                PotentialFeature {
                    feature_type: FeatureType::Caching,
                    name: "Predictive Caching".to_string(),
                    benefit: "Pre-computed common responses".to_string(),
                },
            ],
            ConstraintType::Resource => vec![PotentialFeature {
                feature_type: FeatureType::Optimization,
                name: "Efficient Algorithms".to_string(),
                benefit: "Better performance with less".to_string(),
            }],
            ConstraintType::External => vec![
                PotentialFeature {
                    feature_type: FeatureType::Batching,
                    name: "Request Batching".to_string(),
                    benefit: "Fewer calls, more data per call".to_string(),
                },
                PotentialFeature {
                    feature_type: FeatureType::Fallback,
                    name: "Graceful Degradation".to_string(),
                    benefit: "Works even when external services fail".to_string(),
                },
            ],
            _ => Vec::new(),
        }
    }

    /// Transform a constraint into a feature.
    pub fn transform_to_feature(
        &mut self,
        constraint_id: &str,
        feature_name: String,
        feature_type: FeatureType,
        implementation: String,
        benefit: String,
        insight: String,
    ) -> Result<Transformation, String> {
        if !self.constraints.contains_key(constraint_id) {
            return Err(format!("Constraint {} not found", constraint_id));
        }

        let feature_id = generate_id(&format!("{}{}", feature_name, constraint_id));

        let constraint_name = self
            .constraints
            .get(constraint_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let feature = Feature {
            id: feature_id.clone(),
            name: feature_name,
            feature_type,
            description: format!("Feature derived from constraint: {}", constraint_name),
            source_constraint_id: constraint_id.to_string(),
            implementation: implementation.clone(),
            benefit,
            created: Utc::now(),
        };

        self.features.insert(feature.id.clone(), feature);

        let transformation = Transformation {
            constraint_id: constraint_id.to_string(),
            feature_id,
            insight,
            method: implementation,
            confidence: 0.8,
            verified: false,
            created: Utc::now(),
        };

        let trans_id = format!("{}_{}", constraint_id, transformation.feature_id);
        self.transformations
            .insert(trans_id.clone(), transformation.clone());

        Ok(transformation)
    }

    /// Apply constraint thinking to a problem.
    pub fn apply_constraint_thinking(&self, problem: &str) -> ConstraintThinkingResult {
        let mut result = ConstraintThinkingResult {
            problem: problem.to_string(),
            relevant_constraints: Vec::new(),
            suggested_features: Vec::new(),
            transformation_strategy: String::new(),
        };

        let problem_lower = problem.to_lowercase();

        for constraint in self.constraints.values() {
            let relevance = self.calculate_relevance(constraint, &problem_lower);
            if relevance > 0.3 {
                if let Some(inversion) = self.invert_constraint(&constraint.id) {
                    result.relevant_constraints.push(RelevantConstraint {
                        constraint: constraint.clone(),
                        relevance,
                        inversion,
                    });
                    result
                        .suggested_features
                        .extend(self.generate_potential_features(&constraint.constraint_type));
                }
            }
        }

        if !result.relevant_constraints.is_empty() {
            result.transformation_strategy = result
                .relevant_constraints
                .iter()
                .map(|rc| {
                    format!(
                        "Transform '{}': {}",
                        rc.constraint.name, rc.inversion.inverted_view
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
        }

        result
    }

    fn calculate_relevance(&self, constraint: &Constraint, problem: &str) -> f64 {
        let keywords: Vec<&str> = match constraint.constraint_type {
            ConstraintType::Capacity => vec!["token", "limit", "size", "capacity", "context"],
            ConstraintType::Time => vec!["time", "fast", "slow", "latency", "response"],
            ConstraintType::Resource => vec!["memory", "cpu", "resource", "efficient"],
            ConstraintType::External => vec!["api", "rate", "external", "service"],
            ConstraintType::Physical => vec!["network", "hardware", "physical"],
            ConstraintType::Structural => vec!["architecture", "structure", "design"],
            ConstraintType::Functional => vec!["function", "feature", "capability", "process"],
        };

        let matches = keywords.iter().filter(|kw| problem.contains(*kw)).count();
        (matches as f64 * 0.2).min(1.0)
    }

    pub fn get_all_constraints(&self) -> Vec<&Constraint> {
        self.constraints.values().collect()
    }

    pub fn get_all_features(&self) -> Vec<&Feature> {
        self.features.values().collect()
    }

    pub fn get_all_transformations(&self) -> Vec<TransformationSummary> {
        self.transformations
            .values()
            .filter_map(|t| {
                let constraint = self.constraints.get(&t.constraint_id)?;
                let feature = self.features.get(&t.feature_id)?;
                Some(TransformationSummary {
                    constraint_name: constraint.name.clone(),
                    feature_name: feature.name.clone(),
                    insight: t.insight.clone(),
                    verified: t.verified,
                    confidence: t.confidence,
                })
            })
            .collect()
    }
}

// Result types
#[derive(Debug, Clone, Serialize)]
pub struct ConstraintAnalysis {
    pub constraint: Constraint,
    pub why_exists: String,
    pub what_prevents: Vec<String>,
    pub what_enables: Vec<String>,
    pub transformation_potential: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstraintInversion {
    pub constraint: Constraint,
    pub original_view: String,
    pub inverted_view: String,
    pub potential_features: Vec<PotentialFeature>,
    pub matching_patterns: Vec<TransformationPattern>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PotentialFeature {
    pub feature_type: FeatureType,
    pub name: String,
    pub benefit: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstraintThinkingResult {
    pub problem: String,
    pub relevant_constraints: Vec<RelevantConstraint>,
    pub suggested_features: Vec<PotentialFeature>,
    pub transformation_strategy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelevantConstraint {
    pub constraint: Constraint,
    pub relevance: f64,
    pub inversion: ConstraintInversion,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransformationSummary {
    pub constraint_name: String,
    pub feature_name: String,
    pub insight: String,
    pub verified: bool,
    pub confidence: f64,
}

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
        let engine = ConstraintFeatureEngine::new();
        assert!(!engine.constraints.is_empty());
    }

    #[test]
    fn test_constraint_analysis() {
        let engine = ConstraintFeatureEngine::new();
        let constraints: Vec<_> = engine.get_all_constraints();

        if let Some(constraint) = constraints.first() {
            let analysis = engine.analyze_constraint(&constraint.id);
            assert!(analysis.is_some());
        }
    }
}
