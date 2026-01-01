use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use sena_core::{
    Error, Message, MessageRole, MessageStore, Query, QueryResponse, Result, Session,
    SessionCreateConfig, SessionId, SessionManager, SessionState,
};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio_rusqlite::Connection;

pub struct SqliteMessageStore {
    conn: Arc<Connection>,
    max_per_session: usize,
}

impl SqliteMessageStore {
    pub async fn new(path: impl AsRef<Path>, max_per_session: usize) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)
            .await
            .map_err(|e| Error::internal(format!("Failed to open database: {}", e)))?;

        conn.call(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT NOT NULL,
                    metadata TEXT NOT NULL DEFAULT '{}',
                    created_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, id);

                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    name TEXT,
                    capabilities TEXT NOT NULL DEFAULT '[]',
                    state TEXT NOT NULL DEFAULT 'active',
                    metadata TEXT NOT NULL DEFAULT '{}',
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    expires_at INTEGER
                );
                "#,
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to initialize database: {}", e)))?;

        Ok(Self {
            conn: Arc::new(conn),
            max_per_session,
        })
    }

    pub async fn in_memory(max_per_session: usize) -> Result<Self> {
        let conn = Connection::open(":memory:")
            .await
            .map_err(|e| Error::internal(format!("Failed to open in-memory database: {}", e)))?;

        conn.call(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT NOT NULL,
                    metadata TEXT NOT NULL DEFAULT '{}',
                    created_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, id);

                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    name TEXT,
                    capabilities TEXT NOT NULL DEFAULT '[]',
                    state TEXT NOT NULL DEFAULT 'active',
                    metadata TEXT NOT NULL DEFAULT '{}',
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    expires_at INTEGER
                );
                "#,
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to initialize database: {}", e)))?;

        Ok(Self {
            conn: Arc::new(conn),
            max_per_session,
        })
    }

    async fn enforce_limit(&self, session_id: &SessionId) -> Result<()> {
        let session_str = session_id.to_string();
        let max = self.max_per_session as i64;
        let conn = self.conn.clone();

        conn.call(move |conn| {
            conn.execute(
                r#"
                DELETE FROM messages WHERE session_id = ?1 AND id NOT IN (
                    SELECT id FROM messages WHERE session_id = ?1 ORDER BY id DESC LIMIT ?2
                )
                "#,
                rusqlite::params![session_str, max],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to enforce limit: {}", e)))
    }
}

#[async_trait]
impl MessageStore for SqliteMessageStore {
    async fn append(&self, session_id: &SessionId, message: Message) -> Result<()> {
        let session_str = session_id.to_string();
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
        .to_string();
        let content = message.content;
        let metadata = serde_json::to_string(&message.metadata).unwrap_or_default();
        let now = Utc::now().timestamp();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT INTO messages (session_id, role, content, metadata, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                rusqlite::params![session_str, role, content, metadata, now],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to append message: {}", e)))?;

        self.enforce_limit(session_id).await
    }

    async fn get_history(&self, session_id: &SessionId, limit: Option<usize>) -> Result<Vec<Message>> {
        let session_str = session_id.to_string();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let sql = match limit {
                Some(n) => format!(
                    "SELECT role, content, metadata FROM messages WHERE session_id = ?1 ORDER BY id ASC LIMIT {}",
                    n
                ),
                None => "SELECT role, content, metadata FROM messages WHERE session_id = ?1 ORDER BY id ASC".to_string(),
            };

            let mut stmt = conn.prepare(&sql)?;
            let messages: Vec<Message> = stmt
                .query_map([&session_str], |row| {
                    let role_str: String = row.get(0)?;
                    let content: String = row.get(1)?;
                    let metadata_str: String = row.get(2)?;

                    let role = match role_str.as_str() {
                        "system" => MessageRole::System,
                        "user" => MessageRole::User,
                        _ => MessageRole::Assistant,
                    };

                    let metadata = serde_json::from_str(&metadata_str).unwrap_or_default();

                    Ok(Message {
                        role,
                        content,
                        metadata,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(messages)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to get history: {}", e)))
    }

    async fn get_recent(&self, session_id: &SessionId, count: usize) -> Result<Vec<Message>> {
        let session_str = session_id.to_string();
        let count = count as i64;
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT role, content, metadata FROM (
                    SELECT id, role, content, metadata FROM messages
                    WHERE session_id = ?1 ORDER BY id DESC LIMIT ?2
                ) ORDER BY id ASC
                "#,
            )?;

            let messages: Vec<Message> = stmt
                .query_map(rusqlite::params![session_str, count], |row| {
                    let role_str: String = row.get(0)?;
                    let content: String = row.get(1)?;
                    let metadata_str: String = row.get(2)?;

                    let role = match role_str.as_str() {
                        "system" => MessageRole::System,
                        "user" => MessageRole::User,
                        _ => MessageRole::Assistant,
                    };

                    let metadata = serde_json::from_str(&metadata_str).unwrap_or_default();

                    Ok(Message {
                        role,
                        content,
                        metadata,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(messages)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to get recent: {}", e)))
    }

    async fn clear(&self, session_id: &SessionId) -> Result<()> {
        let session_str = session_id.to_string();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            conn.execute("DELETE FROM messages WHERE session_id = ?1", [&session_str])?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to clear messages: {}", e)))
    }

    async fn count(&self, session_id: &SessionId) -> Result<usize> {
        let session_str = session_id.to_string();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
                [&session_str],
                |row| row.get(0),
            )?;
            Ok(count as usize)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to count messages: {}", e)))
    }

    async fn summarize(&self, session_id: &SessionId, max_tokens: usize) -> Result<String> {
        let messages = self.get_history(session_id, None).await?;

        if messages.is_empty() {
            return Ok(String::new());
        }

        let mut summary = String::new();
        let mut token_estimate = 0;

        for message in messages.iter().rev() {
            let prefix = match message.role {
                MessageRole::System => "[System] ",
                MessageRole::User => "[User] ",
                MessageRole::Assistant => "[Assistant] ",
            };

            let line = format!("{}{}\n", prefix, message.content);
            let line_tokens = line.len() / 4;

            if token_estimate + line_tokens > max_tokens {
                break;
            }

            summary.insert_str(0, &line);
            token_estimate += line_tokens;
        }

        Ok(summary.trim().to_string())
    }
}

pub struct SqliteSessionManager {
    conn: Arc<Connection>,
    cache: DashMap<SessionId, Session>,
    default_ttl: Option<u64>,
}

impl SqliteSessionManager {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)
            .await
            .map_err(|e| Error::internal(format!("Failed to open database: {}", e)))?;

        conn.call(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    name TEXT,
                    capabilities TEXT NOT NULL DEFAULT '[]',
                    state TEXT NOT NULL DEFAULT 'active',
                    metadata TEXT NOT NULL DEFAULT '{}',
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    expires_at INTEGER
                );
                "#,
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to initialize database: {}", e)))?;

        Ok(Self {
            conn: Arc::new(conn),
            cache: DashMap::new(),
            default_ttl: None,
        })
    }

    pub async fn in_memory() -> Result<Self> {
        let conn = Connection::open(":memory:")
            .await
            .map_err(|e| Error::internal(format!("Failed to open in-memory database: {}", e)))?;

        conn.call(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    name TEXT,
                    capabilities TEXT NOT NULL DEFAULT '[]',
                    state TEXT NOT NULL DEFAULT 'active',
                    metadata TEXT NOT NULL DEFAULT '{}',
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    expires_at INTEGER
                );
                "#,
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to initialize database: {}", e)))?;

        Ok(Self {
            conn: Arc::new(conn),
            cache: DashMap::new(),
            default_ttl: None,
        })
    }

    pub fn with_default_ttl(mut self, ttl_secs: u64) -> Self {
        self.default_ttl = Some(ttl_secs);
        self
    }

    fn parse_session_row(
        id_str: &str,
        name: Option<String>,
        capabilities_str: &str,
        state_str: &str,
        metadata_str: &str,
        created_at: i64,
        updated_at: i64,
    ) -> Option<Session> {
        let id = SessionId::from_str(id_str).ok()?;
        let state = match state_str {
            "active" => SessionState::Active,
            "idle" => SessionState::Idle,
            "suspended" => SessionState::Suspended,
            _ => SessionState::Terminated,
        };
        let capabilities: Vec<String> = serde_json::from_str(capabilities_str).unwrap_or_default();
        let metadata = serde_json::from_str(metadata_str).unwrap_or_default();

        Some(Session {
            id,
            name,
            capabilities,
            state,
            metadata,
            created_at: chrono::DateTime::from_timestamp(created_at, 0)
                .unwrap_or_else(chrono::Utc::now),
            updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                .unwrap_or_else(chrono::Utc::now),
        })
    }

    async fn load_session(&self, id: &SessionId) -> Result<Option<Session>> {
        let id_str = id.to_string();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, capabilities, state, metadata, created_at, updated_at FROM sessions WHERE id = ?1",
            )?;

            let session = stmt
                .query_row([&id_str], |row| {
                    let id_str: String = row.get(0)?;
                    let name: Option<String> = row.get(1)?;
                    let capabilities_str: String = row.get(2)?;
                    let state_str: String = row.get(3)?;
                    let metadata_str: String = row.get(4)?;
                    let created_at: i64 = row.get(5)?;
                    let updated_at: i64 = row.get(6)?;

                    Ok((id_str, name, capabilities_str, state_str, metadata_str, created_at, updated_at))
                })
                .ok()
                .and_then(|(id_str, name, cap, state, meta, created, updated)| {
                    Self::parse_session_row(&id_str, name, &cap, &state, &meta, created, updated)
                });

            Ok(session)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to load session: {}", e)))
    }

    async fn save_session(&self, session: &Session, expires_at: Option<i64>) -> Result<()> {
        let id_str = session.id.to_string();
        let name = session.name.clone();
        let capabilities = serde_json::to_string(&session.capabilities).unwrap_or_default();
        let state_str = match session.state {
            SessionState::Active => "active",
            SessionState::Idle => "idle",
            SessionState::Suspended => "suspended",
            SessionState::Terminated => "terminated",
        }
        .to_string();
        let metadata = serde_json::to_string(&session.metadata).unwrap_or_default();
        let created_at = session.created_at.timestamp();
        let updated_at = session.updated_at.timestamp();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT INTO sessions (id, name, capabilities, state, metadata, created_at, updated_at, expires_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(id) DO UPDATE SET
                    name = ?2, capabilities = ?3, state = ?4, metadata = ?5, updated_at = ?7, expires_at = ?8
                "#,
                rusqlite::params![id_str, name, capabilities, state_str, metadata, created_at, updated_at, expires_at],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to save session: {}", e)))
    }
}

#[async_trait]
impl SessionManager for SqliteSessionManager {
    async fn create(&self, config: SessionCreateConfig) -> Result<Session> {
        let now = Utc::now();
        let session = Session {
            id: SessionId::new(),
            name: config.name,
            capabilities: config.capabilities,
            state: SessionState::Active,
            metadata: config.metadata,
            created_at: now,
            updated_at: now,
        };

        let ttl = config.ttl_secs.or(self.default_ttl);
        let expires_at = ttl.map(|t| now.timestamp() + t as i64);

        self.save_session(&session, expires_at).await?;
        self.cache.insert(session.id, session.clone());

        Ok(session)
    }

    async fn get(&self, id: &SessionId) -> Result<Option<Session>> {
        if let Some(session) = self.cache.get(id) {
            return Ok(Some(session.clone()));
        }

        if let Some(session) = self.load_session(id).await? {
            self.cache.insert(*id, session.clone());
            return Ok(Some(session));
        }

        Ok(None)
    }

    async fn list(&self) -> Result<Vec<Session>> {
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, capabilities, state, metadata, created_at, updated_at FROM sessions WHERE state != 'terminated' ORDER BY updated_at DESC",
            )?;

            let sessions: Vec<Session> = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: Option<String> = row.get(1)?;
                    let capabilities_str: String = row.get(2)?;
                    let state_str: String = row.get(3)?;
                    let metadata_str: String = row.get(4)?;
                    let created_at: i64 = row.get(5)?;
                    let updated_at: i64 = row.get(6)?;

                    Ok((id_str, name, capabilities_str, state_str, metadata_str, created_at, updated_at))
                })?
                .filter_map(|r| r.ok())
                .filter_map(|(id_str, name, cap, state, meta, created, updated)| {
                    Self::parse_session_row(&id_str, name, &cap, &state, &meta, created, updated)
                })
                .collect();

            Ok(sessions)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to list sessions: {}", e)))
    }

    async fn list_active(&self) -> Result<Vec<Session>> {
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, capabilities, state, metadata, created_at, updated_at FROM sessions WHERE state = 'active' ORDER BY updated_at DESC",
            )?;

            let sessions: Vec<Session> = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: Option<String> = row.get(1)?;
                    let capabilities_str: String = row.get(2)?;
                    let state_str: String = row.get(3)?;
                    let metadata_str: String = row.get(4)?;
                    let created_at: i64 = row.get(5)?;
                    let updated_at: i64 = row.get(6)?;

                    Ok((id_str, name, capabilities_str, state_str, metadata_str, created_at, updated_at))
                })?
                .filter_map(|r| r.ok())
                .filter_map(|(id_str, name, cap, state, meta, created, updated)| {
                    Self::parse_session_row(&id_str, name, &cap, &state, &meta, created, updated)
                })
                .collect();

            Ok(sessions)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to list active sessions: {}", e)))
    }

    async fn find_by_capability(&self, capability: &str) -> Result<Vec<Session>> {
        let capability = capability.to_string();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, capabilities, state, metadata, created_at, updated_at FROM sessions WHERE state = 'active' ORDER BY updated_at DESC",
            )?;

            let sessions: Vec<Session> = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: Option<String> = row.get(1)?;
                    let capabilities_str: String = row.get(2)?;
                    let state_str: String = row.get(3)?;
                    let metadata_str: String = row.get(4)?;
                    let created_at: i64 = row.get(5)?;
                    let updated_at: i64 = row.get(6)?;

                    Ok((id_str, name, capabilities_str, state_str, metadata_str, created_at, updated_at))
                })?
                .filter_map(|r| r.ok())
                .filter_map(|(id_str, name, cap, state, meta, created, updated)| {
                    Self::parse_session_row(&id_str, name, &cap, &state, &meta, created, updated)
                })
                .filter(|s| s.capabilities.contains(&capability))
                .collect();

            Ok(sessions)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to find sessions by capability: {}", e)))
    }

    async fn update(&self, session: &Session) -> Result<()> {
        let mut updated = session.clone();
        updated.updated_at = Utc::now();
        self.save_session(&updated, None).await?;
        self.cache.insert(session.id, updated);
        Ok(())
    }

    async fn terminate(&self, id: &SessionId) -> Result<()> {
        if let Some(mut session) = self.get(id).await? {
            session.state = SessionState::Terminated;
            session.updated_at = Utc::now();
            self.save_session(&session, None).await?;
            self.cache.remove(id);
        }
        Ok(())
    }

    async fn query(&self, target: &SessionId, query: Query) -> Result<QueryResponse> {
        let session = self.get(target).await?
            .ok_or_else(|| Error::not_found(format!("Session {} not found", target)))?;

        Ok(QueryResponse {
            source: session.id,
            query_type: query.query_type,
            content: format!("Query received: {}", query.content),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn broadcast(&self, query: Query) -> Result<Vec<QueryResponse>> {
        let sessions = self.list_active().await?;
        let mut responses = Vec::new();

        for session in sessions {
            responses.push(QueryResponse {
                source: session.id,
                query_type: query.query_type,
                content: format!("Broadcast received: {}", query.content),
                metadata: std::collections::HashMap::new(),
            });
        }

        Ok(responses)
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let now = Utc::now().timestamp();
        let conn = self.conn.clone();

        let count = conn.call(move |conn| {
            let deleted = conn.execute(
                "DELETE FROM sessions WHERE expires_at IS NOT NULL AND expires_at < ?1",
                [now],
            )?;
            Ok(deleted)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to cleanup expired sessions: {}", e)))?;

        self.cache.retain(|_, session| session.state != SessionState::Terminated);

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_message_store() {
        let store = SqliteMessageStore::in_memory(100).await.unwrap();
        let session_id = SessionId::new();

        store.append(&session_id, Message::user("Hello")).await.unwrap();
        store.append(&session_id, Message::assistant("Hi there")).await.unwrap();

        let count = store.count(&session_id).await.unwrap();
        assert_eq!(count, 2);

        let recent = store.get_recent(&session_id, 1).await.unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].content, "Hi there");

        let history = store.get_history(&session_id, None).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_sqlite_session_manager() {
        let manager = SqliteSessionManager::in_memory().await.unwrap();

        let config = SessionCreateConfig::default().with_name("test-session");
        let session = manager.create(config).await.unwrap();
        assert_eq!(session.name, Some("test-session".to_string()));

        let retrieved = manager.get(&session.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, Some("test-session".to_string()));

        let sessions = manager.list().await.unwrap();
        assert_eq!(sessions.len(), 1);

        manager.terminate(&session.id).await.unwrap();
        let sessions = manager.list().await.unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_message_limit() {
        let store = SqliteMessageStore::in_memory(5).await.unwrap();
        let session_id = SessionId::new();

        for i in 0..10 {
            store
                .append(&session_id, Message::user(format!("Message {}", i)))
                .await
                .unwrap();
        }

        let count = store.count(&session_id).await.unwrap();
        assert_eq!(count, 5);

        let history = store.get_history(&session_id, None).await.unwrap();
        assert_eq!(history[0].content, "Message 5");
    }

    #[tokio::test]
    async fn test_find_by_capability() {
        let manager = SqliteSessionManager::in_memory().await.unwrap();

        let config = SessionCreateConfig::default()
            .with_name("session1")
            .with_capability("code");
        manager.create(config).await.unwrap();

        let config = SessionCreateConfig::default()
            .with_name("session2")
            .with_capability("chat");
        manager.create(config).await.unwrap();

        let code_sessions = manager.find_by_capability("code").await.unwrap();
        assert_eq!(code_sessions.len(), 1);
        assert_eq!(code_sessions[0].name, Some("session1".to_string()));
    }
}
