//! Session Registry
//!
//! Manages active sessions in the collaboration hub
//! Includes command history, preferences, and cross-session continuity
//! (Merged from deprecated session/manager.rs)

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use super::HubConfig;

/// Maximum command history size per session
const MAX_COMMAND_HISTORY: usize = 100;

/// Session role types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionRole {
    Android,
    Web,
    Backend,
    IoT,
    General,
    Custom,
}

impl SessionRole {
    /// Get emoji for role
    pub fn emoji(&self) -> &'static str {
        match self {
            SessionRole::Android => "🤖",
            SessionRole::Web => "🌐",
            SessionRole::Backend => "⚙️",
            SessionRole::IoT => "📡",
            SessionRole::General => "💻",
            SessionRole::Custom => "🔧",
        }
    }

    /// Get role name
    pub fn name(&self) -> &'static str {
        match self {
            SessionRole::Android => "android",
            SessionRole::Web => "web",
            SessionRole::Backend => "backend",
            SessionRole::IoT => "iot",
            SessionRole::General => "general",
            SessionRole::Custom => "custom",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "android" => SessionRole::Android,
            "web" | "frontend" => SessionRole::Web,
            "backend" | "server" | "api" => SessionRole::Backend,
            "iot" | "embedded" | "hardware" => SessionRole::IoT,
            "general" => SessionRole::General,
            _ => SessionRole::Custom,
        }
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Busy,
    Away,
}

impl SessionStatus {
    pub fn indicator(&self) -> &'static str {
        match self {
            SessionStatus::Active => "🟢",
            SessionStatus::Idle => "🟡",
            SessionStatus::Busy => "🔴",
            SessionStatus::Away => "⚪",
        }
    }
}

/// A collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub role: SessionRole,
    pub name: String,
    pub status: SessionStatus,
    pub working_on: Option<String>,
    pub working_directory: Option<String>,
    pub joined_at: u64,
    pub last_heartbeat: u64,
    pub pid: u32,

    // Merged from old SessionManager
    pub command_history: Vec<String>,
    pub preferences: HashMap<String, serde_json::Value>,
    pub commands_executed: u64,
    pub errors_encountered: u64,
    pub last_command: Option<String>,
}

impl Session {
    /// Create a new session
    pub fn new(role: SessionRole, name: Option<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = Self::generate_id(&role, timestamp);
        let display_name = name.unwrap_or_else(|| role.name().to_string());

        Self {
            id,
            role,
            name: display_name,
            status: SessionStatus::Active,
            working_on: None,
            working_directory: std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string()),
            joined_at: timestamp,
            last_heartbeat: timestamp,
            pid: std::process::id(),
            // Merged features
            command_history: Vec::new(),
            preferences: HashMap::new(),
            commands_executed: 0,
            errors_encountered: 0,
            last_command: None,
        }
    }

    /// Generate unique session ID
    fn generate_id(role: &SessionRole, timestamp: u64) -> String {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let data = format!("{}{}{}{}", role.name(), hostname, timestamp, std::process::id());
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{}-{}", role.name(), hex::encode(&hasher.finalize()[..4]))
    }

    /// Update heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Check if session is stale (no heartbeat for 24 hours)
    /// CLI sessions don't have continuous heartbeats, so we use a long timeout
    pub fn is_stale(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        const STALE_TIMEOUT_SECONDS: u64 = 24 * 60 * 60;
        now.saturating_sub(self.last_heartbeat) > STALE_TIMEOUT_SECONDS
    }

    /// Get time since last activity
    pub fn idle_time(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        now - self.last_heartbeat
    }

    /// Format idle time for display
    pub fn idle_display(&self) -> String {
        let secs = self.idle_time();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else {
            format!("{}h ago", secs / 3600)
        }
    }

    // ========================================
    // Merged from old SessionManager
    // ========================================

    /// Record a command execution
    pub fn record_command(&mut self, command: &str, success: bool) {
        self.last_command = Some(command.to_string());
        self.commands_executed += 1;

        if !success {
            self.errors_encountered += 1;
        }

        // Add to history with limit
        self.command_history.push(command.to_string());
        if self.command_history.len() > MAX_COMMAND_HISTORY {
            self.command_history.remove(0);
        }

        self.heartbeat();
    }

    /// Set a preference
    pub fn set_preference(&mut self, key: &str, value: serde_json::Value) {
        self.preferences.insert(key.to_string(), value);
        self.heartbeat();
    }

    /// Get a preference
    pub fn get_preference(&self, key: &str) -> Option<&serde_json::Value> {
        self.preferences.get(key)
    }

    /// Get command history
    pub fn get_command_history(&self, limit: usize) -> Vec<String> {
        let start = self.command_history.len().saturating_sub(limit);
        self.command_history[start..].to_vec()
    }

    /// Get session duration in seconds
    pub fn duration(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        now - self.joined_at
    }

    /// Get session stats
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "session_id": self.id,
            "role": self.role.name(),
            "duration_seconds": self.duration(),
            "commands_executed": self.commands_executed,
            "errors_encountered": self.errors_encountered,
            "error_rate": if self.commands_executed > 0 {
                self.errors_encountered as f64 / self.commands_executed as f64
            } else {
                0.0
            },
            "preferences_count": self.preferences.len(),
            "history_size": self.command_history.len(),
        })
    }
}

/// Persisted sessions data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionsData {
    version: String,
    sessions: HashMap<String, Session>,
    last_updated: u64,
}

/// Global preferences data (persists across all sessions)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GlobalPreferencesData {
    version: String,
    preferences: HashMap<String, serde_json::Value>,
    last_updated: u64,
}

/// Session Registry
pub struct SessionRegistry {
    sessions: HashMap<String, Session>,
    sessions_file: PathBuf,
    global_preferences: HashMap<String, serde_json::Value>,
    preferences_file: PathBuf,
}

impl SessionRegistry {
    /// Create a new session registry
    pub fn new(config: &HubConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            sessions_file: config.hub_dir.join("sessions.json"),
            global_preferences: HashMap::new(),
            preferences_file: config.hub_dir.join("global_preferences.json"),
        }
    }

    /// Register a new session
    pub fn register(&mut self, role: SessionRole, name: Option<String>) -> Result<Session, String> {
        let mut session = Session::new(role, name);

        // Copy global preferences to new session
        session.preferences = self.global_preferences.clone();

        // Check if role is already taken by active session
        for existing in self.sessions.values() {
            if existing.role == role && !existing.is_stale() {
                return Err(format!(
                    "Role '{}' is already taken by session {}",
                    role.name(),
                    existing.id
                ));
            }
        }

        self.sessions.insert(session.id.clone(), session.clone());
        self.save()?;

        Ok(session)
    }

    /// Unregister a session
    pub fn unregister(&mut self, session_id: &str) -> Result<(), String> {
        self.sessions.remove(session_id);
        self.save()?;
        Ok(())
    }

    /// Get a session by ID
    pub fn get(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    /// Get a session by role
    pub fn get_by_role(&self, role: SessionRole) -> Option<&Session> {
        self.sessions.values().find(|s| s.role == role && !s.is_stale())
    }

    /// Get all active sessions
    pub fn get_active(&self) -> Vec<Session> {
        self.sessions
            .values()
            .filter(|s| !s.is_stale())
            .cloned()
            .collect()
    }

    /// Get all sessions (including stale)
    pub fn get_all(&self) -> Vec<Session> {
        self.sessions.values().cloned().collect()
    }

    /// Update session heartbeat
    pub fn heartbeat(&mut self, session_id: &str) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.heartbeat();
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Update session status
    pub fn set_status(&mut self, session_id: &str, status: SessionStatus) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.status = status;
            session.heartbeat();
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Update what session is working on
    pub fn set_working_on(&mut self, session_id: &str, file_path: Option<&str>) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.working_on = file_path.map(|s| s.to_string());
            session.heartbeat();
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Clean up stale sessions
    pub fn cleanup_stale(&mut self) {
        let stale_ids: Vec<String> = self.sessions
            .iter()
            .filter(|(_, s)| s.is_stale())
            .map(|(id, _)| id.clone())
            .collect();

        for id in stale_ids {
            self.sessions.remove(&id);
        }
    }

    /// Save sessions to disk
    pub fn save(&self) -> Result<(), String> {
        let data = SessionsData {
            version: crate::VERSION.to_string(),
            sessions: self.sessions.clone(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Cannot serialize sessions: {}", e))?;

        // Ensure parent directory exists
        if let Some(parent) = self.sessions_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create sessions directory: {}", e))?;
        }

        fs::write(&self.sessions_file, json)
            .map_err(|e| format!("Cannot write sessions file: {}", e))?;

        Ok(())
    }

    /// Load sessions from disk
    pub fn load(&mut self) -> Result<(), String> {
        if !self.sessions_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.sessions_file)
            .map_err(|e| format!("Cannot read sessions file: {}", e))?;

        let data: SessionsData = serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse sessions file: {}", e))?;

        self.sessions = data.sessions;

        // Clean up stale sessions on load
        self.cleanup_stale();

        Ok(())
    }

    /// Get count of active sessions
    pub fn count(&self) -> usize {
        self.get_active().len()
    }

    // ========================================
    // Global Preferences (Merged from old SessionManager)
    // ========================================

    /// Set a global preference (persists across sessions)
    pub fn set_global_preference(&mut self, key: &str, value: serde_json::Value) -> Result<(), String> {
        self.global_preferences.insert(key.to_string(), value);
        self.save_preferences()
    }

    /// Get a global preference
    pub fn get_global_preference(&self, key: &str) -> Option<&serde_json::Value> {
        self.global_preferences.get(key)
    }

    /// Get all global preferences
    pub fn get_all_global_preferences(&self) -> &HashMap<String, serde_json::Value> {
        &self.global_preferences
    }

    /// Save global preferences to disk
    fn save_preferences(&self) -> Result<(), String> {
        let data = GlobalPreferencesData {
            version: crate::VERSION.to_string(),
            preferences: self.global_preferences.clone(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Cannot serialize preferences: {}", e))?;

        // Ensure parent directory exists
        if let Some(parent) = self.preferences_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create preferences directory: {}", e))?;
        }

        fs::write(&self.preferences_file, json)
            .map_err(|e| format!("Cannot write preferences file: {}", e))?;

        Ok(())
    }

    /// Load global preferences from disk
    pub fn load_preferences(&mut self) -> Result<(), String> {
        if !self.preferences_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.preferences_file)
            .map_err(|e| format!("Cannot read preferences file: {}", e))?;

        let data: GlobalPreferencesData = serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse preferences file: {}", e))?;

        self.global_preferences = data.preferences;
        Ok(())
    }

    // ========================================
    // Session Command/Preference Helpers
    // ========================================

    /// Record a command for a session
    pub fn record_command(&mut self, session_id: &str, command: &str, success: bool) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.record_command(command, success);
            self.save()?;
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Set a preference for a session
    pub fn set_session_preference(&mut self, session_id: &str, key: &str, value: serde_json::Value) -> Result<(), String> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.set_preference(key, value);
            self.save()?;
            Ok(())
        } else {
            Err(format!("Session {} not found", session_id))
        }
    }

    /// Get session command history
    pub fn get_session_history(&self, session_id: &str, limit: usize) -> Option<Vec<String>> {
        self.sessions.get(session_id)
            .map(|s| s.get_command_history(limit))
    }

    /// Get session stats
    pub fn get_session_stats(&self, session_id: &str) -> Option<serde_json::Value> {
        self.sessions.get(session_id)
            .map(|s| s.stats())
    }

    /// Get mutable session reference
    pub fn get_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_role_emoji() {
        assert_eq!(SessionRole::Android.emoji(), "🤖");
        assert_eq!(SessionRole::Web.emoji(), "🌐");
        assert_eq!(SessionRole::Backend.emoji(), "⚙️");
    }

    #[test]
    fn test_session_role_from_str() {
        assert_eq!(SessionRole::parse("android"), SessionRole::Android);
        assert_eq!(SessionRole::parse("web"), SessionRole::Web);
        assert_eq!(SessionRole::parse("frontend"), SessionRole::Web);
        assert_eq!(SessionRole::parse("backend"), SessionRole::Backend);
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(SessionRole::Android, Some("test".to_string()));
        assert!(session.id.starts_with("android-"));
        assert_eq!(session.name, "test");
    }

    #[test]
    fn test_session_idle_time() {
        let session = Session::new(SessionRole::Web, None);
        assert!(session.idle_time() < 2); // Should be very recent
    }
}
