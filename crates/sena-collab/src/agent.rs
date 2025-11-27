use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::Result,
    message::{AgentStatus, CollabMessage},
    permission::PermissionSet,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
}

impl AgentInfo {
    pub fn new(provider: &str, model: &str) -> Self {
        let id = format!(
            "{}_{}",
            provider,
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );
        let name = format!("{} ({})", provider, model);

        Self {
            id,
            name,
            provider: provider.to_string(),
            model: model.to_string(),
            capabilities: Vec::new(),
            status: AgentStatus::Idle,
            joined_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn update_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.last_active = chrono::Utc::now();
    }

    pub fn touch(&mut self) {
        self.last_active = chrono::Utc::now();
    }

    pub fn is_available(&self) -> bool {
        !matches!(self.status, AgentStatus::Offline | AgentStatus::Error)
    }

    pub fn idle_duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.last_active
    }
}

#[async_trait]
pub trait CollabAgent: Send + Sync {
    fn info(&self) -> &AgentInfo;

    fn permissions(&self) -> &PermissionSet;

    async fn process_message(&self, message: &CollabMessage) -> Result<Option<CollabMessage>>;

    async fn handle_request(&self, message: &CollabMessage) -> Result<CollabMessage>;

    fn update_status(&mut self, status: AgentStatus);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub strength: f32,
}

impl AgentCapability {
    pub fn new(name: &str, description: &str, strength: f32) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            strength: strength.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: std::collections::HashMap<String, AgentInfo>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, agent: AgentInfo) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn unregister(&mut self, agent_id: &str) -> Option<AgentInfo> {
        self.agents.remove(agent_id)
    }

    pub fn get(&self, agent_id: &str) -> Option<&AgentInfo> {
        self.agents.get(agent_id)
    }

    pub fn get_mut(&mut self, agent_id: &str) -> Option<&mut AgentInfo> {
        self.agents.get_mut(agent_id)
    }

    pub fn list(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }

    pub fn available(&self) -> Vec<&AgentInfo> {
        self.agents.values().filter(|a| a.is_available()).collect()
    }

    pub fn by_provider(&self, provider: &str) -> Vec<&AgentInfo> {
        self.agents
            .values()
            .filter(|a| a.provider == provider)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_creation() {
        let agent = AgentInfo::new("claude", "claude-sonnet-4-5");
        assert!(agent.id.starts_with("claude_"));
        assert_eq!(agent.provider, "claude");
        assert!(agent.is_available());
    }

    #[test]
    fn test_agent_status_update() {
        let mut agent = AgentInfo::new("openai", "gpt-4.1");
        agent.update_status(AgentStatus::Thinking);
        assert_eq!(agent.status, AgentStatus::Thinking);

        agent.update_status(AgentStatus::Offline);
        assert!(!agent.is_available());
    }

    #[test]
    fn test_agent_registry() {
        let mut registry = AgentRegistry::new();
        assert!(registry.is_empty());

        let agent = AgentInfo::new("claude", "claude-sonnet-4-5");
        let agent_id = agent.id.clone();
        registry.register(agent);

        assert_eq!(registry.count(), 1);
        assert!(registry.get(&agent_id).is_some());

        let agents = registry.by_provider("claude");
        assert_eq!(agents.len(), 1);
    }

    #[test]
    fn test_agent_capability() {
        let cap = AgentCapability::new("code_review", "Review code for issues", 0.9);
        assert_eq!(cap.strength, 0.9);

        let capped = AgentCapability::new("test", "Test", 1.5);
        assert_eq!(capped.strength, 1.0);
    }
}
