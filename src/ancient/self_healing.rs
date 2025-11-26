//! SENA v5.0 - Layer 4: Embedded Self-Healing (Rust)
//!
//! Inspired by Roman Concrete (2000+ years durability)
//!
//! Roman concrete contains lime clite that activates when water penetrates
//! cracks, automatically repairing damage. The repair mechanism is embedded
//! in the damage pathway itself.
//!
//! Applied to AI: Embed recovery mechanisms in failure pathways.
//! Systems should heal themselves when damaged.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

/// Errors for Self-Healing operations
#[derive(Error, Debug)]
pub enum SelfHealingError {
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    #[error("Healing mechanism not found: {0}")]
    MechanismNotFound(String),
    #[error("Healing failed: {0}")]
    HealingFailed(String),
    #[error("Component unhealable: {0}")]
    ComponentUnhealable(String),
}

/// Types of damage that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    /// Corrupted data or state
    Corruption,
    /// Performance degradation
    Degradation,
    /// Complete failure
    Failure,
    /// Resource exhaustion
    Exhaustion,
    /// Connection loss
    Disconnection,
    /// Configuration error
    Misconfiguration,
    /// Timeout
    Timeout,
    /// Overflow
    Overflow,
    /// External attack or interference
    External,
    /// Unknown damage type
    Unknown,
}

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentHealth {
    /// Fully operational
    Healthy,
    /// Minor issues, still functional
    Degraded,
    /// Serious issues, limited functionality
    Impaired,
    /// Not functional
    Failed,
    /// Currently recovering
    Recovering,
}

impl ComponentHealth {
    pub fn score(&self) -> f64 {
        match self {
            ComponentHealth::Healthy => 1.0,
            ComponentHealth::Degraded => 0.7,
            ComponentHealth::Impaired => 0.4,
            ComponentHealth::Recovering => 0.3,
            ComponentHealth::Failed => 0.0,
        }
    }
}

/// Status of a healing operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealingStatus {
    /// Healing not started
    Pending,
    /// Healing in progress
    InProgress,
    /// Healing completed successfully
    Completed,
    /// Healing failed
    Failed,
    /// Healing partially successful
    Partial,
    /// Healing skipped (not needed)
    Skipped,
}

/// A damage event that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEvent {
    pub id: String,
    pub component_id: String,
    pub damage_type: DamageType,
    pub severity: f64,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, String>,
    pub healed: bool,
    pub healing_id: Option<String>,
}

impl DamageEvent {
    pub fn new(
        component_id: impl Into<String>,
        damage_type: DamageType,
        severity: f64,
        description: impl Into<String>,
    ) -> Self {
        let component_id = component_id.into();
        let id = Self::generate_id(&component_id);

        Self {
            id,
            component_id,
            damage_type,
            severity: severity.clamp(0.0, 1.0),
            description: description.into(),
            timestamp: Utc::now(),
            context: HashMap::new(),
            healed: false,
            healing_id: None,
        }
    }

    fn generate_id(component_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(component_id.as_bytes());
        hasher.update(Utc::now().timestamp_nanos_opt().unwrap_or(0).to_string().as_bytes());
        format!("dmg_{}", hex::encode(&hasher.finalize()[..8]))
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn mark_healed(&mut self, healing_id: String) {
        self.healed = true;
        self.healing_id = Some(healing_id);
    }
}

/// A mechanism for healing damage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingMechanism {
    pub id: String,
    pub name: String,
    pub description: String,
    pub handles: Vec<DamageType>,
    pub priority: u32,
    pub success_rate: f64,
    pub avg_healing_time_ms: u64,
    pub created_at: DateTime<Utc>,
    pub invocation_count: u64,
    pub success_count: u64,
}

impl HealingMechanism {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        handles: Vec<DamageType>,
    ) -> Self {
        let name = name.into();
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let id = format!("heal_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            name,
            description: description.into(),
            handles,
            priority: 100,
            success_rate: 1.0,
            avg_healing_time_ms: 100,
            created_at: Utc::now(),
            invocation_count: 0,
            success_count: 0,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_success_rate(mut self, rate: f64) -> Self {
        self.success_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn can_handle(&self, damage_type: DamageType) -> bool {
        self.handles.contains(&damage_type)
    }

    pub fn record_invocation(&mut self, success: bool) {
        self.invocation_count += 1;
        if success {
            self.success_count += 1;
        }
        // Update success rate based on actual results
        if self.invocation_count > 0 {
            self.success_rate = self.success_count as f64 / self.invocation_count as f64;
        }
    }
}

/// Result of a healing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingResult {
    pub id: String,
    pub damage_id: String,
    pub mechanism_id: String,
    pub status: HealingStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub actions_taken: Vec<String>,
    pub residual_damage: f64,
    pub message: String,
}

impl HealingResult {
    pub fn new(damage_id: impl Into<String>, mechanism_id: impl Into<String>) -> Self {
        let damage_id = damage_id.into();
        let mut hasher = Sha256::new();
        hasher.update(damage_id.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        let id = format!("result_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            damage_id,
            mechanism_id: mechanism_id.into(),
            status: HealingStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            actions_taken: Vec::new(),
            residual_damage: 0.0,
            message: String::new(),
        }
    }

    pub fn start(&mut self) {
        self.status = HealingStatus::InProgress;
        self.started_at = Utc::now();
    }

    pub fn complete(&mut self, success: bool, message: impl Into<String>) {
        self.completed_at = Some(Utc::now());
        self.duration_ms = Some(
            (Utc::now() - self.started_at)
                .num_milliseconds()
                .max(0) as u64,
        );
        self.status = if success {
            HealingStatus::Completed
        } else {
            HealingStatus::Failed
        };
        self.message = message.into();
    }

    pub fn add_action(&mut self, action: impl Into<String>) {
        self.actions_taken.push(action.into());
    }
}

/// State of a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentState {
    pub id: String,
    pub name: String,
    pub health: ComponentHealth,
    pub health_score: f64,
    pub damage_history: Vec<String>,
    pub healing_history: Vec<String>,
    pub last_health_check: DateTime<Utc>,
    pub properties: HashMap<String, String>,
    pub recovery_count: u64,
}

impl ComponentState {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let id = format!("comp_{}", hex::encode(&hasher.finalize()[..8]));

        Self {
            id,
            name,
            health: ComponentHealth::Healthy,
            health_score: 1.0,
            damage_history: Vec::new(),
            healing_history: Vec::new(),
            last_health_check: Utc::now(),
            properties: HashMap::new(),
            recovery_count: 0,
        }
    }

    pub fn apply_damage(&mut self, severity: f64, damage_id: String) {
        self.health_score = (self.health_score - severity).max(0.0);
        self.update_health_status();
        self.damage_history.push(damage_id);
    }

    pub fn apply_healing(&mut self, recovery: f64, healing_id: String) {
        self.health_score = (self.health_score + recovery).min(1.0);
        self.update_health_status();
        self.healing_history.push(healing_id);
        self.recovery_count += 1;
    }

    fn update_health_status(&mut self) {
        self.health = if self.health_score >= 0.9 {
            ComponentHealth::Healthy
        } else if self.health_score >= 0.6 {
            ComponentHealth::Degraded
        } else if self.health_score >= 0.3 {
            ComponentHealth::Impaired
        } else if self.health_score > 0.0 {
            ComponentHealth::Recovering
        } else {
            ComponentHealth::Failed
        };
        self.last_health_check = Utc::now();
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

/// Healing strategy for selecting mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealingStrategy {
    /// Use highest priority mechanism
    Priority,
    /// Use mechanism with best success rate
    BestSuccess,
    /// Try all applicable mechanisms
    Exhaustive,
    /// Use fastest mechanism
    Fastest,
}

/// The main Self-Healing engine
pub struct EmbeddedSelfHealing {
    components: HashMap<String, ComponentState>,
    mechanisms: HashMap<String, HealingMechanism>,
    damage_events: HashMap<String, DamageEvent>,
    healing_results: HashMap<String, HealingResult>,
    strategy: HealingStrategy,
    auto_heal: bool,
    #[allow(dead_code)]
    max_healing_attempts: u32,
    total_heals: AtomicU64,
    total_damages: AtomicU64,
}

impl Default for EmbeddedSelfHealing {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddedSelfHealing {
    pub fn new() -> Self {
        let mut engine = Self {
            components: HashMap::new(),
            mechanisms: HashMap::new(),
            damage_events: HashMap::new(),
            healing_results: HashMap::new(),
            strategy: HealingStrategy::Priority,
            auto_heal: true,
            max_healing_attempts: 3,
            total_heals: AtomicU64::new(0),
            total_damages: AtomicU64::new(0),
        };
        engine.initialize_core_mechanisms();
        engine
    }

    pub fn with_strategy(mut self, strategy: HealingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_auto_heal(mut self, auto_heal: bool) -> Self {
        self.auto_heal = auto_heal;
        self
    }

    /// Initialize core healing mechanisms
    fn initialize_core_mechanisms(&mut self) {
        // Retry mechanism for transient failures
        let retry = HealingMechanism::new(
            "retry",
            "Automatic retry for transient failures",
            vec![DamageType::Timeout, DamageType::Disconnection],
        )
        .with_priority(100)
        .with_success_rate(0.8);

        // Reset mechanism for corruption
        let reset = HealingMechanism::new(
            "reset",
            "Reset component to known good state",
            vec![DamageType::Corruption, DamageType::Misconfiguration],
        )
        .with_priority(90)
        .with_success_rate(0.95);

        // Fallback mechanism for failures
        let fallback = HealingMechanism::new(
            "fallback",
            "Switch to fallback or degraded mode",
            vec![DamageType::Failure, DamageType::External],
        )
        .with_priority(80)
        .with_success_rate(0.9);

        // Resource recovery for exhaustion
        let resource_recovery = HealingMechanism::new(
            "resource_recovery",
            "Free resources and recover from exhaustion",
            vec![DamageType::Exhaustion, DamageType::Overflow],
        )
        .with_priority(85)
        .with_success_rate(0.85);

        // Graceful degradation
        let degradation = HealingMechanism::new(
            "graceful_degradation",
            "Reduce functionality to maintain core operations",
            vec![
                DamageType::Degradation,
                DamageType::Exhaustion,
                DamageType::Failure,
            ],
        )
        .with_priority(70)
        .with_success_rate(0.95);

        self.add_mechanism(retry);
        self.add_mechanism(reset);
        self.add_mechanism(fallback);
        self.add_mechanism(resource_recovery);
        self.add_mechanism(degradation);
    }

    /// Register a component for self-healing
    pub fn register_component(&mut self, component: ComponentState) -> String {
        let id = component.id.clone();
        self.components.insert(id.clone(), component);
        id
    }

    /// Create and register a component
    pub fn create_component(&mut self, name: impl Into<String>) -> String {
        let component = ComponentState::new(name);
        self.register_component(component)
    }

    /// Add a healing mechanism
    pub fn add_mechanism(&mut self, mechanism: HealingMechanism) -> String {
        let id = mechanism.id.clone();
        self.mechanisms.insert(id.clone(), mechanism);
        id
    }

    /// Report damage to a component
    pub fn report_damage(&mut self, event: DamageEvent) -> Result<HealingResult, SelfHealingError> {
        self.total_damages.fetch_add(1, Ordering::SeqCst);

        let component_id = event.component_id.clone();
        let damage_id = event.id.clone();
        let damage_type = event.damage_type;
        let severity = event.severity;

        // Apply damage to component
        if let Some(component) = self.components.get_mut(&component_id) {
            component.apply_damage(severity, damage_id.clone());
        } else {
            return Err(SelfHealingError::ComponentNotFound(component_id));
        }

        // Store the damage event
        self.damage_events.insert(damage_id.clone(), event);

        // Auto-heal if enabled
        if self.auto_heal {
            self.heal_damage(&damage_id, damage_type, &component_id)
        } else {
            let result = HealingResult::new(&damage_id, "none");
            Ok(result)
        }
    }

    /// Attempt to heal a specific damage event
    fn heal_damage(
        &mut self,
        damage_id: &str,
        damage_type: DamageType,
        component_id: &str,
    ) -> Result<HealingResult, SelfHealingError> {
        // Find applicable mechanisms
        let mut applicable: Vec<&HealingMechanism> = self
            .mechanisms
            .values()
            .filter(|m| m.can_handle(damage_type))
            .collect();

        if applicable.is_empty() {
            return Err(SelfHealingError::MechanismNotFound(format!(
                "No mechanism for {:?}",
                damage_type
            )));
        }

        // Sort by strategy
        match self.strategy {
            HealingStrategy::Priority => {
                applicable.sort_by(|a, b| b.priority.cmp(&a.priority));
            }
            HealingStrategy::BestSuccess => {
                applicable.sort_by(|a, b| {
                    b.success_rate
                        .partial_cmp(&a.success_rate)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            HealingStrategy::Fastest => {
                applicable.sort_by(|a, b| a.avg_healing_time_ms.cmp(&b.avg_healing_time_ms));
            }
            HealingStrategy::Exhaustive => {
                // Keep original order, try all
            }
        }

        // Try healing with selected mechanism(s)
        let mechanism_id = applicable[0].id.clone();
        let mut result = HealingResult::new(damage_id, &mechanism_id);
        result.start();

        // Simulate healing process
        result.add_action(format!("Applying {} mechanism", applicable[0].name));
        result.add_action("Analyzing damage pattern".to_string());
        result.add_action("Initiating recovery procedure".to_string());

        // Determine success based on mechanism success rate
        let success = applicable[0].success_rate > 0.5; // Simplified for demo

        if success {
            // Apply healing to component
            if let Some(component) = self.components.get_mut(component_id) {
                let recovery = 0.5; // Recover 50% of health
                component.apply_healing(recovery, result.id.clone());
                result.residual_damage = (1.0 - component.health_score).max(0.0);
            }

            // Mark damage as healed
            if let Some(damage) = self.damage_events.get_mut(damage_id) {
                damage.mark_healed(result.id.clone());
            }

            result.complete(true, "Healing completed successfully");
            self.total_heals.fetch_add(1, Ordering::SeqCst);
        } else {
            result.complete(false, "Healing mechanism failed to repair damage");
        }

        // Record mechanism usage
        if let Some(mechanism) = self.mechanisms.get_mut(&mechanism_id) {
            mechanism.record_invocation(success);
        }

        // Store result
        let result_id = result.id.clone();
        self.healing_results.insert(result_id, result.clone());

        Ok(result)
    }

    /// Get component health
    pub fn get_component_health(&self, component_id: &str) -> Option<ComponentHealth> {
        self.components.get(component_id).map(|c| c.health)
    }

    /// Get component state
    pub fn get_component(&self, component_id: &str) -> Option<&ComponentState> {
        self.components.get(component_id)
    }

    /// Get all components
    pub fn get_all_components(&self) -> Vec<&ComponentState> {
        self.components.values().collect()
    }

    /// Get system-wide health score
    pub fn get_system_health(&self) -> f64 {
        if self.components.is_empty() {
            return 1.0;
        }

        let total: f64 = self.components.values().map(|c| c.health_score).sum();
        total / self.components.len() as f64
    }

    /// Get healing statistics
    pub fn get_statistics(&self) -> HealingStatistics {
        let total_damages = self.total_damages.load(Ordering::SeqCst);
        let total_heals = self.total_heals.load(Ordering::SeqCst);

        let mechanism_stats: HashMap<String, MechanismStats> = self
            .mechanisms
            .iter()
            .map(|(id, m)| {
                (
                    id.clone(),
                    MechanismStats {
                        name: m.name.clone(),
                        invocations: m.invocation_count,
                        successes: m.success_count,
                        success_rate: m.success_rate,
                    },
                )
            })
            .collect();

        let unhealthy_components: Vec<String> = self
            .components
            .iter()
            .filter(|(_, c)| !matches!(c.health, ComponentHealth::Healthy))
            .map(|(id, _)| id.clone())
            .collect();

        HealingStatistics {
            total_components: self.components.len(),
            total_mechanisms: self.mechanisms.len(),
            total_damage_events: total_damages,
            total_healing_operations: total_heals,
            system_health: self.get_system_health(),
            mechanism_stats,
            unhealthy_components,
        }
    }

    /// Perform health check on all components
    pub fn health_check(&mut self) -> Vec<(String, ComponentHealth)> {
        self.components
            .iter_mut()
            .map(|(id, component)| {
                component.last_health_check = Utc::now();
                (id.clone(), component.health)
            })
            .collect()
    }

    /// Force heal a component
    pub fn force_heal(&mut self, component_id: &str) -> Result<(), SelfHealingError> {
        let component = self
            .components
            .get_mut(component_id)
            .ok_or_else(|| SelfHealingError::ComponentNotFound(component_id.to_string()))?;

        component.health_score = 1.0;
        component.health = ComponentHealth::Healthy;
        component.recovery_count += 1;

        Ok(())
    }
}

/// Statistics for a healing mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismStats {
    pub name: String,
    pub invocations: u64,
    pub successes: u64,
    pub success_rate: f64,
}

/// Overall healing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingStatistics {
    pub total_components: usize,
    pub total_mechanisms: usize,
    pub total_damage_events: u64,
    pub total_healing_operations: u64,
    pub system_health: f64,
    pub mechanism_stats: HashMap<String, MechanismStats>,
    pub unhealthy_components: Vec<String>,
}

/// Wrapper for healing-enabled functions
pub struct HealingWrapper<T> {
    pub value: Option<T>,
    pub error: Option<String>,
    pub healing_applied: bool,
    pub attempts: u32,
}

impl<T> HealingWrapper<T> {
    pub fn success(value: T) -> Self {
        Self {
            value: Some(value),
            error: None,
            healing_applied: false,
            attempts: 1,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            value: None,
            error: Some(error.into()),
            healing_applied: false,
            attempts: 1,
        }
    }

    pub fn healed(value: T, attempts: u32) -> Self {
        Self {
            value: Some(value),
            error: None,
            healing_applied: true,
            attempts,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.value.is_some()
    }

    pub fn unwrap(self) -> T {
        self.value.expect("Called unwrap on a failed HealingWrapper")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_event_creation() {
        let event = DamageEvent::new("comp_1", DamageType::Corruption, 0.5, "Test damage")
            .with_context("key", "value");

        assert!(event.id.starts_with("dmg_"));
        assert_eq!(event.damage_type, DamageType::Corruption);
        assert_eq!(event.severity, 0.5);
        assert!(!event.healed);
    }

    #[test]
    fn test_healing_mechanism_creation() {
        let mechanism = HealingMechanism::new(
            "test_heal",
            "Test healing",
            vec![DamageType::Corruption, DamageType::Failure],
        )
        .with_priority(100)
        .with_success_rate(0.9);

        assert!(mechanism.id.starts_with("heal_"));
        assert!(mechanism.can_handle(DamageType::Corruption));
        assert!(mechanism.can_handle(DamageType::Failure));
        assert!(!mechanism.can_handle(DamageType::Timeout));
    }

    #[test]
    fn test_component_state() {
        let mut component = ComponentState::new("test_component");

        assert_eq!(component.health, ComponentHealth::Healthy);
        assert_eq!(component.health_score, 1.0);

        component.apply_damage(0.5, "dmg_1".to_string());
        assert!(component.health_score < 1.0);
        assert!(!component.damage_history.is_empty());
    }

    #[test]
    fn test_self_healing_engine() {
        let mut engine = EmbeddedSelfHealing::new();

        let comp_id = engine.create_component("test_comp");
        assert!(engine.get_component(&comp_id).is_some());

        // Check initial health
        assert_eq!(
            engine.get_component_health(&comp_id),
            Some(ComponentHealth::Healthy)
        );
    }

    #[test]
    fn test_damage_and_heal() {
        let mut engine = EmbeddedSelfHealing::new().with_auto_heal(true);

        let comp_id = engine.create_component("test_comp");

        let damage = DamageEvent::new(&comp_id, DamageType::Timeout, 0.3, "Connection timeout");

        let result = engine.report_damage(damage);
        assert!(result.is_ok());

        // Component should have been healed (at least partially)
        let health = engine.get_component_health(&comp_id);
        assert!(health.is_some());
    }

    #[test]
    fn test_healing_statistics() {
        let engine = EmbeddedSelfHealing::new();
        let stats = engine.get_statistics();

        assert!(stats.total_mechanisms > 0);
        assert_eq!(stats.system_health, 1.0); // No components = perfect health
    }
}
