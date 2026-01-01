use sena_observe::Observer;
use sena_security::Sanitizer;
use std::sync::Arc;

use crate::tools::{self, Tool, ToolCall, ToolResult};

pub struct McpServer {
    observer: Arc<Observer>,
    sanitizer: Sanitizer,
}

impl McpServer {
    pub fn new(observer: Arc<Observer>) -> Self {
        Self {
            observer,
            sanitizer: Sanitizer::new(),
        }
    }

    pub fn list_tools(&self) -> Vec<Tool> {
        tools::all_tools()
    }

    pub async fn call_tool(&self, call: ToolCall) -> ToolResult {
        match call.name.as_str() {
            "sena_health" => self.handle_health().await,
            "sena_metrics" => self.handle_metrics().await,
            "sena_detect_format" => self.handle_detect_format(&call).await,
            "sena_validate" => self.handle_validate(&call).await,
            "sena_process" => self.handle_process(&call).await,
            "sena_format_table" => self.handle_format_table(&call).await,
            "sena_progress" => self.handle_progress(&call).await,
            _ => ToolResult::error(format!("Unknown tool: {}", call.name)),
        }
    }

    async fn handle_health(&self) -> ToolResult {
        let health = self.observer.health_check();
        ToolResult::text(serde_json::to_string_pretty(&health).unwrap_or_default())
    }

    async fn handle_metrics(&self) -> ToolResult {
        ToolResult::text(self.observer.metrics())
    }

    async fn handle_detect_format(&self, call: &ToolCall) -> ToolResult {
        let input = match call.arguments.get("input").and_then(|v| v.as_str()) {
            Some(i) => i,
            None => return ToolResult::error("Missing 'input' argument"),
        };

        let format = if input.trim().starts_with('{') || input.trim().starts_with('[') {
            "json"
        } else if input.contains("---") || input.lines().any(|l| l.contains(": ")) {
            "yaml"
        } else if input.trim().starts_with('<') {
            "xml"
        } else {
            "text"
        };

        ToolResult::text(format!("Detected format: {}", format))
    }

    async fn handle_validate(&self, call: &ToolCall) -> ToolResult {
        let input = match call.arguments.get("input").and_then(|v| v.as_str()) {
            Some(i) => i,
            None => return ToolResult::error("Missing 'input' argument"),
        };

        match self.sanitizer.sanitize_input(input) {
            Ok(sanitized) => ToolResult::text(format!("Valid input: {}", sanitized)),
            Err(e) => ToolResult::error(format!("Validation failed: {}", e)),
        }
    }

    async fn handle_process(&self, call: &ToolCall) -> ToolResult {
        let action = match call.arguments.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::error("Missing 'action' argument"),
        };

        let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::error("Missing 'text' argument"),
        };

        match action {
            "summarize" => {
                let summarizer = sena_local::TextRankSummarizer::new();
                match summarizer.summarize(text, 3) {
                    Ok(summary) => ToolResult::text(summary),
                    Err(e) => ToolResult::error(format!("Summarization failed: {}", e)),
                }
            }
            "keywords" => {
                let summarizer = sena_local::TextRankSummarizer::new();
                let keywords = summarizer.extract_keywords(text, 5);
                ToolResult::text(format!("Keywords: {}", keywords.join(", ")))
            }
            _ => ToolResult::error(format!("Unknown action: {}", action)),
        }
    }

    async fn handle_format_table(&self, call: &ToolCall) -> ToolResult {
        let headers: Vec<String> = match call.arguments.get("headers") {
            Some(h) => serde_json::from_value(h.clone()).unwrap_or_default(),
            None => return ToolResult::error("Missing 'headers' argument"),
        };

        let rows: Vec<Vec<String>> = match call.arguments.get("rows") {
            Some(r) => serde_json::from_value(r.clone()).unwrap_or_default(),
            None => return ToolResult::error("Missing 'rows' argument"),
        };

        let mut table = String::new();

        table.push_str("| ");
        table.push_str(&headers.join(" | "));
        table.push_str(" |\n");

        table.push_str("|");
        for _ in &headers {
            table.push_str(" --- |");
        }
        table.push('\n');

        for row in rows {
            table.push_str("| ");
            table.push_str(&row.join(" | "));
            table.push_str(" |\n");
        }

        ToolResult::text(table)
    }

    async fn handle_progress(&self, call: &ToolCall) -> ToolResult {
        let task = call
            .arguments
            .get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("Task");
        let current = call
            .arguments
            .get("current")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total = call
            .arguments
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);
        let message = call
            .arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let percentage = (current as f64 / total as f64 * 100.0) as u32;
        let bar_width = 20;
        let filled = (percentage as usize * bar_width / 100).min(bar_width);
        let empty = bar_width - filled;

        let progress_bar = format!(
            "[{}{}] {}% ({}/{})",
            "█".repeat(filled),
            "░".repeat(empty),
            percentage,
            current,
            total
        );

        let output = if message.is_empty() {
            format!("{}: {}", task, progress_bar)
        } else {
            format!("{}: {} - {}", task, progress_bar, message)
        };

        ToolResult::text(output)
    }
}
