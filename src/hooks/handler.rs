//! Hook Handlers
//!
//! Handles Claude Code hooks for SENA integration

use crate::cli::args::HookType;
use crate::integration::AutoIntegration;
use crate::ancient::HarmonyValidationEngine;
use serde::{Deserialize, Serialize};

/// Hook processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub should_block: bool,
}

impl HookResult {
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
            should_block: false,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn block(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
            should_block: true,
        }
    }
}

/// Handle incoming hook
pub async fn handle_hook(hook_type: HookType, input: &str) -> Result<HookResult, String> {
    match hook_type {
        HookType::UserPromptSubmit => handle_pre_prompt(input).await,
        HookType::AssistantResponse => handle_post_response(input).await,
        HookType::ToolExecution => handle_pre_tool(input).await,
        HookType::PreValidation => handle_pre_validation(input).await,
        HookType::PostValidation => handle_post_validation(input).await,
    }
}

/// Pre-prompt hook - analyze user input before Claude processes
async fn handle_pre_prompt(input: &str) -> Result<HookResult, String> {
    let ai = AutoIntegration::new();

    // Detect format requirements
    let detected = ai.detect_format(input);

    // Check for SENA trigger keywords
    let triggers = detect_triggers(input);

    let result = HookResult::success("Pre-prompt analysis complete")
        .with_data(serde_json::json!({
            "detected_format": detected.as_ref().map(|f| f.name()),
            "triggers": triggers,
            "input_length": input.len(),
            "word_count": input.split_whitespace().count(),
        }));

    Ok(result)
}

/// Post-response hook - validate Claude's response
async fn handle_post_response(input: &str) -> Result<HookResult, String> {
    // Parse the response input (JSON with conversation context)
    let response_data: Result<serde_json::Value, _> = serde_json::from_str(input);

    let response_text = match &response_data {
        Ok(v) => v.get("response").and_then(|r| r.as_str()).unwrap_or(input),
        Err(_) => input,
    };

    // Validate response with harmony engine
    let mut engine = HarmonyValidationEngine::new();
    let validation = engine.validate(response_text);

    // Check for SENA format compliance
    let has_sena_format = check_sena_format_compliance(response_text);

    let result = HookResult::success("Response validation complete")
        .with_data(serde_json::json!({
            "valid": validation.is_valid(),
            "confidence": validation.overall_confidence,
            "sena_compliant": has_sena_format,
            "response_length": response_text.len(),
        }));

    Ok(result)
}

/// Pre-tool hook - validate tool calls before execution
async fn handle_pre_tool(input: &str) -> Result<HookResult, String> {
    let tool_data: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| format!("Invalid tool data: {}", e))?;

    let tool_name = tool_data.get("tool")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    // Check for potentially dangerous tools
    let dangerous_tools = ["Bash", "Write", "Edit"];
    let is_dangerous = dangerous_tools.contains(&tool_name);

    // Get tool arguments for analysis
    let args = tool_data.get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let result = HookResult::success("Tool validation complete")
        .with_data(serde_json::json!({
            "tool": tool_name,
            "is_dangerous": is_dangerous,
            "arguments": args,
            "approved": true,
        }));

    Ok(result)
}

/// Post-tool hook - process tool results
#[allow(dead_code)]
async fn handle_post_tool(input: &str) -> Result<HookResult, String> {
    let tool_result: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| format!("Invalid tool result: {}", e))?;

    let tool_name = tool_result.get("tool")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    let success = tool_result.get("success")
        .and_then(|s| s.as_bool())
        .unwrap_or(true);

    let result = HookResult::success("Tool result processed")
        .with_data(serde_json::json!({
            "tool": tool_name,
            "success": success,
            "processed": true,
        }));

    Ok(result)
}

/// Pre-validation hook - validate before final processing
async fn handle_pre_validation(input: &str) -> Result<HookResult, String> {
    let mut engine = HarmonyValidationEngine::new();
    let validation = engine.validate(input);

    let result = HookResult::success("Pre-validation complete")
        .with_data(serde_json::json!({
            "valid": validation.is_valid(),
            "confidence": validation.overall_confidence,
            "status": format!("{:?}", validation.overall_status),
        }));

    Ok(result)
}

/// Post-validation hook - finalize validation results
async fn handle_post_validation(input: &str) -> Result<HookResult, String> {
    let mut engine = HarmonyValidationEngine::new();
    let validation = engine.validate(input);

    let result = HookResult::success("Post-validation complete")
        .with_data(serde_json::json!({
            "valid": validation.is_valid(),
            "confidence": validation.overall_confidence,
            "violations": validation.rule_violations.len(),
            "suggestions": validation.corrections_suggested,
        }));

    Ok(result)
}

/// Detect SENA trigger keywords in text
fn detect_triggers(text: &str) -> Vec<String> {
    let mut triggers = Vec::new();
    let lower = text.to_lowercase();

    // Trigger keywords for different formats
    let trigger_map = [
        (vec!["table", "tabular", "in table form"], "TABLE_FORMAT"),
        (vec!["why", "how", "explain"], "BRILLIANT_THINKING"),
        (vec!["is it true", "fact check", "verify", "true or false"], "TRUTH_VERIFICATION"),
        (vec!["analyze code", "code review", "review this"], "CODE_ANALYSIS"),
        (vec!["progress", "status", "show tasks"], "PROGRESS_BAR"),
    ];

    for (keywords, trigger_name) in trigger_map {
        for keyword in keywords {
            if lower.contains(keyword) {
                triggers.push(trigger_name.to_string());
                break;
            }
        }
    }

    triggers
}

/// Check if response has proper SENA formatting
fn check_sena_format_compliance(text: &str) -> bool {
    // Check for SENA markers
    let markers = [
        "SENA 🦁",
        "SENA BRILLIANT THINKING",
        "SENA TRUTH VERIFICATION",
        "SENA CODE ANALYSIS",
        "╔═",  // Box drawing
        "┌─",  // Table
    ];

    markers.iter().any(|m| text.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_triggers_table() {
        let triggers = detect_triggers("show me a table of data");
        assert!(triggers.contains(&"TABLE_FORMAT".to_string()));
    }

    #[test]
    fn test_detect_triggers_thinking() {
        let triggers = detect_triggers("why does this work?");
        assert!(triggers.contains(&"BRILLIANT_THINKING".to_string()));
    }

    #[test]
    fn test_detect_triggers_multiple() {
        let triggers = detect_triggers("explain this in a table format");
        assert!(triggers.contains(&"TABLE_FORMAT".to_string()));
        assert!(triggers.contains(&"BRILLIANT_THINKING".to_string()));
    }

    #[test]
    fn test_sena_format_compliance() {
        assert!(check_sena_format_compliance("SENA 🦁 Response here"));
        assert!(check_sena_format_compliance("╔══════════════╗"));
        assert!(!check_sena_format_compliance("Plain text response"));
    }

    #[tokio::test]
    async fn test_handle_pre_prompt() {
        let result = handle_pre_prompt("show me a table").await.unwrap();
        assert!(result.success);
        assert!(result.data.is_some());
    }

    #[tokio::test]
    async fn test_handle_pre_validation() {
        let result = handle_pre_validation("Test content").await.unwrap();
        assert!(result.success);
    }
}
