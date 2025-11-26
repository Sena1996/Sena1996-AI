//! SENA Controller v10.0 - Full Agent Suite
//!
//! This version features domain-specific agents for complete development support:
//! - 3,000 years of ancient engineering wisdom (7 layers)
//! - Multi-session collaboration hub
//! - Multi-level knowledge system
//! - Extended thinking & specialized sub-agents
//! - Self-improvement evolution engine
//! - Configuration file support (~/.sena/config.toml)
//! - Robust error handling throughout
//!
//! Core Principle: Build systems that embody truth, learn continuously, and improve autonomously.
//!
//! # The 7 Ancient Wisdom Layers
//!
//! - **Layer 0**: First Principles Engine (Eratosthenes, 240 BCE)
//! - **Layer 1**: Constraint-as-Feature Engine (Persian Qanats, 3000+ years)
//! - **Layer 2**: Negative Space Architecture (Sushruta, 600 BCE)
//! - **Layer 3**: Relationship Data Model (Mayan Mathematics)
//! - **Layer 4**: Embedded Self-Healing (Roman Concrete, 2000+ years)
//! - **Layer 5**: Harmony Validation Engine (Antikythera, 150 BCE)
//! - **Layer 6**: Millennium Test Framework (All Ancient Wisdom)
//!
//! # Knowledge System
//!
//! - Multi-level memory (Session, Project, Global, Permanent)
//! - Reasoning frameworks (First Principles, 5 Whys, Systems Thinking)
//! - Security patterns (OWASP, Auth, Crypto)
//! - Performance patterns (Big O, Caching, Async)
//! - Architecture patterns (SOLID, Design Patterns, DDD)
//!
//! # Intelligence Engine
//!
//! - Extended thinking mode (unlimited capacity)
//! - Specialized sub-agents (Security, Performance, Architecture)
//! - Multi-model routing (Fast/Balanced/Powerful)
//! - Autonomous skills (auto-activating capabilities)
//!
//! # Evolution System
//!
//! - Pattern learning (unlimited storage)
//! - Self-optimization for quality, speed, accuracy
//! - Feedback loop for continuous improvement
//! - Knowledge evolution over time
//!
//! # Collaboration Hub
//!
//! - Real-time session communication
//! - Task management across sessions
//! - Conflict detection and resolution
//! - Lightning-fast Unix socket messaging
//!
//! # Usage
//!
//! ```rust,no_run
//! use sena_v10::{SenaUnifiedSystem, ProcessingRequest, KnowledgeSystem, IntelligenceSystem, ThinkingDepth};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut system = SenaUnifiedSystem::new();
//!
//!     // Search knowledge base
//!     let results = system.knowledge().search("sql injection");
//!
//!     // Use extended thinking
//!     let analysis = system.intelligence().analyze("complex problem", ThinkingDepth::Deep);
//!
//!     // Process request
//!     let request = ProcessingRequest::new("Hello, SENA!", "greeting");
//!     let result = system.process(request).await;
//!
//!     println!("Result: {:?}", result);
//! }
//! ```
//!
//! Version: 9.0.1
//! Date: 2025-11-25

pub mod ancient;
pub mod base;
pub mod config;
pub mod sync;
pub mod metrics;
pub mod integration;
pub mod cli;
pub mod mcp;
pub mod hooks;
pub mod output;
pub mod daemon;
pub mod hub;
pub mod knowledge;
pub mod intelligence;
pub mod evolution;
pub mod agents;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

// Re-export ancient wisdom layers
pub use ancient::*;

// Re-export base components
pub use base::{
    BaseComponent, ComponentRegistry, ComponentMetrics, ComponentStatus, ComponentState,
    IVerifier, IStorage, IExecutor, IPermissionManager, ICodebaseMemory, IResearchSystem,
};

// Re-export configuration
pub use config::{SenaConfig, ConfigError};

// Re-export sync
pub use sync::{OfflineSync, CRDT, Change};

// Re-export metrics
pub use metrics::{SenaHealth, SenaMetrics};

// Re-export integration
pub use integration::{AutoIntegration, FormatType};

// Re-export CLI
pub use cli::{Cli, Commands, HookType, execute_command};

// Re-export MCP
pub use mcp::run_server;

// Re-export hooks
pub use hooks::{handle_hook, HookResult};

// Re-export output formatting
pub use output::{
    TableBuilder, ProgressBar, ProgressConfig, LiveProgress,
    MultiProgress, Spinner, FormatBox, render_progress_box, ansi
};

// Re-export collaboration hub
pub use hub::{
    Hub, HubConfig, HubStatus,
    Session, SessionRegistry, SessionRole, SessionStatus,
    HubState, SharedState,
    Task, TaskBoard, TaskPriority, TaskStatus,
    Message, MessageQueue, Broadcast,
    ConflictDetector, FileConflict,
    HubServer, HubClient,
};

// Re-export knowledge system
pub use knowledge::{
    KnowledgeSystem, KnowledgeStats, SearchResult,
    MemoryLevel, MemorySystem, KnowledgeEntry,
    ReasoningFramework, ThinkingMode,
    SecurityPattern, VulnerabilityType, SecurityAudit,
    PerformancePattern, ComplexityClass, OptimizationSuggestion,
    ArchitecturePattern, DesignPattern, SolidPrinciple,
};

// Re-export intelligence system
pub use intelligence::{
    IntelligenceSystem, IntelligenceStatus,
    ThinkingEngine, ThinkingDepth, ThinkingResult,
    Agent, AgentType, AgentPool, AgentResult,
    ModelRouter, ModelType, RoutingDecision,
    Skill, SkillRegistry, SkillExecution,
};

// Re-export evolution system
pub use evolution::{
    EvolutionSystem, EvolutionStatus, EvolutionResult,
    PatternLearner, LearnedPattern, PatternType,
    SelfOptimizer, OptimizationResult, OptimizationTarget,
    FeedbackLoop, FeedbackEntry, FeedbackType,
};

// Re-export domain agents
pub use agents::{
    DomainAgentType, DomainAgentPool, DomainAnalysis, Finding, Severity,
    BackendAgent, IoTAgent, IOSAgent, AndroidAgent, WebAgent,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CODENAME: &str = "Full Agent Suite";

#[derive(Error, Debug)]
pub enum SenaError {
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Safety violation: {0}")]
    SafetyViolation(String),
    #[error("Component error: {0}")]
    ComponentError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Processing phases in the pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessingPhase {
    /// Initial intake and classification
    Intake,
    /// First principles analysis
    Analysis,
    /// Constraint transformation
    Constraint,
    /// Safety and boundary checking
    Safety,
    /// Context and relationship building
    Context,
    /// Response generation
    Generation,
    /// Harmony validation
    Validation,
    /// Final delivery
    Delivery,
}

impl ProcessingPhase {
    pub fn all() -> Vec<Self> {
        vec![
            ProcessingPhase::Intake,
            ProcessingPhase::Analysis,
            ProcessingPhase::Constraint,
            ProcessingPhase::Safety,
            ProcessingPhase::Context,
            ProcessingPhase::Generation,
            ProcessingPhase::Validation,
            ProcessingPhase::Delivery,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ProcessingPhase::Intake => "Intake",
            ProcessingPhase::Analysis => "Analysis",
            ProcessingPhase::Constraint => "Constraint",
            ProcessingPhase::Safety => "Safety",
            ProcessingPhase::Context => "Context",
            ProcessingPhase::Generation => "Generation",
            ProcessingPhase::Validation => "Validation",
            ProcessingPhase::Delivery => "Delivery",
        }
    }
}

/// Health status of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemHealth {
    Excellent,
    Good,
    Degraded,
    Critical,
    Unknown,
}

impl SystemHealth {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            SystemHealth::Excellent
        } else if score >= 0.7 {
            SystemHealth::Good
        } else if score >= 0.5 {
            SystemHealth::Degraded
        } else if score > 0.0 {
            SystemHealth::Critical
        } else {
            SystemHealth::Unknown
        }
    }
}

/// A processing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRequest {
    pub id: String,
    pub content: String,
    pub request_type: String,
    pub context: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub priority: u32,
}

impl ProcessingRequest {
    pub fn new(content: impl Into<String>, request_type: impl Into<String>) -> Self {
        let content = content.into();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        let id = format!("req_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            content,
            request_type: request_type.into(),
            context: HashMap::new(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            priority: 100,
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// Result of processing a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub id: String,
    pub request_id: String,
    pub success: bool,
    pub content: String,
    pub phase_results: HashMap<String, PhaseResult>,
    pub validation_score: f64,
    pub safety_score: f64,
    pub harmony_score: f64,
    pub processing_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ProcessingResult {
    pub fn new(request_id: impl Into<String>) -> Self {
        let request_id = request_id.into();
        let mut hasher = Sha256::new();
        hasher.update(request_id.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        let id = format!("res_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            request_id,
            success: false,
            content: String::new(),
            phase_results: HashMap::new(),
            validation_score: 0.0,
            safety_score: 0.0,
            harmony_score: 0.0,
            processing_time_ms: 0,
            timestamp: Utc::now(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn overall_score(&self) -> f64 {
        (self.validation_score + self.safety_score + self.harmony_score) / 3.0
    }
}

/// Result of a single processing phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub phase: String,
    pub success: bool,
    pub duration_ms: u64,
    pub output: HashMap<String, String>,
    pub score: f64,
}

/// The unified SENA system integrating all capabilities
pub struct SenaUnifiedSystem {
    // Layer 0: First Principles
    first_principles: FirstPrinciplesEngine,
    // Layer 1: Constraint-Feature
    constraint_feature: ConstraintFeatureEngine,
    // Layer 2: Negative Space
    negative_space: NegativeSpaceArchitecture,
    // Layer 3: Relationship Model
    relationship_model: RelationshipDataModel,
    // Layer 4: Self-Healing
    self_healing: EmbeddedSelfHealing,
    // Layer 5: Harmony Validation
    harmony_validation: HarmonyValidationEngine,
    // Layer 6: Millennium Test
    millennium_test: MillenniumTestFramework,

    // NEW: Knowledge System
    knowledge_system: KnowledgeSystem,
    // NEW: Intelligence System
    intelligence_system: IntelligenceSystem,
    // NEW: Evolution System
    evolution_system: EvolutionSystem,

    // Processing state
    request_count: u64,
    successful_count: u64,
    failed_count: u64,
    created_at: DateTime<Utc>,
}

impl Default for SenaUnifiedSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SenaUnifiedSystem {
    /// Create a new unified SENA system
    pub fn new() -> Self {
        Self {
            first_principles: FirstPrinciplesEngine::new(),
            constraint_feature: ConstraintFeatureEngine::new(),
            negative_space: NegativeSpaceArchitecture::new(),
            relationship_model: RelationshipDataModel::new(),
            self_healing: EmbeddedSelfHealing::new(),
            harmony_validation: HarmonyValidationEngine::new(),
            millennium_test: MillenniumTestFramework::new(),
            knowledge_system: KnowledgeSystem::new(),
            intelligence_system: IntelligenceSystem::new(),
            evolution_system: EvolutionSystem::new(),
            request_count: 0,
            successful_count: 0,
            failed_count: 0,
            created_at: Utc::now(),
        }
    }

    /// Process a request through all layers
    pub async fn process(&mut self, request: ProcessingRequest) -> ProcessingResult {
        let start_time = std::time::Instant::now();
        let mut result = ProcessingResult::new(&request.id);

        self.request_count += 1;

        // Phase 1: Intake
        let intake_result = self.phase_intake(&request);
        result.phase_results.insert("intake".to_string(), intake_result);

        // Phase 2: Analysis (First Principles)
        let analysis_result = self.phase_analysis(&request);
        result.phase_results.insert("analysis".to_string(), analysis_result);

        // Phase 3: Constraint (Transform constraints to features)
        let constraint_result = self.phase_constraint(&request);
        result.phase_results.insert("constraint".to_string(), constraint_result);

        // Phase 4: Safety (Negative Space check)
        let safety_result = self.phase_safety(&request);
        result.safety_score = safety_result.score;
        if !safety_result.success {
            result.errors.push("Safety check failed".to_string());
            result.processing_time_ms = start_time.elapsed().as_millis() as u64;
            self.failed_count += 1;
            return result;
        }
        result.phase_results.insert("safety".to_string(), safety_result);

        // Phase 5: Context (Relationship building)
        let context_result = self.phase_context(&request);
        result.phase_results.insert("context".to_string(), context_result);

        // Phase 6: Generation
        let generation_result = self.phase_generation(&request);
        result.content = generation_result
            .output
            .get("response")
            .cloned()
            .unwrap_or_default();
        result.phase_results.insert("generation".to_string(), generation_result);

        // Phase 7: Validation (Harmony check)
        let validation_result = self.phase_validation(&result.content);
        result.harmony_score = validation_result.score;
        result.validation_score = validation_result.score;
        result.phase_results.insert("validation".to_string(), validation_result);

        // Phase 8: Delivery
        let delivery_result = self.phase_delivery(&mut result);
        result.phase_results.insert("delivery".to_string(), delivery_result);

        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        result.success = true;
        self.successful_count += 1;

        result
    }

    fn phase_intake(&self, request: &ProcessingRequest) -> PhaseResult {
        let start = std::time::Instant::now();
        let mut output = HashMap::new();

        output.insert("request_type".to_string(), request.request_type.clone());
        output.insert("content_length".to_string(), request.content.len().to_string());
        output.insert("priority".to_string(), request.priority.to_string());

        PhaseResult {
            phase: "intake".to_string(),
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score: 1.0,
        }
    }

    fn phase_analysis(&mut self, request: &ProcessingRequest) -> PhaseResult {
        let start = std::time::Instant::now();
        let mut output = HashMap::new();

        // Use first principles engine
        let context: HashMap<String, serde_json::Value> = request.context.iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        let observation = self.first_principles.observe(
            request.content.clone(),
            context,
            "user_input".to_string(),
        );

        output.insert("observation_id".to_string(), observation.id.clone());
        output.insert("questions_raised".to_string(), observation.questions_raised.len().to_string());

        PhaseResult {
            phase: "analysis".to_string(),
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score: 0.9,
        }
    }

    fn phase_constraint(&mut self, request: &ProcessingRequest) -> PhaseResult {
        let start = std::time::Instant::now();
        let mut output = HashMap::new();

        // Identify and register a constraint from the request
        let _constraint_id = self.constraint_feature.identify_constraint(
            format!("Request constraint: {}", &request.request_type),
            ConstraintType::Functional,
            format!("Processing {} request", request.request_type),
            serde_json::json!(100),  // limit_value
            "request".to_string(),   // unit
            false,                   // is_hard
        );
        output.insert("constraints_found".to_string(), "1".to_string());

        // Get all features generated
        let features_count = self.constraint_feature.get_all_features().len();
        output.insert("features_generated".to_string(), features_count.to_string());

        PhaseResult {
            phase: "constraint".to_string(),
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score: 0.85,
        }
    }

    fn phase_safety(&mut self, request: &ProcessingRequest) -> PhaseResult {
        let start = std::time::Instant::now();
        let mut output = HashMap::new();

        // Check against negative space
        let check_result = self.negative_space.check_action(&request.content, &request.context);

        output.insert("violations_found".to_string(), check_result.violations.len().to_string());
        output.insert("allowed".to_string(), check_result.allowed.to_string());
        output.insert("risk_score".to_string(), format!("{:.2}", check_result.risk_score));

        let score = if check_result.allowed { 1.0 - check_result.risk_score } else { 0.0 };

        PhaseResult {
            phase: "safety".to_string(),
            success: check_result.allowed,
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score,
        }
    }

    fn phase_context(&mut self, request: &ProcessingRequest) -> PhaseResult {
        let start = std::time::Instant::now();
        let mut output = HashMap::new();

        // Build context in relationship model
        let node_id = self.relationship_model.create_node(
            format!("request_{}", &request.id[..8]),
            NodeType::Event,
        );

        output.insert("context_node".to_string(), node_id);
        output.insert("total_nodes".to_string(), self.relationship_model.get_all_nodes().len().to_string());

        PhaseResult {
            phase: "context".to_string(),
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score: 0.9,
        }
    }

    fn phase_generation(&self, request: &ProcessingRequest) -> PhaseResult {
        let start = std::time::Instant::now();
        let mut output = HashMap::new();

        // Generate response (simplified - in real implementation, this would involve LLM)
        let response = format!(
            "Processed request '{}' of type '{}' through SENA v{} Truth-Embedded Architecture.",
            &request.content,
            request.request_type,
            VERSION
        );

        output.insert("response".to_string(), response);
        output.insert("generation_method".to_string(), "direct".to_string());

        PhaseResult {
            phase: "generation".to_string(),
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score: 0.95,
        }
    }

    fn phase_validation(&mut self, content: &str) -> PhaseResult {
        let start = std::time::Instant::now();
        let mut output = HashMap::new();

        // Validate harmony
        let validation = self.harmony_validation.validate(content);

        output.insert("harmony_status".to_string(), format!("{:?}", validation.overall_status));
        output.insert("confidence".to_string(), format!("{:.2}", validation.overall_confidence));
        output.insert("violations".to_string(), validation.rule_violations.len().to_string());

        PhaseResult {
            phase: "validation".to_string(),
            success: validation.is_valid(),
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score: validation.overall_confidence,
        }
    }

    fn phase_delivery(&self, result: &mut ProcessingResult) -> PhaseResult {
        let start = std::time::Instant::now();
        let output = HashMap::new();

        // Calculate final scores
        let phase_scores: Vec<f64> = result.phase_results.values().map(|p| p.score).collect();
        let avg_score = if phase_scores.is_empty() {
            0.0
        } else {
            phase_scores.iter().sum::<f64>() / phase_scores.len() as f64
        };

        result.validation_score = avg_score;

        PhaseResult {
            phase: "delivery".to_string(),
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            output,
            score: 1.0,
        }
    }

    /// Get system health status
    pub fn get_health(&self) -> SystemHealth {
        let healing_health = self.self_healing.get_system_health();
        SystemHealth::from_score(healing_health)
    }

    /// Get system report
    pub fn get_system_report(&self) -> SystemReport {
        let healing_stats = self.self_healing.get_statistics();
        let harmony_stats = self.harmony_validation.get_statistics();
        let millennium_stats = self.millennium_test.get_statistics();

        let success_rate = if self.request_count > 0 {
            self.successful_count as f64 / self.request_count as f64
        } else {
            1.0
        };

        SystemReport {
            version: VERSION.to_string(),
            codename: CODENAME.to_string(),
            health: self.get_health(),
            uptime_seconds: (Utc::now() - self.created_at).num_seconds() as u64,
            request_count: self.request_count,
            successful_count: self.successful_count,
            failed_count: self.failed_count,
            success_rate,
            healing_stats,
            harmony_stats,
            millennium_stats,
        }
    }

    // Accessors for individual layers

    /// Get first principles engine
    pub fn first_principles(&mut self) -> &mut FirstPrinciplesEngine {
        &mut self.first_principles
    }

    /// Get constraint feature engine
    pub fn constraint_feature(&mut self) -> &mut ConstraintFeatureEngine {
        &mut self.constraint_feature
    }

    /// Get negative space architecture
    pub fn negative_space(&mut self) -> &mut NegativeSpaceArchitecture {
        &mut self.negative_space
    }

    /// Get relationship data model
    pub fn relationship_model(&mut self) -> &mut RelationshipDataModel {
        &mut self.relationship_model
    }

    /// Get self healing engine
    pub fn self_healing(&mut self) -> &mut EmbeddedSelfHealing {
        &mut self.self_healing
    }

    /// Get harmony validation engine
    pub fn harmony_validation(&mut self) -> &mut HarmonyValidationEngine {
        &mut self.harmony_validation
    }

    /// Get millennium test framework
    pub fn millennium_test(&mut self) -> &mut MillenniumTestFramework {
        &mut self.millennium_test
    }

    /// Get knowledge system
    pub fn knowledge(&mut self) -> &mut KnowledgeSystem {
        &mut self.knowledge_system
    }

    /// Get intelligence system
    pub fn intelligence(&mut self) -> &mut IntelligenceSystem {
        &mut self.intelligence_system
    }

    /// Get evolution system
    pub fn evolution(&mut self) -> &mut EvolutionSystem {
        &mut self.evolution_system
    }

    /// Search knowledge base
    pub fn search_knowledge(&self, query: &str) -> Vec<SearchResult> {
        self.knowledge_system.search(query)
    }

    /// Analyze with extended thinking
    pub fn analyze(&self, problem: &str, depth: ThinkingDepth) -> ThinkingResult {
        self.intelligence_system.analyze(problem, depth)
    }

    /// Dispatch to specialized agent
    pub fn dispatch_agent(&self, task: &str, agent_type: AgentType) -> AgentResult {
        self.intelligence_system.dispatch(task, agent_type)
    }

    /// Learn from interaction
    pub fn learn(&mut self, context: &str, outcome: &str, success: bool) {
        self.evolution_system.learn(context, outcome, success);
    }

    /// Run evolution cycle
    pub fn evolve(&mut self) -> EvolutionResult {
        self.evolution_system.evolve()
    }
}

/// System status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReport {
    pub version: String,
    pub codename: String,
    pub health: SystemHealth,
    pub uptime_seconds: u64,
    pub request_count: u64,
    pub successful_count: u64,
    pub failed_count: u64,
    pub success_rate: f64,
    pub healing_stats: HealingStatistics,
    pub harmony_stats: HarmonyStatistics,
    pub millennium_stats: MillenniumStatistics,
}

/// Create a default pipeline for quick usage
pub fn create_system() -> SenaUnifiedSystem {
    SenaUnifiedSystem::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_system_creation() {
        let system = SenaUnifiedSystem::new();
        assert_eq!(system.get_health(), SystemHealth::Excellent);
    }

    #[tokio::test]
    async fn test_process_request() {
        let mut system = SenaUnifiedSystem::new();

        let request = ProcessingRequest::new("Hello, SENA!", "greeting")
            .with_context("user", "test")
            .with_priority(100);

        let result = system.process(request).await;

        assert!(result.success);
        assert!(!result.content.is_empty());
        assert!(result.processing_time_ms > 0);
    }

    #[tokio::test]
    async fn test_system_report() {
        let mut system = SenaUnifiedSystem::new();

        let request = ProcessingRequest::new("Test request", "test");
        let _ = system.process(request).await;

        let report = system.get_system_report();

        assert_eq!(report.version, VERSION);
        assert_eq!(report.request_count, 1);
        assert_eq!(report.successful_count, 1);
    }

    #[test]
    fn test_processing_phases() {
        let phases = ProcessingPhase::all();
        assert_eq!(phases.len(), 8);
    }
}
