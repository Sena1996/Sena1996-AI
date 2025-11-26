//! SENA Collaboration Hub v1.0
//!
//! Multi-session collaboration system for SENA v10.0.0
//! Enables Android Claude, Web Claude, Backend Claude, IoT Claude to work together
//!
//! Features:
//! - Real-time session discovery and communication
//! - Task management across sessions
//! - Live state synchronization via CRDT
//! - Conflict detection and warnings
//! - Lightning-fast Unix socket messaging

pub mod session;
pub mod state;
pub mod tasks;
pub mod messages;
pub mod conflicts;
pub mod socket;

pub use session::{Session, SessionRegistry, SessionRole, SessionStatus};
pub use state::{HubState, SharedState};
pub use tasks::{Task, TaskBoard, TaskPriority, TaskStatus};
pub use messages::{Message, MessageQueue, Broadcast};
pub use conflicts::{ConflictDetector, FileConflict};
pub use socket::{HubServer, HubClient};

use std::path::PathBuf;
use std::fs;

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
        fs::create_dir_all(&self.hub_dir)
            .map_err(|e| format!("Cannot create hub dir: {}", e))?;
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
}

impl Hub {
    /// Create a new Hub instance
    pub fn new() -> Result<Self, String> {
        let config = HubConfig::new();
        config.ensure_dirs()?;

        Ok(Self {
            sessions: SessionRegistry::new(&config),
            state: HubState::new(&config),
            tasks: TaskBoard::new(&config),
            messages: MessageQueue::new(&config),
            conflicts: ConflictDetector::new(),
            config,
        })
    }

    /// Join the hub as a specific role
    pub fn join(&mut self, role: SessionRole, name: Option<String>) -> Result<Session, String> {
        let session = self.sessions.register(role, name)?;
        self.state.set_session_active(&session.id, true);
        Ok(session)
    }

    /// Leave the hub
    pub fn leave(&mut self, session_id: &str) -> Result<(), String> {
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

    /// Create a new task
    pub fn create_task(&mut self, title: &str, assignee: &str, priority: TaskPriority) -> Result<Task, String> {
        self.tasks.create(title, assignee, priority)
    }

    /// Get all tasks
    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.get_all()
    }

    /// Get tasks for a specific session
    pub fn get_my_tasks(&self, session_id: &str) -> Vec<Task> {
        self.tasks.get_for_session(session_id)
    }

    /// Update task status
    pub fn update_task(&mut self, task_id: u64, status: TaskStatus) -> Result<(), String> {
        self.tasks.update_status(task_id, status)
    }

    /// Set working state for a session
    pub fn set_working_on(&mut self, session_id: &str, file_path: &str) -> Result<(), String> {
        // Check for conflicts first
        if let Some(conflict) = self.conflicts.check_file(file_path, session_id, &self.state) {
            // Still allow but return warning
            eprintln!("⚠️  Warning: {} is also editing {}", conflict.other_session, file_path);
        }

        self.state.set_working_on(session_id, file_path);
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

    /// Load state from disk
    pub fn load(&mut self) -> Result<(), String> {
        self.sessions.load()?;
        self.state.load()?;
        self.tasks.load()?;
        Ok(())
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new().expect("Failed to create hub")
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
