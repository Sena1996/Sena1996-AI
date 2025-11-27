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

use super::HubConfig;

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
}

impl HubServer {
    /// Create a new hub server
    pub fn new(config: &HubConfig) -> Self {
        Self {
            socket_path: config.socket_path.clone(),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the server
    pub fn start(&self) -> Result<(), String> {
        // Remove old socket if exists
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)
                .map_err(|e| format!("Cannot remove old socket: {}", e))?;
        }

        // Create listener
        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| format!("Cannot bind socket: {}", e))?;

        // Set non-blocking for graceful shutdown
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Cannot set non-blocking: {}", e))?;

        *self.running.lock().expect("running lock poisoned") = true;

        eprintln!("Hub server listening on {:?}", self.socket_path);

        // Accept connections
        while *self.running.lock().expect("running lock poisoned") {
            match listener.accept() {
                Ok((stream, _)) => {
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream) {
                            eprintln!("Client error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection, sleep briefly
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }

        // Cleanup
        let _ = fs::remove_file(&self.socket_path);

        Ok(())
    }

    /// Stop the server
    pub fn stop(&self) {
        *self.running.lock().expect("running lock poisoned") = false;
    }

    /// Handle a client connection
    fn handle_client(mut stream: UnixStream) -> Result<(), String> {
        let reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;

            // Parse command
            let command: HubCommand =
                serde_json::from_str(&line).map_err(|e| format!("Invalid command: {}", e))?;

            // Process command
            let response = Self::process_command(command);

            // Send response
            let response_json = serde_json::to_string(&response)
                .map_err(|e| format!("Cannot serialize response: {}", e))?;

            writeln!(stream, "{}", response_json)
                .map_err(|e| format!("Cannot write response: {}", e))?;

            stream.flush().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Process a command
    fn process_command(command: HubCommand) -> HubResponse {
        match command {
            HubCommand::Ping => HubResponse::pong(),

            HubCommand::Status => HubResponse::ok_with_data(
                "Hub status",
                serde_json::json!({
                    "running": true,
                    "version": crate::VERSION
                }),
            ),

            HubCommand::Who => {
                // TODO: Get from actual hub state
                HubResponse::ok_with_data(
                    "Active sessions",
                    serde_json::json!({
                        "sessions": []
                    }),
                )
            }

            HubCommand::Shutdown => HubResponse::ok("Shutting down"),

            _ => {
                // For now, acknowledge other commands
                HubResponse::ok("Command received")
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

    /// Check if hub is available
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
