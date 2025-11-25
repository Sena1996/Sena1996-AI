//! Session Manager
//! Cross-Session Continuity - Persist state, restore context, remember preferences

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;

use crate::base::component::{BaseComponent, ComponentMetrics, ComponentState, ComponentStatus};

/// Session state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub last_command: Option<String>,
    pub last_result: Option<serde_json::Value>,
    pub working_directory: Option<String>,
    pub environment: HashMap<String, String>,
    pub preferences: HashMap<String, serde_json::Value>,
    pub command_history: Vec<String>,
    pub commands_executed: u64,
    pub errors_encountered: u64,
    pub duration_seconds: f64,
}

impl SessionState {
    /// Create a new session
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            started_at: Utc::now(),
            ended_at: None,
            last_command: None,
            last_result: None,
            working_directory: None,
            environment: HashMap::new(),
            preferences: HashMap::new(),
            command_history: Vec::new(),
            commands_executed: 0,
            errors_encountered: 0,
            duration_seconds: 0.0,
        }
    }

    /// Mark session as ended
    pub fn end(&mut self) {
        self.ended_at = Some(Utc::now());
        self.duration_seconds = (Utc::now() - self.started_at).num_milliseconds() as f64 / 1000.0;
    }
}

/// Session history data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionHistoryData {
    version: String,
    sessions: Vec<SessionState>,
    last_saved: String,
}

/// Preferences data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreferencesData {
    version: String,
    preferences: HashMap<String, serde_json::Value>,
    last_saved: String,
}

/// Session Manager metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagerMetrics {
    pub sessions_started: u64,
    pub sessions_restored: u64,
    pub total_commands_recorded: u64,
    pub preferences_saved: u64,
}

/// Session Manager
/// Cross-session continuity with state persistence and preference memory
pub struct SessionManager {
    state: ComponentState,
    current_session: RwLock<Option<SessionState>>,
    session_history: RwLock<Vec<SessionState>>,
    global_preferences: RwLock<HashMap<String, serde_json::Value>>,
    metrics: RwLock<SessionManagerMetrics>,
    storage_dir: PathBuf,
    max_history_size: usize,
    max_sessions_history: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(max_history_size: usize) -> Self {
        let storage_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("data")
            .join("sessions");

        // Create directory if it doesn't exist
        let _ = fs::create_dir_all(&storage_dir);

        Self {
            state: ComponentState::new("SessionManager"),
            current_session: RwLock::new(None),
            session_history: RwLock::new(Vec::new()),
            global_preferences: RwLock::new(HashMap::new()),
            metrics: RwLock::new(SessionManagerMetrics {
                sessions_started: 0,
                sessions_restored: 0,
                total_commands_recorded: 0,
                preferences_saved: 0,
            }),
            storage_dir,
            max_history_size,
            max_sessions_history: 50,
        }
    }

    /// Generate a unique session ID
    fn generate_session_id() -> String {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let data = format!("{}{}", hostname, Utc::now().to_rfc3339());
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(&hasher.finalize()[..8])
    }

    /// Start a new session
    pub fn start_session(&self) -> SessionState {
        let session_id = Self::generate_session_id();
        let global_prefs = self.global_preferences.read().unwrap().clone();

        let mut session = SessionState::new(session_id);
        session.preferences = global_prefs;

        // Store as current session
        {
            let mut current = self.current_session.write().unwrap();
            *current = Some(session.clone());
        }

        // Save to disk
        self.save_current_session();

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.sessions_started += 1;
        }

        session
    }

    /// End the current session
    pub fn end_session(&self) {
        let session = {
            let mut current = self.current_session.write().unwrap();
            current.take()
        };

        if let Some(mut session) = session {
            session.end();

            // Add to history
            {
                let mut history = self.session_history.write().unwrap();
                history.push(session);

                // Limit history size
                if history.len() > self.max_sessions_history {
                    let keep_from = history.len() - self.max_sessions_history;
                    *history = history.split_off(keep_from);
                }
            }

            // Save history
            self.save_session_history();

            // Clear current session file
            let _ = fs::remove_file(self.current_session_file());
        }
    }

    /// Record a command execution
    pub fn record_command(&self, command: &str, result: Option<serde_json::Value>) {
        let mut current = self.current_session.write().unwrap();

        // Start session if needed
        if current.is_none() {
            drop(current);
            self.start_session();
            current = self.current_session.write().unwrap();
        }

        if let Some(ref mut session) = *current {
            session.last_command = Some(command.to_string());
            session.last_result = result.clone();
            session.commands_executed += 1;

            // Add to command history
            session.command_history.push(command.to_string());

            // Limit history
            if session.command_history.len() > self.max_history_size {
                session.command_history = session.command_history
                    .split_off(session.command_history.len() - self.max_history_size);
            }

            // Check for errors
            if let Some(ref res) = result {
                if let Some(success) = res.get("success") {
                    if !success.as_bool().unwrap_or(true) {
                        session.errors_encountered += 1;
                    }
                }
            }
        }

        drop(current);
        self.save_current_session();

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_commands_recorded += 1;
        }
    }

    /// Set a user preference
    pub fn set_preference(&self, key: &str, value: serde_json::Value, persist: bool) {
        {
            let mut current = self.current_session.write().unwrap();

            if current.is_none() {
                drop(current);
                self.start_session();
                current = self.current_session.write().unwrap();
            }

            if let Some(ref mut session) = *current {
                session.preferences.insert(key.to_string(), value.clone());
            }
        }

        if persist {
            let mut global = self.global_preferences.write().unwrap();
            global.insert(key.to_string(), value);
            drop(global);
            self.save_global_preferences();
        }

        self.save_current_session();

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.preferences_saved += 1;
        }
    }

    /// Get a user preference
    pub fn get_preference(&self, key: &str) -> Option<serde_json::Value> {
        let current = self.current_session.read().unwrap();
        if let Some(ref session) = *current {
            if let Some(value) = session.preferences.get(key) {
                return Some(value.clone());
            }
        }
        drop(current);

        let global = self.global_preferences.read().unwrap();
        global.get(key).cloned()
    }

    /// Get the current session
    pub fn get_current_session(&self) -> Option<SessionState> {
        self.current_session.read().unwrap().clone()
    }

    /// Get the last completed session
    pub fn get_last_session(&self) -> Option<SessionState> {
        let history = self.session_history.read().unwrap();
        history.last().cloned()
    }

    /// Get session history
    pub fn get_session_history(&self, limit: usize) -> Vec<SessionState> {
        let history = self.session_history.read().unwrap();
        let start = history.len().saturating_sub(limit);
        history[start..].iter().rev().cloned().collect()
    }

    /// Get command history from current session
    pub fn get_command_history(&self, limit: usize) -> Vec<String> {
        let current = self.current_session.read().unwrap();
        if let Some(ref session) = *current {
            let start = session.command_history.len().saturating_sub(limit);
            return session.command_history[start..].to_vec();
        }
        Vec::new()
    }

    /// Get session manager statistics
    pub fn get_stats(&self) -> serde_json::Value {
        let metrics = self.metrics.read().unwrap();
        let history = self.session_history.read().unwrap();
        let global = self.global_preferences.read().unwrap();
        let current = self.current_session.read().unwrap();

        let mut stats = serde_json::json!({
            "total_sessions": history.len(),
            "global_preferences_count": global.len(),
            "sessions_started": metrics.sessions_started,
            "sessions_restored": metrics.sessions_restored,
            "total_commands_recorded": metrics.total_commands_recorded,
            "preferences_saved": metrics.preferences_saved,
        });

        if let Some(ref session) = *current {
            stats["current_session_id"] = serde_json::json!(session.session_id);
            stats["current_session_duration"] = serde_json::json!(
                (Utc::now() - session.started_at).num_seconds()
            );
            stats["current_session_commands"] = serde_json::json!(session.commands_executed);
        }

        stats
    }

    // File path helpers
    fn current_session_file(&self) -> PathBuf {
        self.storage_dir.join("current_session.json")
    }

    fn session_history_file(&self) -> PathBuf {
        self.storage_dir.join("session_history.json")
    }

    fn preferences_file(&self) -> PathBuf {
        self.storage_dir.join("user_preferences.json")
    }

    // Persistence methods
    fn save_current_session(&self) {
        let current = self.current_session.read().unwrap();
        if let Some(ref session) = *current {
            if let Ok(json) = serde_json::to_string_pretty(session) {
                let _ = fs::write(self.current_session_file(), json);
            }
        }
    }

    fn save_session_history(&self) {
        let history = self.session_history.read().unwrap();
        let data = SessionHistoryData {
            version: "5.0.0".to_string(),
            sessions: history.clone(),
            last_saved: Utc::now().to_rfc3339(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(self.session_history_file(), json);
        }
    }

    fn load_session_history(&self) {
        if let Ok(content) = fs::read_to_string(self.session_history_file()) {
            if let Ok(data) = serde_json::from_str::<SessionHistoryData>(&content) {
                let mut history = self.session_history.write().unwrap();
                *history = data.sessions;
            }
        }
    }

    fn save_global_preferences(&self) {
        let prefs = self.global_preferences.read().unwrap();
        let data = PreferencesData {
            version: "5.0.0".to_string(),
            preferences: prefs.clone(),
            last_saved: Utc::now().to_rfc3339(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(self.preferences_file(), json);
        }
    }

    fn load_global_preferences(&self) {
        if let Ok(content) = fs::read_to_string(self.preferences_file()) {
            if let Ok(data) = serde_json::from_str::<PreferencesData>(&content) {
                let mut prefs = self.global_preferences.write().unwrap();
                *prefs = data.preferences;
            }
        }
    }

    fn get_last_incomplete_session(&self) -> Option<SessionState> {
        if let Ok(content) = fs::read_to_string(self.current_session_file()) {
            if let Ok(session) = serde_json::from_str::<SessionState>(&content) {
                // Only restore if recent (within last 24 hours)
                let age = Utc::now() - session.started_at;
                if age < Duration::hours(24) {
                    return Some(session);
                }
            }
        }
        None
    }
}

impl BaseComponent for SessionManager {
    fn name(&self) -> &str {
        &self.state.name
    }

    fn initialize(&mut self) -> Result<(), String> {
        // Load persisted data
        self.load_global_preferences();
        self.load_session_history();

        // Try to restore last session
        if let Some(mut session) = self.get_last_incomplete_session() {
            session.started_at = Utc::now();
            {
                let mut current = self.current_session.write().unwrap();
                *current = Some(session);
            }
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.sessions_restored += 1;
            }
        } else {
            self.start_session();
        }

        self.state.mark_initialized();
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), String> {
        self.end_session();
        Ok(())
    }

    fn get_status(&self) -> ComponentStatus {
        ComponentStatus {
            name: self.state.name.clone(),
            initialized: self.state.initialized,
            healthy: true,
            details: {
                let mut details = HashMap::new();
                if let Ok(stats) = serde_json::to_value(self.get_stats()) {
                    details.insert("stats".to_string(), stats);
                }
                details
            },
        }
    }

    fn is_initialized(&self) -> bool {
        self.state.initialized
    }

    fn get_metrics(&self) -> ComponentMetrics {
        self.state.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_creation() {
        let session = SessionState::new("test-123".to_string());
        assert_eq!(session.session_id, "test-123");
        assert!(session.ended_at.is_none());
    }

    #[test]
    fn test_session_end() {
        let mut session = SessionState::new("test-123".to_string());
        session.end();
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new(100);
        assert!(!manager.is_initialized());
    }

    #[test]
    fn test_generate_session_id() {
        let id1 = SessionManager::generate_session_id();
        let id2 = SessionManager::generate_session_id();
        // IDs should be different (though in fast tests they might be same)
        assert_eq!(id1.len(), 16);
        assert_eq!(id2.len(), 16);
    }
}
