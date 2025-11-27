use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingMode {
    Quick,
    Standard,
    Deep,
    Maximum,
}

impl std::fmt::Display for ThinkingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThinkingMode::Quick => write!(f, "Quick"),
            ThinkingMode::Standard => write!(f, "Standard"),
            ThinkingMode::Deep => write!(f, "Deep"),
            ThinkingMode::Maximum => write!(f, "Maximum"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningFramework {
    pub name: String,
    pub description: String,
    pub process: Vec<String>,
    pub use_cases: Vec<String>,
    pub example: Option<String>,
}

impl ReasoningFramework {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            process: Vec::new(),
            use_cases: Vec::new(),
            example: None,
        }
    }

    pub fn with_step(mut self, step: &str) -> Self {
        self.process.push(step.to_string());
        self
    }

    pub fn with_steps(mut self, steps: &[&str]) -> Self {
        for step in steps {
            self.process.push(step.to_string());
        }
        self
    }

    pub fn with_use_case(mut self, use_case: &str) -> Self {
        self.use_cases.push(use_case.to_string());
        self
    }

    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }

    pub fn analyze(&self, problem: &str) -> FrameworkAnalysis {
        FrameworkAnalysis {
            framework: self.name.clone(),
            problem: problem.to_string(),
            steps: self
                .process
                .iter()
                .enumerate()
                .map(|(i, step)| AnalysisStep {
                    number: i + 1,
                    instruction: step.clone(),
                    result: None,
                })
                .collect(),
            conclusion: None,
        }
    }
}

impl std::fmt::Display for ReasoningFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f, "  {}", self.name)?;
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)?;
        writeln!(f)?;

        if !self.process.is_empty() {
            writeln!(f, "Process:")?;
            for (i, step) in self.process.iter().enumerate() {
                writeln!(f, "  {}. {}", i + 1, step)?;
            }
            writeln!(f)?;
        }

        if !self.use_cases.is_empty() {
            writeln!(f, "Use cases:")?;
            for use_case in &self.use_cases {
                writeln!(f, "  • {}", use_case)?;
            }
            writeln!(f)?;
        }

        if let Some(example) = &self.example {
            writeln!(f, "Example:")?;
            writeln!(f, "{}", example)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkAnalysis {
    pub framework: String,
    pub problem: String,
    pub steps: Vec<AnalysisStep>,
    pub conclusion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStep {
    pub number: usize,
    pub instruction: String,
    pub result: Option<String>,
}

pub fn default_frameworks() -> Vec<ReasoningFramework> {
    vec![
        ReasoningFramework::new(
            "First Principles Thinking",
            "Break complex problems down to fundamental truths and rebuild from there.",
        )
        .with_steps(&[
            "Identify and define current assumptions - What do we currently believe?",
            "Break down the problem into fundamental principles - What are the basic laws/truths?",
            "Rebuild from the ground up - Start with verified fundamentals",
        ])
        .with_use_case("Complex system design")
        .with_use_case("Performance optimization")
        .with_use_case("Debugging difficult issues")
        .with_example(
            "Problem: 'Why is system slow?'\n\
            Bad approach: 'Add more servers' (assumption-based)\n\
            First principles:\n\
            - Fundamental: Response time = Processing + I/O + Network\n\
            - Question: Which component dominates?\n\
            - Measure: Profile to find bottleneck\n\
            - Solution: Optimize the actual bottleneck",
        ),
        ReasoningFramework::new(
            "Root Cause Analysis (5 Whys)",
            "Identify underlying causes, not just symptoms, by asking 'Why?' repeatedly.",
        )
        .with_steps(&[
            "State the problem clearly",
            "Ask 'Why did this happen?' and document the answer",
            "For each answer, ask 'Why?' again",
            "Continue until you reach the root cause (typically 5 iterations)",
            "Verify the root cause by checking if fixing it prevents recurrence",
        ])
        .with_use_case("Bug investigation")
        .with_use_case("System failures")
        .with_use_case("Process breakdowns")
        .with_example(
            "Problem: Website is down\n\
            Why? → Server crashed\n\
            Why? → Out of memory\n\
            Why? → Memory leak in code\n\
            Why? → Unclosed database connections\n\
            Why? → Missing connection pool cleanup\n\
            Root Cause: No connection lifecycle management",
        ),
        ReasoningFramework::new(
            "Systems Thinking",
            "View problems as part of larger interconnected systems with feedback loops.",
        )
        .with_steps(&[
            "Map system components and their relationships",
            "Identify feedback loops (reinforcing and balancing)",
            "Find leverage points where small changes have big effects",
            "Consider second and third-order effects",
            "Look for unintended consequences",
        ])
        .with_use_case("Architecture decisions")
        .with_use_case("Organizational changes")
        .with_use_case("Feature planning")
        .with_example(
            "Reinforcing loop: More customers → More revenue → More marketing → More customers\n\
            Balancing loop: High prices → Reduced demand → Lower prices → Increased demand",
        ),
        ReasoningFramework::new(
            "Decision Matrix",
            "Systematically evaluate options against weighted criteria.",
        )
        .with_steps(&[
            "List all options/alternatives",
            "Define evaluation criteria",
            "Assign weights to criteria (importance)",
            "Score each option on each criterion",
            "Calculate weighted totals",
            "Select the highest-scoring option",
        ])
        .with_use_case("Technology selection")
        .with_use_case("Vendor evaluation")
        .with_use_case("Prioritization")
        .with_example(
            "Options: React vs Vue vs Angular\n\
            Criteria: Performance (3), Learning curve (2), Ecosystem (4)\n\
            React: 8×3 + 7×2 + 9×4 = 74\n\
            Vue: 8×3 + 9×2 + 7×4 = 70\n\
            Angular: 7×3 + 5×2 + 8×4 = 63",
        ),
        ReasoningFramework::new(
            "Inversion Thinking",
            "Instead of asking how to succeed, ask how to guarantee failure and avoid those.",
        )
        .with_steps(&[
            "State your goal",
            "Invert: Ask 'How would I guarantee failure?'",
            "List all ways to fail",
            "Invert again: Do the opposite of each failure mode",
        ])
        .with_use_case("Risk mitigation")
        .with_use_case("Project planning")
        .with_use_case("Quality assurance")
        .with_example(
            "Goal: Build a successful product\n\
            Invert: How to guarantee failure?\n\
            - Ignore user feedback\n\
            - Add unnecessary complexity\n\
            - Skip testing\n\
            - Poor documentation\n\
            Prevention: Do the opposite!",
        ),
        ReasoningFramework::new(
            "Probabilistic Thinking",
            "Think in terms of probabilities and expected values rather than certainties.",
        )
        .with_steps(&[
            "Identify possible outcomes",
            "Estimate probability of each outcome",
            "Calculate expected value: Σ(Probability × Outcome)",
            "Consider variance and worst-case scenarios",
            "Make decision based on expected value and risk tolerance",
        ])
        .with_use_case("Risk assessment")
        .with_use_case("Investment decisions")
        .with_use_case("A/B testing")
        .with_example(
            "Investment decision:\n\
            60% chance of $100K gain = 0.6 × $100K = $60K\n\
            30% chance of $0 = 0.3 × $0 = $0\n\
            10% chance of $50K loss = 0.1 × -$50K = -$5K\n\
            Expected Value = $55K",
        ),
        ReasoningFramework::new(
            "Theory of Constraints",
            "A system's throughput is limited by its constraint (bottleneck). Focus there.",
        )
        .with_steps(&[
            "IDENTIFY the system constraint (bottleneck)",
            "EXPLOIT the constraint (maximize its output)",
            "SUBORDINATE everything else to the constraint",
            "ELEVATE the constraint (increase its capacity)",
            "REPEAT - find the next constraint",
        ])
        .with_use_case("Process optimization")
        .with_use_case("Resource allocation")
        .with_use_case("Pipeline improvement")
        .with_example(
            "Software development bottleneck:\n\
            1. Identify: Code review is slowest step\n\
            2. Exploit: Prioritize reviews, clear blockers\n\
            3. Subordinate: Slow down coding to match review capacity\n\
            4. Elevate: Add reviewers, automate checks\n\
            5. Repeat: Next bottleneck might be testing",
        ),
        ReasoningFramework::new(
            "Second-Order Thinking",
            "Consider not just the immediate effects, but the effects of the effects.",
        )
        .with_steps(&[
            "Identify the first-order effect (immediate consequence)",
            "Ask 'And then what?' for each effect",
            "Map out second and third-order effects",
            "Evaluate the full chain of consequences",
            "Consider if long-term effects outweigh short-term gains",
        ])
        .with_use_case("Strategic decisions")
        .with_use_case("Policy changes")
        .with_use_case("Feature design")
        .with_example(
            "Decision: Fire an employee to save money\n\
            First-order: Save $100K salary\n\
            Second-order: Lose institutional knowledge, remaining team demoralized\n\
            Third-order: Others leave, productivity drops, hiring costs increase",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_frameworks() {
        let frameworks = default_frameworks();
        assert!(frameworks.len() >= 5);
    }

    #[test]
    fn test_framework_analysis() {
        let framework = &default_frameworks()[0];
        let analysis = framework.analyze("Why is the system slow?");
        assert_eq!(analysis.framework, "First Principles Thinking");
        assert!(analysis.steps.len() > 0);
    }

    #[test]
    fn test_framework_display() {
        let framework = &default_frameworks()[0];
        let display = format!("{}", framework);
        assert!(display.contains("First Principles"));
    }
}
