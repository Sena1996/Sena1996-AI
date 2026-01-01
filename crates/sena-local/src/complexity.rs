use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplexityLevel {
    Trivial,
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

impl ComplexityLevel {
    pub fn to_score(&self) -> f32 {
        match self {
            Self::Trivial => 0.1,
            Self::Simple => 0.3,
            Self::Moderate => 0.5,
            Self::Complex => 0.7,
            Self::VeryComplex => 0.9,
        }
    }

    pub fn from_score(score: f32) -> Self {
        match score {
            s if s < 0.2 => Self::Trivial,
            s if s < 0.4 => Self::Simple,
            s if s < 0.6 => Self::Moderate,
            s if s < 0.8 => Self::Complex,
            _ => Self::VeryComplex,
        }
    }

    pub fn recommended_model(&self) -> &'static str {
        match self {
            Self::Trivial | Self::Simple => "gpt-4o-mini",
            Self::Moderate => "gpt-4o",
            Self::Complex => "claude-sonnet-4-20250514",
            Self::VeryComplex => "claude-opus-4-20250514",
        }
    }

    pub fn max_tokens(&self) -> u32 {
        match self {
            Self::Trivial => 256,
            Self::Simple => 512,
            Self::Moderate => 1024,
            Self::Complex => 2048,
            Self::VeryComplex => 4096,
        }
    }
}

pub struct ComplexityScorer {
    technical_terms: HashSet<String>,
    complexity_patterns: Vec<(Regex, f32)>,
}

impl ComplexityScorer {
    pub fn new() -> Self {
        let mut scorer = Self {
            technical_terms: HashSet::new(),
            complexity_patterns: Vec::new(),
        };
        scorer.init_technical_terms();
        scorer.init_patterns();
        scorer
    }

    fn init_technical_terms(&mut self) {
        let terms = [
            "algorithm", "optimization", "concurrency", "parallelism", "async",
            "distributed", "microservice", "architecture", "design pattern",
            "polymorphism", "inheritance", "abstraction", "encapsulation",
            "recursion", "dynamic programming", "graph", "tree", "hash",
            "cryptography", "encryption", "authentication", "authorization",
            "database", "query", "index", "transaction", "acid",
            "api", "rest", "graphql", "grpc", "websocket",
            "kubernetes", "docker", "container", "orchestration",
            "machine learning", "neural network", "deep learning", "transformer",
            "compiler", "parser", "lexer", "ast", "bytecode",
            "memory management", "garbage collection", "pointer", "reference",
            "thread", "mutex", "semaphore", "deadlock", "race condition",
            "cache", "memoization", "lazy evaluation", "streaming",
            "regex", "pattern matching", "finite automata",
            "security", "vulnerability", "exploit", "sanitization",
            "performance", "profiling", "benchmark", "latency", "throughput",
        ];

        for term in terms {
            self.technical_terms.insert(term.to_string());
        }
    }

    fn init_patterns(&mut self) {
        let patterns: Vec<(&str, f32)> = vec![
            (r"(?i)\b(implement|build|create)\b.*(system|architecture|framework)", 0.3),
            (r"(?i)\b(optimize|performance|efficient)\b", 0.2),
            (r"(?i)\b(secure|security|encrypt|auth)\b", 0.25),
            (r"(?i)\b(distributed|concurrent|parallel|async)\b", 0.3),
            (r"(?i)\b(multiple|several|many)\b.*(files?|components?|modules?)", 0.2),
            (r"(?i)\b(integration|integrate|connect)\b.*(api|service|system)", 0.2),
            (r"(?i)\b(database|sql|query|schema)\b", 0.15),
            (r"(?i)\b(test|testing|coverage|tdd)\b", 0.1),
            (r"(?i)\b(refactor|rewrite|restructure)\b.*(entire|whole|complete)", 0.25),
            (r"(?i)\b(complex|complicated|intricate|sophisticated)\b", 0.15),
            (r"(?i)\b(edge case|corner case|error handling)\b", 0.15),
            (r"(?i)\b(backward.?compatible|migration|upgrade)\b", 0.2),
            (r"(?i)\b(real.?time|streaming|live)\b", 0.2),
            (r"(?i)\b(scale|scaling|scalable|load)\b", 0.2),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.complexity_patterns.push((regex, weight));
            }
        }
    }

    pub fn score(&self, text: &str) -> ComplexityResult {
        let mut total_score = 0.0;
        let mut factors = Vec::new();

        let length_score = self.score_length(text);
        if length_score > 0.0 {
            total_score += length_score;
            factors.push(("length".to_string(), length_score));
        }

        let term_score = self.score_technical_terms(text);
        if term_score > 0.0 {
            total_score += term_score;
            factors.push(("technical_terms".to_string(), term_score));
        }

        let pattern_score = self.score_patterns(text);
        if pattern_score > 0.0 {
            total_score += pattern_score;
            factors.push(("complexity_patterns".to_string(), pattern_score));
        }

        let structure_score = self.score_structure(text);
        if structure_score > 0.0 {
            total_score += structure_score;
            factors.push(("structure".to_string(), structure_score));
        }

        let code_score = self.score_code_blocks(text);
        if code_score > 0.0 {
            total_score += code_score;
            factors.push(("code_blocks".to_string(), code_score));
        }

        let normalized_score = (total_score / 2.0).min(1.0);
        let level = ComplexityLevel::from_score(normalized_score);

        ComplexityResult {
            score: normalized_score,
            level,
            factors,
            recommended_model: level.recommended_model().to_string(),
            suggested_max_tokens: level.max_tokens(),
        }
    }

    fn score_length(&self, text: &str) -> f32 {
        let word_count = text.split_whitespace().count();
        match word_count {
            0..=10 => 0.0,
            11..=50 => 0.1,
            51..=150 => 0.2,
            151..=300 => 0.3,
            _ => 0.4,
        }
    }

    fn score_technical_terms(&self, text: &str) -> f32 {
        let text_lower = text.to_lowercase();
        let mut found_terms = 0;

        for term in &self.technical_terms {
            if text_lower.contains(term) {
                found_terms += 1;
            }
        }

        match found_terms {
            0 => 0.0,
            1..=2 => 0.1,
            3..=5 => 0.2,
            6..=10 => 0.3,
            _ => 0.4,
        }
    }

    fn score_patterns(&self, text: &str) -> f32 {
        let mut score: f32 = 0.0;

        for (pattern, weight) in &self.complexity_patterns {
            if pattern.is_match(text) {
                score += weight;
            }
        }

        score.min(0.5)
    }

    fn score_structure(&self, text: &str) -> f32 {
        let mut score = 0.0;

        let bullet_count = text.matches(&['-', '*', '•'][..]).count();
        if bullet_count > 3 {
            score += 0.1;
        }

        let numbered_items = Regex::new(r"^\d+\.\s").ok()
            .map(|r| r.find_iter(text).count())
            .unwrap_or(0);
        if numbered_items > 3 {
            score += 0.1;
        }

        if text.contains("```") || text.contains("   ") {
            score += 0.1;
        }

        score
    }

    fn score_code_blocks(&self, text: &str) -> f32 {
        let code_block_count = text.matches("```").count() / 2;
        let inline_code_count = text.matches('`').count() / 2;

        let mut score = 0.0;

        if code_block_count > 0 {
            score += 0.1 * (code_block_count as f32).min(3.0);
        }

        if inline_code_count > 5 {
            score += 0.1;
        }

        score.min(0.4)
    }

    pub fn score_batch(&self, texts: &[&str]) -> Vec<ComplexityResult> {
        texts.iter().map(|t| self.score(t)).collect()
    }

    pub fn should_use_premium_model(&self, text: &str) -> bool {
        let result = self.score(text);
        matches!(result.level, ComplexityLevel::Complex | ComplexityLevel::VeryComplex)
    }
}

impl Default for ComplexityScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ComplexityResult {
    pub score: f32,
    pub level: ComplexityLevel,
    pub factors: Vec<(String, f32)>,
    pub recommended_model: String,
    pub suggested_max_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let scorer = ComplexityScorer::new();
        let result = scorer.score("What is a variable?");
        assert!(result.level <= ComplexityLevel::Simple);
    }

    #[test]
    fn test_complex_query() {
        let scorer = ComplexityScorer::new();
        let result = scorer.score(
            "Implement a distributed caching system with consistent hashing, \
             replication across multiple nodes, automatic failover, \
             and support for concurrent read/write operations with ACID guarantees."
        );
        assert!(result.level >= ComplexityLevel::Moderate, "Expected at least Moderate, got {:?}", result.level);
    }

    #[test]
    fn test_model_recommendation() {
        let scorer = ComplexityScorer::new();

        let simple = scorer.score("Hello world");
        assert_eq!(simple.recommended_model, "gpt-4o-mini");

        let complex = scorer.score(
            "Design a microservice architecture with kubernetes orchestration, \
             distributed tracing, circuit breakers, and zero-downtime deployments."
        );
        assert!(complex.recommended_model.contains("claude") || complex.recommended_model.contains("gpt-4o"));
    }

    #[test]
    fn test_technical_terms_detection() {
        let scorer = ComplexityScorer::new();
        let result = scorer.score(
            "Implement polymorphism with inheritance and encapsulation using design patterns."
        );
        assert!(result.factors.iter().any(|(name, _)| name == "technical_terms"));
    }
}
