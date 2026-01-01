use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ContentItem>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
}

impl ToolResult {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem::Text {
                text: content.into(),
            }],
            is_error: false,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem::Text {
                text: message.into(),
            }],
            is_error: true,
        }
    }
}

pub fn health_tool() -> Tool {
    Tool {
        name: "sena_health".to_string(),
        description: "Check SENA system health status".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

pub fn metrics_tool() -> Tool {
    Tool {
        name: "sena_metrics".to_string(),
        description: "Get SENA metrics in Prometheus format".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

pub fn detect_format_tool() -> Tool {
    Tool {
        name: "sena_detect_format".to_string(),
        description: "Detect input format (json, yaml, xml, text)".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input text to analyze"
                }
            },
            "required": ["input"]
        }),
    }
}

pub fn validate_tool() -> Tool {
    Tool {
        name: "sena_validate".to_string(),
        description: "Validate and sanitize user input".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input to validate"
                }
            },
            "required": ["input"]
        }),
    }
}

pub fn process_tool() -> Tool {
    Tool {
        name: "sena_process".to_string(),
        description: "Process text using local AI capabilities".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["summarize", "keywords", "embed"],
                    "description": "Processing action"
                },
                "text": {
                    "type": "string",
                    "description": "Text to process"
                },
                "options": {
                    "type": "object",
                    "description": "Optional parameters"
                }
            },
            "required": ["action", "text"]
        }),
    }
}

pub fn format_table_tool() -> Tool {
    Tool {
        name: "sena_format_table".to_string(),
        description: "Format data as a markdown table".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "headers": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Column headers"
                },
                "rows": {
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": {"type": "string"}
                    },
                    "description": "Table rows"
                }
            },
            "required": ["headers", "rows"]
        }),
    }
}

pub fn progress_tool() -> Tool {
    Tool {
        name: "sena_progress".to_string(),
        description: "Report task progress".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Task description"
                },
                "current": {
                    "type": "integer",
                    "description": "Current step"
                },
                "total": {
                    "type": "integer",
                    "description": "Total steps"
                },
                "message": {
                    "type": "string",
                    "description": "Status message"
                }
            },
            "required": ["task", "current", "total"]
        }),
    }
}

pub fn all_tools() -> Vec<Tool> {
    vec![
        health_tool(),
        metrics_tool(),
        detect_format_tool(),
        validate_tool(),
        process_tool(),
        format_table_tool(),
        progress_tool(),
    ]
}
