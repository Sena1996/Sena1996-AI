use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub type EventId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Request,
    Response,
    Error,
    Metric,
    Session,
    Security,
    Cache,
    Provider,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub event_type: EventType,
    pub level: EventLevel,
    pub source: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
}

impl Event {
    pub fn new(event_type: EventType, level: EventLevel, source: &str, message: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            level,
            source: source.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            span_id: None,
            trace_id: None,
        }
    }

    pub fn info(source: &str, message: &str) -> Self {
        Self::new(EventType::Custom("info".to_string()), EventLevel::Info, source, message)
    }

    pub fn error(source: &str, message: &str) -> Self {
        Self::new(EventType::Error, EventLevel::Error, source, message)
    }

    pub fn request(source: &str, message: &str) -> Self {
        Self::new(EventType::Request, EventLevel::Info, source, message)
    }

    pub fn response(source: &str, message: &str) -> Self {
        Self::new(EventType::Response, EventLevel::Info, source, message)
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    pub fn with_trace(mut self, trace_id: &str, span_id: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self.span_id = Some(span_id.to_string());
        self
    }
}

pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Event);
    fn event_types(&self) -> Vec<EventType>;
}

const BUS_CAPACITY: usize = 4096;

pub struct EventBus {
    sender: broadcast::Sender<Event>,
    handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
    running: Arc<RwLock<bool>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BUS_CAPACITY);
        Self {
            sender,
            handlers: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn emit(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) {
        self.handlers.write().await.push(handler);
    }

    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let mut receiver = self.sender.subscribe();
        let handlers = self.handlers.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().await {
                match receiver.recv().await {
                    Ok(event) => {
                        let handlers = handlers.read().await;
                        for handler in handlers.iter() {
                            handler.handle(&event);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Event bus lagged by {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });
    }

    pub async fn stop(&self) {
        *self.running.write().await = false;
    }

    pub fn emit_request(&self, source: &str, message: &str, metadata: HashMap<String, serde_json::Value>) {
        let mut event = Event::request(source, message);
        event.metadata = metadata;
        self.emit(event);
    }

    pub fn emit_response(&self, source: &str, message: &str, duration_ms: u64) {
        let event = Event::response(source, message)
            .with_metadata("duration_ms", serde_json::json!(duration_ms));
        self.emit(event);
    }

    pub fn emit_error(&self, source: &str, error: &str, details: Option<&str>) {
        let mut event = Event::error(source, error);
        if let Some(d) = details {
            event = event.with_metadata("details", serde_json::json!(d));
        }
        self.emit(event);
    }

    pub fn emit_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let event = Event::new(EventType::Metric, EventLevel::Debug, "metrics", name)
            .with_metadata("value", serde_json::json!(value))
            .with_metadata("labels", serde_json::json!(labels));
        self.emit(event);
    }

    pub fn emit_session(&self, session_id: &str, action: &str) {
        let event = Event::new(EventType::Session, EventLevel::Info, "session", action)
            .with_metadata("session_id", serde_json::json!(session_id));
        self.emit(event);
    }

    pub fn emit_security(&self, source: &str, message: &str, severity: &str) {
        let event = Event::new(EventType::Security, EventLevel::Warn, source, message)
            .with_metadata("severity", serde_json::json!(severity));
        self.emit(event);
    }

    pub fn emit_cache(&self, action: &str, key: &str, hit: bool) {
        let event = Event::new(EventType::Cache, EventLevel::Debug, "cache", action)
            .with_metadata("key", serde_json::json!(key))
            .with_metadata("hit", serde_json::json!(hit));
        self.emit(event);
    }

    pub fn emit_provider(&self, provider: &str, action: &str, success: bool) {
        let level = if success { EventLevel::Info } else { EventLevel::Warn };
        let event = Event::new(EventType::Provider, level, provider, action)
            .with_metadata("success", serde_json::json!(success));
        self.emit(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LoggingHandler {
    min_level: EventLevel,
}

impl LoggingHandler {
    pub fn new(min_level: EventLevel) -> Self {
        Self { min_level }
    }

    fn should_log(&self, level: &EventLevel) -> bool {
        let level_order = |l: &EventLevel| match l {
            EventLevel::Debug => 0,
            EventLevel::Info => 1,
            EventLevel::Warn => 2,
            EventLevel::Error => 3,
            EventLevel::Critical => 4,
        };
        level_order(level) >= level_order(&self.min_level)
    }
}

impl EventHandler for LoggingHandler {
    fn handle(&self, event: &Event) {
        if !self.should_log(&event.level) {
            return;
        }

        match event.level {
            EventLevel::Debug => tracing::debug!(
                event_type = ?event.event_type,
                source = %event.source,
                message = %event.message,
                "Event"
            ),
            EventLevel::Info => tracing::info!(
                event_type = ?event.event_type,
                source = %event.source,
                message = %event.message,
                "Event"
            ),
            EventLevel::Warn => tracing::warn!(
                event_type = ?event.event_type,
                source = %event.source,
                message = %event.message,
                "Event"
            ),
            EventLevel::Error | EventLevel::Critical => tracing::error!(
                event_type = ?event.event_type,
                source = %event.source,
                message = %event.message,
                "Event"
            ),
        }
    }

    fn event_types(&self) -> Vec<EventType> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_emit() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe();

        let event = Event::info("test", "test message");
        bus.emit(event.clone());

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.source, "test");
        assert_eq!(received.message, "test message");
    }

    #[test]
    fn test_event_builder() {
        let event = Event::request("api", "GET /users")
            .with_metadata("method", serde_json::json!("GET"))
            .with_trace("trace-123", "span-456");

        assert_eq!(event.trace_id, Some("trace-123".to_string()));
        assert_eq!(event.span_id, Some("span-456".to_string()));
        assert!(event.metadata.contains_key("method"));
    }
}
