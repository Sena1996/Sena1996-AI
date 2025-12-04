//! SENA Collaboration Hub v2.0
//!
//! Multi-session collaboration system with Cross-Hub Federation
//! Enables Android Claude, Web Claude, Backend Claude, IoT Claude to work together
//!
//! Features:
//! - Real-time session discovery and communication
//! - Task management across sessions
//! - Live state synchronization via CRDT
//! - Conflict detection and warnings
//! - Lightning-fast Unix socket messaging
//! - Cross-hub peer federation (v2.0)
//! - Hub identity with persistent UUID
//! - Auth passkey for secure hub connections
//! - Federated sessions across multiple hubs

pub mod conflicts;
pub mod context;
pub mod identity;
pub mod messages;
pub mod peers;
pub mod session;
#[cfg(unix)]
pub mod socket;
pub mod state;
pub mod tasks;

pub use conflicts::{ConflictDetector, FileConflict};
pub use context::{ContextManager, SessionContext};
pub use identity::{ConnectedHub, ConnectionRequest, DiscoveredHub, HubIdentity};
pub use messages::{Broadcast, Message, MessageQueue};
pub use peers::{FederatedSession, PeerManager, RemoteSession, ResolvedTarget};
pub use session::{Session, SessionRegistry, SessionRole, SessionStatus};
#[cfg(unix)]
pub use socket::{HubClient, HubServer};
pub use state::{HubState, SharedState};
pub use tasks::{Task, TaskBoard, TaskPriority, TaskStatus};

use std::fs;
use std::path::PathBuf;

/// Hub configuration
pub struct HubConfig {
    pub hub_dir: PathBuf,
    pub socket_path: PathBuf,
    pub state_file: PathBuf,
    pub tasks_file: PathBuf,
    pub messages_dir: PathBuf,
}

impl HubConfig {
    /// Create hub config with default paths
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let hub_dir = home.join(".claude").join("hub");

        Self {
            socket_path: hub_dir.join("hub.sock"),
            state_file: hub_dir.join("state.json"),
            tasks_file: hub_dir.join("tasks.json"),
            messages_dir: hub_dir.join("messages"),
            hub_dir,
        }
    }

    /// Ensure all hub directories exist
    pub fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(&self.hub_dir).map_err(|e| format!("Cannot create hub dir: {}", e))?;
        fs::create_dir_all(&self.messages_dir)
            .map_err(|e| format!("Cannot create messages dir: {}", e))?;
        Ok(())
    }
}

impl Default for HubConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Main Hub controller
pub struct Hub {
    pub config: HubConfig,
    pub sessions: SessionRegistry,
    pub state: HubState,
    pub tasks: TaskBoard,
    pub messages: MessageQueue,
    pub conflicts: ConflictDetector,
    pub context: ContextManager,
}

impl Hub {
    pub fn new() -> Result<Self, String> {
        let config = HubConfig::new();
        config.ensure_dirs()?;

        Ok(Self {
            sessions: SessionRegistry::new(&config),
            state: HubState::new(&config),
            tasks: TaskBoard::new(&config),
            messages: MessageQueue::new(&config),
            conflicts: ConflictDetector::new(),
            context: ContextManager::new(&config),
            config,
        })
    }

    pub fn join(&mut self, role: SessionRole, name: Option<String>) -> Result<Session, String> {
        let session = self.sessions.register(role, name.clone())?;
        self.state.set_session_active(&session.id, true);

        let context = SessionContext::new(&session.id, &session.name, role.name());
        self.context.save_context(&context)?;

        Ok(session)
    }

    pub fn get_current_session_id(&self) -> Option<String> {
        self.context.get_current_session_id()
    }

    pub fn get_current_session(&self) -> Option<Session> {
        self.get_current_session_id()
            .and_then(|id| self.sessions.get(&id).cloned())
    }

    pub fn leave(&mut self, session_id: &str) -> Result<(), String> {
        if let Some(current_id) = self.get_current_session_id() {
            if current_id == session_id {
                let _ = self.context.clear_current_context();
            }
        }
        self.sessions.unregister(session_id)?;
        self.state.set_session_active(session_id, false);
        Ok(())
    }

    /// Get all online sessions
    pub fn who(&self) -> Vec<Session> {
        self.sessions.get_active()
    }

    /// Send message to a specific session
    pub fn tell(&mut self, from: &str, to: &str, message: &str) -> Result<(), String> {
        self.messages.send(from, to, message)
    }

    /// Broadcast message to all sessions
    pub fn broadcast(&mut self, from: &str, message: &str) -> Result<(), String> {
        self.messages.broadcast(from, message)
    }

    /// Get messages for a session
    pub fn inbox(&self, session_id: &str) -> Vec<Message> {
        self.messages.get_inbox(session_id)
    }

    /// Create a new task and broadcast to all sessions
    pub fn create_task(
        &mut self,
        title: &str,
        assignee: &str,
        priority: TaskPriority,
    ) -> Result<Task, String> {
        let task = self.tasks.create(title, assignee, priority)?;
        self.broadcast_task_update(&task, "created")?;
        Ok(task)
    }

    /// Create task from specific session and broadcast
    pub fn create_task_from(
        &mut self,
        title: &str,
        assignee: &str,
        creator: &str,
        priority: TaskPriority,
    ) -> Result<Task, String> {
        let task = self.tasks.create_from(title, assignee, creator, priority)?;
        self.broadcast_task_update(&task, "created")?;
        Ok(task)
    }

    /// Get all tasks
    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.get_all()
    }

    /// Get tasks for a specific session
    pub fn get_my_tasks(&self, session_id: &str) -> Vec<Task> {
        self.tasks.get_for_session(session_id)
    }

    /// Update task status and broadcast to all sessions
    pub fn update_task(&mut self, task_id: u64, status: TaskStatus) -> Result<(), String> {
        self.tasks.update_status(task_id, status)?;
        let task_info = self.tasks.get(task_id).cloned();
        if let Some(task) = task_info {
            self.broadcast_task_update(&task, "updated")?;
        }
        Ok(())
    }

    /// Reassign task and broadcast
    pub fn reassign_task(&mut self, task_id: u64, new_assignee: &str) -> Result<(), String> {
        self.tasks.reassign(task_id, new_assignee)?;
        let task_info = self.tasks.get(task_id).cloned();
        if let Some(task) = task_info {
            self.broadcast_task_update(&task, "reassigned")?;
        }
        Ok(())
    }

    /// Broadcast task update to all collab sessions
    fn broadcast_task_update(&mut self, task: &Task, action: &str) -> Result<(), String> {
        let message = format!(
            "[Task {}] #{} {} - {} ({})",
            action,
            task.id,
            task.priority.emoji(),
            task.title,
            task.status.name()
        );
        self.messages.broadcast("hub", &message)
    }

    /// Set working state for a session
    pub fn set_working_on(&mut self, session_id: &str, file_path: &str) -> Result<(), String> {
        if let Some(conflict) = self
            .conflicts
            .check_file(file_path, session_id, &self.state)
        {
            eprintln!(
                "⚠️  Warning: {} is also editing {}",
                conflict.other_session, file_path
            );
        }

        self.state.set_working_on(session_id, file_path);
        self.state.save()?;
        Ok(())
    }

    /// Get current conflicts
    pub fn get_conflicts(&self) -> Vec<FileConflict> {
        self.conflicts.get_all(&self.state)
    }

    /// Get hub status summary
    pub fn status(&self) -> HubStatus {
        HubStatus {
            online_sessions: self.sessions.get_active().len(),
            total_tasks: self.tasks.get_all().len(),
            pending_tasks: self.tasks.get_by_status(TaskStatus::Pending).len(),
            active_conflicts: self.get_conflicts().len(),
            sessions: self.sessions.get_active(),
        }
    }

    /// Save all state to disk
    pub fn save(&self) -> Result<(), String> {
        self.sessions.save()?;
        self.state.save()?;
        self.tasks.save()?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        self.sessions.load()?;
        self.state.load()?;
        self.tasks.load()?;
        self.messages.load()?;
        Ok(())
    }
}

impl Default for Hub {
    fn default() -> Self {
        match Self::new() {
            Ok(hub) => hub,
            Err(e) => {
                eprintln!("Warning: Failed to create hub with defaults: {}", e);
                let config = HubConfig::new();
                Self {
                    sessions: SessionRegistry::new(&config),
                    state: HubState::new(&config),
                    tasks: TaskBoard::new(&config),
                    messages: MessageQueue::new(&config),
                    conflicts: ConflictDetector::new(),
                    context: ContextManager::new(&config),
                    config,
                }
            }
        }
    }
}

/// Hub status summary
#[derive(Debug, Clone)]
pub struct HubStatus {
    pub online_sessions: usize,
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub active_conflicts: usize,
    pub sessions: Vec<Session>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_config_creation() {
        let config = HubConfig::new();
        assert!(config.hub_dir.to_string_lossy().contains("hub"));
    }

    #[test]
    fn test_hub_creation() {
        let hub = Hub::new();
        assert!(hub.is_ok());
    }
}
