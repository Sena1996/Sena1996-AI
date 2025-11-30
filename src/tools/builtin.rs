use std::collections::HashMap;

use super::{ToolCall, ToolCategory, ToolDefinition, ToolResponse};

pub struct BuiltinTools;

impl BuiltinTools {
    pub fn file_read(path: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path));
        ToolCall::new("file_read", params)
    }

    pub fn file_write(path: &str, content: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path));
        params.insert("content".to_string(), serde_json::json!(content));
        ToolCall::new("file_write", params)
    }

    pub fn file_append(path: &str, content: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path));
        params.insert("content".to_string(), serde_json::json!(content));
        params.insert("append".to_string(), serde_json::json!(true));
        ToolCall::new("file_write", params)
    }

    pub fn file_list(path: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path));
        ToolCall::new("file_list", params)
    }

    pub fn file_list_recursive(path: &str, pattern: Option<&str>) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path));
        params.insert("recursive".to_string(), serde_json::json!(true));
        if let Some(p) = pattern {
            params.insert("pattern".to_string(), serde_json::json!(p));
        }
        ToolCall::new("file_list", params)
    }

    pub fn file_exists(path: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path));
        ToolCall::new("file_exists", params)
    }

    pub fn shell_exec(command: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("command".to_string(), serde_json::json!(command));
        ToolCall::new("shell_exec", params)
    }

    pub fn shell_exec_in_dir(command: &str, cwd: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("command".to_string(), serde_json::json!(command));
        params.insert("cwd".to_string(), serde_json::json!(cwd));
        ToolCall::new("shell_exec", params)
    }

    pub fn web_fetch(url: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("url".to_string(), serde_json::json!(url));
        ToolCall::new("web_fetch", params)
    }

    pub fn web_post(url: &str, body: Option<&str>) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("url".to_string(), serde_json::json!(url));
        params.insert("method".to_string(), serde_json::json!("POST"));
        if let Some(b) = body {
            params.insert("body".to_string(), serde_json::json!(b));
        }
        ToolCall::new("web_fetch", params)
    }

    pub fn code_search(pattern: &str, path: Option<&str>) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), serde_json::json!(pattern));
        if let Some(p) = path {
            params.insert("path".to_string(), serde_json::json!(p));
        }
        ToolCall::new("code_search", params)
    }

    pub fn code_search_files(pattern: &str, path: &str, file_pattern: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), serde_json::json!(pattern));
        params.insert("path".to_string(), serde_json::json!(path));
        params.insert("file_pattern".to_string(), serde_json::json!(file_pattern));
        ToolCall::new("code_search", params)
    }

    pub fn code_analyze(path: &str, analysis_type: &str) -> ToolCall {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path));
        params.insert("analysis_type".to_string(), serde_json::json!(analysis_type));
        ToolCall::new("code_analyze", params)
    }

    pub fn code_analyze_structure(path: &str) -> ToolCall {
        Self::code_analyze(path, "structure")
    }

    pub fn code_analyze_complexity(path: &str) -> ToolCall {
        Self::code_analyze(path, "complexity")
    }

    pub fn code_analyze_security(path: &str) -> ToolCall {
        Self::code_analyze(path, "security")
    }
}

pub fn get_tool_summary(tool: &ToolDefinition) -> String {
    format!(
        "{} - {} [{}]",
        tool.name,
        tool.description,
        format!("{:?}", tool.category).to_lowercase()
    )
}

pub fn format_tool_help(tool: &ToolDefinition) -> String {
    let mut output = String::new();

    output.push_str(&format!("Tool: {}\n", tool.name));
    output.push_str(&format!("Description: {}\n", tool.description));
    output.push_str(&format!("Category: {:?}\n", tool.category));
    output.push_str(&format!("Returns: {}\n", tool.returns));
    output.push_str(&format!(
        "Requires Confirmation: {}\n",
        tool.requires_confirmation
    ));
    output.push_str(&format!("Timeout: {}s\n\n", tool.timeout_seconds));

    output.push_str("Parameters:\n");
    for param in &tool.parameters {
        let required = if param.required { "*" } else { "" };
        output.push_str(&format!(
            "  {}{} ({:?}): {}\n",
            param.name, required, param.param_type, param.description
        ));
        if let Some(default) = &param.default {
            output.push_str(&format!("    Default: {}\n", default));
        }
    }

    if !tool.examples.is_empty() {
        output.push_str("\nExamples:\n");
        for example in &tool.examples {
            output.push_str(&format!("  {}\n", example.description));
            output.push_str(&format!("    Parameters: {:?}\n", example.parameters));
            output.push_str(&format!("    Expected: {}\n", example.expected_output));
        }
    }

    output
}

pub fn format_tool_list(tools: &[&ToolDefinition]) -> String {
    let mut output = String::new();

    let mut by_category: HashMap<&ToolCategory, Vec<&&ToolDefinition>> = HashMap::new();
    for tool in tools {
        by_category.entry(&tool.category).or_default().push(tool);
    }

    for (category, cat_tools) in by_category {
        output.push_str(&format!("\n{:?} Tools:\n", category));
        output.push_str(&"-".repeat(40));
        output.push('\n');

        for tool in cat_tools {
            output.push_str(&format!("  {} - {}\n", tool.name, tool.description));
        }
    }

    output
}

pub fn format_tool_response(response: &ToolResponse) -> String {
    let status = if response.success { "SUCCESS" } else { "FAILED" };

    let mut output = format!(
        "[{}] {} ({}ms)\n",
        status, response.tool_name, response.execution_time_ms
    );

    if response.success {
        output.push_str(&format!(
            "Output: {}\n",
            serde_json::to_string_pretty(&response.output).unwrap_or_else(|_| "{}".to_string())
        ));
    } else if let Some(err) = &response.error {
        output.push_str(&format!("Error: {}\n", err));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_file_read() {
        let call = BuiltinTools::file_read("/tmp/test.txt");
        assert_eq!(call.tool_name, "file_read");
        assert!(call.parameters.contains_key("path"));
    }

    #[test]
    fn test_builtin_shell_exec() {
        let call = BuiltinTools::shell_exec("echo hello");
        assert_eq!(call.tool_name, "shell_exec");
        assert_eq!(
            call.parameters.get("command").unwrap().as_str().unwrap(),
            "echo hello"
        );
    }

    #[test]
    fn test_builtin_code_search() {
        let call = BuiltinTools::code_search_files("TODO", "./src", "*.rs");
        assert_eq!(call.tool_name, "code_search");
        assert!(call.parameters.contains_key("file_pattern"));
    }

    #[test]
    fn test_format_tool_response() {
        let response = ToolResponse::success(
            "call_1".to_string(),
            "file_read".to_string(),
            serde_json::json!({"content": "hello"}),
            50,
        );

        let formatted = format_tool_response(&response);
        assert!(formatted.contains("SUCCESS"));
        assert!(formatted.contains("file_read"));
    }
}
