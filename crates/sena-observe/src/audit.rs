use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sena_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type AuditId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    SessionCreate,
    SessionTerminate,
    MessageSend,
    MessageReceive,
    ProviderCall,
    ProviderFailover,
    KeyAccess,
    KeyModify,
    RateLimitHit,
    ValidationFail,
    SecurityEvent,
    ConfigChange,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: AuditId,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub outcome: AuditOutcome,
    pub actor: Option<String>,
    pub session_id: Option<String>,
    pub resource: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditEntry {
    pub fn new(action: AuditAction, outcome: AuditOutcome) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action,
            outcome,
            actor: None,
            session_id: None,
            resource: None,
            details: HashMap::new(),
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_actor(mut self, actor: &str) -> Self {
        self.actor = Some(actor.to_string());
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_resource(mut self, resource: &str) -> Self {
        self.resource = Some(resource.to_string());
        self
    }

    pub fn with_detail(mut self, key: &str, value: serde_json::Value) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }

    pub fn with_client_info(mut self, ip: &str, user_agent: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self.user_agent = Some(user_agent.to_string());
        self
    }
}

#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log(&self, entry: AuditEntry) -> Result<()>;
    async fn query(
        &self,
        filter: AuditFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AuditEntry>>;
}

#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub action: Option<AuditAction>,
    pub actor: Option<String>,
    pub session_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub outcome_success: Option<bool>,
}

impl AuditFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action(mut self, action: AuditAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn actor(mut self, actor: &str) -> Self {
        self.actor = Some(actor.to_string());
        self
    }

    pub fn session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn time_range(mut self, from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        self.from = Some(from);
        self.to = Some(to);
        self
    }

    pub fn successful_only(mut self) -> Self {
        self.outcome_success = Some(true);
        self
    }

    pub fn failed_only(mut self) -> Self {
        self.outcome_success = Some(false);
        self
    }
}

pub struct InMemoryAuditLogger {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
    max_entries: usize,
}

impl InMemoryAuditLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            max_entries,
        }
    }

    fn matches_filter(entry: &AuditEntry, filter: &AuditFilter) -> bool {
        if let Some(ref actor) = filter.actor {
            if entry.actor.as_ref() != Some(actor) {
                return false;
            }
        }

        if let Some(ref session_id) = filter.session_id {
            if entry.session_id.as_ref() != Some(session_id) {
                return false;
            }
        }

        if let Some(from) = filter.from {
            if entry.timestamp < from {
                return false;
            }
        }

        if let Some(to) = filter.to {
            if entry.timestamp > to {
                return false;
            }
        }

        if let Some(success) = filter.outcome_success {
            let is_success = matches!(entry.outcome, AuditOutcome::Success);
            if is_success != success {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl AuditLogger for InMemoryAuditLogger {
    async fn log(&self, entry: AuditEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.push(entry);

        if entries.len() > self.max_entries {
            let excess = entries.len() - self.max_entries;
            entries.drain(0..excess);
        }

        Ok(())
    }

    async fn query(
        &self,
        filter: AuditFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AuditEntry>> {
        let entries = self.entries.read().await;
        let filtered: Vec<AuditEntry> = entries
            .iter()
            .filter(|e| Self::matches_filter(e, &filter))
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }
}

pub struct FileAuditLogger {
    path: PathBuf,
    memory_logger: InMemoryAuditLogger,
}

impl FileAuditLogger {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            memory_logger: InMemoryAuditLogger::new(10000),
        }
    }

    async fn append_to_file(&self, entry: &AuditEntry) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .map_err(|e| sena_core::Error::internal(format!("Failed to open audit log: {}", e)))?;

        let json = serde_json::to_string(entry)
            .map_err(|e| sena_core::Error::internal(format!("Failed to serialize: {}", e)))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| sena_core::Error::internal(format!("Failed to write: {}", e)))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| sena_core::Error::internal(format!("Failed to write newline: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl AuditLogger for FileAuditLogger {
    async fn log(&self, entry: AuditEntry) -> Result<()> {
        self.append_to_file(&entry).await?;
        self.memory_logger.log(entry).await
    }

    async fn query(
        &self,
        filter: AuditFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AuditEntry>> {
        self.memory_logger.query(filter, limit, offset).await
    }
}

pub struct AuditService {
    logger: Arc<dyn AuditLogger>,
}

impl AuditService {
    pub fn new<L: AuditLogger + 'static>(logger: L) -> Self {
        Self {
            logger: Arc::new(logger),
        }
    }

    pub fn in_memory(max_entries: usize) -> Self {
        Self::new(InMemoryAuditLogger::new(max_entries))
    }

    pub fn file_backed(path: PathBuf) -> Self {
        Self::new(FileAuditLogger::new(path))
    }

    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        self.logger.log(entry).await
    }

    pub async fn log_session_create(&self, session_id: &str, actor: Option<&str>) -> Result<()> {
        let mut entry = AuditEntry::new(AuditAction::SessionCreate, AuditOutcome::Success)
            .with_session(session_id);
        if let Some(a) = actor {
            entry = entry.with_actor(a);
        }
        self.logger.log(entry).await
    }

    pub async fn log_session_terminate(&self, session_id: &str) -> Result<()> {
        let entry = AuditEntry::new(AuditAction::SessionTerminate, AuditOutcome::Success)
            .with_session(session_id);
        self.logger.log(entry).await
    }

    pub async fn log_provider_call(
        &self,
        provider: &str,
        model: &str,
        success: bool,
        latency_ms: u64,
    ) -> Result<()> {
        let outcome = if success {
            AuditOutcome::Success
        } else {
            AuditOutcome::Failure
        };
        let entry = AuditEntry::new(AuditAction::ProviderCall, outcome)
            .with_resource(provider)
            .with_detail("model", serde_json::json!(model))
            .with_detail("latency_ms", serde_json::json!(latency_ms));
        self.logger.log(entry).await
    }

    pub async fn log_provider_failover(&self, from: &str, to: &str, reason: &str) -> Result<()> {
        let entry = AuditEntry::new(AuditAction::ProviderFailover, AuditOutcome::Success)
            .with_detail("from", serde_json::json!(from))
            .with_detail("to", serde_json::json!(to))
            .with_detail("reason", serde_json::json!(reason));
        self.logger.log(entry).await
    }

    pub async fn log_key_access(&self, key_name: &str, actor: &str) -> Result<()> {
        let entry = AuditEntry::new(AuditAction::KeyAccess, AuditOutcome::Success)
            .with_actor(actor)
            .with_resource(key_name);
        self.logger.log(entry).await
    }

    pub async fn log_rate_limit(&self, resource: &str, limit: u32) -> Result<()> {
        let entry = AuditEntry::new(AuditAction::RateLimitHit, AuditOutcome::Denied)
            .with_resource(resource)
            .with_detail("limit", serde_json::json!(limit));
        self.logger.log(entry).await
    }

    pub async fn log_security_event(
        &self,
        event_type: &str,
        details: &str,
        severity: &str,
    ) -> Result<()> {
        let entry = AuditEntry::new(AuditAction::SecurityEvent, AuditOutcome::Success)
            .with_detail("event_type", serde_json::json!(event_type))
            .with_detail("details", serde_json::json!(details))
            .with_detail("severity", serde_json::json!(severity));
        self.logger.log(entry).await
    }

    pub async fn query(
        &self,
        filter: AuditFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AuditEntry>> {
        self.logger.query(filter, limit, offset).await
    }

    pub async fn recent(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        self.logger.query(AuditFilter::new(), limit, 0).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging() {
        let service = AuditService::in_memory(100);

        service
            .log_session_create("sess-123", Some("user-1"))
            .await
            .unwrap();
        service
            .log_provider_call("anthropic", "claude-3", true, 150)
            .await
            .unwrap();

        let entries = service.recent(10).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_audit_filter() {
        let service = AuditService::in_memory(100);

        service
            .log_session_create("sess-1", Some("user-a"))
            .await
            .unwrap();
        service
            .log_session_create("sess-2", Some("user-b"))
            .await
            .unwrap();

        let filter = AuditFilter::new().actor("user-a");
        let entries = service.query(filter, 10, 0).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].actor, Some("user-a".to_string()));
    }
}
