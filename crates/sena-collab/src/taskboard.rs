use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Open,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Blocked => write!(f, "blocked"),
            Self::Done => write!(f, "done"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "in_progress" | "inprogress" | "progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "done" | "complete" | "completed" => Ok(Self::Done),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" | "l" => Ok(Self::Low),
            "medium" | "med" | "m" => Ok(Self::Medium),
            "high" | "h" => Ok(Self::High),
            "critical" | "crit" | "c" => Ok(Self::Critical),
            _ => Err(format!("Unknown priority: {}", s)),
        }
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_by: String,
    pub created_by_id: Option<Uuid>,
    pub assigned_to: Option<String>,
    pub assigned_to_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub comments: Vec<TaskComment>,
    pub blockers: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub id: Uuid,
    pub author: String,
    pub author_id: Option<Uuid>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TaskBoardState {
    tasks: HashMap<Uuid, Task>,
    by_assignee: HashMap<String, Vec<Uuid>>,
    by_status: HashMap<String, Vec<Uuid>>,
    by_tag: HashMap<String, Vec<Uuid>>,
}

pub struct TaskBoard {
    state: RwLock<TaskBoardState>,
    state_file: PathBuf,
}

impl TaskBoard {
    pub fn new() -> Self {
        let state_file = Self::default_state_file();
        let state = Self::load_state(&state_file).unwrap_or_default();
        Self {
            state: RwLock::new(state),
            state_file,
        }
    }

    pub fn with_state_file(path: PathBuf) -> Self {
        let state = Self::load_state(&path).unwrap_or_default();
        Self {
            state: RwLock::new(state),
            state_file: path,
        }
    }

    fn default_state_file() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sena")
            .join("taskboard_state.json")
    }

    fn load_state(path: &PathBuf) -> Option<TaskBoardState> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_state(&self) {
        let state = self.state.read();
        if let Some(parent) = self.state_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&*state) {
            let _ = std::fs::write(&self.state_file, content);
        }
    }

    #[allow(dead_code)]
    fn rebuild_indexes(&self) {
        let mut state = self.state.write();
        state.by_assignee.clear();
        state.by_status.clear();
        state.by_tag.clear();

        let task_info: Vec<_> = state.tasks.iter()
            .map(|(id, task)| {
                (*id, task.assigned_to.clone(), task.status.to_string(), task.tags.clone())
            })
            .collect();

        for (id, assignee, status, tags) in task_info {
            if let Some(assignee) = assignee {
                state.by_assignee
                    .entry(assignee)
                    .or_default()
                    .push(id);
            }

            state.by_status
                .entry(status)
                .or_default()
                .push(id);

            for tag in tags {
                state.by_tag
                    .entry(tag)
                    .or_default()
                    .push(id);
            }
        }
    }

    pub fn create_task(
        &self,
        title: &str,
        description: Option<&str>,
        priority: TaskPriority,
        created_by: &str,
        created_by_id: Option<Uuid>,
        assigned_to: Option<&str>,
        tags: Vec<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let task = Task {
            id,
            title: title.to_string(),
            description: description.map(String::from),
            status: TaskStatus::Open,
            priority,
            created_by: created_by.to_string(),
            created_by_id,
            assigned_to: assigned_to.map(String::from),
            assigned_to_id: None,
            created_at: now,
            updated_at: now,
            due_at: None,
            tags,
            comments: Vec::new(),
            blockers: Vec::new(),
        };

        {
            let mut state = self.state.write();
            if let Some(ref assignee) = task.assigned_to {
                state.by_assignee
                    .entry(assignee.clone())
                    .or_default()
                    .push(id);
            }
            state.by_status
                .entry(task.status.to_string())
                .or_default()
                .push(id);
            for tag in &task.tags {
                state.by_tag
                    .entry(tag.clone())
                    .or_default()
                    .push(id);
            }
            state.tasks.insert(id, task);
        }

        self.save_state();
        id
    }

    pub fn get_task(&self, id: Uuid) -> Option<Task> {
        self.state.read().tasks.get(&id).cloned()
    }

    pub fn update_status(&self, id: Uuid, status: TaskStatus) -> Result<(), String> {
        {
            let mut state = self.state.write();
            let task = state.tasks.get_mut(&id)
                .ok_or_else(|| format!("Task {} not found", id))?;

            let old_status = task.status.to_string();
            task.status = status;
            task.updated_at = Utc::now();

            if let Some(ids) = state.by_status.get_mut(&old_status) {
                ids.retain(|&tid| tid != id);
            }
            state.by_status
                .entry(status.to_string())
                .or_default()
                .push(id);
        }

        self.save_state();
        Ok(())
    }

    pub fn assign_task(&self, id: Uuid, assignee: &str) -> Result<(), String> {
        {
            let mut state = self.state.write();

            let old_assignee = state.tasks.get(&id)
                .ok_or_else(|| format!("Task {} not found", id))?
                .assigned_to.clone();

            if let Some(old) = old_assignee {
                if let Some(ids) = state.by_assignee.get_mut(&old) {
                    ids.retain(|&tid| tid != id);
                }
            }

            let task = state.tasks.get_mut(&id).unwrap();
            task.assigned_to = Some(assignee.to_string());
            task.updated_at = Utc::now();

            state.by_assignee
                .entry(assignee.to_string())
                .or_default()
                .push(id);
        }

        self.save_state();
        Ok(())
    }

    pub fn unassign_task(&self, id: Uuid) -> Result<(), String> {
        {
            let mut state = self.state.write();

            let old_assignee = state.tasks.get(&id)
                .ok_or_else(|| format!("Task {} not found", id))?
                .assigned_to.clone();

            if let Some(old) = old_assignee {
                if let Some(ids) = state.by_assignee.get_mut(&old) {
                    ids.retain(|&tid| tid != id);
                }
            }

            let task = state.tasks.get_mut(&id).unwrap();
            task.assigned_to = None;
            task.assigned_to_id = None;
            task.updated_at = Utc::now();
        }

        self.save_state();
        Ok(())
    }

    pub fn add_comment(&self, task_id: Uuid, author: &str, author_id: Option<Uuid>, content: &str) -> Result<Uuid, String> {
        let comment_id = Uuid::new_v4();

        {
            let mut state = self.state.write();
            let task = state.tasks.get_mut(&task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            task.comments.push(TaskComment {
                id: comment_id,
                author: author.to_string(),
                author_id,
                content: content.to_string(),
                created_at: Utc::now(),
            });
            task.updated_at = Utc::now();
        }

        self.save_state();
        Ok(comment_id)
    }

    pub fn add_blocker(&self, task_id: Uuid, blocker_id: Uuid) -> Result<(), String> {
        {
            let mut state = self.state.write();

            if !state.tasks.contains_key(&blocker_id) {
                return Err(format!("Blocker task {} not found", blocker_id));
            }

            let task = state.tasks.get_mut(&task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            if !task.blockers.contains(&blocker_id) {
                task.blockers.push(blocker_id);
                task.updated_at = Utc::now();
            }
        }

        self.save_state();
        Ok(())
    }

    pub fn remove_blocker(&self, task_id: Uuid, blocker_id: Uuid) -> Result<(), String> {
        {
            let mut state = self.state.write();
            let task = state.tasks.get_mut(&task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            task.blockers.retain(|&id| id != blocker_id);
            task.updated_at = Utc::now();
        }

        self.save_state();
        Ok(())
    }

    pub fn add_tag(&self, task_id: Uuid, tag: &str) -> Result<(), String> {
        {
            let mut state = self.state.write();
            let task = state.tasks.get_mut(&task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            let tag_str = tag.to_string();
            if !task.tags.contains(&tag_str) {
                task.tags.push(tag_str.clone());
                task.updated_at = Utc::now();

                state.by_tag
                    .entry(tag_str)
                    .or_default()
                    .push(task_id);
            }
        }

        self.save_state();
        Ok(())
    }

    pub fn remove_tag(&self, task_id: Uuid, tag: &str) -> Result<(), String> {
        {
            let mut state = self.state.write();
            let task = state.tasks.get_mut(&task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            task.tags.retain(|t| t != tag);
            task.updated_at = Utc::now();

            if let Some(ids) = state.by_tag.get_mut(tag) {
                ids.retain(|&id| id != task_id);
            }
        }

        self.save_state();
        Ok(())
    }

    pub fn set_priority(&self, task_id: Uuid, priority: TaskPriority) -> Result<(), String> {
        {
            let mut state = self.state.write();
            let task = state.tasks.get_mut(&task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            task.priority = priority;
            task.updated_at = Utc::now();
        }

        self.save_state();
        Ok(())
    }

    pub fn delete_task(&self, id: Uuid) -> Result<Task, String> {
        let task = {
            let mut state = self.state.write();
            let task = state.tasks.remove(&id)
                .ok_or_else(|| format!("Task {} not found", id))?;

            if let Some(ref assignee) = task.assigned_to {
                if let Some(ids) = state.by_assignee.get_mut(assignee) {
                    ids.retain(|&tid| tid != id);
                }
            }

            if let Some(ids) = state.by_status.get_mut(&task.status.to_string()) {
                ids.retain(|&tid| tid != id);
            }

            for tag in &task.tags {
                if let Some(ids) = state.by_tag.get_mut(tag) {
                    ids.retain(|&tid| tid != id);
                }
            }

            task
        };

        self.save_state();
        Ok(task)
    }

    pub fn list_all(&self) -> Vec<Task> {
        self.state.read().tasks.values().cloned().collect()
    }

    pub fn list_by_status(&self, status: TaskStatus) -> Vec<Task> {
        let state = self.state.read();
        state.by_status
            .get(&status.to_string())
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| state.tasks.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn list_by_assignee(&self, assignee: &str) -> Vec<Task> {
        let state = self.state.read();
        state.by_assignee
            .get(assignee)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| state.tasks.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn list_by_tag(&self, tag: &str) -> Vec<Task> {
        let state = self.state.read();
        state.by_tag
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| state.tasks.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn list_open(&self) -> Vec<Task> {
        let state = self.state.read();
        let mut tasks: Vec<Task> = state.tasks
            .values()
            .filter(|t| matches!(t.status, TaskStatus::Open | TaskStatus::InProgress))
            .cloned()
            .collect();

        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        tasks
    }

    pub fn count_by_status(&self) -> HashMap<String, usize> {
        let state = self.state.read();
        state.by_status
            .iter()
            .map(|(status, ids)| (status.clone(), ids.len()))
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<Task> {
        let query_lower = query.to_lowercase();
        self.state.read().tasks
            .values()
            .filter(|task| {
                task.title.to_lowercase().contains(&query_lower)
                    || task.description.as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || task.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }
}

impl Default for TaskBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_board() -> TaskBoard {
        let temp_file = PathBuf::from("/tmp/sena_taskboard_test.json");
        let _ = std::fs::remove_file(&temp_file);
        TaskBoard::with_state_file(temp_file)
    }

    #[test]
    fn test_create_and_get_task() {
        let board = test_board();

        let id = board.create_task(
            "Implement feature X",
            Some("This is a detailed description"),
            TaskPriority::High,
            "BackendDev",
            None,
            Some("AndroidDev"),
            vec!["backend".to_string(), "api".to_string()],
        );

        let task = board.get_task(id).unwrap();
        assert_eq!(task.title, "Implement feature X");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(task.assigned_to, Some("AndroidDev".to_string()));
    }

    #[test]
    fn test_update_status() {
        let board = test_board();

        let id = board.create_task(
            "Fix bug",
            None,
            TaskPriority::Critical,
            "QA",
            None,
            None,
            vec![],
        );

        board.update_status(id, TaskStatus::InProgress).unwrap();
        let task = board.get_task(id).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);

        board.update_status(id, TaskStatus::Done).unwrap();
        let task = board.get_task(id).unwrap();
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn test_assign_task() {
        let board = test_board();

        let id = board.create_task(
            "Review PR",
            None,
            TaskPriority::Medium,
            "Lead",
            None,
            None,
            vec![],
        );

        board.assign_task(id, "BackendDev").unwrap();
        let task = board.get_task(id).unwrap();
        assert_eq!(task.assigned_to, Some("BackendDev".to_string()));

        let assigned = board.list_by_assignee("BackendDev");
        assert_eq!(assigned.len(), 1);
    }

    #[test]
    fn test_comments() {
        let board = test_board();

        let id = board.create_task(
            "Discuss architecture",
            None,
            TaskPriority::Low,
            "Architect",
            None,
            None,
            vec![],
        );

        board.add_comment(id, "BackendDev", None, "I think we should use microservices").unwrap();
        board.add_comment(id, "FrontendDev", None, "Agreed, but let's keep it simple").unwrap();

        let task = board.get_task(id).unwrap();
        assert_eq!(task.comments.len(), 2);
    }

    #[test]
    fn test_list_by_status() {
        let board = test_board();

        board.create_task("Task 1", None, TaskPriority::Low, "Dev", None, None, vec![]);
        let id2 = board.create_task("Task 2", None, TaskPriority::Medium, "Dev", None, None, vec![]);
        board.create_task("Task 3", None, TaskPriority::High, "Dev", None, None, vec![]);

        board.update_status(id2, TaskStatus::InProgress).unwrap();

        let open = board.list_by_status(TaskStatus::Open);
        assert_eq!(open.len(), 2);

        let in_progress = board.list_by_status(TaskStatus::InProgress);
        assert_eq!(in_progress.len(), 1);
    }

    #[test]
    fn test_search() {
        let board = test_board();

        board.create_task("API endpoint for users", None, TaskPriority::High, "Dev", None, None, vec!["api".to_string()]);
        board.create_task("Frontend button fix", None, TaskPriority::Low, "Dev", None, None, vec!["ui".to_string()]);
        board.create_task("API rate limiting", None, TaskPriority::Critical, "Dev", None, None, vec!["api".to_string()]);

        let results = board.search("api");
        assert_eq!(results.len(), 2);

        let results = board.search("button");
        assert_eq!(results.len(), 1);

        let results = board.search("endpoint");
        assert_eq!(results.len(), 1);
    }
}
