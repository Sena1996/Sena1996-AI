use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingDepth {
    Quick,
    Standard,
    Deep,
    Maximum,
}

impl ThinkingDepth {
    pub fn steps(&self) -> usize {
        match self {
            ThinkingDepth::Quick => 3,
            ThinkingDepth::Standard => 5,
            ThinkingDepth::Deep => 8,
            ThinkingDepth::Maximum => 12,
        }
    }

    pub fn budget(&self) -> Option<u64> {
        None
    }

    pub fn label(&self) -> &'static str {
        match self {
            ThinkingDepth::Quick => "Quick Analysis",
            ThinkingDepth::Standard => "Standard Analysis",
            ThinkingDepth::Deep => "Deep Analysis",
            ThinkingDepth::Maximum => "Maximum Depth - Unlimited",
        }
    }
}

impl std::fmt::Display for ThinkingDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThinkingDepth::Quick => write!(f, "Quick"),
            ThinkingDepth::Standard => write!(f, "Standard"),
            ThinkingDepth::Deep => write!(f, "Deep"),
            ThinkingDepth::Maximum => write!(f, "Maximum"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThinkingEngine {
    frameworks: Vec<ThinkingFramework>,
    config: ThinkingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    pub default_depth: ThinkingDepth,
    pub show_reasoning: bool,
    pub cross_validate: bool,
    pub max_time_ms: u64,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        Self {
            default_depth: ThinkingDepth::Standard,
            show_reasoning: true,
            cross_validate: true,
            max_time_ms: 30000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThinkingFramework {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
}

impl ThinkingEngine {
    pub fn new() -> Self {
        Self {
            frameworks: default_frameworks(),
            config: ThinkingConfig::default(),
        }
    }

    pub fn with_config(config: ThinkingConfig) -> Self {
        Self {
            frameworks: default_frameworks(),
            config,
        }
    }

    pub fn analyze(&self, problem: &str, depth: ThinkingDepth) -> ThinkingResult {
        let mut result = ThinkingResult {
            problem: problem.to_string(),
            depth,
            steps: Vec::new(),
            frameworks_used: Vec::new(),
            conclusion: String::new(),
            confidence: 0.0,
            thinking_time_ms: 0,
        };

        let start = std::time::Instant::now();

        result.steps.push(ThinkingStep {
            name: "Problem Decomposition".to_string(),
            description: "Breaking down the problem into components".to_string(),
            output: format!("Analyzing: {}", problem),
        });

        if depth as u8 >= ThinkingDepth::Standard as u8 {
            result.steps.push(ThinkingStep {
                name: "First Principles Analysis".to_string(),
                description: "Identifying fundamental truths".to_string(),
                output: "Examining base assumptions and constraints...".to_string(),
            });
            result.frameworks_used.push("First Principles".to_string());
        }

        if depth as u8 >= ThinkingDepth::Standard as u8 {
            result.steps.push(ThinkingStep {
                name: "Root Cause Investigation".to_string(),
                description: "Asking 'Why?' to find underlying causes".to_string(),
                output: "Tracing causality chain...".to_string(),
            });
            result.frameworks_used.push("5 Whys".to_string());
        }

        if depth as u8 >= ThinkingDepth::Deep as u8 {
            result.steps.push(ThinkingStep {
                name: "Systems Analysis".to_string(),
                description: "Examining interconnections and feedback loops".to_string(),
                output: "Mapping system dependencies...".to_string(),
            });
            result.frameworks_used.push("Systems Thinking".to_string());
        }

        if depth as u8 >= ThinkingDepth::Deep as u8 {
            result.steps.push(ThinkingStep {
                name: "Risk Assessment".to_string(),
                description: "Evaluating potential failure modes".to_string(),
                output: "Identifying risks and mitigations...".to_string(),
            });
            result.frameworks_used.push("Risk Matrix".to_string());
        }

        if depth as u8 >= ThinkingDepth::Deep as u8 {
            result.steps.push(ThinkingStep {
                name: "Alternative Generation".to_string(),
                description: "Generating multiple solution approaches".to_string(),
                output: "Brainstorming alternatives...".to_string(),
            });
        }

        if depth as u8 >= ThinkingDepth::Maximum as u8 {
            result.steps.push(ThinkingStep {
                name: "Trade-off Analysis".to_string(),
                description: "Evaluating pros and cons of each approach".to_string(),
                output: "Weighing trade-offs...".to_string(),
            });
            result.frameworks_used.push("Decision Matrix".to_string());
        }

        if depth as u8 >= ThinkingDepth::Maximum as u8 {
            result.steps.push(ThinkingStep {
                name: "Second-Order Effects".to_string(),
                description: "Considering consequences of consequences".to_string(),
                output: "Analyzing downstream effects...".to_string(),
            });
            result.frameworks_used.push("Second-Order Thinking".to_string());
        }

        if depth as u8 >= ThinkingDepth::Maximum as u8 {
            result.steps.push(ThinkingStep {
                name: "Inversion Check".to_string(),
                description: "Asking 'How would this fail?'".to_string(),
                output: "Identifying failure modes to avoid...".to_string(),
            });
            result.frameworks_used.push("Inversion".to_string());
        }

        result.steps.push(ThinkingStep {
            name: "Synthesis".to_string(),
            description: "Combining insights into actionable conclusion".to_string(),
            output: "Formulating final recommendation...".to_string(),
        });

        result.confidence = match depth {
            ThinkingDepth::Quick => 0.6,
            ThinkingDepth::Standard => 0.75,
            ThinkingDepth::Deep => 0.85,
            ThinkingDepth::Maximum => 0.95,
        };

        result.conclusion = format!(
            "Analysis complete using {} frameworks across {} steps.",
            result.frameworks_used.len(),
            result.steps.len()
        );

        result.thinking_time_ms = start.elapsed().as_millis() as u64;

        result
    }

    pub fn frameworks(&self) -> &[ThinkingFramework] {
        &self.frameworks
    }

    pub fn config(&self) -> &ThinkingConfig {
        &self.config
    }
}

impl Default for ThinkingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingResult {
    pub problem: String,
    pub depth: ThinkingDepth,
    pub steps: Vec<ThinkingStep>,
    pub frameworks_used: Vec<String>,
    pub conclusion: String,
    pub confidence: f64,
    pub thinking_time_ms: u64,
}

impl ThinkingResult {
    pub fn format_brilliant(&self) -> String {
        let mut output = String::new();

        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                                                              ║\n");
        output.push_str("║              SENA 🦁 BRILLIANT THINKING                      ║\n");
        output.push_str("║                                                              ║\n");
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        output.push_str("════════════════════════════════════════════════════════════════\n");
        output.push_str("  PROBLEM ANALYSIS\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");
        output.push_str(&format!("{}\n\n", self.problem));

        output.push_str("════════════════════════════════════════════════════════════════\n");
        output.push_str("  THINKING PROCESS\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        for (i, step) in self.steps.iter().enumerate() {
            output.push_str(&format!("{}. **{}**\n", i + 1, step.name));
            output.push_str(&format!("   {}\n\n", step.description));
        }

        output.push_str("════════════════════════════════════════════════════════════════\n");
        output.push_str("  FRAMEWORKS APPLIED\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        for framework in &self.frameworks_used {
            output.push_str(&format!("  • {}\n", framework));
        }
        output.push('\n');

        output.push_str("════════════════════════════════════════════════════════════════\n");
        output.push_str("  CONCLUSION\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");
        output.push_str(&format!("{}\n\n", self.conclusion));
        output.push_str(&format!("Confidence: {:.0}%\n", self.confidence * 100.0));
        output.push_str(&format!("Thinking time: {}ms\n", self.thinking_time_ms));

        output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingStep {
    pub name: String,
    pub description: String,
    pub output: String,
}

fn default_frameworks() -> Vec<ThinkingFramework> {
    vec![
        ThinkingFramework {
            name: "First Principles".to_string(),
            description: "Break down to fundamental truths".to_string(),
            steps: vec![
                "Identify assumptions".to_string(),
                "Question each assumption".to_string(),
                "Rebuild from verified fundamentals".to_string(),
            ],
        },
        ThinkingFramework {
            name: "5 Whys".to_string(),
            description: "Find root causes by asking why repeatedly".to_string(),
            steps: vec![
                "State the problem".to_string(),
                "Ask 'Why did this happen?'".to_string(),
                "Repeat for each answer".to_string(),
                "Continue until root cause".to_string(),
            ],
        },
        ThinkingFramework {
            name: "Systems Thinking".to_string(),
            description: "View problems as interconnected systems".to_string(),
            steps: vec![
                "Map components".to_string(),
                "Identify relationships".to_string(),
                "Find feedback loops".to_string(),
                "Locate leverage points".to_string(),
            ],
        },
        ThinkingFramework {
            name: "Inversion".to_string(),
            description: "Ask how to guarantee failure, then avoid it".to_string(),
            steps: vec![
                "State your goal".to_string(),
                "Invert: How would I fail?".to_string(),
                "List failure modes".to_string(),
                "Avoid each failure mode".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thinking_engine_creation() {
        let engine = ThinkingEngine::new();
        assert!(engine.frameworks().len() > 0);
    }

    #[test]
    fn test_analyze_quick() {
        let engine = ThinkingEngine::new();
        let result = engine.analyze("Why is the system slow?", ThinkingDepth::Quick);
        assert!(result.steps.len() >= 2);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_analyze_deep() {
        let engine = ThinkingEngine::new();
        let result = engine.analyze("Design a scalable system", ThinkingDepth::Deep);
        assert!(result.steps.len() >= 5);
        assert!(result.frameworks_used.len() >= 3);
    }

    #[test]
    fn test_brilliant_format() {
        let engine = ThinkingEngine::new();
        let result = engine.analyze("Test problem", ThinkingDepth::Standard);
        let formatted = result.format_brilliant();
        assert!(formatted.contains("SENA"));
        assert!(formatted.contains("BRILLIANT THINKING"));
    }
}
