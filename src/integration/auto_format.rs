//! SENA Auto Integration System
//! Automatic keyword detection and format application

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Format type for SENA responses
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatType {
    BrilliantThinking,
    TruthVerification,
    CodeAnalysis,
    TableFormat,
    ProgressDisplay,
}

impl FormatType {
    /// Get format name
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

/// Trigger configuration
#[derive(Debug, Clone)]
struct TriggerConfig {
    keywords: Vec<&'static str>,
    patterns: Vec<Regex>,
    format: FormatType,
}

/// Format instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInstructions {
    pub title: String,
    pub sections: Vec<String>,
    pub use_boxes: bool,
}

/// Auto Integration for SENA format detection
pub struct AutoIntegration {
    triggers: HashMap<&'static str, TriggerConfig>,
}

impl AutoIntegration {
    /// Create new auto integration instance
    pub fn new() -> Self {
        let mut triggers = HashMap::new();

        // Brilliant Thinking triggers
        triggers.insert(
            "brilliant_thinking",
            TriggerConfig {
                keywords: vec!["why", "how", "explain", "reasoning", "understanding", "logic", "rationale", "because"],
                patterns: vec![
                    Regex::new(r"(?i)\bwhy\s+(?:does|is|are|do|did|would|should)\b").unwrap(),
                    Regex::new(r"(?i)\bhow\s+(?:does|is|are|do|did|can|could|should)\b").unwrap(),
                    Regex::new(r"(?i)\bexplain\s+(?:why|how|what|the)\b").unwrap(),
                    Regex::new(r"(?i)\bwhat(?:'s|\s+is)\s+the\s+(?:reason|logic|rationale)\b").unwrap(),
                ],
                format: FormatType::BrilliantThinking,
            },
        );

        // Truth Verification triggers
        triggers.insert(
            "truth_verification",
            TriggerConfig {
                keywords: vec![],
                patterns: vec![
                    Regex::new(r"(?i)\bis\s+(?:it|this|that)\s+(?:true|false|correct|accurate|real|valid)\b").unwrap(),
                    Regex::new(r"(?i)\bis\s+(?:the|a)\s+\w+\s+(?:flat|round|hollow|fake|real)\b").unwrap(),
                    Regex::new(r"(?i)\bis\s+\w+\s+(?:true|false|correct|accurate|valid|real)\b").unwrap(),
                    Regex::new(r"(?i)\b(?:fact\s+check|verify|confirm)\s+(?:that|if|whether)\b").unwrap(),
                    Regex::new(r"(?i)\bmyth\s+(?:or|vs|versus)\s+(?:fact|reality|truth)\b").unwrap(),
                ],
                format: FormatType::TruthVerification,
            },
        );

        // Code Analysis triggers
        triggers.insert(
            "code_analysis",
            TriggerConfig {
                keywords: vec!["analyze", "review", "quality", "refactor", "optimize", "debug", "fix", "improve"],
                patterns: vec![
                    Regex::new(r"(?i)\b(?:analyze|review)\s+(?:this|the|my)?\s*code\b").unwrap(),
                    Regex::new(r"(?i)\bcode\s+(?:review|analysis|quality)\b").unwrap(),
                    Regex::new(r"(?i)\b(?:refactor|optimize|debug|fix)\s+(?:this|the|my)\b").unwrap(),
                    Regex::new(r"(?i)\bcheck\s+(?:for|the)\s+(?:bugs|errors|issues)\b").unwrap(),
                ],
                format: FormatType::CodeAnalysis,
            },
        );

        // Table Format triggers
        triggers.insert(
            "table_format",
            TriggerConfig {
                keywords: vec!["table", "tabular", "grid", "matrix", "columns", "rows"],
                patterns: vec![
                    Regex::new(r"(?i)\b(?:in|as|with)?\s*(?:a\s+)?table\b").unwrap(),
                    Regex::new(r"(?i)\btabular\s+(?:format|form|data)\b").unwrap(),
                    Regex::new(r"(?i)\b(?:show|display|present)\s+(?:in|as)\s+(?:table|grid)\b").unwrap(),
                ],
                format: FormatType::TableFormat,
            },
        );

        // Progress Display triggers
        triggers.insert(
            "progress_display",
            TriggerConfig {
                keywords: vec!["find", "search", "locate", "scan", "process"],
                patterns: vec![
                    Regex::new(r"(?i)\b(?:find|search|locate)\s+(?:all|the|files|in)\b").unwrap(),
                    Regex::new(r"(?i)\b(?:scan|process|analyze)\s+(?:multiple|all|the)\b").unwrap(),
                    Regex::new(r"(?i)\b(?:read|check|examine)\s+(?:multiple|several|all)\b").unwrap(),
                ],
                format: FormatType::ProgressDisplay,
            },
        );

        Self { triggers }
    }

    /// Detect format from user input
    pub fn detect_format(&self, user_input: &str) -> Option<FormatType> {
        let input_lower = user_input.to_lowercase();

        // Process in priority order
        let priority_order = [
            "table_format",
            "code_analysis",
            "brilliant_thinking",
            "truth_verification",
            "progress_display",
        ];

        for trigger_type in priority_order {
            if let Some(config) = self.triggers.get(trigger_type) {
                // Check keywords
                for keyword in &config.keywords {
                    if input_lower.contains(keyword) {
                        return Some(config.format.clone());
                    }
                }

                // Check patterns
                for pattern in &config.patterns {
                    if pattern.is_match(&input_lower) {
                        return Some(config.format.clone());
                    }
                }
            }
        }

        None
    }

    /// Check if progress bars should be shown
    pub fn should_show_progress(&self, operation_type: &str, step_count: usize) -> bool {
        let progress_operations = [
            "file_search",
            "multi_read",
            "code_analysis",
            "bulk_write",
            "multi_tool",
            "research",
        ];

        progress_operations.contains(&operation_type) || step_count >= 2
    }

    /// Get format instructions
    pub fn get_format_instructions(&self, format_type: &FormatType) -> FormatInstructions {
        match format_type {
            FormatType::BrilliantThinking => FormatInstructions {
                title: "SENA 🦁 BRILLIANT THINKING".to_string(),
                sections: vec![
                    "QUESTION ANALYSIS".to_string(),
                    "FIRST PRINCIPLES BREAKDOWN".to_string(),
                    "STRUCTURED ANALYSIS".to_string(),
                    "CONCLUSION".to_string(),
                ],
                use_boxes: true,
            },
            FormatType::TruthVerification => FormatInstructions {
                title: "SENA 🦁 TRUTH VERIFICATION SYSTEM".to_string(),
                sections: vec![
                    "CLAIM BEING VERIFIED".to_string(),
                    "VERIFICATION ANALYSIS".to_string(),
                    "EVIDENCE".to_string(),
                    "FINAL VERDICT".to_string(),
                ],
                use_boxes: true,
            },
            FormatType::CodeAnalysis => FormatInstructions {
                title: "SENA 🦁 CODE QUALITY ANALYSIS".to_string(),
                sections: vec![
                    "CODE OVERVIEW".to_string(),
                    "QUALITY METRICS".to_string(),
                    "ISSUES & RECOMMENDATIONS".to_string(),
                ],
                use_boxes: true,
            },
            FormatType::TableFormat => FormatInstructions {
                title: "".to_string(),
                sections: vec![],
                use_boxes: false,
            },
            FormatType::ProgressDisplay => FormatInstructions {
                title: "SENA 🦁 TASK PROGRESS".to_string(),
                sections: vec![],
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

/// Helper function to check user input
pub fn check_user_input(text: &str) -> Option<FormatType> {
    AutoIntegration::new().detect_format(text)
}

/// Helper function to check if progress should be shown
pub fn should_show_progress(op_type: &str, steps: usize) -> bool {
    AutoIntegration::new().should_show_progress(op_type, steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_integration_creation() {
        let ai = AutoIntegration::new();
        assert!(!ai.triggers.is_empty());
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
