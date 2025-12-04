use crate::intelligence::autonomous::{AgentExecution, AutonomousAgent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTask {
    pub id: String,
    pub name: String,
    pub task_description: String,
    pub working_dir: PathBuf,
    pub max_steps: usize,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution: Option<AgentExecution>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl BackgroundTask {
    pub fn new(name: &str, task_description: &str, working_dir: PathBuf, max_steps: usize) -> Self {
        Self {
            id: Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("task")
                .to_string(),
            name: name.to_string(),
            task_description: task_description.to_string(),
            working_dir,
            max_steps,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            execution: None,
            error: None,
        }
    }

    pub fn elapsed_ms(&self) -> Option<u64> {
        let start = self.started_at?;
        let end = self.completed_at.unwrap_or_else(Utc::now);
        Some((end - start).num_milliseconds() as u64)
    }
}

pub enum TaskCommand {
    Submit(Box<BackgroundTask>),
    Cancel(String),
    Status(String),
    List,
    Shutdown,
}

pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub execution: Option<AgentExecution>,
    pub error: Option<String>,
}

pub struct BackgroundAgentManager {
    tasks: Arc<RwLock<HashMap<String, BackgroundTask>>>,
    command_tx: Option<mpsc::Sender<TaskCommand>>,
    result_rx: Option<mpsc::Receiver<TaskResult>>,
}

impl BackgroundAgentManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            command_tx: None,
            result_rx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<TaskCommand>(100);
        let (result_tx, result_rx) = mpsc::channel::<TaskResult>(100);

        self.command_tx = Some(cmd_tx);
        self.result_rx = Some(result_rx);

        let tasks = Arc::clone(&self.tasks);

        tokio::spawn(async move {
            while let Some(command) = cmd_rx.recv().await {
                match command {
                    TaskCommand::Submit(boxed_task) => {
                        let mut task = *boxed_task;
                        let task_id = task.id.clone();
                        task.status = TaskStatus::Running;
                        task.started_at = Some(Utc::now());

                        {
                            let mut tasks_guard = tasks.write().unwrap();
                            tasks_guard.insert(task_id.clone(), task.clone());
                        }

                        let tasks_clone = Arc::clone(&tasks);
                        let result_tx_clone = result_tx.clone();

                        tokio::spawn(async move {
                            let mut agent = AutonomousAgent::new();
                            let exec_result = agent
                                .execute(
                                    &task.task_description,
                                    task.working_dir.clone(),
                                    task.max_steps,
                                    false,
                                )
                                .await;

                            let (status, execution, error) = match exec_result {
                                Ok(exec) => (TaskStatus::Completed, Some(exec), None),
                                Err(e) => (TaskStatus::Failed, None, Some(e.to_string())),
                            };

                            {
                                let mut tasks_guard = tasks_clone.write().unwrap();
                                if let Some(t) = tasks_guard.get_mut(&task_id) {
                                    t.status = status;
                                    t.completed_at = Some(Utc::now());
                                    t.execution = execution.clone();
                                    t.error = error.clone();
                                }
                            }

                            let _ = result_tx_clone
                                .send(TaskResult {
                                    task_id,
                                    status,
                                    execution,
                                    error,
                                })
                                .await;
                        });
                    }

                    TaskCommand::Cancel(task_id) => {
                        let mut tasks_guard = tasks.write().unwrap();
                        if let Some(task) = tasks_guard.get_mut(&task_id) {
                            if task.status == TaskStatus::Pending
                                || task.status == TaskStatus::Running
                            {
                                task.status = TaskStatus::Cancelled;
                                task.completed_at = Some(Utc::now());
                            }
                        }
                    }

                    TaskCommand::Status(_task_id) => {}

                    TaskCommand::List => {}

                    TaskCommand::Shutdown => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn submit_task(
        &self,
        name: &str,
        task_description: &str,
        working_dir: PathBuf,
        max_steps: usize,
    ) -> Result<String, String> {
        let task = BackgroundTask::new(name, task_description, working_dir, max_steps);
        let task_id = task.id.clone();

        if let Some(ref tx) = self.command_tx {
            tx.send(TaskCommand::Submit(Box::new(task)))
                .await
                .map_err(|e| format!("Failed to submit task: {}", e))?;
        } else {
            return Err("Background manager not started".to_string());
        }

        Ok(task_id)
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<(), String> {
        if let Some(ref tx) = self.command_tx {
            tx.send(TaskCommand::Cancel(task_id.to_string()))
                .await
                .map_err(|e| format!("Failed to cancel task: {}", e))?;
        }
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Option<BackgroundTask> {
        let tasks = self.tasks.read().ok()?;
        tasks.get(task_id).cloned()
    }

    pub fn list_tasks(&self) -> Vec<BackgroundTask> {
        let tasks = match self.tasks.read() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        tasks.values().cloned().collect()
    }

    pub fn pending_count(&self) -> usize {
        let tasks = match self.tasks.read() {
            Ok(t) => t,
            Err(_) => return 0,
        };
        tasks
            .values()
            .filter(|t| t.status == TaskStatus::Pending)
            .count()
    }

    pub fn running_count(&self) -> usize {
        let tasks = match self.tasks.read() {
            Ok(t) => t,
            Err(_) => return 0,
        };
        tasks
            .values()
            .filter(|t| t.status == TaskStatus::Running)
            .count()
    }

    pub fn completed_count(&self) -> usize {
        let tasks = match self.tasks.read() {
            Ok(t) => t,
            Err(_) => return 0,
        };
        tasks
            .values()
            .filter(|t| t.status == TaskStatus::Completed)
            .count()
    }

    pub async fn poll_results(&mut self) -> Vec<TaskResult> {
        let mut results = Vec::new();

        if let Some(ref mut rx) = self.result_rx {
            while let Ok(result) = rx.try_recv() {
                results.push(result);
            }
        }

        results
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        if let Some(ref tx) = self.command_tx {
            tx.send(TaskCommand::Shutdown)
                .await
                .map_err(|e| format!("Failed to send shutdown: {}", e))?;
        }
        Ok(())
    }
}

impl Default for BackgroundAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TaskQueue {
    tasks: Vec<BackgroundTask>,
    max_concurrent: usize,
}

impl TaskQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            tasks: Vec::new(),
            max_concurrent,
        }
    }

    pub fn enqueue(&mut self, task: BackgroundTask) {
        self.tasks.push(task);
    }

    pub fn dequeue(&mut self) -> Option<BackgroundTask> {
        let running = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Running)
            .count();

        if running >= self.max_concurrent {
            return None;
        }

        let pending_idx = self
            .tasks
            .iter()
            .position(|t| t.status == TaskStatus::Pending)?;

        Some(self.tasks.remove(pending_idx))
    }

    pub fn mark_completed(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Utc::now());
        }
    }

    pub fn mark_failed(&mut self, task_id: &str, error: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Failed;
            task.completed_at = Some(Utc::now());
            task.error = Some(error.to_string());
        }
    }

    pub fn pending_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count()
    }

    pub fn running_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Running)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_task_creation() {
        let task = BackgroundTask::new("test", "Read files", PathBuf::from("/tmp"), 5);

        assert_eq!(task.name, "test");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.started_at.is_none());
    }

    #[test]
    fn test_task_queue() {
        let mut queue = TaskQueue::new(2);

        queue.enqueue(BackgroundTask::new(
            "task1",
            "Test 1",
            PathBuf::from("/tmp"),
            5,
        ));
        queue.enqueue(BackgroundTask::new(
            "task2",
            "Test 2",
            PathBuf::from("/tmp"),
            5,
        ));

        assert_eq!(queue.pending_count(), 2);

        let task = queue.dequeue();
        assert!(task.is_some());
    }

    #[tokio::test]
    async fn test_background_manager_creation() {
        let manager = BackgroundAgentManager::new();
        assert_eq!(manager.pending_count(), 0);
        assert_eq!(manager.running_count(), 0);
    }
}
