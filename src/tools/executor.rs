use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use glob::glob;
use regex::Regex;
use tokio::time::timeout;

use super::{ToolDefinition, ToolError, ToolResult};

pub struct ToolExecutor {
    allowed_paths: Vec<String>,
    blocked_commands: Vec<String>,
    max_output_size: usize,
}

#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub output: serde_json::Value,
    pub execution_time_ms: u64,
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            allowed_paths: vec![],
            blocked_commands: vec![
                "rm -rf /".to_string(),
                "sudo rm".to_string(),
                "mkfs".to_string(),
                "dd if=".to_string(),
                "> /dev/".to_string(),
            ],
            max_output_size: 1024 * 1024,
        }
    }

    pub fn with_allowed_paths(mut self, paths: Vec<String>) -> Self {
        self.allowed_paths = paths;
        self
    }

    pub async fn execute(
        &self,
        tool: &ToolDefinition,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let timeout_duration = Duration::from_secs(tool.timeout_seconds);

        let result = timeout(timeout_duration, async {
            match tool.name.as_str() {
                "file_read" => self.execute_file_read(params).await,
                "file_write" => self.execute_file_write(params).await,
                "file_list" => self.execute_file_list(params).await,
                "file_exists" => self.execute_file_exists(params).await,
                "shell_exec" => self.execute_shell(params).await,
                "web_fetch" => self.execute_web_fetch(params).await,
                "code_search" => self.execute_code_search(params).await,
                "code_analyze" => self.execute_code_analyze(params).await,
                _ => Err(ToolError::NotFound(tool.name.clone())),
            }
        })
        .await;

        match result {
            Ok(inner) => inner,
            Err(_) => Err(ToolError::Timeout(format!(
                "Tool {} timed out after {} seconds",
                tool.name, tool.timeout_seconds
            ))),
        }
    }

    async fn execute_file_read(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("path is required".to_string()))?;

        self.validate_path(path)?;

        let content = fs::read_to_string(path)?;

        let truncated = content.len() > self.max_output_size;
        let content = if truncated {
            content[..self.max_output_size].to_string()
        } else {
            content
        };

        Ok(serde_json::json!({
            "content": content,
            "path": path,
            "size": content.len(),
            "truncated": truncated
        }))
    }

    async fn execute_file_write(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("path is required".to_string()))?;

        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("content is required".to_string()))?;

        let append = params
            .get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        self.validate_path(path)?;

        let mut file = if append {
            OpenOptions::new().append(true).create(true).open(path)?
        } else {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?
        };

        let bytes_written = file.write(content.as_bytes())?;

        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "bytes_written": bytes_written,
            "append": append
        }))
    }

    async fn execute_file_list(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("path is required".to_string()))?;

        let recursive = params
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let pattern = params.get("pattern").and_then(|v| v.as_str());

        self.validate_path(path)?;

        let mut files = Vec::new();

        if recursive {
            let glob_pattern = if let Some(pat) = pattern {
                format!("{}/**/{}", path, pat)
            } else {
                format!("{}/**/*", path)
            };

            for p in glob(&glob_pattern)
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?
                .flatten()
            {
                if p.is_file() {
                    files.push(p.to_string_lossy().to_string());
                }
            }
        } else {
            let glob_pattern = if let Some(pat) = pattern {
                format!("{}/{}", path, pat)
            } else {
                format!("{}/*", path)
            };

            for p in glob(&glob_pattern)
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?
                .flatten()
            {
                files.push(p.to_string_lossy().to_string());
            }
        }

        Ok(serde_json::json!({
            "files": files,
            "count": files.len(),
            "path": path,
            "recursive": recursive
        }))
    }

    async fn execute_file_exists(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("path is required".to_string()))?;

        let exists = Path::new(path).exists();
        let is_file = Path::new(path).is_file();
        let is_dir = Path::new(path).is_dir();

        Ok(serde_json::json!({
            "exists": exists,
            "is_file": is_file,
            "is_directory": is_dir,
            "path": path
        }))
    }

    async fn execute_shell(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let command = params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("command is required".to_string()))?;

        self.validate_command(command)?;

        let cwd = params.get("cwd").and_then(|v| v.as_str());

        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let shell_arg = if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        };

        let mut cmd = Command::new(shell);
        cmd.arg(shell_arg).arg(command);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let stdout = if stdout.len() > self.max_output_size {
            stdout[..self.max_output_size].to_string()
        } else {
            stdout.to_string()
        };

        let stderr = if stderr.len() > self.max_output_size {
            stderr[..self.max_output_size].to_string()
        } else {
            stderr.to_string()
        };

        Ok(serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code().unwrap_or(-1),
            "success": output.status.success(),
            "command": command
        }))
    }

    async fn execute_web_fetch(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("url is required".to_string()))?;

        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let request = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => return Err(ToolError::InvalidParameters(format!("Invalid HTTP method: {}", method))),
        };

        let response = request
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response
            .text()
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let body = if body.len() > self.max_output_size {
            body[..self.max_output_size].to_string()
        } else {
            body
        };

        Ok(serde_json::json!({
            "url": url,
            "status": status,
            "headers": headers,
            "body": body,
            "truncated": body.len() >= self.max_output_size
        }))
    }

    async fn execute_code_search(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let pattern = params
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("pattern is required".to_string()))?;

        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let file_pattern = params.get("file_pattern").and_then(|v| v.as_str());

        self.validate_path(path)?;

        let regex = Regex::new(pattern)
            .map_err(|e| ToolError::InvalidParameters(format!("Invalid regex: {}", e)))?;

        let glob_pattern = if let Some(fp) = file_pattern {
            format!("{}/**/{}", path, fp)
        } else {
            format!("{}/**/*", path)
        };

        let mut matches = Vec::new();

        for file_path in glob(&glob_pattern)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?
            .flatten()
        {
            if !file_path.is_file() {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&file_path) {
                for (line_num, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        matches.push(serde_json::json!({
                            "file": file_path.to_string_lossy(),
                            "line": line_num + 1,
                            "content": line.trim(),
                        }));

                        if matches.len() >= 100 {
                            break;
                        }
                    }
                }
            }

            if matches.len() >= 100 {
                break;
            }
        }

        Ok(serde_json::json!({
            "matches": matches,
            "count": matches.len(),
            "pattern": pattern,
            "path": path,
            "truncated": matches.len() >= 100
        }))
    }

    async fn execute_code_analyze(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> ToolResult<serde_json::Value> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("path is required".to_string()))?;

        let analysis_type = params
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("structure");

        self.validate_path(path)?;

        let file_path = Path::new(path);

        if !file_path.exists() {
            return Err(ToolError::ExecutionFailed(format!("Path not found: {}", path)));
        }

        let content = if file_path.is_file() {
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        let analysis = match analysis_type {
            "structure" => self.analyze_structure(&content, path),
            "complexity" => self.analyze_complexity(&content),
            "security" => self.analyze_security(&content),
            _ => self.analyze_structure(&content, path),
        };

        Ok(analysis)
    }

    fn analyze_structure(&self, content: &str, path: &str) -> serde_json::Value {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        let code_lines = lines.iter().filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//")).count();
        let comment_lines = lines.iter().filter(|l| l.trim().starts_with("//")).count();
        let blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count();

        let functions = Regex::new(r"(?m)^\s*(pub\s+)?(async\s+)?fn\s+\w+")
            .map(|r| r.find_iter(content).count())
            .unwrap_or(0);

        let structs = Regex::new(r"(?m)^\s*(pub\s+)?struct\s+\w+")
            .map(|r| r.find_iter(content).count())
            .unwrap_or(0);

        let impls = Regex::new(r"(?m)^\s*impl\s+")
            .map(|r| r.find_iter(content).count())
            .unwrap_or(0);

        let traits = Regex::new(r"(?m)^\s*(pub\s+)?trait\s+\w+")
            .map(|r| r.find_iter(content).count())
            .unwrap_or(0);

        serde_json::json!({
            "path": path,
            "analysis_type": "structure",
            "metrics": {
                "total_lines": total_lines,
                "code_lines": code_lines,
                "comment_lines": comment_lines,
                "blank_lines": blank_lines,
                "functions": functions,
                "structs": structs,
                "impls": impls,
                "traits": traits
            }
        })
    }

    fn analyze_complexity(&self, content: &str) -> serde_json::Value {
        let if_count = content.matches("if ").count();
        let loop_count = content.matches("for ").count() + content.matches("while ").count() + content.matches("loop ").count();
        let match_count = content.matches("match ").count();
        let unwrap_count = content.matches(".unwrap()").count();

        let cyclomatic_estimate = 1 + if_count + loop_count + match_count;

        serde_json::json!({
            "analysis_type": "complexity",
            "metrics": {
                "cyclomatic_complexity_estimate": cyclomatic_estimate,
                "if_statements": if_count,
                "loops": loop_count,
                "match_expressions": match_count,
                "unwrap_calls": unwrap_count
            },
            "suggestions": {
                "high_unwrap": if unwrap_count > 5 { "Consider using ? operator instead of unwrap()" } else { "" },
                "high_complexity": if cyclomatic_estimate > 20 { "Consider breaking down into smaller functions" } else { "" }
            }
        })
    }

    fn analyze_security(&self, content: &str) -> serde_json::Value {
        let mut issues = Vec::new();

        if content.contains("unsafe") {
            issues.push(serde_json::json!({
                "severity": "high",
                "type": "unsafe_code",
                "message": "Contains unsafe code blocks"
            }));
        }

        if Regex::new(r#"(?i)(password|secret|api_key|token)\s*=\s*["'][^"']+["']"#)
            .map(|r| r.is_match(content))
            .unwrap_or(false)
        {
            issues.push(serde_json::json!({
                "severity": "critical",
                "type": "hardcoded_secret",
                "message": "Possible hardcoded secret detected"
            }));
        }

        if content.contains(".unwrap()") {
            issues.push(serde_json::json!({
                "severity": "medium",
                "type": "unwrap_usage",
                "message": "Uses unwrap() which can panic"
            }));
        }

        if content.contains("format!(") && content.contains("SQL") {
            issues.push(serde_json::json!({
                "severity": "high",
                "type": "sql_injection",
                "message": "Possible SQL injection vulnerability"
            }));
        }

        serde_json::json!({
            "analysis_type": "security",
            "issues": issues,
            "issue_count": issues.len(),
            "risk_level": if issues.iter().any(|i| i["severity"] == "critical") {
                "critical"
            } else if issues.iter().any(|i| i["severity"] == "high") {
                "high"
            } else if !issues.is_empty() {
                "medium"
            } else {
                "low"
            }
        })
    }

    fn validate_path(&self, path: &str) -> ToolResult<()> {
        let path = Path::new(path);

        if path.to_string_lossy().contains("..") {
            return Err(ToolError::PermissionDenied(
                "Path traversal not allowed".to_string(),
            ));
        }

        if !self.allowed_paths.is_empty() {
            let allowed = self.allowed_paths.iter().any(|allowed| {
                path.starts_with(allowed) || path.to_string_lossy().starts_with(allowed)
            });

            if !allowed {
                return Err(ToolError::PermissionDenied(format!(
                    "Path not in allowed list: {}",
                    path.display()
                )));
            }
        }

        Ok(())
    }

    fn validate_command(&self, command: &str) -> ToolResult<()> {
        let command_lower = command.to_lowercase();

        for blocked in &self.blocked_commands {
            if command_lower.contains(&blocked.to_lowercase()) {
                return Err(ToolError::PermissionDenied(format!(
                    "Command contains blocked pattern: {}",
                    blocked
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_exists() {
        let executor = ToolExecutor::new();
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!("/tmp"));

        let result = executor.execute_file_exists(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output["exists"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_validate_path_traversal() {
        let executor = ToolExecutor::new();
        let result = executor.validate_path("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_blocked_command() {
        let executor = ToolExecutor::new();
        let result = executor.validate_command("rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_complexity() {
        let executor = ToolExecutor::new();
        let code = r#"
            fn test() {
                if true {
                    for i in 0..10 {
                        match i {
                            0 => println!("zero"),
                            _ => println!("{}", i),
                        }
                    }
                }
            }
        "#;

        let result = executor.analyze_complexity(code);
        assert!(result["metrics"]["cyclomatic_complexity_estimate"].as_i64().unwrap() > 1);
    }
}
