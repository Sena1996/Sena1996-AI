use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intent {
    CodeGeneration,
    CodeReview,
    CodeExplanation,
    Debugging,
    Refactoring,
    Testing,
    Documentation,
    Question,
    Conversation,
    Search,
    Summarization,
    Translation,
    Unknown,
}

impl Intent {
    pub fn requires_ai(&self) -> bool {
        !matches!(self, Intent::Search | Intent::Unknown)
    }

    pub fn suggested_model_tier(&self) -> ModelTier {
        match self {
            Intent::CodeGeneration | Intent::Refactoring => ModelTier::High,
            Intent::CodeReview | Intent::Debugging | Intent::Testing => ModelTier::High,
            Intent::CodeExplanation | Intent::Documentation => ModelTier::Medium,
            Intent::Question | Intent::Summarization => ModelTier::Medium,
            Intent::Conversation | Intent::Translation => ModelTier::Low,
            Intent::Search | Intent::Unknown => ModelTier::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    High,
    Medium,
    Low,
}

pub struct IntentDetector {
    patterns: HashMap<Intent, Vec<Regex>>,
    keyword_weights: HashMap<String, (Intent, f32)>,
}

impl IntentDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            patterns: HashMap::new(),
            keyword_weights: HashMap::new(),
        };
        detector.init_patterns();
        detector.init_keywords();
        detector
    }

    fn init_patterns(&mut self) {
        let pattern_map: Vec<(Intent, &[&str])> = vec![
            (Intent::CodeGeneration, &[
                r"(?i)\b(write|create|generate|implement|build|make|add)\b.*(function|class|method|code|script|program|module)",
                r"(?i)\b(can you|please|could you)\b.*(write|create|generate|implement)",
                r"(?i)\bnew\s+(function|class|component|module)\b",
            ]),
            (Intent::CodeReview, &[
                r"(?i)\b(review|check|analyze|audit|inspect)\b.*(code|implementation|solution)",
                r"(?i)\b(is this|does this|will this)\b.*(correct|work|efficient|good)",
                r"(?i)\b(what do you think|feedback|opinion)\b.*(code|implementation)",
            ]),
            (Intent::CodeExplanation, &[
                r"(?i)\b(explain|describe|what does|how does|what is|why does)\b",
                r"(?i)\b(walk me through|help me understand|clarify)\b",
                r"(?i)\b(meaning|purpose|reason)\b.*(code|function|method)",
            ]),
            (Intent::Debugging, &[
                r"(?i)\b(debug|fix|solve|resolve|troubleshoot)\b.*(error|bug|issue|problem)",
                r"(?i)\b(not working|doesn't work|broken|failing|crashed)\b",
                r"(?i)\b(why is|why does|why am i getting)\b.*(error|exception|fail)",
            ]),
            (Intent::Refactoring, &[
                r"(?i)\b(refactor|improve|optimize|clean up|simplify)\b.*(code|function|class)",
                r"(?i)\b(make.*better|more efficient|more readable)\b",
                r"(?i)\b(restructure|reorganize|rewrite)\b",
            ]),
            (Intent::Testing, &[
                r"(?i)\b(write|create|add|generate)\b.*(test|spec|unittest)",
                r"(?i)\b(test|testing|coverage)\b.*(function|class|module|code)",
                r"(?i)\b(how to test|testing strategy)\b",
            ]),
            (Intent::Documentation, &[
                r"(?i)\b(document|write docs|add comments|docstring)\b",
                r"(?i)\b(readme|documentation|api docs)\b",
                r"(?i)\b(explain.*for documentation|document this)\b",
            ]),
            (Intent::Question, &[
                r"(?i)^(what|how|why|when|where|who|which|can|could|would|should|is|are|do|does)\b",
                r"(?i)\?$",
            ]),
            (Intent::Conversation, &[
                r"(?i)^(hi|hello|hey|thanks|thank you|ok|okay|yes|no|sure)\b",
                r"(?i)\b(good morning|good afternoon|good evening)\b",
            ]),
            (Intent::Search, &[
                r"(?i)\b(find|search|look for|locate|where is)\b.*(file|function|class|variable)",
                r"(?i)\b(grep|search.*codebase|find.*in)\b",
            ]),
            (Intent::Summarization, &[
                r"(?i)\b(summarize|summary|tldr|brief|overview)\b",
                r"(?i)\b(in short|briefly|quick summary)\b",
            ]),
            (Intent::Translation, &[
                r"(?i)\b(translate|convert|port)\b.*(to|from|into)\b.*(python|javascript|rust|go|java)",
                r"(?i)\b(rewrite.*in|convert.*to)\b",
            ]),
        ];

        for (intent, patterns) in pattern_map {
            let compiled: Vec<Regex> = patterns
                .iter()
                .filter_map(|p| Regex::new(p).ok())
                .collect();
            self.patterns.insert(intent, compiled);
        }
    }

    fn init_keywords(&mut self) {
        let keywords: Vec<(&str, Intent, f32)> = vec![
            ("implement", Intent::CodeGeneration, 0.8),
            ("create", Intent::CodeGeneration, 0.7),
            ("write", Intent::CodeGeneration, 0.6),
            ("generate", Intent::CodeGeneration, 0.7),
            ("build", Intent::CodeGeneration, 0.6),
            ("review", Intent::CodeReview, 0.8),
            ("check", Intent::CodeReview, 0.5),
            ("analyze", Intent::CodeReview, 0.7),
            ("explain", Intent::CodeExplanation, 0.8),
            ("understand", Intent::CodeExplanation, 0.6),
            ("clarify", Intent::CodeExplanation, 0.7),
            ("debug", Intent::Debugging, 0.9),
            ("fix", Intent::Debugging, 0.8),
            ("error", Intent::Debugging, 0.7),
            ("bug", Intent::Debugging, 0.8),
            ("broken", Intent::Debugging, 0.7),
            ("refactor", Intent::Refactoring, 0.9),
            ("improve", Intent::Refactoring, 0.6),
            ("optimize", Intent::Refactoring, 0.7),
            ("test", Intent::Testing, 0.8),
            ("unittest", Intent::Testing, 0.9),
            ("spec", Intent::Testing, 0.7),
            ("document", Intent::Documentation, 0.8),
            ("docs", Intent::Documentation, 0.7),
            ("readme", Intent::Documentation, 0.8),
            ("find", Intent::Search, 0.6),
            ("search", Intent::Search, 0.7),
            ("locate", Intent::Search, 0.7),
            ("summarize", Intent::Summarization, 0.9),
            ("summary", Intent::Summarization, 0.8),
            ("translate", Intent::Translation, 0.9),
            ("convert", Intent::Translation, 0.7),
            ("port", Intent::Translation, 0.6),
        ];

        for (keyword, intent, weight) in keywords {
            self.keyword_weights.insert(keyword.to_string(), (intent, weight));
        }
    }

    pub fn detect(&self, text: &str) -> IntentResult {
        let text_lower = text.to_lowercase();
        let mut scores: HashMap<Intent, f32> = HashMap::new();

        for (intent, patterns) in &self.patterns {
            for pattern in patterns {
                if pattern.is_match(text) {
                    *scores.entry(*intent).or_insert(0.0) += 0.5;
                }
            }
        }

        for word in text_lower.split_whitespace() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if let Some((intent, weight)) = self.keyword_weights.get(clean_word) {
                *scores.entry(*intent).or_insert(0.0) += weight;
            }
        }

        let mut scored: Vec<(Intent, f32)> = scores.into_iter().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (primary_intent, confidence) = scored
            .first()
            .map(|(i, s)| (*i, (*s).min(1.0)))
            .unwrap_or((Intent::Unknown, 0.0));

        let secondary_intents: Vec<Intent> = scored
            .iter()
            .skip(1)
            .take(2)
            .filter(|(_, s)| *s > 0.3)
            .map(|(i, _)| *i)
            .collect();

        IntentResult {
            primary: primary_intent,
            confidence,
            secondary: secondary_intents,
            model_tier: primary_intent.suggested_model_tier(),
            requires_ai: primary_intent.requires_ai(),
        }
    }

    pub fn detect_batch(&self, texts: &[&str]) -> Vec<IntentResult> {
        texts.iter().map(|t| self.detect(t)).collect()
    }
}

impl Default for IntentDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IntentResult {
    pub primary: Intent,
    pub confidence: f32,
    pub secondary: Vec<Intent>,
    pub model_tier: ModelTier,
    pub requires_ai: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_generation_detection() {
        let detector = IntentDetector::new();

        let result = detector.detect("Write a function to calculate fibonacci");
        assert_eq!(result.primary, Intent::CodeGeneration);
        assert!(result.confidence > 0.5);

        let result = detector.detect("Create a class for user authentication");
        assert_eq!(result.primary, Intent::CodeGeneration);
    }

    #[test]
    fn test_debugging_detection() {
        let detector = IntentDetector::new();

        let result = detector.detect("Fix this error in my code");
        assert_eq!(result.primary, Intent::Debugging);

        let result = detector.detect("Debug why this function is not working and has bugs");
        assert_eq!(result.primary, Intent::Debugging);
    }

    #[test]
    fn test_explanation_detection() {
        let detector = IntentDetector::new();

        let result = detector.detect("Explain how this algorithm works");
        assert_eq!(result.primary, Intent::CodeExplanation);
    }

    #[test]
    fn test_model_tier_assignment() {
        let detector = IntentDetector::new();

        let result = detector.detect("Refactor this entire module");
        assert_eq!(result.model_tier, ModelTier::High);

        let result = detector.detect("Hi there!");
        assert!(matches!(result.model_tier, ModelTier::Low | ModelTier::Medium));
    }
}
