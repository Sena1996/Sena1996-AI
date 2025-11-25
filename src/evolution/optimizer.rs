use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationTarget {
    Quality,
    Speed,
    Accuracy,
    Satisfaction,
    Balanced,
}

impl std::fmt::Display for OptimizationTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationTarget::Quality => write!(f, "Quality"),
            OptimizationTarget::Speed => write!(f, "Speed"),
            OptimizationTarget::Accuracy => write!(f, "Accuracy"),
            OptimizationTarget::Satisfaction => write!(f, "User Satisfaction"),
            OptimizationTarget::Balanced => write!(f, "Balanced"),
        }
    }
}

#[derive(Debug)]
pub struct SelfOptimizer {
    metrics: OptimizationMetrics,
    history: Vec<OptimizationRecord>,
    config: OptimizerConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    pub quality: f64,
    pub speed: f64,
    pub accuracy: f64,
    pub satisfaction: f64,
}

impl OptimizationMetrics {
    pub fn overall(&self) -> f64 {
        (self.quality + self.speed + self.accuracy + self.satisfaction) / 4.0
    }

    pub fn score_for(&self, target: OptimizationTarget) -> f64 {
        match target {
            OptimizationTarget::Quality => self.quality,
            OptimizationTarget::Speed => self.speed,
            OptimizationTarget::Accuracy => self.accuracy,
            OptimizationTarget::Satisfaction => self.satisfaction,
            OptimizationTarget::Balanced => self.overall(),
        }
    }
}

#[derive(Debug, Clone)]
struct OptimizerConfig {
    learning_rate: f64,
    min_improvement: f64,
    max_history: usize,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            min_improvement: 0.01,
            max_history: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OptimizationRecord {
    target: OptimizationTarget,
    before: f64,
    after: f64,
    timestamp: String,
}

impl SelfOptimizer {
    pub fn new() -> Self {
        Self {
            metrics: OptimizationMetrics {
                quality: 0.7,
                speed: 0.8,
                accuracy: 0.75,
                satisfaction: 0.7,
            },
            history: Vec::new(),
            config: OptimizerConfig::default(),
        }
    }

    pub fn optimize(&mut self, target: OptimizationTarget) -> OptimizationResult {
        let before = self.metrics.score_for(target);

        let improvement = match target {
            OptimizationTarget::Quality => self.optimize_quality(),
            OptimizationTarget::Speed => self.optimize_speed(),
            OptimizationTarget::Accuracy => self.optimize_accuracy(),
            OptimizationTarget::Satisfaction => self.optimize_satisfaction(),
            OptimizationTarget::Balanced => self.optimize_balanced(),
        };

        let after = self.metrics.score_for(target);

        self.history.push(OptimizationRecord {
            target,
            before,
            after,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        if self.history.len() > self.config.max_history {
            self.history.remove(0);
        }

        OptimizationResult {
            target,
            success: improvement > self.config.min_improvement,
            improvement,
            new_score: after,
            suggestions: self.generate_suggestions(target),
        }
    }

    fn optimize_quality(&mut self) -> f64 {
        let improvement = self.config.learning_rate * (1.0 - self.metrics.quality) * 0.5;
        self.metrics.quality = (self.metrics.quality + improvement).min(1.0);
        improvement
    }

    fn optimize_speed(&mut self) -> f64 {
        let improvement = self.config.learning_rate * (1.0 - self.metrics.speed) * 0.5;
        self.metrics.speed = (self.metrics.speed + improvement).min(1.0);
        improvement
    }

    fn optimize_accuracy(&mut self) -> f64 {
        let improvement = self.config.learning_rate * (1.0 - self.metrics.accuracy) * 0.5;
        self.metrics.accuracy = (self.metrics.accuracy + improvement).min(1.0);
        improvement
    }

    fn optimize_satisfaction(&mut self) -> f64 {
        let improvement = self.config.learning_rate * (1.0 - self.metrics.satisfaction) * 0.5;
        self.metrics.satisfaction = (self.metrics.satisfaction + improvement).min(1.0);
        improvement
    }

    fn optimize_balanced(&mut self) -> f64 {
        let q = self.optimize_quality();
        let s = self.optimize_speed();
        let a = self.optimize_accuracy();
        let sat = self.optimize_satisfaction();
        (q + s + a + sat) / 4.0
    }

    fn generate_suggestions(&self, target: OptimizationTarget) -> Vec<String> {
        let mut suggestions = Vec::new();

        match target {
            OptimizationTarget::Quality => {
                if self.metrics.quality < 0.8 {
                    suggestions.push("Apply more reasoning frameworks".to_string());
                    suggestions.push("Increase analysis depth".to_string());
                }
            }
            OptimizationTarget::Speed => {
                if self.metrics.speed < 0.8 {
                    suggestions.push("Cache frequently used patterns".to_string());
                    suggestions.push("Optimize search algorithms".to_string());
                }
            }
            OptimizationTarget::Accuracy => {
                if self.metrics.accuracy < 0.8 {
                    suggestions.push("Cross-validate conclusions".to_string());
                    suggestions.push("Use multiple reasoning approaches".to_string());
                }
            }
            OptimizationTarget::Satisfaction => {
                if self.metrics.satisfaction < 0.8 {
                    suggestions.push("Improve response formatting".to_string());
                    suggestions.push("Better align with user intent".to_string());
                }
            }
            OptimizationTarget::Balanced => {
                suggestions.push("Continue balanced optimization".to_string());
            }
        }

        suggestions
    }

    pub fn metrics(&self) -> &OptimizationMetrics {
        &self.metrics
    }

    pub fn update_metrics(&mut self, quality: Option<f64>, speed: Option<f64>,
                          accuracy: Option<f64>, satisfaction: Option<f64>) {
        if let Some(q) = quality {
            self.metrics.quality = self.blend(self.metrics.quality, q);
        }
        if let Some(s) = speed {
            self.metrics.speed = self.blend(self.metrics.speed, s);
        }
        if let Some(a) = accuracy {
            self.metrics.accuracy = self.blend(self.metrics.accuracy, a);
        }
        if let Some(sat) = satisfaction {
            self.metrics.satisfaction = self.blend(self.metrics.satisfaction, sat);
        }
    }

    fn blend(&self, current: f64, new: f64) -> f64 {
        let alpha = self.config.learning_rate;
        alpha * new + (1.0 - alpha) * current
    }

    pub fn history(&self) -> Vec<(OptimizationTarget, f64, f64)> {
        self.history.iter()
            .map(|r| (r.target, r.before, r.after))
            .collect()
    }
}

impl Default for SelfOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub target: OptimizationTarget,
    pub success: bool,
    pub improvement: f64,
    pub new_score: f64,
    pub suggestions: Vec<String>,
}

impl OptimizationResult {
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Optimization Target: {}\n", self.target));
        output.push_str(&format!("Success: {}\n", if self.success { "✅" } else { "❌" }));
        output.push_str(&format!("Improvement: +{:.1}%\n", self.improvement * 100.0));
        output.push_str(&format!("New Score: {:.1}%\n", self.new_score * 100.0));

        if !self.suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = SelfOptimizer::new();
        assert!(optimizer.metrics().overall() > 0.0);
    }

    #[test]
    fn test_optimization() {
        let mut optimizer = SelfOptimizer::new();
        let result = optimizer.optimize(OptimizationTarget::Quality);
        assert!(result.new_score > 0.0);
    }

    #[test]
    fn test_metrics_update() {
        let mut optimizer = SelfOptimizer::new();
        let before = optimizer.metrics().quality;
        optimizer.update_metrics(Some(1.0), None, None, None);
        assert!(optimizer.metrics().quality > before);
    }
}
