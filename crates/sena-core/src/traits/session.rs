use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::Result;
use crate::types::{Message, Query, QueryResponse, SessionId};

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub name: Option<String>,
    pub capabilities: Vec<String>,
    pub state: SessionState,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: SessionId::new(),
            name: None,
            capabilities: Vec::new(),
            state: SessionState::Active,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(id: SessionId) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name: None,
            capabilities: Vec::new(),
            state: SessionState::Active,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Idle,
    Suspended,
    Terminated,
}

#[async_trait]
pub trait SessionManager: Send + Sync {
    async fn create(&self, config: SessionCreateConfig) -> Result<Session>;

    async fn get(&self, id: &SessionId) -> Result<Option<Session>>;

    async fn update(&self, session: &Session) -> Result<()>;

    async fn terminate(&self, id: &SessionId) -> Result<()>;

    async fn list(&self) -> Result<Vec<Session>>;

    async fn list_active(&self) -> Result<Vec<Session>>;

    async fn find_by_capability(&self, capability: &str) -> Result<Vec<Session>>;

    async fn query(&self, target: &SessionId, query: Query) -> Result<QueryResponse>;

    async fn broadcast(&self, query: Query) -> Result<Vec<QueryResponse>>;

    async fn cleanup_expired(&self) -> Result<usize>;
}

#[derive(Debug, Clone, Default)]
pub struct SessionCreateConfig {
    pub name: Option<String>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub ttl_secs: Option<u64>,
}

impl SessionCreateConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = Some(ttl_secs);
        self
    }
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn append(&self, session_id: &SessionId, message: Message) -> Result<()>;

    async fn get_history(&self, session_id: &SessionId, limit: Option<usize>) -> Result<Vec<Message>>;

    async fn get_recent(&self, session_id: &SessionId, count: usize) -> Result<Vec<Message>>;

    async fn clear(&self, session_id: &SessionId) -> Result<()>;

    async fn count(&self, session_id: &SessionId) -> Result<usize>;

    async fn summarize(&self, session_id: &SessionId, max_tokens: usize) -> Result<String>;
}

#[async_trait]
pub trait SessionSync: Send + Sync {
    async fn sync(&self, local: &Session, remote: &Session) -> Result<Session>;

    async fn merge_conflict(&self, local: &Session, remote: &Session) -> Result<Session>;

    async fn replicate(&self, session: &Session, targets: &[SessionId]) -> Result<()>;
}
