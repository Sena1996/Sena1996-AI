mod architecture;
mod memory;
mod performance;
mod reasoning;
mod security;

pub use architecture::{ArchitecturePattern, DesignPattern, SolidPrinciple};
pub use memory::{KnowledgeEntry, MemoryLevel, MemorySystem};
pub use performance::{ComplexityClass, OptimizationSuggestion, PerformancePattern};
pub use reasoning::{ReasoningFramework, ThinkingMode};
pub use security::{SecurityAudit, SecurityPattern, VulnerabilityType};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSystem {
    pub memory: MemorySystem,
    pub reasoning_frameworks: Vec<ReasoningFramework>,
    pub security_patterns: Vec<SecurityPattern>,
    pub performance_patterns: Vec<PerformancePattern>,
    pub architecture_patterns: Vec<ArchitecturePattern>,
    pub stats: KnowledgeStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub total_entries: usize,
    pub reasoning_count: usize,
    pub security_count: usize,
    pub performance_count: usize,
    pub architecture_count: usize,
    pub last_updated: Option<String>,
}

impl KnowledgeSystem {
    pub fn new() -> Self {
        let mut system = Self {
            memory: MemorySystem::new(),
            reasoning_frameworks: reasoning::default_frameworks(),
            security_patterns: security::default_patterns(),
            performance_patterns: performance::default_patterns(),
            architecture_patterns: architecture::default_patterns(),
            stats: KnowledgeStats::default(),
        };
        system.update_stats();
        system
    }

    pub fn update_stats(&mut self) {
        self.stats = KnowledgeStats {
            total_entries: self.memory.total_entries()
                + self.reasoning_frameworks.len()
                + self.security_patterns.len()
                + self.performance_patterns.len()
                + self.architecture_patterns.len(),
            reasoning_count: self.reasoning_frameworks.len(),
            security_count: self.security_patterns.len(),
            performance_count: self.performance_patterns.len(),
            architecture_count: self.architecture_patterns.len(),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
        };
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for framework in &self.reasoning_frameworks {
            if framework.name.to_lowercase().contains(&query_lower)
                || framework.description.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult {
                    domain: "reasoning".to_string(),
                    title: framework.name.clone(),
                    description: framework.description.clone(),
                    relevance: calculate_relevance(
                        &query_lower,
                        &framework.name,
                        &framework.description,
                    ),
                });
            }
        }

        for pattern in &self.security_patterns {
            if pattern.name.to_lowercase().contains(&query_lower)
                || pattern.description.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult {
                    domain: "security".to_string(),
                    title: pattern.name.clone(),
                    description: pattern.description.clone(),
                    relevance: calculate_relevance(
                        &query_lower,
                        &pattern.name,
                        &pattern.description,
                    ),
                });
            }
        }

        for pattern in &self.performance_patterns {
            if pattern.name.to_lowercase().contains(&query_lower)
                || pattern.description.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult {
                    domain: "performance".to_string(),
                    title: pattern.name.clone(),
                    description: pattern.description.clone(),
                    relevance: calculate_relevance(
                        &query_lower,
                        &pattern.name,
                        &pattern.description,
                    ),
                });
            }
        }

        for pattern in &self.architecture_patterns {
            if pattern.name.to_lowercase().contains(&query_lower)
                || pattern.description.to_lowercase().contains(&query_lower)
            {
                results.push(SearchResult {
                    domain: "architecture".to_string(),
                    title: pattern.name.clone(),
                    description: pattern.description.clone(),
                    relevance: calculate_relevance(
                        &query_lower,
                        &pattern.name,
                        &pattern.description,
                    ),
                });
            }
        }

        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn get_pattern(&self, domain: &str, name: &str) -> Option<String> {
        match domain {
            "reasoning" => self
                .reasoning_frameworks
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.to_string()),
            "security" => self
                .security_patterns
                .iter()
                .find(|p| p.name == name)
                .map(|p| p.to_string()),
            "performance" => self
                .performance_patterns
                .iter()
                .find(|p| p.name == name)
                .map(|p| p.to_string()),
            "architecture" => self
                .architecture_patterns
                .iter()
                .find(|p| p.name == name)
                .map(|p| p.to_string()),
            _ => None,
        }
    }

    pub fn get_domain_patterns(&self, domain: &str) -> Vec<String> {
        match domain {
            "reasoning" => self
                .reasoning_frameworks
                .iter()
                .map(|f| f.name.clone())
                .collect(),
            "security" => self
                .security_patterns
                .iter()
                .map(|p| p.name.clone())
                .collect(),
            "performance" => self
                .performance_patterns
                .iter()
                .map(|p| p.name.clone())
                .collect(),
            "architecture" => self
                .architecture_patterns
                .iter()
                .map(|p| p.name.clone())
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl Default for KnowledgeSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub domain: String,
    pub title: String,
    pub description: String,
    pub relevance: f64,
}

fn calculate_relevance(query: &str, title: &str, description: &str) -> f64 {
    let title_lower = title.to_lowercase();
    let desc_lower = description.to_lowercase();
    let mut score: f64 = 0.0;

    if title_lower == query {
        score += 1.0;
    } else if title_lower.contains(query) {
        score += 0.7;
    }

    if desc_lower.contains(query) {
        score += 0.3;
    }

    for word in query.split_whitespace() {
        if title_lower.contains(word) {
            score += 0.2;
        }
        if desc_lower.contains(word) {
            score += 0.1;
        }
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_system_creation() {
        let system = KnowledgeSystem::new();
        assert!(system.stats.total_entries > 0);
        assert!(system.reasoning_frameworks.len() > 0);
        assert!(system.security_patterns.len() > 0);
    }

    #[test]
    fn test_knowledge_search() {
        let system = KnowledgeSystem::new();
        let results = system.search("sql injection");
        assert!(results.len() > 0);
        assert_eq!(results[0].domain, "security");
    }

    #[test]
    fn test_get_domain_patterns() {
        let system = KnowledgeSystem::new();
        let patterns = system.get_domain_patterns("reasoning");
        assert!(patterns.len() > 0);
    }
}
