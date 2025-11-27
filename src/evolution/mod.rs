mod feedback;
mod learner;
mod optimizer;

pub use feedback::{FeedbackEntry, FeedbackLoop, FeedbackType};
pub use learner::{LearnedPattern, PatternLearner, PatternType};
pub use optimizer::{OptimizationResult, OptimizationTarget, SelfOptimizer};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug)]
pub struct EvolutionSystem {
    pub learner: PatternLearner,
    pub optimizer: SelfOptimizer,
    pub feedback: FeedbackLoop,
    pub stats: EvolutionStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvolutionStats {
    pub patterns_learned: usize,
    pub optimizations_applied: usize,
    pub feedback_count: usize,
    pub improvement_score: f64,
    pub learning_rate: f64,
    pub last_evolution: Option<String>,
}

impl EvolutionSystem {
    pub fn new() -> Self {
        Self {
            learner: PatternLearner::new(),
            optimizer: SelfOptimizer::new(),
            feedback: FeedbackLoop::new(),
            stats: EvolutionStats {
                learning_rate: 0.1,
                ..Default::default()
            },
        }
    }

    pub fn learn(&mut self, context: &str, outcome: &str, success: bool) {
        if success {
            self.learner.learn(context, outcome);
            self.stats.patterns_learned = self.learner.pattern_count();
        }
        self.update_stats();
    }

    pub fn optimize(&mut self, target: OptimizationTarget) -> OptimizationResult {
        let result = self.optimizer.optimize(target);
        if result.success {
            self.stats.optimizations_applied += 1;
        }
        self.update_stats();
        result
    }

    pub fn process_feedback(&mut self, feedback_type: FeedbackType, content: &str) {
        self.feedback.add(feedback_type, content);
        self.stats.feedback_count = self.feedback.count();
        self.update_stats();
    }

    pub fn status(&self) -> EvolutionStatus {
        EvolutionStatus {
            is_learning: true,
            patterns_count: self.learner.pattern_count(),
            optimizations_count: self.stats.optimizations_applied,
            feedback_count: self.feedback.count(),
            improvement_score: self.stats.improvement_score,
            health: self.calculate_health(),
        }
    }

    pub fn evolve(&mut self) -> EvolutionResult {
        let feedback_insights = self.feedback.analyze();
        let patterns_applied = self.learner.apply_learnings();
        let optimization = self.optimizer.optimize(OptimizationTarget::Quality);

        self.stats.improvement_score = self.calculate_improvement();
        self.stats.last_evolution = Some(chrono::Utc::now().to_rfc3339());
        self.update_stats();

        EvolutionResult {
            patterns_applied,
            optimizations_made: if optimization.success { 1 } else { 0 },
            feedback_processed: feedback_insights.len(),
            new_improvement_score: self.stats.improvement_score,
        }
    }

    fn update_stats(&mut self) {
        self.stats.patterns_learned = self.learner.pattern_count();
        self.stats.feedback_count = self.feedback.count();
    }

    fn calculate_improvement(&self) -> f64 {
        let pattern_factor = (self.stats.patterns_learned as f64 / 100.0).min(0.3);
        let optimization_factor = (self.stats.optimizations_applied as f64 / 50.0).min(0.3);
        let feedback_factor = (self.stats.feedback_count as f64 / 100.0).min(0.2);

        let base = 0.2;
        (base + pattern_factor + optimization_factor + feedback_factor).min(1.0)
    }

    fn calculate_health(&self) -> String {
        let score = self.stats.improvement_score;
        if score >= 0.8 {
            "Excellent".to_string()
        } else if score >= 0.6 {
            "Good".to_string()
        } else if score >= 0.4 {
            "Fair".to_string()
        } else {
            "Needs Improvement".to_string()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let evolution_dir = home.join(".sena").join("evolution");

        std::fs::create_dir_all(&evolution_dir)
            .map_err(|e| format!("Failed to create evolution directory: {}", e))?;

        self.learner.save(&evolution_dir.join("patterns.json"))?;
        self.feedback.save(&evolution_dir.join("feedback.json"))?;

        let stats_json = serde_json::to_string_pretty(&self.stats)
            .map_err(|e| format!("Failed to serialize stats: {}", e))?;
        std::fs::write(evolution_dir.join("stats.json"), stats_json)
            .map_err(|e| format!("Failed to write stats: {}", e))?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let evolution_dir = home.join(".sena").join("evolution");

        if !evolution_dir.exists() {
            return Ok(());
        }

        let _ = self.learner.load(&evolution_dir.join("patterns.json"));
        let _ = self.feedback.load(&evolution_dir.join("feedback.json"));

        if let Ok(content) = std::fs::read_to_string(evolution_dir.join("stats.json")) {
            if let Ok(stats) = serde_json::from_str(&content) {
                self.stats = stats;
            }
        }

        Ok(())
    }
}

impl Default for EvolutionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionStatus {
    pub is_learning: bool,
    pub patterns_count: usize,
    pub optimizations_count: usize,
    pub feedback_count: usize,
    pub improvement_score: f64,
    pub health: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub patterns_applied: usize,
    pub optimizations_made: usize,
    pub feedback_processed: usize,
    pub new_improvement_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_system_creation() {
        let system = EvolutionSystem::new();
        let status = system.status();
        assert!(status.is_learning);
    }

    #[test]
    fn test_learning() {
        let mut system = EvolutionSystem::new();
        system.learn("test context", "good outcome", true);
        assert!(system.learner.pattern_count() > 0);
    }

    #[test]
    fn test_feedback_processing() {
        let mut system = EvolutionSystem::new();
        system.process_feedback(FeedbackType::Positive, "Great response!");
        assert_eq!(system.feedback.count(), 1);
    }

    #[test]
    fn test_evolution_cycle() {
        let mut system = EvolutionSystem::new();
        system.learn("context", "outcome", true);
        system.process_feedback(FeedbackType::Positive, "Good");

        let result = system.evolve();
        assert!(result.new_improvement_score > 0.0);
    }
}
