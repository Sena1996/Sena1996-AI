//! Unix Socket Communication
//!
//! Lightning-fast IPC for real-time session communication

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::tasks::TaskPriority;
use super::{Hub, HubConfig, SessionRole};

/// Hub command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HubCommand {
    // Session commands
    Join {
        role: String,
        name: Option<String>,
    },
    Leave {
        session_id: String,
    },
    Heartbeat {
        session_id: String,
    },
    Who,

    // Message commands
    Tell {
        from: String,
        to: String,
        message: String,
    },
    Broadcast {
        from: String,
        message: String,
    },
    GetInbox {
        session_id: String,
    },

    // Task commands
    CreateTask {
        title: String,
        assignee: String,
        priority: String,
    },
    ListTasks,
    UpdateTask {
        id: u64,
        status: String,
    },

    // State commands
    SetWorkingOn {
        session_id: String,
        file_path: String,
    },
    ClearWorkingOn {
        session_id: String,
    },
    GetState,
    GetConflicts,

    // System commands
    Status,
    Ping,
    Shutdown,
}

/// Hub response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl HubResponse {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn ok_with_data(message: &str, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn pong() -> Self {
        Self::ok("pong")
    }
}

/// Hub Server (runs as daemon)
pub struct HubServer {
    socket_path: PathBuf,
    running: Arc<Mutex<bool>>,
    hub: Arc<Mutex<Hub>>,
}

impl HubServer {
    pub fn new(config: &HubConfig) -> Result<Self, String> {
        let hub = Hub::new()?;
        Ok(Self {
            socket_path: config.socket_path.clone(),
            running: Arc::new(Mutex::new(false)),
            hub: Arc::new(Mutex::new(hub)),
        })
    }

    pub fn with_hub(config: &HubConfig, hub: Hub) -> Self {
        Self {
            socket_path: config.socket_path.clone(),
            running: Arc::new(Mutex::new(false)),
            hub: Arc::new(Mutex::new(hub)),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)
                .map_err(|e| format!("Cannot remove old socket: {}", e))?;
        }

        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| format!("Cannot bind socket: {}", e))?;

        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Cannot set non-blocking: {}", e))?;

        *self.running.lock().expect("running lock poisoned") = true;

        eprintln!("Hub server listening on {:?}", self.socket_path);

        while *self.running.lock().expect("running lock poisoned") {
            match listener.accept() {
                Ok((stream, _)) => {
                    let hub_clone = Arc::clone(&self.hub);
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, hub_clone) {
                            eprintln!("Client error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }

        let _ = fs::remove_file(&self.socket_path);

        Ok(())
    }

    pub fn stop(&self) {
        *self.running.lock().expect("running lock poisoned") = false;
    }

    fn handle_client(mut stream: UnixStream, hub: Arc<Mutex<Hub>>) -> Result<(), String> {
        let reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;

            let command: HubCommand =
                serde_json::from_str(&line).map_err(|e| format!("Invalid command: {}", e))?;

            let response = Self::process_command(command, &hub);

            let response_json = serde_json::to_string(&response)
                .map_err(|e| format!("Cannot serialize response: {}", e))?;

            writeln!(stream, "{}", response_json)
                .map_err(|e| format!("Cannot write response: {}", e))?;

            stream.flush().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn process_command(command: HubCommand, hub: &Arc<Mutex<Hub>>) -> HubResponse {
        let mut hub_guard = match hub.lock() {
            Ok(guard) => guard,
            Err(_) => return HubResponse::error("Hub lock poisoned"),
        };

        match command {
            HubCommand::Ping => HubResponse::pong(),

            HubCommand::Status => {
                let status = hub_guard.status();
                HubResponse::ok_with_data(
                    "Hub status",
                    serde_json::json!({
                        "running": true,
                        "version": crate::VERSION,
                        "online_sessions": status.online_sessions,
                        "total_tasks": status.total_tasks,
                        "pending_tasks": status.pending_tasks,
                        "active_conflicts": status.active_conflicts
                    }),
                )
            }

            HubCommand::Who => {
                let sessions = hub_guard.who();
                let session_data: Vec<serde_json::Value> = sessions
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "role": format!("{:?}", s.role),
                            "name": s.name,
                            "status": format!("{:?}", s.status)
                        })
                    })
                    .collect();
                HubResponse::ok_with_data(
                    "Active sessions",
                    serde_json::json!({ "sessions": session_data }),
                )
            }

            HubCommand::Join { role, name } => {
                let session_role = match role.to_lowercase().as_str() {
                    "android" => SessionRole::Android,
                    "web" => SessionRole::Web,
                    "backend" => SessionRole::Backend,
                    "iot" => SessionRole::IoT,
                    _ => {
                        return HubResponse::error("Invalid role. Use: android, web, backend, iot")
                    }
                };
                match hub_guard.join(session_role, name) {
                    Ok(session) => HubResponse::ok_with_data(
                        "Joined hub",
                        serde_json::json!({ "session_id": session.id, "role": format!("{:?}", session.role) }),
                    ),
                    Err(e) => HubResponse::error(&e),
                }
            }

            HubCommand::Leave { session_id } => match hub_guard.leave(&session_id) {
                Ok(()) => HubResponse::ok("Left hub"),
                Err(e) => HubResponse::error(&e),
            },

            HubCommand::Tell { from, to, message } => match hub_guard.tell(&from, &to, &message) {
                Ok(()) => HubResponse::ok("Message sent"),
                Err(e) => HubResponse::error(&e),
            },

            HubCommand::Broadcast { from, message } => match hub_guard.broadcast(&from, &message) {
                Ok(()) => HubResponse::ok("Broadcast sent"),
                Err(e) => HubResponse::error(&e),
            },

            HubCommand::GetInbox { session_id } => {
                let messages = hub_guard.inbox(&session_id);
                let message_data: Vec<serde_json::Value> = messages
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "from": m.from,
                            "to": m.to,
                            "content": m.content,
                            "type": format!("{:?}", m.message_type),
                            "timestamp": m.timestamp,
                            "read": m.read
                        })
                    })
                    .collect();
                HubResponse::ok_with_data(
                    &format!("Inbox for {}", session_id),
                    serde_json::json!({ "messages": message_data, "count": message_data.len() }),
                )
            }

            HubCommand::CreateTask {
                title,
                assignee,
                priority,
            } => {
                let task_priority = match priority.to_lowercase().as_str() {
                    "low" => TaskPriority::Low,
                    "medium" => TaskPriority::Medium,
                    "high" => TaskPriority::High,
                    "critical" => TaskPriority::Critical,
                    _ => TaskPriority::Medium,
                };
                match hub_guard.create_task(&title, &assignee, task_priority) {
                    Ok(task) => HubResponse::ok_with_data(
                        "Task created",
                        serde_json::json!({ "task_id": task.id, "title": task.title }),
                    ),
                    Err(e) => HubResponse::error(&e),
                }
            }

            HubCommand::ListTasks => {
                let tasks = hub_guard.get_tasks();
                let task_data: Vec<serde_json::Value> = tasks
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "title": t.title,
                            "assignee": t.assignee,
                            "status": format!("{:?}", t.status),
                            "priority": format!("{:?}", t.priority)
                        })
                    })
                    .collect();
                HubResponse::ok_with_data("Tasks", serde_json::json!({ "tasks": task_data }))
            }

            HubCommand::UpdateTask { id, status } => {
                let task_status = match status.to_lowercase().as_str() {
                    "pending" => super::tasks::TaskStatus::Pending,
                    "in_progress" | "inprogress" => super::tasks::TaskStatus::InProgress,
                    "blocked" => super::tasks::TaskStatus::Blocked,
                    "done" | "completed" => super::tasks::TaskStatus::Done,
                    _ => return HubResponse::error("Invalid status"),
                };
                match hub_guard.update_task(id, task_status) {
                    Ok(()) => HubResponse::ok("Task updated"),
                    Err(e) => HubResponse::error(&e),
                }
            }

            HubCommand::SetWorkingOn {
                session_id,
                file_path,
            } => match hub_guard.set_working_on(&session_id, &file_path) {
                Ok(()) => HubResponse::ok(&format!("Now working on {}", file_path)),
                Err(e) => HubResponse::error(&e),
            },

            HubCommand::ClearWorkingOn { session_id } => {
                hub_guard.state.clear_working_on(&session_id);
                HubResponse::ok("Cleared working state")
            }

            HubCommand::GetState => {
                let state_data = hub_guard.state.get_all_working();
                HubResponse::ok_with_data(
                    "Current state",
                    serde_json::json!({ "working_on": state_data }),
                )
            }

            HubCommand::GetConflicts => {
                let conflicts = hub_guard.get_conflicts();
                let conflict_data: Vec<serde_json::Value> = conflicts
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "file": c.file_path,
                            "sessions": c.sessions
                        })
                    })
                    .collect();
                HubResponse::ok_with_data(
                    "Conflicts",
                    serde_json::json!({ "conflicts": conflict_data }),
                )
            }

            HubCommand::Heartbeat { session_id } => {
                hub_guard.state.set_session_active(&session_id, true);
                HubResponse::ok("Heartbeat received")
            }

            HubCommand::Shutdown => {
                let _ = hub_guard.save();
                HubResponse::ok("Shutting down")
            }
        }
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.socket_path.exists()
    }
}

/// Hub Client (used by sessions)
pub struct HubClient {
    socket_path: PathBuf,
}

impl HubClient {
    /// Create a new hub client
    pub fn new(config: &HubConfig) -> Self {
        Self {
            socket_path: config.socket_path.clone(),
        }
    }

    /// Connect to the hub
    fn connect(&self) -> Result<UnixStream, String> {
        UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Cannot connect to hub: {}. Is the hub running?", e))
    }

    /// Send a command and get response
    pub fn send(&self, command: HubCommand) -> Result<HubResponse, String> {
        let mut stream = self.connect()?;

        // Send command
        let command_json = serde_json::to_string(&command)
            .map_err(|e| format!("Cannot serialize command: {}", e))?;

        writeln!(stream, "{}", command_json).map_err(|e| format!("Cannot send command: {}", e))?;

        stream.flush().map_err(|e| e.to_string())?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .map_err(|e| format!("Cannot read response: {}", e))?;

        let response: HubResponse = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Cannot parse response: {}", e))?;

        Ok(response)
    }

    /// Ping the hub
    pub fn ping(&self) -> Result<bool, String> {
        let response = self.send(HubCommand::Ping)?;
        Ok(response.success && response.message == "pong")
    }

    /// Join the hub
    pub fn join(&self, role: &str, name: Option<String>) -> Result<HubResponse, String> {
        self.send(HubCommand::Join {
            role: role.to_string(),
            name,
        })
    }

    /// Leave the hub
    pub fn leave(&self, session_id: &str) -> Result<HubResponse, String> {
        self.send(HubCommand::Leave {
            session_id: session_id.to_string(),
        })
    }

    /// Get who's online
    pub fn who(&self) -> Result<HubResponse, String> {
        self.send(HubCommand::Who)
    }

    /// Send a message
    pub fn tell(&self, from: &str, to: &str, message: &str) -> Result<HubResponse, String> {
        self.send(HubCommand::Tell {
            from: from.to_string(),
            to: to.to_string(),
            message: message.to_string(),
        })
    }

    /// Broadcast a message
    pub fn broadcast(&self, from: &str, message: &str) -> Result<HubResponse, String> {
        self.send(HubCommand::Broadcast {
            from: from.to_string(),
            message: message.to_string(),
        })
    }

    /// Create a task
    pub fn create_task(
        &self,
        title: &str,
        assignee: &str,
        priority: &str,
    ) -> Result<HubResponse, String> {
        self.send(HubCommand::CreateTask {
            title: title.to_string(),
            assignee: assignee.to_string(),
            priority: priority.to_string(),
        })
    }

    /// Get status
    pub fn status(&self) -> Result<HubResponse, String> {
        self.send(HubCommand::Status)
    }

    /// Set working on file
    pub fn set_working_on(&self, session_id: &str, file_path: &str) -> Result<HubResponse, String> {
        self.send(HubCommand::SetWorkingOn {
            session_id: session_id.to_string(),
            file_path: file_path.to_string(),
        })
    }

    pub fn get_inbox(&self, session_id: &str) -> Result<HubResponse, String> {
        self.send(HubCommand::GetInbox {
            session_id: session_id.to_string(),
        })
    }

    pub fn is_available(&self) -> bool {
        self.socket_path.exists() && self.ping().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_response_ok() {
        let response = HubResponse::ok("Test");
        assert!(response.success);
        assert_eq!(response.message, "Test");
    }

    #[test]
    fn test_hub_response_error() {
        let response = HubResponse::error("Failed");
        assert!(!response.success);
        assert_eq!(response.message, "Failed");
    }

    #[test]
    fn test_hub_command_serialization() {
        let cmd = HubCommand::Ping;
        let json = serde_json::to_string(&cmd).expect("serialization failed");
        assert!(json.contains("Ping"));
    }

    #[test]
    fn test_hub_client_creation() {
        let config = HubConfig::new();
        let client = HubClient::new(&config);
        assert!(client.socket_path.to_string_lossy().contains("hub.sock"));
    }
}
