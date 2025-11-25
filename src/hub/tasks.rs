//! Task Management System
//!
//! Cross-session task board for collaboration

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use super::HubConfig;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl TaskPriority {
    pub fn emoji(&self) -> &'static str {
        match self {
            TaskPriority::Critical => "🔥",
            TaskPriority::High => "🔴",
            TaskPriority::Medium => "🟡",
            TaskPriority::Low => "🟢",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TaskPriority::Critical => "CRITICAL",
            TaskPriority::High => "HIGH",
            TaskPriority::Medium => "MEDIUM",
            TaskPriority::Low => "LOW",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" | "crit" => TaskPriority::Critical,
            "high" | "h" => TaskPriority::High,
            "medium" | "med" | "m" => TaskPriority::Medium,
            "low" | "l" => TaskPriority::Low,
            _ => TaskPriority::Medium,
        }
    }
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "⏳",
            TaskStatus::InProgress => "🔄",
            TaskStatus::Blocked => "🚫",
            TaskStatus::Done => "✅",
            TaskStatus::Cancelled => "❌",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Blocked => "Blocked",
            TaskStatus::Done => "Done",
            TaskStatus::Cancelled => "Cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" | "p" => TaskStatus::Pending,
            "inprogress" | "in_progress" | "progress" | "ip" => TaskStatus::InProgress,
            "blocked" | "b" => TaskStatus::Blocked,
            "done" | "d" | "complete" | "completed" => TaskStatus::Done,
            "cancelled" | "cancel" | "c" => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }
}

/// A collaborative task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub assignee: String,
    pub creator: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub completed_at: Option<u64>,
    pub tags: Vec<String>,
    pub blockers: Vec<String>,
}

impl Task {
    /// Create a new task
    pub fn new(id: u64, title: &str, assignee: &str, creator: &str, priority: TaskPriority) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id,
            title: title.to_string(),
            description: None,
            assignee: assignee.to_string(),
            creator: creator.to_string(),
            priority,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            completed_at: None,
            tags: Vec::new(),
            blockers: Vec::new(),
        }
    }

    /// Check if task is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, TaskStatus::Done | TaskStatus::Cancelled)
    }

    /// Format for display
    pub fn display_line(&self) -> String {
        format!(
            "#{} │ {} {} │ {} │ {} │ {}",
            self.id,
            self.priority.emoji(),
            self.priority.name(),
            self.assignee,
            self.title,
            self.status.name()
        )
    }
}

/// Persisted tasks data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TasksData {
    version: String,
    next_id: u64,
    tasks: HashMap<u64, Task>,
    last_updated: u64,
}

/// Task Board
pub struct TaskBoard {
    tasks: HashMap<u64, Task>,
    next_id: AtomicU64,
    tasks_file: PathBuf,
}

impl TaskBoard {
    /// Create a new task board
    pub fn new(config: &HubConfig) -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: AtomicU64::new(1),
            tasks_file: config.tasks_file.clone(),
        }
    }

    /// Create a new task
    pub fn create(&mut self, title: &str, assignee: &str, priority: TaskPriority) -> Result<Task, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let creator = "hub"; // TODO: Get from context

        let task = Task::new(id, title, assignee, creator, priority);
        self.tasks.insert(id, task.clone());
        self.save()?;

        Ok(task)
    }

    /// Create task with specific creator
    pub fn create_from(&mut self, title: &str, assignee: &str, creator: &str, priority: TaskPriority) -> Result<Task, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let task = Task::new(id, title, assignee, creator, priority);
        self.tasks.insert(id, task.clone());
        self.save()?;

        Ok(task)
    }

    /// Get a task by ID
    pub fn get(&self, id: u64) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// Get all tasks
    pub fn get_all(&self) -> Vec<Task> {
        let mut tasks: Vec<Task> = self.tasks.values().cloned().collect();
        tasks.sort_by(|a, b| {
            // Sort by priority (critical first), then by created_at
            match (a.priority, b.priority) {
                (TaskPriority::Critical, TaskPriority::Critical) => a.created_at.cmp(&b.created_at),
                (TaskPriority::Critical, _) => std::cmp::Ordering::Less,
                (_, TaskPriority::Critical) => std::cmp::Ordering::Greater,
                (TaskPriority::High, TaskPriority::High) => a.created_at.cmp(&b.created_at),
                (TaskPriority::High, _) => std::cmp::Ordering::Less,
                (_, TaskPriority::High) => std::cmp::Ordering::Greater,
                (TaskPriority::Medium, TaskPriority::Medium) => a.created_at.cmp(&b.created_at),
                (TaskPriority::Medium, _) => std::cmp::Ordering::Less,
                (_, TaskPriority::Medium) => std::cmp::Ordering::Greater,
                (TaskPriority::Low, TaskPriority::Low) => a.created_at.cmp(&b.created_at),
            }
        });
        tasks
    }

    /// Get tasks for a specific session/assignee
    pub fn get_for_session(&self, session_id: &str) -> Vec<Task> {
        self.tasks
            .values()
            .filter(|t| t.assignee == session_id || t.assignee.contains(session_id))
            .cloned()
            .collect()
    }

    /// Get tasks by status
    pub fn get_by_status(&self, status: TaskStatus) -> Vec<Task> {
        self.tasks
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    /// Get pending tasks
    pub fn get_pending(&self) -> Vec<Task> {
        self.get_by_status(TaskStatus::Pending)
    }

    /// Get in-progress tasks
    pub fn get_in_progress(&self) -> Vec<Task> {
        self.get_by_status(TaskStatus::InProgress)
    }

    /// Update task status
    pub fn update_status(&mut self, id: u64, status: TaskStatus) -> Result<(), String> {
        let task = self.tasks.get_mut(&id)
            .ok_or_else(|| format!("Task #{} not found", id))?;

        task.status = status;
        task.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if matches!(status, TaskStatus::Done) {
            task.completed_at = Some(task.updated_at);
        }

        self.save()?;
        Ok(())
    }

    /// Update task assignee
    pub fn reassign(&mut self, id: u64, new_assignee: &str) -> Result<(), String> {
        let task = self.tasks.get_mut(&id)
            .ok_or_else(|| format!("Task #{} not found", id))?;

        task.assignee = new_assignee.to_string();
        task.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.save()?;
        Ok(())
    }

    /// Add description to task
    pub fn set_description(&mut self, id: u64, description: &str) -> Result<(), String> {
        let task = self.tasks.get_mut(&id)
            .ok_or_else(|| format!("Task #{} not found", id))?;

        task.description = Some(description.to_string());
        task.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.save()?;
        Ok(())
    }

    /// Delete a task
    pub fn delete(&mut self, id: u64) -> Result<(), String> {
        self.tasks.remove(&id)
            .ok_or_else(|| format!("Task #{} not found", id))?;
        self.save()?;
        Ok(())
    }

    /// Save tasks to disk
    pub fn save(&self) -> Result<(), String> {
        let data = TasksData {
            version: "7.0.0".to_string(),
            next_id: self.next_id.load(Ordering::SeqCst),
            tasks: self.tasks.clone(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Cannot serialize tasks: {}", e))?;

        // Ensure parent directory exists
        if let Some(parent) = self.tasks_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create tasks directory: {}", e))?;
        }

        fs::write(&self.tasks_file, json)
            .map_err(|e| format!("Cannot write tasks file: {}", e))?;

        Ok(())
    }

    /// Load tasks from disk
    pub fn load(&mut self) -> Result<(), String> {
        if !self.tasks_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.tasks_file)
            .map_err(|e| format!("Cannot read tasks file: {}", e))?;

        let data: TasksData = serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse tasks file: {}", e))?;

        self.tasks = data.tasks;
        self.next_id.store(data.next_id, Ordering::SeqCst);

        Ok(())
    }

    /// Get task count
    pub fn count(&self) -> usize {
        self.tasks.len()
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.tasks.values().filter(|t| t.status == TaskStatus::Pending).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority() {
        assert_eq!(TaskPriority::Critical.emoji(), "🔥");
        assert_eq!(TaskPriority::from_str("high"), TaskPriority::High);
    }

    #[test]
    fn test_task_status() {
        assert_eq!(TaskStatus::Done.emoji(), "✅");
        assert_eq!(TaskStatus::from_str("done"), TaskStatus::Done);
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(1, "Fix bug", "backend", "web", TaskPriority::High);
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Fix bug");
        assert_eq!(task.assignee, "backend");
    }

    #[test]
    fn test_task_board() {
        let config = HubConfig::new();
        let mut board = TaskBoard::new(&config);

        let task = board.create("Test task", "web", TaskPriority::Medium);
        assert!(task.is_ok());
        assert_eq!(board.count(), 1);
    }
}
