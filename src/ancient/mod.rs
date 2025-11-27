//! SENA v5.0 - Ancient Wisdom Layers (Rust)
//!
//! These 7 layers form the foundation of SENA v5.0,
//! derived from 3,000 years of proven engineering wisdom.
//!
//! Each layer addresses a fundamental aspect that modern AI systems miss:
//!
//! 1. First Principles - Understand WHY before building (Eratosthenes, 240 BCE)
//! 2. Constraints - Treat limitations as features (Persian Qanats, 3000+ years)
//! 3. Negative Space - Define failure before success (Sushruta, 600 BCE)
//! 4. Relationships - Store connections, not just values (Mayan Calendar)
//! 5. Self-Healing - Embed repair in damage pathways (Roman Concrete)
//! 6. Harmony - Ensure model mirrors reality (Antikythera Mechanism)
//! 7. Millennium Test - Build for 1,000+ years (All Ancient Wisdom)

// Layer 0: First Principles Engine
pub mod first_principles;

// Layer 1: Constraint-as-Feature Engine
pub mod constraint_feature;

// Layer 2: Negative Space Architecture
pub mod negative_space;

// Layer 3: Relationship Data Model
pub mod relationship_model;

// Layer 4: Embedded Self-Healing
pub mod self_healing;

// Layer 5: Harmony Validation Engine
pub mod harmony_validation;

// Layer 6: Millennium Test Framework
pub mod millennium_test;

// Re-export main types from each layer
pub use first_principles::{
    Assumption, FirstPrinciple, FirstPrinciplesEngine, Observation, TruthStatus,
};

pub use constraint_feature::{
    Constraint, ConstraintFeatureEngine, ConstraintType, Feature, FeatureType, Transformation,
};

pub use negative_space::{
    BoundaryType, BoundaryValue, NegativeSpaceArchitecture, NegativeSpaceCheckResult,
    NegativeSpaceDefinition, Prohibition, ProhibitionCategory, ProhibitionLevel, SafeBoundary,
    SafetyReport, ViolationAttempt, ViolationSeverity,
};

pub use relationship_model::{
    ModelStatistics, NodeType, RelationType, Relationship, RelationshipCluster,
    RelationshipDataModel, RelationshipNode, RelationshipPath, RelationshipQuery,
};

pub use self_healing::{
    ComponentHealth, ComponentState, DamageEvent, DamageType, EmbeddedSelfHealing,
    HealingMechanism, HealingResult, HealingStatistics, HealingStatus, HealingStrategy,
    HealingWrapper,
};

pub use harmony_validation::{
    AnchorValidation, AnchorValue, ConfidenceLevel, HarmonyCheck, HarmonyRule, HarmonyStatistics,
    HarmonyStatus, HarmonyType, HarmonyValidationEngine, RealityAnchor, RuleCondition,
    RuleViolation, ValidationResult,
};

pub use millennium_test::{
    ComponentInfo, CriterionCategory, DurabilityAssessment, DurabilityRating, EvolutionPath,
    EvolutionStage, FailureMode, IdentifiedFailureMode, MaintenanceType, MillenniumCriterion,
    MillenniumStatistics, MillenniumTestFramework, MillenniumTestResult, RecoveryPlan,
    RecoveryStep,
};
