//! Session Context Management
//!
//! Tracks the current terminal's session for proper message routing

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::HubConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: String,
    pub session_name: String,
    pub role: String,
    pub created_at: u64,
    pub terminal_id: String,
}

impl SessionContext {
    pub fn new(session_id: &str, session_name: &str, role: &str) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let terminal_id = Self::get_terminal_id();

        Self {
            session_id: session_id.to_string(),
            session_name: session_name.to_string(),
            role: role.to_string(),
            created_at,
            terminal_id,
        }
    }

    fn get_terminal_id() -> String {
        std::env::var("TERM_SESSION_ID")
            .or_else(|_| std::env::var("WINDOWID"))
            .or_else(|_| std::env::var("TMUX_PANE"))
            .unwrap_or_else(|_| format!("term-{}", std::process::id()))
    }
}

pub struct ContextManager {
    context_dir: PathBuf,
}

impl ContextManager {
    pub fn new(config: &HubConfig) -> Self {
        let context_dir = config.hub_dir.join("contexts");
        Self { context_dir }
    }

    fn ensure_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.context_dir)
            .map_err(|e| format!("Cannot create context directory: {}", e))
    }

    fn context_file_for_terminal(&self, terminal_id: &str) -> PathBuf {
        self.context_dir.join(format!("{}.json", terminal_id))
    }

    fn current_terminal_id(&self) -> String {
        SessionContext::get_terminal_id()
    }

    pub fn save_context(&self, context: &SessionContext) -> Result<(), String> {
        self.ensure_dir()?;
        let file_path = self.context_file_for_terminal(&context.terminal_id);
        let json = serde_json::to_string_pretty(context)
            .map_err(|e| format!("Cannot serialize context: {}", e))?;
        fs::write(&file_path, json).map_err(|e| format!("Cannot write context: {}", e))
    }

    pub fn load_current_context(&self) -> Option<SessionContext> {
        let terminal_id = self.current_terminal_id();
        let file_path = self.context_file_for_terminal(&terminal_id);

        if !file_path.exists() {
            return None;
        }

        fs::read_to_string(&file_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }

    pub fn get_current_session_id(&self) -> Option<String> {
        self.load_current_context().map(|c| c.session_id)
    }

    pub fn clear_current_context(&self) -> Result<(), String> {
        let terminal_id = self.current_terminal_id();
        let file_path = self.context_file_for_terminal(&terminal_id);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Cannot remove context: {}", e))?;
        }
        Ok(())
    }

    pub fn get_all_contexts(&self) -> Vec<SessionContext> {
        if !self.context_dir.exists() {
            return Vec::new();
        }

        fs::read_dir(&self.context_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    fs::read_to_string(&path)
                        .ok()
                        .and_then(|content| serde_json::from_str(&content).ok())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_context_creation() {
        let context = SessionContext::new("test-123", "TestSession", "web");
        assert_eq!(context.session_id, "test-123");
        assert_eq!(context.session_name, "TestSession");
    }
}
