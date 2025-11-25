//! Inter-Session Messaging System
//!
//! Real-time messaging between collaborative sessions

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use super::HubConfig;

/// Message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Direct,      // One-to-one
    Broadcast,   // One-to-all
    System,      // System notification
    Alert,       // Important alert
    TaskUpdate,  // Task-related update
}

impl MessageType {
    pub fn emoji(&self) -> &'static str {
        match self {
            MessageType::Direct => "💬",
            MessageType::Broadcast => "📢",
            MessageType::System => "⚙️",
            MessageType::Alert => "🚨",
            MessageType::TaskUpdate => "📋",
        }
    }
}

/// A message between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,  // Session ID or "all" for broadcast
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: u64,
    pub read: bool,
}

impl Message {
    /// Create a new message
    pub fn new(from: &str, to: &str, content: &str, message_type: MessageType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = format!("{}-{}-{}", from, to, timestamp);

        Self {
            id,
            from: from.to_string(),
            to: to.to_string(),
            content: content.to_string(),
            message_type,
            timestamp,
            read: false,
        }
    }

    /// Create a broadcast message
    pub fn broadcast(from: &str, content: &str) -> Self {
        Self::new(from, "all", content, MessageType::Broadcast)
    }

    /// Create a system message
    pub fn system(content: &str) -> Self {
        Self::new("system", "all", content, MessageType::System)
    }

    /// Create an alert message
    pub fn alert(from: &str, to: &str, content: &str) -> Self {
        Self::new(from, to, content, MessageType::Alert)
    }

    /// Format timestamp for display
    pub fn time_display(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let diff = now - self.timestamp;

        if diff < 60 {
            format!("{}s ago", diff)
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }

    /// Format for display
    pub fn display_line(&self) -> String {
        format!(
            "[{}] {} {} → {}: {}",
            self.time_display(),
            self.message_type.emoji(),
            self.from,
            self.to,
            self.content
        )
    }
}

/// Broadcast message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Broadcast {
    pub message: Message,
    pub recipients: Vec<String>,
    pub delivered_to: Vec<String>,
}

impl Broadcast {
    pub fn new(message: Message, recipients: Vec<String>) -> Self {
        Self {
            message,
            recipients,
            delivered_to: Vec::new(),
        }
    }
}

/// Persisted messages data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessagesData {
    version: String,
    messages: Vec<Message>,
    last_updated: u64,
}

/// Message Queue
pub struct MessageQueue {
    messages: Vec<Message>,
    messages_dir: PathBuf,
}

impl MessageQueue {
    /// Create a new message queue
    pub fn new(config: &HubConfig) -> Self {
        Self {
            messages: Vec::new(),
            messages_dir: config.messages_dir.clone(),
        }
    }

    /// Send a direct message
    pub fn send(&mut self, from: &str, to: &str, content: &str) -> Result<(), String> {
        let message = Message::new(from, to, content, MessageType::Direct);
        self.messages.push(message.clone());

        // Also save to recipient's inbox file
        self.save_to_inbox(to, &message)?;

        Ok(())
    }

    /// Send a broadcast message
    pub fn broadcast(&mut self, from: &str, content: &str) -> Result<(), String> {
        let message = Message::broadcast(from, content);
        self.messages.push(message.clone());

        // Save to broadcast file
        self.save_broadcast(&message)?;

        Ok(())
    }

    /// Send a system message
    pub fn system_message(&mut self, content: &str) -> Result<(), String> {
        let message = Message::system(content);
        self.messages.push(message.clone());
        self.save_broadcast(&message)?;
        Ok(())
    }

    /// Send an alert
    pub fn alert(&mut self, from: &str, to: &str, content: &str) -> Result<(), String> {
        let message = Message::alert(from, to, content);
        self.messages.push(message.clone());
        self.save_to_inbox(to, &message)?;
        Ok(())
    }

    /// Get inbox for a session
    pub fn get_inbox(&self, session_id: &str) -> Vec<Message> {
        // Get direct messages to this session + broadcasts
        self.messages
            .iter()
            .filter(|m| m.to == session_id || m.to == "all")
            .cloned()
            .collect()
    }

    /// Get unread messages for a session
    pub fn get_unread(&self, session_id: &str) -> Vec<Message> {
        self.messages
            .iter()
            .filter(|m| (m.to == session_id || m.to == "all") && !m.read)
            .cloned()
            .collect()
    }

    /// Mark message as read
    pub fn mark_read(&mut self, message_id: &str) {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == message_id) {
            msg.read = true;
        }
    }

    /// Mark all messages for a session as read
    pub fn mark_all_read(&mut self, session_id: &str) {
        for msg in self.messages.iter_mut() {
            if msg.to == session_id || msg.to == "all" {
                msg.read = true;
            }
        }
    }

    /// Get recent messages (last N)
    pub fn get_recent(&self, count: usize) -> Vec<Message> {
        let mut messages: Vec<Message> = self.messages.clone();
        messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        messages.into_iter().take(count).collect()
    }

    /// Get messages between two sessions
    pub fn get_conversation(&self, session1: &str, session2: &str) -> Vec<Message> {
        self.messages
            .iter()
            .filter(|m| {
                (m.from == session1 && m.to == session2) ||
                (m.from == session2 && m.to == session1)
            })
            .cloned()
            .collect()
    }

    /// Clear old messages (older than N seconds)
    pub fn cleanup(&mut self, max_age_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.messages.retain(|m| now - m.timestamp < max_age_secs);
    }

    /// Save message to inbox file
    fn save_to_inbox(&self, session_id: &str, message: &Message) -> Result<(), String> {
        let inbox_file = self.messages_dir.join(format!("{}.json", session_id));

        let mut inbox: Vec<Message> = if inbox_file.exists() {
            let content = fs::read_to_string(&inbox_file)
                .map_err(|e| format!("Cannot read inbox: {}", e))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        inbox.push(message.clone());

        let json = serde_json::to_string_pretty(&inbox)
            .map_err(|e| format!("Cannot serialize inbox: {}", e))?;

        fs::write(&inbox_file, json)
            .map_err(|e| format!("Cannot write inbox: {}", e))?;

        Ok(())
    }

    /// Save broadcast message
    fn save_broadcast(&self, message: &Message) -> Result<(), String> {
        let broadcast_file = self.messages_dir.join("broadcast.json");

        let mut broadcasts: Vec<Message> = if broadcast_file.exists() {
            let content = fs::read_to_string(&broadcast_file)
                .map_err(|e| format!("Cannot read broadcasts: {}", e))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        broadcasts.push(message.clone());

        // Keep only last 100 broadcasts
        if broadcasts.len() > 100 {
            broadcasts = broadcasts.split_off(broadcasts.len() - 100);
        }

        let json = serde_json::to_string_pretty(&broadcasts)
            .map_err(|e| format!("Cannot serialize broadcasts: {}", e))?;

        fs::write(&broadcast_file, json)
            .map_err(|e| format!("Cannot write broadcasts: {}", e))?;

        Ok(())
    }

    /// Load messages from disk
    pub fn load(&mut self) -> Result<(), String> {
        // Load broadcasts
        let broadcast_file = self.messages_dir.join("broadcast.json");
        if broadcast_file.exists() {
            let content = fs::read_to_string(&broadcast_file)
                .map_err(|e| format!("Cannot read broadcasts: {}", e))?;
            let broadcasts: Vec<Message> = serde_json::from_str(&content).unwrap_or_default();
            self.messages.extend(broadcasts);
        }

        Ok(())
    }

    /// Get message count
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    /// Get unread count for a session
    pub fn unread_count(&self, session_id: &str) -> usize {
        self.get_unread(session_id).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new("from", "to", "Hello", MessageType::Direct);
        assert_eq!(msg.from, "from");
        assert_eq!(msg.to, "to");
        assert!(!msg.read);
    }

    #[test]
    fn test_broadcast_message() {
        let msg = Message::broadcast("sender", "Hello everyone!");
        assert_eq!(msg.to, "all");
        assert_eq!(msg.message_type, MessageType::Broadcast);
    }

    #[test]
    fn test_message_queue() {
        let config = HubConfig::new();
        let mut queue = MessageQueue::new(&config);

        let _ = queue.send("web", "backend", "API ready?");
        assert_eq!(queue.count(), 1);

        let inbox = queue.get_inbox("backend");
        assert_eq!(inbox.len(), 1);
    }
}
