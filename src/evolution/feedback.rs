use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedbackType {
    Positive,
    Negative,
    Neutral,
    Bug,
    FeatureRequest,
    Correction,
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackType::Positive => write!(f, "👍 Positive"),
            FeedbackType::Negative => write!(f, "👎 Negative"),
            FeedbackType::Neutral => write!(f, "➖ Neutral"),
            FeedbackType::Bug => write!(f, "🐛 Bug"),
            FeedbackType::FeatureRequest => write!(f, "✨ Feature Request"),
            FeedbackType::Correction => write!(f, "📝 Correction"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    pub id: String,
    pub feedback_type: FeedbackType,
    pub content: String,
    pub context: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub processed: bool,
    pub tags: Vec<String>,
}

impl FeedbackEntry {
    pub fn new(feedback_type: FeedbackType, content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            feedback_type,
            content: content.to_string(),
            context: None,
            timestamp: Utc::now(),
            processed: false,
            tags: Self::extract_tags(content),
        }
    }

    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    fn extract_tags(content: &str) -> Vec<String> {
        let keywords = [
            ("security", "security"),
            ("performance", "performance"),
            ("slow", "performance"),
            ("fast", "performance"),
            ("bug", "bug"),
            ("error", "bug"),
            ("feature", "feature"),
            ("ui", "ui"),
            ("format", "formatting"),
        ];

        let content_lower = content.to_lowercase();
        let mut tags: Vec<String> = keywords
            .iter()
            .filter(|(keyword, _)| content_lower.contains(keyword))
            .map(|(_, tag)| tag.to_string())
            .collect();

        tags.dedup();
        tags
    }

    pub fn mark_processed(&mut self) {
        self.processed = true;
    }
}

#[derive(Debug)]
pub struct FeedbackLoop {
    entries: Vec<FeedbackEntry>,
    type_counts: HashMap<FeedbackType, usize>,
    tag_counts: HashMap<String, usize>,
}

impl FeedbackLoop {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            type_counts: HashMap::new(),
            tag_counts: HashMap::new(),
        }
    }

    pub fn add(&mut self, feedback_type: FeedbackType, content: &str) {
        let entry = FeedbackEntry::new(feedback_type, content);

        *self.type_counts.entry(feedback_type).or_insert(0) += 1;

        for tag in &entry.tags {
            *self.tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }

        self.entries.push(entry);
    }

    pub fn add_with_context(&mut self, feedback_type: FeedbackType, content: &str, context: &str) {
        let entry = FeedbackEntry::new(feedback_type, content).with_context(context);

        *self.type_counts.entry(feedback_type).or_insert(0) += 1;

        for tag in &entry.tags {
            *self.tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }

        self.entries.push(entry);
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn unprocessed(&self) -> Vec<&FeedbackEntry> {
        self.entries.iter().filter(|e| !e.processed).collect()
    }

    pub fn analyze(&self) -> Vec<FeedbackInsight> {
        let mut insights = Vec::new();

        let positive = self.type_counts.get(&FeedbackType::Positive).unwrap_or(&0);
        let negative = self.type_counts.get(&FeedbackType::Negative).unwrap_or(&0);

        if *positive + *negative > 0 {
            let sentiment = *positive as f64 / (*positive + *negative) as f64;
            insights.push(FeedbackInsight {
                category: "Sentiment".to_string(),
                description: format!("{:.0}% positive feedback", sentiment * 100.0),
                action: if sentiment < 0.7 {
                    Some("Review negative feedback patterns".to_string())
                } else {
                    None
                },
                priority: if sentiment < 0.5 {
                    1
                } else if sentiment < 0.7 {
                    2
                } else {
                    3
                },
            });
        }

        if let Some(bug_count) = self.type_counts.get(&FeedbackType::Bug) {
            if *bug_count > 0 {
                insights.push(FeedbackInsight {
                    category: "Bugs".to_string(),
                    description: format!("{} bug reports received", bug_count),
                    action: Some("Address reported bugs".to_string()),
                    priority: 1,
                });
            }
        }

        if let Some(feature_count) = self.type_counts.get(&FeedbackType::FeatureRequest) {
            if *feature_count > 0 {
                insights.push(FeedbackInsight {
                    category: "Features".to_string(),
                    description: format!("{} feature requests", feature_count),
                    action: Some("Review and prioritize feature requests".to_string()),
                    priority: 2,
                });
            }
        }

        let performance_issues = self.tag_counts.get("performance").unwrap_or(&0);
        if *performance_issues > 2 {
            insights.push(FeedbackInsight {
                category: "Performance".to_string(),
                description: format!("{} performance-related feedback items", performance_issues),
                action: Some("Investigate performance issues".to_string()),
                priority: 1,
            });
        }

        insights.sort_by(|a, b| a.priority.cmp(&b.priority));

        insights
    }

    pub fn summary(&self) -> FeedbackSummary {
        FeedbackSummary {
            total: self.entries.len(),
            positive: *self.type_counts.get(&FeedbackType::Positive).unwrap_or(&0),
            negative: *self.type_counts.get(&FeedbackType::Negative).unwrap_or(&0),
            bugs: *self.type_counts.get(&FeedbackType::Bug).unwrap_or(&0),
            features: *self
                .type_counts
                .get(&FeedbackType::FeatureRequest)
                .unwrap_or(&0),
            unprocessed: self.unprocessed().len(),
        }
    }

    pub fn mark_all_processed(&mut self) {
        for entry in &mut self.entries {
            entry.mark_processed();
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| format!("Failed to serialize feedback: {}", e))?;

        std::fs::write(path, json).map_err(|e| format!("Failed to write feedback: {}", e))?;

        Ok(())
    }

    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read feedback: {}", e))?;

        let entries: Vec<FeedbackEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse feedback: {}", e))?;

        self.entries = entries;
        self.type_counts.clear();
        self.tag_counts.clear();

        for entry in &self.entries {
            *self.type_counts.entry(entry.feedback_type).or_insert(0) += 1;
            for tag in &entry.tags {
                *self.tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        Ok(())
    }
}

impl Default for FeedbackLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackInsight {
    pub category: String,
    pub description: String,
    pub action: Option<String>,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSummary {
    pub total: usize,
    pub positive: usize,
    pub negative: usize,
    pub bugs: usize,
    pub features: usize,
    pub unprocessed: usize,
}

impl FeedbackSummary {
    pub fn format(&self) -> String {
        format!(
            "Feedback Summary:\n\
            ├─ Total: {}\n\
            ├─ Positive: {} 👍\n\
            ├─ Negative: {} 👎\n\
            ├─ Bugs: {} 🐛\n\
            ├─ Features: {} ✨\n\
            └─ Unprocessed: {}",
            self.total, self.positive, self.negative, self.bugs, self.features, self.unprocessed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_loop_creation() {
        let loop_sys = FeedbackLoop::new();
        assert_eq!(loop_sys.count(), 0);
    }

    #[test]
    fn test_add_feedback() {
        let mut loop_sys = FeedbackLoop::new();
        loop_sys.add(FeedbackType::Positive, "Great response!");
        assert_eq!(loop_sys.count(), 1);
    }

    #[test]
    fn test_feedback_analysis() {
        let mut loop_sys = FeedbackLoop::new();
        loop_sys.add(FeedbackType::Positive, "Good!");
        loop_sys.add(FeedbackType::Positive, "Excellent!");
        loop_sys.add(FeedbackType::Negative, "Could be better");
        loop_sys.add(FeedbackType::Bug, "Found a bug in performance");

        let insights = loop_sys.analyze();
        assert!(insights.len() > 0);
    }

    #[test]
    fn test_summary() {
        let mut loop_sys = FeedbackLoop::new();
        loop_sys.add(FeedbackType::Positive, "Good!");
        loop_sys.add(FeedbackType::Bug, "Bug found");

        let summary = loop_sys.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.positive, 1);
        assert_eq!(summary.bugs, 1);
    }
}
