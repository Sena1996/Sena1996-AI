//! Hub State Management
//!
//! CRDT-powered shared state for real-time synchronization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::HubConfig;
use crate::sync::CRDT;

/// Shared state entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedState {
    pub key: String,
    pub value: serde_json::Value,
    pub author: String,
    pub timestamp: u64,
}

/// Session working state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWorkState {
    pub session_id: String,
    pub working_on: Option<String>,
    pub status: String,
    pub last_update: u64,
    pub active: bool,
}

/// Persisted hub state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HubStateData {
    version: String,
    states: HashMap<String, serde_json::Value>,
    session_states: HashMap<String, SessionWorkState>,
    last_updated: u64,
}

/// Hub State Manager
pub struct HubState {
    crdt: CRDT,
    session_states: HashMap<String, SessionWorkState>,
    state_file: PathBuf,
}

impl HubState {
    /// Create a new hub state manager
    pub fn new(config: &HubConfig) -> Self {
        let author_id = format!("hub-{}", std::process::id());

        Self {
            crdt: CRDT::new(&author_id),
            session_states: HashMap::new(),
            state_file: config.state_file.clone(),
        }
    }

    /// Set a shared state value
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.crdt.set(key, value);
    }

    /// Get a shared state value
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.crdt.get(key)
    }

    /// Get all shared state
    pub fn get_all(&self) -> HashMap<String, serde_json::Value> {
        self.crdt.get_all()
    }

    /// Set session active status
    pub fn set_session_active(&mut self, session_id: &str, active: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let state = self
            .session_states
            .entry(session_id.to_string())
            .or_insert_with(|| SessionWorkState {
                session_id: session_id.to_string(),
                working_on: None,
                status: "idle".to_string(),
                last_update: now,
                active: false,
            });

        state.active = active;
        state.last_update = now;
    }

    /// Set what a session is working on
    pub fn set_working_on(&mut self, session_id: &str, file_path: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let state = self
            .session_states
            .entry(session_id.to_string())
            .or_insert_with(|| SessionWorkState {
                session_id: session_id.to_string(),
                working_on: None,
                status: "active".to_string(),
                last_update: now,
                active: true,
            });

        state.working_on = Some(file_path.to_string());
        state.status = "active".to_string();
        state.last_update = now;

        // Also store in CRDT for sync
        self.crdt.set(
            &format!("working:{}", session_id),
            serde_json::json!({
                "file": file_path,
                "timestamp": now
            }),
        );
    }

    /// Clear working state for a session
    pub fn clear_working_on(&mut self, session_id: &str) {
        if let Some(state) = self.session_states.get_mut(session_id) {
            state.working_on = None;
            state.status = "idle".to_string();
            state.last_update = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
        }

        self.crdt.delete(&format!("working:{}", session_id));
    }

    /// Get session work state
    pub fn get_session_state(&self, session_id: &str) -> Option<&SessionWorkState> {
        self.session_states.get(session_id)
    }

    /// Get all session work states
    pub fn get_all_session_states(&self) -> Vec<&SessionWorkState> {
        self.session_states.values().collect()
    }

    /// Get all sessions working on files
    pub fn get_working_sessions(&self) -> Vec<(&String, &String)> {
        self.session_states
            .iter()
            .filter_map(|(id, state)| state.working_on.as_ref().map(|file| (id, file)))
            .collect()
    }

    /// Check if any session is working on a specific file
    pub fn who_is_working_on(&self, file_path: &str) -> Vec<String> {
        self.session_states
            .iter()
            .filter_map(|(id, state)| {
                if state
                    .working_on
                    .as_ref()
                    .map(|f| f == file_path)
                    .unwrap_or(false)
                {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Save state to disk
    pub fn save(&self) -> Result<(), String> {
        let data = HubStateData {
            version: crate::VERSION.to_string(),
            states: self.crdt.get_all(),
            session_states: self.session_states.clone(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Cannot serialize state: {}", e))?;

        fs::write(&self.state_file, json).map_err(|e| format!("Cannot write state file: {}", e))?;

        Ok(())
    }

    /// Load state from disk
    pub fn load(&mut self) -> Result<(), String> {
        if !self.state_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.state_file)
            .map_err(|e| format!("Cannot read state file: {}", e))?;

        let data: HubStateData = serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse state file: {}", e))?;

        // Restore CRDT state
        for (key, value) in data.states {
            self.crdt.set(&key, value);
        }

        // Restore session states
        self.session_states = data.session_states;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_state_creation() {
        let config = HubConfig::new();
        let state = HubState::new(&config);
        assert!(state.get_all().is_empty());
    }

    #[test]
    fn test_set_and_get() {
        let config = HubConfig::new();
        let mut state = HubState::new(&config);

        state.set("key1", serde_json::json!("value1"));
        assert_eq!(state.get("key1"), Some(serde_json::json!("value1")));
    }

    #[test]
    fn test_working_on() {
        let config = HubConfig::new();
        let mut state = HubState::new(&config);

        state.set_working_on("session-1", "src/main.rs");

        let sessions = state.who_is_working_on("src/main.rs");
        assert!(sessions.contains(&"session-1".to_string()));
    }
}
