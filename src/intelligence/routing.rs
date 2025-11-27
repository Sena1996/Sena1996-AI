use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    Fast,
    Balanced,
    Powerful,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Fast => write!(f, "Fast (Haiku)"),
            ModelType::Balanced => write!(f, "Balanced (Sonnet)"),
            ModelType::Powerful => write!(f, "Powerful (Opus)"),
        }
    }
}

impl ModelType {
    pub fn typical_latency_ms(&self) -> u64 {
        match self {
            ModelType::Fast => 500,
            ModelType::Balanced => 2000,
            ModelType::Powerful => 5000,
        }
    }

    pub fn cost_factor(&self) -> f64 {
        match self {
            ModelType::Fast => 0.25,
            ModelType::Balanced => 1.0,
            ModelType::Powerful => 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelRouter {
    mode: RoutingMode,
    thresholds: ComplexityThresholds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingMode {
    Fixed(ModelType),
    Auto,
    Speed,
    Quality,
    Cost,
}

impl std::fmt::Display for RoutingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingMode::Fixed(model) => write!(f, "Fixed ({})", model),
            RoutingMode::Auto => write!(f, "Auto"),
            RoutingMode::Speed => write!(f, "Speed"),
            RoutingMode::Quality => write!(f, "Quality"),
            RoutingMode::Cost => write!(f, "Cost"),
        }
    }
}

#[derive(Debug, Clone)]
struct ComplexityThresholds {
    fast_max: f64,
    balanced_max: f64,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            fast_max: 0.3,
            balanced_max: 0.7,
        }
    }
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            mode: RoutingMode::Auto,
            thresholds: ComplexityThresholds::default(),
        }
    }

    pub fn with_mode(mode: RoutingMode) -> Self {
        Self {
            mode,
            thresholds: ComplexityThresholds::default(),
        }
    }

    pub fn current_mode(&self) -> String {
        format!("{}", self.mode)
    }

    pub fn set_mode(&mut self, mode: RoutingMode) {
        self.mode = mode;
    }

    pub fn route(&self, task: &str) -> RoutingDecision {
        match self.mode {
            RoutingMode::Fixed(model) => RoutingDecision {
                model,
                reason: "Fixed model selected".to_string(),
                complexity: self.analyze_complexity(task),
                confidence: 1.0,
            },
            RoutingMode::Speed => RoutingDecision {
                model: ModelType::Fast,
                reason: "Optimizing for speed".to_string(),
                complexity: self.analyze_complexity(task),
                confidence: 1.0,
            },
            RoutingMode::Quality => RoutingDecision {
                model: ModelType::Powerful,
                reason: "Optimizing for quality".to_string(),
                complexity: self.analyze_complexity(task),
                confidence: 1.0,
            },
            RoutingMode::Cost => RoutingDecision {
                model: ModelType::Fast,
                reason: "Optimizing for cost".to_string(),
                complexity: self.analyze_complexity(task),
                confidence: 1.0,
            },
            RoutingMode::Auto => self.auto_route(task),
        }
    }

    fn auto_route(&self, task: &str) -> RoutingDecision {
        let complexity = self.analyze_complexity(task);

        let (model, reason) = if complexity < self.thresholds.fast_max {
            (ModelType::Fast, "Simple task - using fast model")
        } else if complexity < self.thresholds.balanced_max {
            (ModelType::Balanced, "Standard task - using balanced model")
        } else {
            (ModelType::Powerful, "Complex task - using powerful model")
        };

        RoutingDecision {
            model,
            reason: reason.to_string(),
            complexity,
            confidence: self.calculate_confidence(complexity),
        }
    }

    fn analyze_complexity(&self, task: &str) -> f64 {
        let mut complexity = 0.0;
        let task_lower = task.to_lowercase();

        complexity += (task.len() as f64 / 500.0).min(0.3);

        let complex_keywords = [
            "analyze",
            "design",
            "architect",
            "optimize",
            "security",
            "performance",
            "refactor",
            "explain",
            "why",
            "how",
            "implement",
            "debug",
            "review",
            "evaluate",
            "complex",
        ];

        let keyword_matches = complex_keywords
            .iter()
            .filter(|k| task_lower.contains(*k))
            .count();

        complexity += (keyword_matches as f64 / complex_keywords.len() as f64) * 0.4;

        if task_lower.contains("why") || task_lower.contains("how") {
            complexity += 0.1;
        }

        if task_lower.contains("code")
            || task_lower.contains("function")
            || task_lower.contains("class")
        {
            complexity += 0.1;
        }

        if task_lower.contains("and then")
            || task_lower.contains("also")
            || task_lower.contains("additionally")
        {
            complexity += 0.1;
        }

        complexity.min(1.0)
    }

    fn calculate_confidence(&self, complexity: f64) -> f64 {
        let distance_to_thresholds = (complexity - self.thresholds.fast_max)
            .abs()
            .min((complexity - self.thresholds.balanced_max).abs());

        0.7 + (distance_to_thresholds * 0.3)
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub model: ModelType,
    pub reason: String,
    pub complexity: f64,
    pub confidence: f64,
}

impl RoutingDecision {
    pub fn format(&self) -> String {
        format!(
            "Model: {}\nReason: {}\nComplexity: {:.0}%\nConfidence: {:.0}%",
            self.model,
            self.reason,
            self.complexity * 100.0,
            self.confidence * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = ModelRouter::new();
        assert_eq!(router.current_mode(), "Auto");
    }

    #[test]
    fn test_simple_task_routing() {
        let router = ModelRouter::new();
        let decision = router.route("Hello world");
        assert_eq!(decision.model, ModelType::Fast);
    }

    #[test]
    fn test_complex_task_routing() {
        let router = ModelRouter::new();
        let decision = router.route(
            "Analyze the security vulnerabilities in this complex authentication system \
            and explain why the current design has performance issues. Also review \
            the architecture for SOLID principle compliance.",
        );
        assert_eq!(decision.model, ModelType::Powerful);
    }

    #[test]
    fn test_fixed_mode() {
        let router = ModelRouter::with_mode(RoutingMode::Fixed(ModelType::Balanced));
        let decision = router.route("Any task");
        assert_eq!(decision.model, ModelType::Balanced);
    }
}
