use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    agent::AgentInfo,
    error::{CollabError, Result},
    message::{AgentStatus, CollabMessage},
    permission::{Permission, PermissionSet},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Initializing,
    Active,
    Paused,
    Completed,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    Discussion,
    CodeReview,
    PairProgramming,
    Analysis,
    Brainstorm,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_type: SessionType,
    pub max_participants: usize,
    pub message_history_limit: usize,
    pub require_permission_for_tools: bool,
    pub auto_summarize: bool,
    pub timeout_minutes: Option<u32>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_type: SessionType::Discussion,
            max_participants: 10,
            message_history_limit: 1000,
            require_permission_for_tools: true,
            auto_summarize: true,
            timeout_minutes: Some(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabSession {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub config: SessionConfig,
    pub state: SessionState,
    pub host_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    participants: HashMap<String, Participant>,
    messages: Vec<CollabMessage>,
    context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub agent: AgentInfo,
    pub permissions: PermissionSet,
    pub is_host: bool,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

impl CollabSession {
    pub fn new(name: &str, host: AgentInfo) -> Self {
        let session_id = format!(
            "session_{}",
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );
        let host_id = host.id.clone();

        let host_participant = Participant {
            agent: host,
            permissions: PermissionSet::new(Permission::session_host()),
            is_host: true,
            joined_at: chrono::Utc::now(),
        };

        let mut participants = HashMap::new();
        participants.insert(host_id.clone(), host_participant);

        Self {
            id: session_id,
            name: name.to_string(),
            description: None,
            config: SessionConfig::default(),
            state: SessionState::Initializing,
            host_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            participants,
            messages: Vec::new(),
            context: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_config(mut self, config: SessionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn start(&mut self) -> Result<()> {
        if self.state != SessionState::Initializing {
            return Err(CollabError::InvalidState(
                "Session can only start from initializing state".into(),
            ));
        }
        self.state = SessionState::Active;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        if self.state != SessionState::Active {
            return Err(CollabError::InvalidState(
                "Session can only pause from active state".into(),
            ));
        }
        self.state = SessionState::Paused;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        if self.state != SessionState::Paused {
            return Err(CollabError::InvalidState(
                "Session can only resume from paused state".into(),
            ));
        }
        self.state = SessionState::Active;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn complete(&mut self) -> Result<()> {
        if !matches!(self.state, SessionState::Active | SessionState::Paused) {
            return Err(CollabError::InvalidState(
                "Session can only complete from active or paused state".into(),
            ));
        }
        self.state = SessionState::Completed;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn terminate(&mut self) {
        self.state = SessionState::Terminated;
        self.updated_at = chrono::Utc::now();
    }

    pub fn add_participant(&mut self, agent: AgentInfo, permissions: PermissionSet) -> Result<()> {
        if self.participants.len() >= self.config.max_participants {
            return Err(CollabError::SessionLimitReached(
                self.config.max_participants,
            ));
        }

        if self.participants.contains_key(&agent.id) {
            return Err(CollabError::AgentUnavailable(format!(
                "Agent {} already in session",
                agent.id
            )));
        }

        let participant = Participant {
            agent,
            permissions,
            is_host: false,
            joined_at: chrono::Utc::now(),
        };

        self.participants
            .insert(participant.agent.id.clone(), participant);
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn remove_participant(&mut self, agent_id: &str) -> Result<Option<Participant>> {
        if agent_id == self.host_id {
            return Err(CollabError::PermissionDenied(
                "Cannot remove the session host".into(),
            ));
        }
        self.updated_at = chrono::Utc::now();
        Ok(self.participants.remove(agent_id))
    }

    pub fn get_participant(&self, agent_id: &str) -> Option<&Participant> {
        self.participants.get(agent_id)
    }

    pub fn get_participant_mut(&mut self, agent_id: &str) -> Option<&mut Participant> {
        self.participants.get_mut(agent_id)
    }

    pub fn participants(&self) -> Vec<&Participant> {
        self.participants.values().collect()
    }

    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    pub fn add_message(&mut self, message: CollabMessage) {
        self.messages.push(message);
        self.updated_at = chrono::Utc::now();

        if self.messages.len() > self.config.message_history_limit {
            self.messages.remove(0);
        }
    }

    pub fn messages(&self) -> &[CollabMessage] {
        &self.messages
    }

    pub fn recent_messages(&self, count: usize) -> Vec<&CollabMessage> {
        self.messages.iter().rev().take(count).collect()
    }

    pub fn messages_from(&self, agent_id: &str) -> Vec<&CollabMessage> {
        self.messages
            .iter()
            .filter(|m| m.sender_id == agent_id)
            .collect()
    }

    pub fn set_context(&mut self, key: &str, value: serde_json::Value) {
        self.context.insert(key.to_string(), value);
        self.updated_at = chrono::Utc::now();
    }

    pub fn get_context(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get(key)
    }

    pub fn context(&self) -> &HashMap<String, serde_json::Value> {
        &self.context
    }

    pub fn has_permission(&self, agent_id: &str, permission: Permission) -> bool {
        self.participants
            .get(agent_id)
            .is_some_and(|p| p.permissions.has(permission))
    }

    pub fn is_host(&self, agent_id: &str) -> bool {
        self.host_id == agent_id
    }

    pub fn is_active(&self) -> bool {
        self.state == SessionState::Active
    }

    pub fn update_agent_status(&mut self, agent_id: &str, status: AgentStatus) {
        if let Some(participant) = self.participants.get_mut(agent_id) {
            participant.agent.update_status(status);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionsData {
    version: String,
    sessions: HashMap<String, CollabSession>,
}

#[derive(Debug)]
pub struct SessionManager {
    sessions: HashMap<String, CollabSession>,
    max_sessions: usize,
    sessions_file: PathBuf,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(100)
    }
}

impl SessionManager {
    pub fn new(max_sessions: usize) -> Self {
        let sessions_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("collab")
            .join("sessions.json");

        let mut manager = Self {
            sessions: HashMap::new(),
            max_sessions,
            sessions_file,
        };

        let _ = manager.load();
        manager
    }

    pub fn load(&mut self) -> Result<()> {
        if !self.sessions_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.sessions_file)
            .map_err(|e| CollabError::ConfigError(format!("Cannot read sessions: {}", e)))?;

        let data: SessionsData = serde_json::from_str(&content)
            .map_err(|e| CollabError::ConfigError(format!("Cannot parse sessions: {}", e)))?;

        self.sessions = data.sessions;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.sessions_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CollabError::ConfigError(format!("Cannot create dir: {}", e)))?;
        }

        let data = SessionsData {
            version: "12.0.4".to_string(),
            sessions: self.sessions.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| CollabError::ConfigError(format!("Cannot serialize: {}", e)))?;

        fs::write(&self.sessions_file, json)
            .map_err(|e| CollabError::ConfigError(format!("Cannot write sessions: {}", e)))?;

        Ok(())
    }

    pub fn create_session(&mut self, name: &str, host: AgentInfo) -> Result<&CollabSession> {
        if self.sessions.len() >= self.max_sessions {
            return Err(CollabError::SessionLimitReached(self.max_sessions));
        }

        let session = CollabSession::new(name, host);
        let session_id = session.id.clone();
        self.sessions.insert(session_id.clone(), session);
        let _ = self.save();
        self.sessions
            .get(&session_id)
            .ok_or(CollabError::SessionNotFound(session_id))
    }

    pub fn get_session(&self, session_id: &str) -> Option<&CollabSession> {
        self.sessions.get(session_id)
    }

    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut CollabSession> {
        self.sessions.get_mut(session_id)
    }

    pub fn remove_session(&mut self, session_id: &str) -> Option<CollabSession> {
        let session = self.sessions.remove(session_id);
        let _ = self.save();
        session
    }

    pub fn list_sessions(&self) -> Vec<&CollabSession> {
        self.sessions.values().collect()
    }

    pub fn active_sessions(&self) -> Vec<&CollabSession> {
        self.sessions.values().filter(|s| s.is_active()).collect()
    }

    pub fn sessions_with_agent(&self, agent_id: &str) -> Vec<&CollabSession> {
        self.sessions
            .values()
            .filter(|s| s.get_participant(agent_id).is_some())
            .collect()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_agent() -> AgentInfo {
        AgentInfo::new("claude", "claude-sonnet-4-5")
    }

    #[test]
    fn test_session_creation() {
        let host = test_agent();
        let session = CollabSession::new("Test Session", host);

        assert_eq!(session.name, "Test Session");
        assert_eq!(session.state, SessionState::Initializing);
        assert_eq!(session.participant_count(), 1);
    }

    #[test]
    fn test_session_lifecycle() {
        let host = test_agent();
        let mut session = CollabSession::new("Test", host);

        assert!(session.start().is_ok());
        assert_eq!(session.state, SessionState::Active);

        assert!(session.pause().is_ok());
        assert_eq!(session.state, SessionState::Paused);

        assert!(session.resume().is_ok());
        assert_eq!(session.state, SessionState::Active);

        assert!(session.complete().is_ok());
        assert_eq!(session.state, SessionState::Completed);
    }

    #[test]
    fn test_add_participant() {
        let host = test_agent();
        let mut session = CollabSession::new("Test", host);

        let participant = AgentInfo::new("openai", "gpt-4.1");
        let permissions = PermissionSet::new(Permission::standard_agent());

        assert!(session.add_participant(participant, permissions).is_ok());
        assert_eq!(session.participant_count(), 2);
    }

    #[test]
    fn test_session_messages() {
        let host = test_agent();
        let host_id = host.id.clone();
        let mut session = CollabSession::new("Test", host);

        let msg = CollabMessage::chat(&session.id, &host_id, "Hello!");
        session.add_message(msg);

        assert_eq!(session.messages().len(), 1);
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new(10);
        let host = test_agent();

        let session = manager.create_session("Test", host).unwrap();
        let session_id = session.id.clone();

        assert_eq!(manager.session_count(), 1);
        assert!(manager.get_session(&session_id).is_some());
    }

    #[test]
    fn test_session_context() {
        let host = test_agent();
        let mut session = CollabSession::new("Test", host);

        session.set_context("task", serde_json::json!({"type": "review"}));
        assert!(session.get_context("task").is_some());
    }

    #[test]
    fn test_host_cannot_be_removed() {
        let host = test_agent();
        let host_id = host.id.clone();
        let mut session = CollabSession::new("Test", host);

        let result = session.remove_participant(&host_id);
        assert!(result.is_err());
    }
}
