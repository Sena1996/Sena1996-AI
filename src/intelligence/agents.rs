use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Security,
    Performance,
    Architecture,
    General,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Security => write!(f, "🔒 Security Agent"),
            AgentType::Performance => write!(f, "⚡ Performance Agent"),
            AgentType::Architecture => write!(f, "🏗️  Architecture Agent"),
            AgentType::General => write!(f, "🤖 General Agent"),
        }
    }
}

impl AgentType {
    pub fn description(&self) -> &'static str {
        match self {
            AgentType::Security => "Specialized in security auditing, vulnerability detection, and secure coding practices.",
            AgentType::Performance => "Specialized in performance analysis, optimization strategies, and efficiency improvements.",
            AgentType::Architecture => "Specialized in architecture review, design patterns, and system structure.",
            AgentType::General => "General-purpose agent for varied analysis tasks.",
        }
    }

    pub fn trigger_keywords(&self) -> Vec<&'static str> {
        match self {
            AgentType::Security => vec![
                "security", "vulnerability", "xss", "sql injection", "auth",
                "authentication", "authorization", "csrf", "owasp", "hack",
                "exploit", "secure", "encrypt", "password", "token", "jwt"
            ],
            AgentType::Performance => vec![
                "performance", "slow", "fast", "optimize", "latency", "throughput",
                "cache", "memory", "cpu", "bottleneck", "scalable", "efficient",
                "complexity", "big o", "algorithm"
            ],
            AgentType::Architecture => vec![
                "architecture", "design", "pattern", "solid", "structure",
                "refactor", "module", "component", "layer", "microservice",
                "monolith", "dependency", "coupling", "cohesion"
            ],
            AgentType::General => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub agent_type: AgentType,
    pub name: String,
    pub capabilities: Vec<String>,
    pub executions: u64,
}

impl Agent {
    pub fn new(agent_type: AgentType) -> Self {
        let (name, capabilities) = match agent_type {
            AgentType::Security => (
                "Security Auditor".to_string(),
                vec![
                    "Vulnerability scanning".to_string(),
                    "OWASP Top 10 detection".to_string(),
                    "Secure code review".to_string(),
                    "Authentication analysis".to_string(),
                    "Encryption validation".to_string(),
                ],
            ),
            AgentType::Performance => (
                "Performance Optimizer".to_string(),
                vec![
                    "Complexity analysis".to_string(),
                    "Bottleneck detection".to_string(),
                    "Caching strategies".to_string(),
                    "Query optimization".to_string(),
                    "Memory profiling".to_string(),
                ],
            ),
            AgentType::Architecture => (
                "Architecture Reviewer".to_string(),
                vec![
                    "Pattern recognition".to_string(),
                    "SOLID compliance".to_string(),
                    "Dependency analysis".to_string(),
                    "Coupling evaluation".to_string(),
                    "Design review".to_string(),
                ],
            ),
            AgentType::General => (
                "General Analyst".to_string(),
                vec![
                    "Code analysis".to_string(),
                    "Documentation".to_string(),
                    "Problem solving".to_string(),
                    "Research".to_string(),
                ],
            ),
        };

        Self {
            agent_type,
            name,
            capabilities,
            executions: 0,
        }
    }

    pub fn execute(&mut self, task: &str) -> AgentResult {
        self.executions += 1;

        let analysis = match self.agent_type {
            AgentType::Security => self.security_analysis(task),
            AgentType::Performance => self.performance_analysis(task),
            AgentType::Architecture => self.architecture_analysis(task),
            AgentType::General => self.general_analysis(task),
        };

        AgentResult {
            agent: self.agent_type,
            task: task.to_string(),
            analysis,
            recommendations: self.generate_recommendations(task),
            confidence: self.calculate_confidence(task),
        }
    }

    fn security_analysis(&self, task: &str) -> String {
        let task_lower = task.to_lowercase();
        let mut findings = Vec::new();

        if task_lower.contains("sql") || task_lower.contains("query") {
            findings.push("🔴 Check for SQL injection vulnerabilities - use parameterized queries");
        }
        if task_lower.contains("user") && task_lower.contains("input") {
            findings.push("🔴 Validate and sanitize all user input");
        }
        if task_lower.contains("password") || task_lower.contains("auth") {
            findings.push("🟡 Ensure passwords are hashed with bcrypt/Argon2");
        }
        if task_lower.contains("token") || task_lower.contains("jwt") {
            findings.push("🟡 Use short-lived tokens with refresh rotation");
        }
        if task_lower.contains("api") {
            findings.push("🟡 Implement rate limiting on API endpoints");
        }

        if findings.is_empty() {
            findings.push("✅ No immediate security concerns detected");
        }

        format!("Security Analysis:\n{}", findings.join("\n"))
    }

    fn performance_analysis(&self, task: &str) -> String {
        let task_lower = task.to_lowercase();
        let mut findings = Vec::new();

        if task_lower.contains("loop") && task_lower.contains("nested") {
            findings.push("🔴 Nested loops detected - check for O(n²) complexity");
        }
        if task_lower.contains("database") || task_lower.contains("query") {
            findings.push("🟡 Check for N+1 query problems");
            findings.push("🟡 Ensure proper indexing on queried columns");
        }
        if task_lower.contains("array") && task_lower.contains("search") {
            findings.push("🟡 Consider using Set/Map for O(1) lookups");
        }
        if task_lower.contains("fetch") || task_lower.contains("api") {
            findings.push("🟡 Consider request batching and caching");
        }

        if findings.is_empty() {
            findings.push("✅ No immediate performance concerns detected");
        }

        format!("Performance Analysis:\n{}", findings.join("\n"))
    }

    fn architecture_analysis(&self, task: &str) -> String {
        let task_lower = task.to_lowercase();
        let mut findings = Vec::new();

        if task_lower.contains("class") && (task_lower.contains("many") || task_lower.contains("large")) {
            findings.push("🟡 Consider Single Responsibility Principle - split large classes");
        }
        if task_lower.contains("depend") {
            findings.push("🟡 Check Dependency Inversion - depend on abstractions");
        }
        if task_lower.contains("inherit") {
            findings.push("🟡 Favor composition over inheritance");
        }
        if task_lower.contains("module") || task_lower.contains("component") {
            findings.push("🟡 Ensure loose coupling between modules");
        }

        if findings.is_empty() {
            findings.push("✅ Architecture appears sound");
        }

        format!("Architecture Analysis:\n{}", findings.join("\n"))
    }

    fn general_analysis(&self, task: &str) -> String {
        format!("General Analysis:\nTask received: {}\nReady for detailed analysis.", task)
    }

    fn generate_recommendations(&self, _task: &str) -> Vec<String> {
        match self.agent_type {
            AgentType::Security => vec![
                "Review OWASP Top 10 checklist".to_string(),
                "Run security linter (e.g., eslint-plugin-security)".to_string(),
                "Consider penetration testing".to_string(),
            ],
            AgentType::Performance => vec![
                "Profile code to identify bottlenecks".to_string(),
                "Add performance benchmarks".to_string(),
                "Monitor production metrics".to_string(),
            ],
            AgentType::Architecture => vec![
                "Document architecture decisions (ADRs)".to_string(),
                "Review with team for feedback".to_string(),
                "Consider future extensibility".to_string(),
            ],
            AgentType::General => vec![
                "Break down task into smaller steps".to_string(),
                "Consider edge cases".to_string(),
                "Write tests for validation".to_string(),
            ],
        }
    }

    fn calculate_confidence(&self, task: &str) -> f64 {
        let keywords = self.agent_type.trigger_keywords();
        let task_lower = task.to_lowercase();

        if keywords.is_empty() {
            return 0.7;
        }

        let matches = keywords.iter()
            .filter(|k| task_lower.contains(*k))
            .count();

        let base_confidence = 0.6;
        let keyword_bonus = (matches as f64 / keywords.len() as f64) * 0.3;

        (base_confidence + keyword_bonus).min(0.95)
    }
}

#[derive(Debug)]
pub struct AgentPool {
    agents: HashMap<AgentType, Agent>,
}

impl AgentPool {
    pub fn new() -> Self {
        let mut agents = HashMap::new();
        agents.insert(AgentType::Security, Agent::new(AgentType::Security));
        agents.insert(AgentType::Performance, Agent::new(AgentType::Performance));
        agents.insert(AgentType::Architecture, Agent::new(AgentType::Architecture));
        agents.insert(AgentType::General, Agent::new(AgentType::General));

        Self { agents }
    }

    pub fn count(&self) -> usize {
        self.agents.len()
    }

    pub fn dispatch(&self, task: &str, agent_type: AgentType) -> AgentResult {
        let mut agent = self.agents.get(&agent_type)
            .cloned()
            .unwrap_or_else(|| Agent::new(AgentType::General));

        agent.execute(task)
    }

    pub fn auto_dispatch(&self, task: &str) -> AgentResult {
        let agent_type = self.select_best_agent(task);
        self.dispatch(task, agent_type)
    }

    pub fn select_best_agent(&self, task: &str) -> AgentType {
        let task_lower = task.to_lowercase();
        let mut best_match = AgentType::General;
        let mut best_score = 0;

        for agent_type in [AgentType::Security, AgentType::Performance, AgentType::Architecture] {
            let keywords = agent_type.trigger_keywords();
            let score = keywords.iter()
                .filter(|k| task_lower.contains(*k))
                .count();

            if score > best_score {
                best_score = score;
                best_match = agent_type;
            }
        }

        best_match
    }

    pub fn list_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }
}

impl Default for AgentPool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent: AgentType,
    pub task: String,
    pub analysis: String,
    pub recommendations: Vec<String>,
    pub confidence: f64,
}

impl AgentResult {
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("╔══════════════════════════════════════════════════════════════╗\n"));
        output.push_str(&format!("║  {}                                             ║\n", self.agent));
        output.push_str(&format!("╚══════════════════════════════════════════════════════════════╝\n\n"));

        output.push_str(&format!("{}\n\n", self.analysis));

        output.push_str("Recommendations:\n");
        for rec in &self.recommendations {
            output.push_str(&format!("  • {}\n", rec));
        }

        output.push_str(&format!("\nConfidence: {:.0}%\n", self.confidence * 100.0));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_pool_creation() {
        let pool = AgentPool::new();
        assert_eq!(pool.count(), 4);
    }

    #[test]
    fn test_agent_dispatch() {
        let pool = AgentPool::new();
        let result = pool.dispatch("Check SQL injection", AgentType::Security);
        assert_eq!(result.agent, AgentType::Security);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_auto_dispatch_security() {
        let pool = AgentPool::new();
        let agent_type = pool.select_best_agent("Check for SQL injection vulnerabilities");
        assert_eq!(agent_type, AgentType::Security);
    }

    #[test]
    fn test_auto_dispatch_performance() {
        let pool = AgentPool::new();
        let agent_type = pool.select_best_agent("Optimize database query performance");
        assert_eq!(agent_type, AgentType::Performance);
    }

    #[test]
    fn test_auto_dispatch_architecture() {
        let pool = AgentPool::new();
        let agent_type = pool.select_best_agent("Review design patterns and SOLID principles");
        assert_eq!(agent_type, AgentType::Architecture);
    }
}
