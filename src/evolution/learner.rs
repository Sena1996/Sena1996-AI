use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    ProblemSolution,
    QueryResponse,
    ContextAction,
    ErrorFix,
    Optimization,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::ProblemSolution => write!(f, "Problem-Solution"),
            PatternType::QueryResponse => write!(f, "Query-Response"),
            PatternType::ContextAction => write!(f, "Context-Action"),
            PatternType::ErrorFix => write!(f, "Error-Fix"),
            PatternType::Optimization => write!(f, "Optimization"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub context: String,
    pub outcome: String,
    pub keywords: Vec<String>,
    pub usage_count: u64,
    pub success_rate: f64,
    pub learned_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl LearnedPattern {
    pub fn new(context: &str, outcome: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pattern_type: Self::detect_type(context),
            context: context.to_string(),
            outcome: outcome.to_string(),
            keywords: Self::extract_keywords(context),
            usage_count: 0,
            success_rate: 1.0,
            learned_at: Utc::now(),
            last_used: None,
        }
    }

    fn detect_type(context: &str) -> PatternType {
        let context_lower = context.to_lowercase();

        if context_lower.contains("error") || context_lower.contains("fix") || context_lower.contains("bug") {
            PatternType::ErrorFix
        } else if context_lower.contains("optimize") || context_lower.contains("performance") || context_lower.contains("improve") {
            PatternType::Optimization
        } else if context_lower.contains("how") || context_lower.contains("why") || context_lower.contains("what") {
            PatternType::QueryResponse
        } else if context_lower.contains("problem") || context_lower.contains("issue") || context_lower.contains("solve") {
            PatternType::ProblemSolution
        } else {
            PatternType::ContextAction
        }
    }

    fn extract_keywords(context: &str) -> Vec<String> {
        let stop_words = ["the", "a", "an", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would",
            "could", "should", "may", "might", "must", "shall", "can", "need",
            "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
            "into", "through", "during", "before", "after", "above", "below",
            "between", "under", "again", "further", "then", "once", "here",
            "there", "when", "where", "why", "how", "all", "each", "few",
            "more", "most", "other", "some", "such", "no", "nor", "not",
            "only", "own", "same", "so", "than", "too", "very", "just"];

        context.split_whitespace()
            .map(|w| w.to_lowercase())
            .filter(|w| w.len() > 2)
            .filter(|w| !stop_words.contains(&w.as_str()))
            .take(10)
            .collect()
    }

    pub fn record_usage(&mut self, success: bool) {
        self.usage_count += 1;
        self.last_used = Some(Utc::now());

        let alpha = 0.2;
        let success_val = if success { 1.0 } else { 0.0 };
        self.success_rate = alpha * success_val + (1.0 - alpha) * self.success_rate;
    }

    pub fn relevance(&self, context: &str) -> f64 {
        let context_lower = context.to_lowercase();
        let mut score = 0.0;

        for keyword in &self.keywords {
            if context_lower.contains(keyword) {
                score += 0.2;
            }
        }

        let detected_type = Self::detect_type(context);
        if detected_type == self.pattern_type {
            score += 0.3;
        }

        score *= self.success_rate;
        score += (self.usage_count as f64 / 100.0).min(0.2);

        score.min(1.0)
    }
}

#[derive(Debug)]
pub struct PatternLearner {
    patterns: HashMap<String, LearnedPattern>,
    relevance_threshold: f64,
}

impl PatternLearner {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            relevance_threshold: 0.3,
        }
    }

    pub fn learn(&mut self, context: &str, outcome: &str) {
        let pattern = LearnedPattern::new(context, outcome);

        if let Some(similar) = self.find_similar(context) {
            let existing = self.patterns.get_mut(&similar).unwrap();
            existing.record_usage(true);
            return;
        }

        self.patterns.insert(pattern.id.clone(), pattern);
    }

    fn find_similar(&self, context: &str) -> Option<String> {
        for (id, pattern) in &self.patterns {
            if pattern.relevance(context) > 0.7 {
                return Some(id.clone());
            }
        }
        None
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn find_relevant(&self, context: &str) -> Vec<&LearnedPattern> {
        let mut relevant: Vec<_> = self.patterns.values()
            .filter(|p| p.relevance(context) >= self.relevance_threshold)
            .collect();

        relevant.sort_by(|a, b| {
            b.relevance(context).partial_cmp(&a.relevance(context))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        relevant.into_iter().take(5).collect()
    }

    pub fn apply_learnings(&mut self) -> usize {
        let mut count = 0;
        for pattern in self.patterns.values_mut() {
            if pattern.success_rate > 0.7 && pattern.usage_count > 0 {
                count += 1;
            }
        }
        count
    }

    pub fn prune_to(&mut self, max_patterns: usize) {
        if self.patterns.len() <= max_patterns {
            return;
        }

        let mut scores: Vec<(String, f64)> = self.patterns.iter()
            .map(|(id, p)| {
                let value = p.success_rate * (p.usage_count as f64 + 1.0).ln();
                (id.clone(), value)
            })
            .collect();

        scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let to_remove = self.patterns.len() - max_patterns;
        for (id, _) in scores.into_iter().take(to_remove) {
            self.patterns.remove(&id);
        }
    }

    pub fn memory_usage(&self) -> usize {
        self.patterns.len() * 500
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let patterns: Vec<&LearnedPattern> = self.patterns.values().collect();
        let json = serde_json::to_string_pretty(&patterns)
            .map_err(|e| format!("Failed to serialize patterns: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write patterns: {}", e))?;

        Ok(())
    }

    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read patterns: {}", e))?;

        let patterns: Vec<LearnedPattern> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse patterns: {}", e))?;

        for pattern in patterns {
            self.patterns.insert(pattern.id.clone(), pattern);
        }

        Ok(())
    }
}

impl Default for PatternLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let pattern = LearnedPattern::new("How to prevent SQL injection?", "Use parameterized queries");
        assert_eq!(pattern.pattern_type, PatternType::QueryResponse);
        assert!(pattern.keywords.len() > 0);
    }

    #[test]
    fn test_learner_creation() {
        let learner = PatternLearner::new();
        assert_eq!(learner.pattern_count(), 0);
    }

    #[test]
    fn test_learning() {
        let mut learner = PatternLearner::new();
        learner.learn("How to optimize query?", "Use indexes");
        assert_eq!(learner.pattern_count(), 1);
    }

    #[test]
    fn test_relevance() {
        let mut learner = PatternLearner::new();
        learner.learn("How to make database queries faster", "Use indexes and batch queries");

        let relevant = learner.find_relevant("How to make my database faster");
        assert!(relevant.len() > 0);
    }
}
