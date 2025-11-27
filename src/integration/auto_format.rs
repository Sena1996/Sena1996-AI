use crate::config::SenaConfig;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatType {
    BrilliantThinking,
    TruthVerification,
    CodeAnalysis,
    TableFormat,
    ProgressDisplay,
}

impl FormatType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BrilliantThinking => "brilliant_thinking",
            Self::TruthVerification => "truth_verification",
            Self::CodeAnalysis => "code_analysis",
            Self::TableFormat => "table_format",
            Self::ProgressDisplay => "progress",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInstructions {
    pub title: String,
    pub sections: Vec<String>,
    pub use_boxes: bool,
}

static BRILLIANT_THINKING_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\bwhy\s+(?:does|is|are|do|did|would|should)\b").expect("valid regex"),
        Regex::new(r"(?i)\bhow\s+(?:does|is|are|do|did|can|could|should)\b").expect("valid regex"),
        Regex::new(r"(?i)\bexplain\s+(?:why|how|what|the)\b").expect("valid regex"),
        Regex::new(r"(?i)\bwhat(?:'s|\s+is)\s+the\s+(?:reason|logic|rationale)\b")
            .expect("valid regex"),
    ]
});

static TRUTH_VERIFICATION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\bis\s+(?:it|this|that)\s+(?:true|false|correct|accurate|real|valid)\b")
            .expect("valid regex"),
        Regex::new(r"(?i)\bis\s+(?:the|a)\s+\w+\s+(?:flat|round|hollow|fake|real)\b")
            .expect("valid regex"),
        Regex::new(r"(?i)\bis\s+\w+\s+(?:true|false|correct|accurate|valid|real)\b")
            .expect("valid regex"),
        Regex::new(r"(?i)\b(?:fact\s+check|verify|confirm)\s+(?:that|if|whether)\b")
            .expect("valid regex"),
        Regex::new(r"(?i)\bmyth\s+(?:or|vs|versus)\s+(?:fact|reality|truth)\b")
            .expect("valid regex"),
    ]
});

static CODE_ANALYSIS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\b(?:analyze|review)\s+(?:this|the|my)?\s*code\b").expect("valid regex"),
        Regex::new(r"(?i)\bcode\s+(?:review|analysis|quality)\b").expect("valid regex"),
        Regex::new(r"(?i)\b(?:refactor|optimize|debug|fix)\s+(?:this|the|my)\b")
            .expect("valid regex"),
        Regex::new(r"(?i)\bcheck\s+(?:for|the)\s+(?:bugs|errors|issues)\b").expect("valid regex"),
    ]
});

static TABLE_FORMAT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\b(?:in|as|with)?\s*(?:a\s+)?table\b").expect("valid regex"),
        Regex::new(r"(?i)\btabular\s+(?:format|form|data)\b").expect("valid regex"),
        Regex::new(r"(?i)\b(?:show|display|present)\s+(?:in|as)\s+(?:table|grid)\b")
            .expect("valid regex"),
    ]
});

static PROGRESS_DISPLAY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\b(?:find|search|locate)\s+(?:all|the|files|in)\b").expect("valid regex"),
        Regex::new(r"(?i)\b(?:scan|process|analyze)\s+(?:multiple|all|the)\b")
            .expect("valid regex"),
        Regex::new(r"(?i)\b(?:read|check|examine)\s+(?:multiple|several|all)\b")
            .expect("valid regex"),
    ]
});

const BRILLIANT_THINKING_KEYWORDS: &[&str] = &[
    "why",
    "how",
    "explain",
    "reasoning",
    "understanding",
    "logic",
    "rationale",
    "because",
];
const CODE_ANALYSIS_KEYWORDS: &[&str] = &[
    "analyze", "review", "quality", "refactor", "optimize", "debug", "fix", "improve",
];
const TABLE_FORMAT_KEYWORDS: &[&str] = &["table", "tabular", "grid", "matrix", "columns", "rows"];
const PROGRESS_DISPLAY_KEYWORDS: &[&str] = &["find", "search", "locate", "scan", "process"];

pub struct AutoIntegration;

impl AutoIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_format(&self, user_input: &str) -> Option<FormatType> {
        let input_lower = user_input.to_lowercase();

        if self.matches_table(&input_lower) {
            return Some(FormatType::TableFormat);
        }

        if self.matches_code_analysis(&input_lower) {
            return Some(FormatType::CodeAnalysis);
        }

        if self.matches_brilliant_thinking(&input_lower) {
            return Some(FormatType::BrilliantThinking);
        }

        if self.matches_truth_verification(&input_lower) {
            return Some(FormatType::TruthVerification);
        }

        if self.matches_progress_display(&input_lower) {
            return Some(FormatType::ProgressDisplay);
        }

        None
    }

    fn matches_table(&self, input: &str) -> bool {
        TABLE_FORMAT_KEYWORDS.iter().any(|kw| input.contains(kw))
            || TABLE_FORMAT_PATTERNS.iter().any(|p| p.is_match(input))
    }

    fn matches_code_analysis(&self, input: &str) -> bool {
        CODE_ANALYSIS_KEYWORDS.iter().any(|kw| input.contains(kw))
            || CODE_ANALYSIS_PATTERNS.iter().any(|p| p.is_match(input))
    }

    fn matches_brilliant_thinking(&self, input: &str) -> bool {
        BRILLIANT_THINKING_KEYWORDS
            .iter()
            .any(|kw| input.contains(kw))
            || BRILLIANT_THINKING_PATTERNS
                .iter()
                .any(|p| p.is_match(input))
    }

    fn matches_truth_verification(&self, input: &str) -> bool {
        TRUTH_VERIFICATION_PATTERNS
            .iter()
            .any(|p| p.is_match(input))
    }

    fn matches_progress_display(&self, input: &str) -> bool {
        PROGRESS_DISPLAY_KEYWORDS
            .iter()
            .any(|kw| input.contains(kw))
            || PROGRESS_DISPLAY_PATTERNS.iter().any(|p| p.is_match(input))
    }

    pub fn should_show_progress(&self, operation_type: &str, step_count: usize) -> bool {
        const PROGRESS_OPERATIONS: &[&str] = &[
            "file_search",
            "multi_read",
            "code_analysis",
            "bulk_write",
            "multi_tool",
            "research",
        ];

        PROGRESS_OPERATIONS.contains(&operation_type) || step_count >= 2
    }

    pub fn get_format_instructions(&self, format_type: &FormatType) -> FormatInstructions {
        match format_type {
            FormatType::BrilliantThinking => FormatInstructions {
                title: SenaConfig::brand_title("BRILLIANT THINKING"),
                sections: vec![
                    "QUESTION ANALYSIS".to_string(),
                    "FIRST PRINCIPLES BREAKDOWN".to_string(),
                    "STRUCTURED ANALYSIS".to_string(),
                    "CONCLUSION".to_string(),
                ],
                use_boxes: true,
            },
            FormatType::TruthVerification => FormatInstructions {
                title: SenaConfig::brand_title("TRUTH VERIFICATION SYSTEM"),
                sections: vec![
                    "CLAIM BEING VERIFIED".to_string(),
                    "VERIFICATION ANALYSIS".to_string(),
                    "EVIDENCE".to_string(),
                    "FINAL VERDICT".to_string(),
                ],
                use_boxes: true,
            },
            FormatType::CodeAnalysis => FormatInstructions {
                title: SenaConfig::brand_title("CODE QUALITY ANALYSIS"),
                sections: vec![
                    "CODE OVERVIEW".to_string(),
                    "QUALITY METRICS".to_string(),
                    "ISSUES & RECOMMENDATIONS".to_string(),
                ],
                use_boxes: true,
            },
            FormatType::TableFormat => FormatInstructions {
                title: String::new(),
                sections: Vec::new(),
                use_boxes: false,
            },
            FormatType::ProgressDisplay => FormatInstructions {
                title: SenaConfig::brand_title("TASK PROGRESS"),
                sections: Vec::new(),
                use_boxes: true,
            },
        }
    }
}

impl Default for AutoIntegration {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_user_input(text: &str) -> Option<FormatType> {
    AutoIntegration::new().detect_format(text)
}

pub fn should_show_progress(op_type: &str, steps: usize) -> bool {
    AutoIntegration::new().should_show_progress(op_type, steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_integration_creation() {
        let _ai = AutoIntegration::new();
    }

    #[test]
    fn test_detect_table() {
        let ai = AutoIntegration::new();
        let format = ai.detect_format("show me a table of planets");
        assert_eq!(format, Some(FormatType::TableFormat));
    }

    #[test]
    fn test_detect_brilliant_thinking() {
        let ai = AutoIntegration::new();
        let format = ai.detect_format("why does the sky look blue?");
        assert_eq!(format, Some(FormatType::BrilliantThinking));
    }

    #[test]
    fn test_detect_truth_verification() {
        let ai = AutoIntegration::new();
        let format = ai.detect_format("is the earth flat?");
        assert_eq!(format, Some(FormatType::TruthVerification));
    }

    #[test]
    fn test_detect_code_analysis() {
        let ai = AutoIntegration::new();
        let format = ai.detect_format("analyze this code for bugs");
        assert_eq!(format, Some(FormatType::CodeAnalysis));
    }

    #[test]
    fn test_no_format_detected() {
        let ai = AutoIntegration::new();
        let format = ai.detect_format("hello world");
        assert_eq!(format, None);
    }

    #[test]
    fn test_should_show_progress() {
        let ai = AutoIntegration::new();
        assert!(ai.should_show_progress("file_search", 1));
        assert!(ai.should_show_progress("other", 2));
        assert!(!ai.should_show_progress("other", 1));
    }

    #[test]
    fn test_format_instructions() {
        let ai = AutoIntegration::new();
        let instructions = ai.get_format_instructions(&FormatType::BrilliantThinking);
        assert!(instructions.title.contains("BRILLIANT THINKING"));
        assert_eq!(instructions.sections.len(), 4);
    }
}
