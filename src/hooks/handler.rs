//! Hook Handlers
//!
//! Handles Claude Code hooks for SENA integration

use crate::ancient::HarmonyValidationEngine;
use crate::cli::args::HookType;
use crate::integration::AutoIntegration;
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
    let detected = ai.detect_format(input);
    let triggers = detect_triggers(input);
    let inbox_messages = check_inbox_messages();
    let is_sena_command = detect_sena_command(input);

    let mut message = build_sena_context();

    if !inbox_messages.is_empty() {
        message.push_str(&format!(
            "\n\nINBOX ({} messages):\n{}",
            inbox_messages.len(),
            inbox_messages.join("\n")
        ));
    }

    if is_sena_command {
        message.push_str(
            "\n\nDETECTED: User wants to run a SENA command. Use Bash tool to execute it.",
        );
    }

    let result = HookResult::success(&message).with_data(serde_json::json!({
        "detected_format": detected.as_ref().map(|f| f.name()),
        "triggers": triggers,
        "is_sena_command": is_sena_command,
        "inbox_count": inbox_messages.len(),
    }));

    Ok(result)
}

fn build_sena_context() -> String {
    use crate::config::SenaConfig;
    let config = SenaConfig::global();
    let user = &config.user;

    format!(
        r#"SENA Controller v{} is installed and ready.

SENA COMMANDS (execute with Bash tool):
- sena health              : System health check
- sena who                 : List active sessions
- sena session list        : List all sessions with IDs
- sena session start --name 'Name' --role <role> : Start session (roles: general, backend, web, android, ios, iot)
- sena session end --id <id> : End session
- sena tell <Name> "msg"   : Send message to session (use session name, not ID)
- sena inbox               : Check messages
- sena task new "title" --to <Name> : Create task
- sena task list           : List tasks
- sena think "question"    : Quick analysis
- sena think --depth deep "question" : Deep analysis
- sena agent security "code" : Security analysis
- sena backend/ios/android/iot/web <action> <code> : Domain agents

RULES:
- Each role can have only ONE active session
- Use session NAME (not ID) for tell/task commands
- All sena commands are auto-approved (no bash prompts needed)

User: {} | Prefix: {} {}"#,
        crate::VERSION,
        user.name,
        user.prefix,
        user.emoji
    )
}

fn detect_sena_command(input: &str) -> bool {
    let lower = input.to_lowercase().trim().to_string();
    lower.starts_with("sena ")
        || lower == "sena"
        || lower.starts_with("sena session")
        || lower.starts_with("sena health")
        || lower.starts_with("sena who")
        || lower.starts_with("sena tell")
        || lower.starts_with("sena inbox")
        || lower.starts_with("sena task")
        || lower.starts_with("sena think")
        || lower.starts_with("sena agent")
        || lower.starts_with("sena backend")
        || lower.starts_with("sena ios")
        || lower.starts_with("sena android")
        || lower.starts_with("sena iot")
        || lower.starts_with("sena web")
}

fn check_inbox_messages() -> Vec<String> {
    use crate::hub::{Hub, Message};

    let mut hub = match Hub::new() {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    if hub.load().is_err() {
        return Vec::new();
    }

    let messages: Vec<Message> = hub.inbox("local").into_iter().filter(|m| !m.read).collect();

    messages
        .iter()
        .map(|m| format!("  [{}] {}: {}", m.time_display(), m.from, m.content))
        .collect()
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

    let result = HookResult::success("Response validation complete").with_data(serde_json::json!({
        "valid": validation.is_valid(),
        "confidence": validation.overall_confidence,
        "sena_compliant": has_sena_format,
        "response_length": response_text.len(),
    }));

    Ok(result)
}

/// Pre-tool hook - validate tool calls before execution
async fn handle_pre_tool(input: &str) -> Result<HookResult, String> {
    let tool_data: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("Invalid tool data: {}", e))?;

    let tool_name = tool_data
        .get("tool")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    // Check for potentially dangerous tools
    let dangerous_tools = ["Bash", "Write", "Edit"];
    let is_dangerous = dangerous_tools.contains(&tool_name);

    // Get tool arguments for analysis
    let args = tool_data
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let result = HookResult::success("Tool validation complete").with_data(serde_json::json!({
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
    let tool_result: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("Invalid tool result: {}", e))?;

    let tool_name = tool_result
        .get("tool")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    let success = tool_result
        .get("success")
        .and_then(|s| s.as_bool())
        .unwrap_or(true);

    let result = HookResult::success("Tool result processed").with_data(serde_json::json!({
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

    let result = HookResult::success("Pre-validation complete").with_data(serde_json::json!({
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

    let result = HookResult::success("Post-validation complete").with_data(serde_json::json!({
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
        (
            vec!["is it true", "fact check", "verify", "true or false"],
            "TRUTH_VERIFICATION",
        ),
        (
            vec!["analyze code", "code review", "review this"],
            "CODE_ANALYSIS",
        ),
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

fn check_sena_format_compliance(text: &str) -> bool {
    use crate::config::SenaConfig;
    let user = SenaConfig::user();
    let brand = user.brand();

    let markers = [
        brand.as_str(),
        &format!("{} BRILLIANT THINKING", user.prefix),
        &format!("{} TRUTH VERIFICATION", user.prefix),
        &format!("{} CODE ANALYSIS", user.prefix),
        "╔═",
        "┌─",
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
        use crate::config::SenaConfig;
        let brand = SenaConfig::brand();
        assert!(check_sena_format_compliance(&format!(
            "{} Response here",
            brand
        )));
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
