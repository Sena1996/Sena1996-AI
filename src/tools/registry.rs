use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::{
    ParameterType, ToolCategory, ToolDefinition, ToolError, ToolExample, ToolParameter, ToolResult,
};

pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn unregister(&mut self, name: &str) -> Option<ToolDefinition> {
        self.tools.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    pub fn list_all(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    pub fn list_by_category(&self, category: &ToolCategory) -> Vec<&ToolDefinition> {
        self.tools
            .values()
            .filter(|t| &t.category == category)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.tools.len()
    }

    pub fn register_builtins(&mut self) {
        self.register(Self::file_read_tool());
        self.register(Self::file_write_tool());
        self.register(Self::file_list_tool());
        self.register(Self::file_exists_tool());
        self.register(Self::shell_exec_tool());
        self.register(Self::web_fetch_tool());
        self.register(Self::code_search_tool());
        self.register(Self::code_analyze_tool());
    }

    pub fn load_custom_tools(&mut self, dir: &Path) -> ToolResult<usize> {
        if !dir.exists() {
            return Ok(0);
        }

        let mut loaded = 0;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "json") {
                match self.load_tool_from_file(&path) {
                    Ok(tool) => {
                        self.register(tool);
                        loaded += 1;
                    }
                    Err(e) => {
                        log::warn!("Failed to load tool from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(loaded)
    }

    fn load_tool_from_file(&self, path: &Path) -> ToolResult<ToolDefinition> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| ToolError::SerializationError(e.to_string()))
    }

    fn file_read_tool() -> ToolDefinition {
        ToolDefinition {
            name: "file_read".to_string(),
            description: "Read contents of a file".to_string(),
            category: ToolCategory::FileSystem,
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "Path to the file to read".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "encoding".to_string(),
                    description: "File encoding (default: utf-8)".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!("utf-8")),
                },
            ],
            returns: "File contents as string".to_string(),
            examples: vec![ToolExample {
                description: "Read a text file".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("path".to_string(), serde_json::json!("/tmp/example.txt"));
                    p
                },
                expected_output: "Contents of the file...".to_string(),
            }],
            requires_confirmation: false,
            timeout_seconds: 30,
        }
    }

    fn file_write_tool() -> ToolDefinition {
        ToolDefinition {
            name: "file_write".to_string(),
            description: "Write contents to a file".to_string(),
            category: ToolCategory::FileSystem,
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "Path to the file to write".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "content".to_string(),
                    description: "Content to write".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "append".to_string(),
                    description: "Append to file instead of overwriting".to_string(),
                    param_type: ParameterType::Boolean,
                    required: false,
                    default: Some(serde_json::json!(false)),
                },
            ],
            returns: "Success status and bytes written".to_string(),
            examples: vec![ToolExample {
                description: "Write to a text file".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("path".to_string(), serde_json::json!("/tmp/output.txt"));
                    p.insert("content".to_string(), serde_json::json!("Hello, World!"));
                    p
                },
                expected_output: r#"{"success": true, "bytes_written": 13}"#.to_string(),
            }],
            requires_confirmation: true,
            timeout_seconds: 30,
        }
    }

    fn file_list_tool() -> ToolDefinition {
        ToolDefinition {
            name: "file_list".to_string(),
            description: "List files in a directory".to_string(),
            category: ToolCategory::FileSystem,
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "Directory path to list".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "recursive".to_string(),
                    description: "List recursively".to_string(),
                    param_type: ParameterType::Boolean,
                    required: false,
                    default: Some(serde_json::json!(false)),
                },
                ToolParameter {
                    name: "pattern".to_string(),
                    description: "Glob pattern to filter".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
            ],
            returns: "List of file paths".to_string(),
            examples: vec![ToolExample {
                description: "List Rust files".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("path".to_string(), serde_json::json!("./src"));
                    p.insert("pattern".to_string(), serde_json::json!("*.rs"));
                    p
                },
                expected_output: r#"["src/main.rs", "src/lib.rs"]"#.to_string(),
            }],
            requires_confirmation: false,
            timeout_seconds: 60,
        }
    }

    fn file_exists_tool() -> ToolDefinition {
        ToolDefinition {
            name: "file_exists".to_string(),
            description: "Check if a file or directory exists".to_string(),
            category: ToolCategory::FileSystem,
            parameters: vec![ToolParameter {
                name: "path".to_string(),
                description: "Path to check".to_string(),
                param_type: ParameterType::String,
                required: true,
                default: None,
            }],
            returns: "Boolean indicating existence".to_string(),
            examples: vec![ToolExample {
                description: "Check if file exists".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("path".to_string(), serde_json::json!("/tmp/test.txt"));
                    p
                },
                expected_output: "true".to_string(),
            }],
            requires_confirmation: false,
            timeout_seconds: 5,
        }
    }

    fn shell_exec_tool() -> ToolDefinition {
        ToolDefinition {
            name: "shell_exec".to_string(),
            description: "Execute a shell command".to_string(),
            category: ToolCategory::Shell,
            parameters: vec![
                ToolParameter {
                    name: "command".to_string(),
                    description: "Command to execute".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "cwd".to_string(),
                    description: "Working directory".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
                ToolParameter {
                    name: "timeout".to_string(),
                    description: "Timeout in seconds".to_string(),
                    param_type: ParameterType::Integer,
                    required: false,
                    default: Some(serde_json::json!(60)),
                },
            ],
            returns: "Command output (stdout, stderr, exit code)".to_string(),
            examples: vec![ToolExample {
                description: "List files".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("command".to_string(), serde_json::json!("ls -la"));
                    p
                },
                expected_output: "File listing...".to_string(),
            }],
            requires_confirmation: true,
            timeout_seconds: 120,
        }
    }

    fn web_fetch_tool() -> ToolDefinition {
        ToolDefinition {
            name: "web_fetch".to_string(),
            description: "Fetch content from a URL".to_string(),
            category: ToolCategory::Web,
            parameters: vec![
                ToolParameter {
                    name: "url".to_string(),
                    description: "URL to fetch".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "method".to_string(),
                    description: "HTTP method".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!("GET")),
                },
                ToolParameter {
                    name: "headers".to_string(),
                    description: "HTTP headers".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: None,
                },
            ],
            returns: "Response body, status, headers".to_string(),
            examples: vec![ToolExample {
                description: "Fetch a webpage".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("url".to_string(), serde_json::json!("https://example.com"));
                    p
                },
                expected_output: "HTML content...".to_string(),
            }],
            requires_confirmation: false,
            timeout_seconds: 30,
        }
    }

    fn code_search_tool() -> ToolDefinition {
        ToolDefinition {
            name: "code_search".to_string(),
            description: "Search for patterns in code".to_string(),
            category: ToolCategory::Code,
            parameters: vec![
                ToolParameter {
                    name: "pattern".to_string(),
                    description: "Search pattern (regex)".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "path".to_string(),
                    description: "Directory to search".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!(".")),
                },
                ToolParameter {
                    name: "file_pattern".to_string(),
                    description: "File glob pattern".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: None,
                },
            ],
            returns: "List of matches with file, line, content".to_string(),
            examples: vec![ToolExample {
                description: "Search for TODO comments".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("pattern".to_string(), serde_json::json!("TODO:"));
                    p.insert("file_pattern".to_string(), serde_json::json!("*.rs"));
                    p
                },
                expected_output: "List of matches...".to_string(),
            }],
            requires_confirmation: false,
            timeout_seconds: 120,
        }
    }

    fn code_analyze_tool() -> ToolDefinition {
        ToolDefinition {
            name: "code_analyze".to_string(),
            description: "Analyze code structure and quality".to_string(),
            category: ToolCategory::Code,
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "File or directory to analyze".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ToolParameter {
                    name: "analysis_type".to_string(),
                    description: "Type of analysis (structure, complexity, security)".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(serde_json::json!("structure")),
                },
            ],
            returns: "Analysis results".to_string(),
            examples: vec![ToolExample {
                description: "Analyze a Rust file".to_string(),
                parameters: {
                    let mut p = HashMap::new();
                    p.insert("path".to_string(), serde_json::json!("src/main.rs"));
                    p
                },
                expected_output: "Analysis results...".to_string(),
            }],
            requires_confirmation: false,
            timeout_seconds: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_builtins() {
        let mut registry = ToolRegistry::new();
        registry.register_builtins();
        assert!(registry.count() >= 8);
    }

    #[test]
    fn test_get_tool() {
        let mut registry = ToolRegistry::new();
        registry.register_builtins();

        let tool = registry.get("file_read");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "file_read");
    }

    #[test]
    fn test_list_by_category() {
        let mut registry = ToolRegistry::new();
        registry.register_builtins();

        let fs_tools = registry.list_by_category(&ToolCategory::FileSystem);
        assert!(!fs_tools.is_empty());
    }
}
