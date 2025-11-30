use super::streaming::{StreamEvent, StreamEventType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    StreamStart,
    StreamText,
    StreamThinking,
    StreamToolCall,
    StreamToolResult,
    StreamProgress,
    StreamError,
    StreamComplete,
    Ping,
    Pong,
    Subscribe,
    Unsubscribe,
}

impl WebSocketMessage {
    pub fn new(message_type: MessageType, payload: serde_json::Value) -> Self {
        Self {
            message_type,
            payload,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    pub fn from_stream_event(event: &StreamEvent) -> Self {
        let message_type = match event.event_type {
            StreamEventType::Start => MessageType::StreamStart,
            StreamEventType::Text => MessageType::StreamText,
            StreamEventType::Thinking => MessageType::StreamThinking,
            StreamEventType::ToolCall => MessageType::StreamToolCall,
            StreamEventType::ToolResult => MessageType::StreamToolResult,
            StreamEventType::Progress => MessageType::StreamProgress,
            StreamEventType::Error => MessageType::StreamError,
            StreamEventType::Complete => MessageType::StreamComplete,
        };

        let payload = serde_json::json!({
            "content": event.content,
            "metadata": event.metadata,
        });

        Self::new(message_type, payload)
    }

    pub fn ping() -> Self {
        Self::new(MessageType::Ping, serde_json::json!({}))
    }

    pub fn pong() -> Self {
        Self::new(MessageType::Pong, serde_json::json!({}))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

pub type ClientId = String;

pub struct WebSocketBroadcaster {
    clients: Arc<RwLock<HashMap<ClientId, mpsc::Sender<WebSocketMessage>>>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<ClientId>>>>,
}

impl WebSocketBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_client(&self, client_id: ClientId, sender: mpsc::Sender<WebSocketMessage>) {
        if let Ok(mut clients) = self.clients.write() {
            clients.insert(client_id, sender);
        }
    }

    pub fn remove_client(&self, client_id: &str) {
        if let Ok(mut clients) = self.clients.write() {
            clients.remove(client_id);
        }

        if let Ok(mut subs) = self.subscriptions.write() {
            for clients in subs.values_mut() {
                clients.retain(|id| id != client_id);
            }
        }
    }

    pub fn subscribe(&self, client_id: ClientId, channel: &str) {
        if let Ok(mut subs) = self.subscriptions.write() {
            subs.entry(channel.to_string())
                .or_default()
                .push(client_id);
        }
    }

    pub fn unsubscribe(&self, client_id: &str, channel: &str) {
        if let Ok(mut subs) = self.subscriptions.write() {
            if let Some(clients) = subs.get_mut(channel) {
                clients.retain(|id| id != client_id);
            }
        }
    }

    pub async fn broadcast(&self, message: WebSocketMessage) {
        let clients = match self.clients.read() {
            Ok(c) => c.clone(),
            Err(_) => return,
        };

        for sender in clients.values() {
            let _ = sender.send(message.clone()).await;
        }
    }

    pub async fn broadcast_to_channel(&self, channel: &str, message: WebSocketMessage) {
        let subscribers = match self.subscriptions.read() {
            Ok(s) => s.get(channel).cloned().unwrap_or_default(),
            Err(_) => return,
        };

        let senders: Vec<_> = {
            let clients = match self.clients.read() {
                Ok(c) => c,
                Err(_) => return,
            };
            subscribers
                .iter()
                .filter_map(|id| clients.get(id).cloned())
                .collect()
        };

        for sender in senders {
            let _ = sender.send(message.clone()).await;
        }
    }

    pub async fn send_to_client(&self, client_id: &str, message: WebSocketMessage) {
        let sender = {
            let clients = match self.clients.read() {
                Ok(c) => c,
                Err(_) => return,
            };
            clients.get(client_id).cloned()
        };

        if let Some(sender) = sender {
            let _ = sender.send(message).await;
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.read().map(|c| c.len()).unwrap_or(0)
    }

    pub fn channel_subscriber_count(&self, channel: &str) -> usize {
        self.subscriptions
            .read()
            .ok()
            .and_then(|s| s.get(channel).map(|v| v.len()))
            .unwrap_or(0)
    }
}

impl Default for WebSocketBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StreamToWebSocket {
    broadcaster: Arc<WebSocketBroadcaster>,
    channel: String,
}

impl StreamToWebSocket {
    pub fn new(broadcaster: Arc<WebSocketBroadcaster>, channel: &str) -> Self {
        Self {
            broadcaster,
            channel: channel.to_string(),
        }
    }

    pub async fn forward_event(&self, event: &StreamEvent) {
        let message = WebSocketMessage::from_stream_event(event);
        self.broadcaster
            .broadcast_to_channel(&self.channel, message)
            .await;
    }

    pub async fn forward_all(&self, events: &[StreamEvent]) {
        for event in events {
            self.forward_event(event).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_creation() {
        let msg = WebSocketMessage::new(MessageType::StreamText, serde_json::json!({"text": "hello"}));
        assert_eq!(msg.message_type, MessageType::StreamText);
    }

    #[test]
    fn test_message_from_stream_event() {
        let event = StreamEvent::new(StreamEventType::Text, "Hello world");
        let msg = WebSocketMessage::from_stream_event(&event);

        assert_eq!(msg.message_type, MessageType::StreamText);
    }

    #[test]
    fn test_message_serialization() {
        let msg = WebSocketMessage::ping();
        let json = msg.to_json();

        assert!(json.contains("Ping"));
    }

    #[test]
    fn test_broadcaster_creation() {
        let broadcaster = WebSocketBroadcaster::new();
        assert_eq!(broadcaster.client_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcaster_add_remove_client() {
        let broadcaster = WebSocketBroadcaster::new();
        let (tx, _rx) = mpsc::channel(10);

        broadcaster.add_client("client1".to_string(), tx);
        assert_eq!(broadcaster.client_count(), 1);

        broadcaster.remove_client("client1");
        assert_eq!(broadcaster.client_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcaster_subscribe() {
        let broadcaster = WebSocketBroadcaster::new();
        let (tx, _rx) = mpsc::channel(10);

        broadcaster.add_client("client1".to_string(), tx);
        broadcaster.subscribe("client1".to_string(), "stream1");

        assert_eq!(broadcaster.channel_subscriber_count("stream1"), 1);
    }
}
