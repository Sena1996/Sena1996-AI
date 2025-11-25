use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub auto_activate: bool,
    pub category: String,
    pub executions: u64,
}

impl Skill {
    pub fn new(name: &str, description: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            triggers: Vec::new(),
            auto_activate: false,
            category: category.to_string(),
            executions: 0,
        }
    }

    pub fn with_triggers(mut self, triggers: &[&str]) -> Self {
        self.triggers = triggers.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_auto_activate(mut self) -> Self {
        self.auto_activate = true;
        self
    }

    pub fn should_activate(&self, context: &str) -> bool {
        if !self.auto_activate {
            return false;
        }

        let context_lower = context.to_lowercase();
        self.triggers.iter().any(|t| context_lower.contains(&t.to_lowercase()))
    }

    pub fn execute(&mut self, context: &str) -> SkillExecution {
        self.executions += 1;

        let output = match self.name.as_str() {
            "Security Auditor" => self.execute_security_audit(context),
            "Performance Optimizer" => self.execute_performance_optimization(context),
            "Truth Verifier" => self.execute_truth_verification(context),
            "Code Reviewer" => self.execute_code_review(context),
            _ => self.execute_generic(context),
        };

        SkillExecution {
            skill: self.name.clone(),
            context: context.to_string(),
            output,
            success: true,
        }
    }

    fn execute_security_audit(&self, context: &str) -> String {
        let mut output = String::new();
        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str("║              🔒 SECURITY AUDIT RESULTS                       ║\n");
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        let context_lower = context.to_lowercase();
        let mut findings = Vec::new();

        if context_lower.contains("sql") || context_lower.contains("query") {
            findings.push(("🔴 HIGH", "SQL Injection Risk", "Use parameterized queries"));
        }
        if context_lower.contains("eval") || context_lower.contains("exec") {
            findings.push(("🔴 HIGH", "Code Injection Risk", "Avoid eval/exec with user input"));
        }
        if context_lower.contains("password") && !context_lower.contains("hash") {
            findings.push(("🟡 MEDIUM", "Password Handling", "Ensure passwords are hashed"));
        }
        if context_lower.contains("token") || context_lower.contains("jwt") {
            findings.push(("🟡 MEDIUM", "Token Security", "Use short-lived tokens"));
        }
        if context_lower.contains("http") && !context_lower.contains("https") {
            findings.push(("🟡 MEDIUM", "Insecure Transport", "Use HTTPS"));
        }

        if findings.is_empty() {
            output.push_str("✅ No immediate security concerns detected.\n\n");
            output.push_str("Recommendations:\n");
            output.push_str("  • Continue following secure coding practices\n");
            output.push_str("  • Consider periodic security audits\n");
        } else {
            output.push_str("Findings:\n\n");
            for (severity, issue, recommendation) in findings {
                output.push_str(&format!("{} {}\n", severity, issue));
                output.push_str(&format!("   └─ {}\n\n", recommendation));
            }
        }

        output
    }

    fn execute_performance_optimization(&self, context: &str) -> String {
        let mut output = String::new();
        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str("║              ⚡ PERFORMANCE ANALYSIS                         ║\n");
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        let context_lower = context.to_lowercase();
        let mut suggestions = Vec::new();

        if context_lower.contains("loop") {
            suggestions.push("Consider loop optimization or vectorization");
        }
        if context_lower.contains("database") || context_lower.contains("query") {
            suggestions.push("Check for N+1 queries, add indexes");
        }
        if context_lower.contains("array") && context_lower.contains("find") {
            suggestions.push("Use Map/Set for O(1) lookups instead of array search");
        }
        if context_lower.contains("async") || context_lower.contains("await") {
            suggestions.push("Run independent async operations in parallel with Promise.all");
        }

        if suggestions.is_empty() {
            output.push_str("✅ No obvious performance issues detected.\n\n");
        } else {
            output.push_str("Optimization Suggestions:\n\n");
            for suggestion in suggestions {
                output.push_str(&format!("  ⚡ {}\n", suggestion));
            }
            output.push('\n');
        }

        output.push_str("General Recommendations:\n");
        output.push_str("  • Profile before optimizing\n");
        output.push_str("  • Measure impact of changes\n");
        output.push_str("  • Consider caching for expensive operations\n");

        output
    }

    fn execute_truth_verification(&self, context: &str) -> String {
        let mut output = String::new();
        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str("║              🔍 TRUTH VERIFICATION                           ║\n");
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        output.push_str(&format!("Claim: {}\n\n", context));
        output.push_str("Verification Status: PENDING ANALYSIS\n\n");
        output.push_str("To verify this claim:\n");
        output.push_str("  1. Identify the core assertion\n");
        output.push_str("  2. Find authoritative sources\n");
        output.push_str("  3. Cross-reference multiple sources\n");
        output.push_str("  4. Check for logical consistency\n");
        output.push_str("  5. Consider potential biases\n");

        output
    }

    fn execute_code_review(&self, context: &str) -> String {
        let mut output = String::new();
        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str("║              📝 CODE REVIEW                                  ║\n");
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        output.push_str("Review Checklist:\n\n");
        output.push_str("  ☐ Code follows project style guide\n");
        output.push_str("  ☐ Functions have single responsibility\n");
        output.push_str("  ☐ Error handling is appropriate\n");
        output.push_str("  ☐ No hardcoded values (use constants)\n");
        output.push_str("  ☐ Tests cover main functionality\n");
        output.push_str("  ☐ No security vulnerabilities\n");
        output.push_str("  ☐ Performance is acceptable\n");
        output.push_str("  ☐ Documentation is adequate\n");

        output
    }

    fn execute_generic(&self, context: &str) -> String {
        format!("Skill '{}' executed with context: {}", self.name, context)
    }
}

#[derive(Debug)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            skills: HashMap::new(),
        };

        registry.register(
            Skill::new("Security Auditor", "Analyzes code for security vulnerabilities", "Security")
                .with_triggers(&["security", "vulnerability", "xss", "sql injection", "auth", "secure"])
                .with_auto_activate()
        );

        registry.register(
            Skill::new("Performance Optimizer", "Analyzes code for performance improvements", "Performance")
                .with_triggers(&["performance", "slow", "optimize", "speed", "latency", "efficient"])
                .with_auto_activate()
        );

        registry.register(
            Skill::new("Truth Verifier", "Verifies factual claims", "Analysis")
                .with_triggers(&["is it true", "fact check", "verify", "is this correct"])
                .with_auto_activate()
        );

        registry.register(
            Skill::new("Code Reviewer", "Reviews code for quality and best practices", "Development")
                .with_triggers(&["review", "code review", "check this code"])
                .with_auto_activate()
        );

        registry
    }

    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }

    pub fn execute(&self, skill_name: &str, context: &str) -> Option<SkillExecution> {
        self.skills.get(skill_name).map(|skill| {
            let mut skill = skill.clone();
            skill.execute(context)
        })
    }

    pub fn auto_execute(&self, context: &str) -> Vec<SkillExecution> {
        self.skills.values()
            .filter(|s| s.should_activate(context))
            .map(|s| {
                let mut skill = s.clone();
                skill.execute(context)
            })
            .collect()
    }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecution {
    pub skill: String,
    pub context: String,
    pub output: String,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_registry_creation() {
        let registry = SkillRegistry::new();
        assert!(registry.count() >= 4);
    }

    #[test]
    fn test_skill_execution() {
        let registry = SkillRegistry::new();
        let result = registry.execute("Security Auditor", "Check SQL query handling");
        assert!(result.is_some());
        assert!(result.unwrap().success);
    }

    #[test]
    fn test_auto_activation() {
        let registry = SkillRegistry::new();
        let results = registry.auto_execute("Check security vulnerabilities");
        assert!(results.len() > 0);
    }

    #[test]
    fn test_skill_triggers() {
        let skill = Skill::new("Test", "Test skill", "Test")
            .with_triggers(&["test", "check"])
            .with_auto_activate();

        assert!(skill.should_activate("run a test"));
        assert!(!skill.should_activate("hello world"));
    }
}
