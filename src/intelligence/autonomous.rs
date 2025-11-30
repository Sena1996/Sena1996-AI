use crate::memory::{MemoryEntry, MemoryType, PersistentMemory};
use crate::tools::{ToolCall, ToolSystem};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AutonomousError {
    #[error("Task failed: {0}")]
    TaskFailed(String),
    #[error("Max steps exceeded")]
    MaxStepsExceeded,
    #[error("Tool error: {0}")]
    ToolError(String),
    #[error("Planning error: {0}")]
    PlanningError(String),
    #[error("Memory error: {0}")]
    MemoryError(String),
}

pub type AutonomousResult<T> = Result<T, AutonomousError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Planning,
    Executing,
    Waiting,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub step_number: usize,
    pub action: String,
    pub tool_name: Option<String>,
    pub tool_params: Option<HashMap<String, serde_json::Value>>,
    pub result: Option<String>,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
}

impl AgentStep {
    pub fn new(step_number: usize, action: &str) -> Self {
        Self {
            step_number,
            action: action.to_string(),
            tool_name: None,
            tool_params: None,
            result: None,
            success: false,
            timestamp: Utc::now(),
            duration_ms: 0,
        }
    }

    pub fn with_tool(mut self, tool_name: &str, params: HashMap<String, serde_json::Value>) -> Self {
        self.tool_name = Some(tool_name.to_string());
        self.tool_params = Some(params);
        self
    }

    pub fn complete(mut self, result: &str, success: bool, duration_ms: u64) -> Self {
        self.result = Some(result.to_string());
        self.success = success;
        self.duration_ms = duration_ms;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlan {
    pub task: String,
    pub steps: Vec<PlannedStep>,
    pub context: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedStep {
    pub description: String,
    pub tool_hint: Option<String>,
    pub estimated_complexity: StepComplexity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepComplexity {
    Simple,
    Medium,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    pub id: String,
    pub task: String,
    pub state: AgentState,
    pub plan: Option<AgentPlan>,
    pub steps: Vec<AgentStep>,
    pub context: HashMap<String, String>,
    pub working_dir: PathBuf,
    pub max_steps: usize,
    pub require_confirmation: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub final_result: Option<String>,
}

impl AgentExecution {
    pub fn elapsed_ms(&self) -> u64 {
        let end = self.completed_at.unwrap_or_else(Utc::now);
        (end - self.started_at).num_milliseconds() as u64
    }

    pub fn steps_taken(&self) -> usize {
        self.steps.len()
    }

    pub fn successful_steps(&self) -> usize {
        self.steps.iter().filter(|s| s.success).count()
    }
}

pub struct AutonomousAgent {
    tool_system: ToolSystem,
    memory: Option<PersistentMemory>,
    current_execution: Option<AgentExecution>,
}

impl AutonomousAgent {
    pub fn new() -> Self {
        let memory = PersistentMemory::new().ok();
        Self {
            tool_system: ToolSystem::new(),
            memory,
            current_execution: None,
        }
    }

    pub fn with_memory(mut self, memory: PersistentMemory) -> Self {
        self.memory = Some(memory);
        self
    }

    pub async fn execute(
        &mut self,
        task: &str,
        working_dir: PathBuf,
        max_steps: usize,
        require_confirmation: bool,
    ) -> AutonomousResult<AgentExecution> {
        let execution_id = format!(
            "exec_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        let mut execution = AgentExecution {
            id: execution_id,
            task: task.to_string(),
            state: AgentState::Planning,
            plan: None,
            steps: Vec::new(),
            context: HashMap::new(),
            working_dir: working_dir.clone(),
            max_steps,
            require_confirmation,
            started_at: Utc::now(),
            completed_at: None,
            final_result: None,
        };

        let plan = self.create_plan(task, &working_dir)?;
        execution.plan = Some(plan.clone());

        execution.state = AgentState::Executing;

        for (idx, planned_step) in plan.steps.iter().enumerate() {
            if execution.steps.len() >= max_steps {
                execution.state = AgentState::Completed;
                execution.final_result = Some(format!(
                    "Completed {} of {} planned steps (max steps reached)",
                    execution.steps.len(),
                    plan.steps.len()
                ));
                break;
            }

            let step_result = self
                .execute_step(idx + 1, planned_step, &execution.context, &working_dir)
                .await;

            match step_result {
                Ok(step) => {
                    if let Some(result) = &step.result {
                        execution
                            .context
                            .insert(format!("step_{}_result", idx + 1), result.clone());
                    }
                    execution.steps.push(step);
                }
                Err(e) => {
                    let failed_step = AgentStep::new(idx + 1, &planned_step.description)
                        .complete(&format!("Error: {}", e), false, 0);
                    execution.steps.push(failed_step);
                }
            }
        }

        if execution.state != AgentState::Completed {
            execution.state = AgentState::Completed;
            execution.final_result = Some(format!(
                "Completed {} steps successfully",
                execution.successful_steps()
            ));
        }

        execution.completed_at = Some(Utc::now());

        if let Some(ref mut memory) = self.memory {
            let summary = format!(
                "Task: {} | Steps: {} | Success: {}",
                task,
                execution.steps_taken(),
                execution.successful_steps()
            );

            let entry = MemoryEntry::new(summary, MemoryType::Context)
                .with_tags(vec!["autonomous".to_string(), "execution".to_string()])
                .with_importance(0.6)
                .with_metadata("execution_id", &execution.id);

            let _ = memory.add(entry);
        }

        self.current_execution = Some(execution.clone());
        Ok(execution)
    }

    fn create_plan(&self, task: &str, working_dir: &Path) -> AutonomousResult<AgentPlan> {
        let task_lower = task.to_lowercase();
        let mut steps = Vec::new();

        if task_lower.contains("read") || task_lower.contains("show") || task_lower.contains("view")
        {
            if let Some(file_hint) = extract_file_hint(&task_lower) {
                steps.push(PlannedStep {
                    description: format!("Read file: {}", file_hint),
                    tool_hint: Some("file_read".to_string()),
                    estimated_complexity: StepComplexity::Simple,
                });
            } else {
                steps.push(PlannedStep {
                    description: "List files in directory".to_string(),
                    tool_hint: Some("file_list".to_string()),
                    estimated_complexity: StepComplexity::Simple,
                });
            }
        }

        if task_lower.contains("list") || task_lower.contains("files") || task_lower.contains("dir")
        {
            steps.push(PlannedStep {
                description: format!("List files in {}", working_dir.display()),
                tool_hint: Some("file_list".to_string()),
                estimated_complexity: StepComplexity::Simple,
            });
        }

        if task_lower.contains("search") || task_lower.contains("find") || task_lower.contains("grep")
        {
            let pattern = extract_search_pattern(&task_lower).unwrap_or("TODO".to_string());
            steps.push(PlannedStep {
                description: format!("Search for pattern: {}", pattern),
                tool_hint: Some("code_search".to_string()),
                estimated_complexity: StepComplexity::Medium,
            });
        }

        if task_lower.contains("analyze") || task_lower.contains("review") {
            steps.push(PlannedStep {
                description: "Analyze code structure".to_string(),
                tool_hint: Some("code_analyze".to_string()),
                estimated_complexity: StepComplexity::Complex,
            });
        }

        if task_lower.contains("write") || task_lower.contains("create") {
            steps.push(PlannedStep {
                description: "Write file contents".to_string(),
                tool_hint: Some("file_write".to_string()),
                estimated_complexity: StepComplexity::Medium,
            });
        }

        if task_lower.contains("run") || task_lower.contains("execute") || task_lower.contains("command")
        {
            steps.push(PlannedStep {
                description: "Execute shell command".to_string(),
                tool_hint: Some("shell_exec".to_string()),
                estimated_complexity: StepComplexity::Complex,
            });
        }

        if steps.is_empty() {
            steps.push(PlannedStep {
                description: "Gather context from working directory".to_string(),
                tool_hint: Some("file_list".to_string()),
                estimated_complexity: StepComplexity::Simple,
            });
        }

        let context = if let Some(ref memory) = self.memory {
            memory.get_context_for_query(task, 3)
        } else {
            String::new()
        };

        Ok(AgentPlan {
            task: task.to_string(),
            steps,
            context,
            created_at: Utc::now(),
        })
    }

    async fn execute_step(
        &mut self,
        step_number: usize,
        planned: &PlannedStep,
        context: &HashMap<String, String>,
        working_dir: &Path,
    ) -> AutonomousResult<AgentStep> {
        let start = std::time::Instant::now();
        let mut step = AgentStep::new(step_number, &planned.description);

        let tool_name = planned
            .tool_hint
            .clone()
            .unwrap_or_else(|| "file_list".to_string());

        let params = self.build_tool_params(&tool_name, planned, context, working_dir);

        step = step.with_tool(&tool_name, params.clone());

        let call = ToolCall::new(&tool_name, params);
        let response = self.tool_system.execute(call).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        let result_str = if response.success {
            serde_json::to_string(&response.output)
                .unwrap_or_else(|_| "Success".to_string())
                .chars()
                .take(500)
                .collect()
        } else {
            response
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        };

        Ok(step.complete(&result_str, response.success, duration_ms))
    }

    fn build_tool_params(
        &self,
        tool_name: &str,
        planned: &PlannedStep,
        _context: &HashMap<String, String>,
        working_dir: &Path,
    ) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();

        match tool_name {
            "file_read" => {
                let file_path = extract_file_hint(&planned.description)
                    .map(|f| working_dir.join(f))
                    .unwrap_or_else(|| working_dir.join("README.md"));
                params.insert("path".to_string(), serde_json::json!(file_path.to_string_lossy()));
            }
            "file_list" => {
                params.insert("path".to_string(), serde_json::json!(working_dir.to_string_lossy()));
            }
            "code_search" => {
                let pattern = extract_search_pattern(&planned.description).unwrap_or_else(|| "TODO".to_string());
                params.insert("pattern".to_string(), serde_json::json!(pattern));
                params.insert("path".to_string(), serde_json::json!(working_dir.to_string_lossy()));
            }
            "code_analyze" => {
                params.insert("path".to_string(), serde_json::json!(working_dir.to_string_lossy()));
                params.insert("analysis_type".to_string(), serde_json::json!("structure"));
            }
            "file_write" => {
                params.insert("path".to_string(), serde_json::json!(working_dir.join("output.txt").to_string_lossy()));
                params.insert("content".to_string(), serde_json::json!(""));
            }
            "shell_exec" => {
                params.insert("command".to_string(), serde_json::json!("echo 'Shell execution disabled for safety'"));
            }
            _ => {
                params.insert("path".to_string(), serde_json::json!(working_dir.to_string_lossy()));
            }
        }

        params
    }

    pub fn current_state(&self) -> Option<AgentState> {
        self.current_execution.as_ref().map(|e| e.state)
    }

    pub fn last_execution(&self) -> Option<&AgentExecution> {
        self.current_execution.as_ref()
    }
}

impl Default for AutonomousAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_file_hint(text: &str) -> Option<String> {
    let file_patterns = [
        "readme", "cargo.toml", "package.json", "config", "main.rs", "lib.rs", "mod.rs", "index.ts",
        "index.js", ".env", "dockerfile", "makefile",
    ];

    for pattern in &file_patterns {
        if text.contains(pattern) {
            return match *pattern {
                "readme" => Some("README.md".to_string()),
                "cargo.toml" => Some("Cargo.toml".to_string()),
                "package.json" => Some("package.json".to_string()),
                "config" => Some("config.toml".to_string()),
                "main.rs" => Some("src/main.rs".to_string()),
                "lib.rs" => Some("src/lib.rs".to_string()),
                "mod.rs" => Some("mod.rs".to_string()),
                "index.ts" => Some("index.ts".to_string()),
                "index.js" => Some("index.js".to_string()),
                ".env" => Some(".env".to_string()),
                "dockerfile" => Some("Dockerfile".to_string()),
                "makefile" => Some("Makefile".to_string()),
                _ => Some(pattern.to_string()),
            };
        }
    }

    None
}

fn extract_search_pattern(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let search_words = ["search", "find", "grep", "look"];

    for (i, word) in words.iter().enumerate() {
        if search_words.contains(word) {
            if let Some(next) = words.get(i + 1) {
                if !["for", "in", "the", "a"].contains(next) {
                    return Some(next.to_string());
                }
                if let Some(after) = words.get(i + 2) {
                    return Some(after.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_step_creation() {
        let step = AgentStep::new(1, "Test action");
        assert_eq!(step.step_number, 1);
        assert_eq!(step.action, "Test action");
        assert!(!step.success);
    }

    #[test]
    fn test_agent_step_with_tool() {
        let params = HashMap::new();
        let step = AgentStep::new(1, "Read file").with_tool("file_read", params);

        assert_eq!(step.tool_name, Some("file_read".to_string()));
    }

    #[test]
    fn test_extract_file_hint() {
        assert_eq!(extract_file_hint("read the readme"), Some("README.md".to_string()));
        assert_eq!(extract_file_hint("show cargo.toml"), Some("Cargo.toml".to_string()));
        assert_eq!(extract_file_hint("random text"), None);
    }

    #[test]
    fn test_extract_search_pattern() {
        assert_eq!(extract_search_pattern("search for TODO"), Some("TODO".to_string()));
        assert_eq!(extract_search_pattern("find error"), Some("error".to_string()));
    }

    #[test]
    fn test_autonomous_agent_creation() {
        let agent = AutonomousAgent::new();
        assert!(agent.current_state().is_none());
    }

    #[tokio::test]
    async fn test_agent_plan_creation() {
        let agent = AutonomousAgent::new();
        let working_dir = PathBuf::from("/tmp");

        let plan = agent.create_plan("read the readme file", &working_dir);
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert!(!plan.steps.is_empty());
    }
}
