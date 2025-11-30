use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

pub mod builtin;
pub mod executor;
pub mod registry;

pub use builtin::BuiltinTools;
pub use executor::{ToolExecutor, ToolExecutionResult};
pub use registry::ToolRegistry;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type ToolResult<T> = Result<T, ToolError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    FileSystem,
    Shell,
    Web,
    Code,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParameterType {
    String,
    Integer,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub parameters: Vec<ToolParameter>,
    pub returns: String,
    pub examples: Vec<ToolExample>,
    pub requires_confirmation: bool,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub expected_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub call_id: String,
}

impl ToolCall {
    pub fn new(tool_name: impl Into<String>, parameters: HashMap<String, serde_json::Value>) -> Self {
        let call_id = format!("call_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
        Self {
            tool_name: tool_name.into(),
            parameters,
            call_id,
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub call_id: String,
    pub tool_name: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

impl ToolResponse {
    pub fn success(call_id: String, tool_name: String, output: serde_json::Value, time_ms: u64) -> Self {
        Self {
            call_id,
            tool_name,
            success: true,
            output,
            error: None,
            execution_time_ms: time_ms,
        }
    }

    pub fn failure(call_id: String, tool_name: String, error: String, time_ms: u64) -> Self {
        Self {
            call_id,
            tool_name,
            success: false,
            output: serde_json::Value::Null,
            error: Some(error),
            execution_time_ms: time_ms,
        }
    }
}

pub struct ToolSystem {
    registry: ToolRegistry,
    executor: ToolExecutor,
    call_history: Vec<ToolResponse>,
}

impl Default for ToolSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolSystem {
    pub fn new() -> Self {
        let mut registry = ToolRegistry::new();
        registry.register_builtins();

        Self {
            registry,
            executor: ToolExecutor::new(),
            call_history: Vec::new(),
        }
    }

    pub fn with_tools_dir(tools_dir: PathBuf) -> ToolResult<Self> {
        let mut registry = ToolRegistry::new();
        registry.register_builtins();
        registry.load_custom_tools(&tools_dir)?;

        Ok(Self {
            registry,
            executor: ToolExecutor::new(),
            call_history: Vec::new(),
        })
    }

    pub async fn execute(&mut self, call: ToolCall) -> ToolResponse {
        let start = std::time::Instant::now();

        let tool = match self.registry.get(&call.tool_name) {
            Some(t) => t.clone(),
            None => {
                let response = ToolResponse::failure(
                    call.call_id,
                    call.tool_name,
                    "Tool not found".to_string(),
                    start.elapsed().as_millis() as u64,
                );
                self.call_history.push(response.clone());
                return response;
            }
        };

        let result = self.executor.execute(&tool, &call.parameters).await;

        let response = match result {
            Ok(output) => ToolResponse::success(
                call.call_id,
                call.tool_name,
                output,
                start.elapsed().as_millis() as u64,
            ),
            Err(e) => ToolResponse::failure(
                call.call_id,
                call.tool_name,
                e.to_string(),
                start.elapsed().as_millis() as u64,
            ),
        };

        self.call_history.push(response.clone());
        response
    }

    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.registry.list_all()
    }

    pub fn list_by_category(&self, category: ToolCategory) -> Vec<&ToolDefinition> {
        self.registry.list_by_category(&category)
    }

    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.registry.get(name)
    }

    pub fn register_tool(&mut self, tool: ToolDefinition) {
        self.registry.register(tool);
    }

    pub fn get_history(&self) -> &[ToolResponse] {
        &self.call_history
    }

    pub fn clear_history(&mut self) {
        self.call_history.clear();
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall::new("file_read", HashMap::new())
            .with_param("path", "/tmp/test.txt");

        assert_eq!(call.tool_name, "file_read");
        assert!(call.parameters.contains_key("path"));
    }

    #[test]
    fn test_tool_response_success() {
        let response = ToolResponse::success(
            "call_1".to_string(),
            "test_tool".to_string(),
            serde_json::json!({"result": "ok"}),
            100,
        );

        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_tool_response_failure() {
        let response = ToolResponse::failure(
            "call_2".to_string(),
            "test_tool".to_string(),
            "Something went wrong".to_string(),
            50,
        );

        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_tool_system_creation() {
        let system = ToolSystem::new();
        let tools = system.list_tools();
        assert!(!tools.is_empty());
    }
}
