use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Chat,
    System,
    Request,
    Response,
    Broadcast,
    DirectMessage,
    ToolInvocation,
    ToolResult,
    ContextUpdate,
    StatusUpdate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabMessage {
    pub id: Uuid,
    pub session_id: String,
    pub sender_id: String,
    pub recipient_id: Option<String>,
    pub message_type: MessageType,
    pub content: MessageContent,
    pub priority: MessagePriority,
    pub in_reply_to: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: MessageMetadata,
}

impl CollabMessage {
    pub fn chat(session_id: &str, sender_id: &str, text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            recipient_id: None,
            message_type: MessageType::Chat,
            content: MessageContent::Text(text.to_string()),
            priority: MessagePriority::Normal,
            in_reply_to: None,
            created_at: chrono::Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    pub fn system(session_id: &str, text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id: session_id.to_string(),
            sender_id: "system".to_string(),
            recipient_id: None,
            message_type: MessageType::System,
            content: MessageContent::Text(text.to_string()),
            priority: MessagePriority::Normal,
            in_reply_to: None,
            created_at: chrono::Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    pub fn direct(session_id: &str, sender_id: &str, recipient_id: &str, text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            recipient_id: Some(recipient_id.to_string()),
            message_type: MessageType::DirectMessage,
            content: MessageContent::Text(text.to_string()),
            priority: MessagePriority::Normal,
            in_reply_to: None,
            created_at: chrono::Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    pub fn request(session_id: &str, sender_id: &str, request: RequestPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            recipient_id: None,
            message_type: MessageType::Request,
            content: MessageContent::Request(request),
            priority: MessagePriority::Normal,
            in_reply_to: None,
            created_at: chrono::Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    pub fn response(session_id: &str, sender_id: &str, in_reply_to: Uuid, response: ResponsePayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            recipient_id: None,
            message_type: MessageType::Response,
            content: MessageContent::Response(response),
            priority: MessagePriority::Normal,
            in_reply_to: Some(in_reply_to),
            created_at: chrono::Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.extra.insert(key.to_string(), value.to_string());
        self
    }

    pub fn is_broadcast(&self) -> bool {
        self.recipient_id.is_none()
    }

    pub fn is_for(&self, agent_id: &str) -> bool {
        self.recipient_id.as_ref().map_or(true, |r| r == agent_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Request(RequestPayload),
    Response(ResponsePayload),
    ToolCall(ToolCallPayload),
    ToolResult(ToolResultPayload),
    ContextUpdate(ContextPayload),
    Status(StatusPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPayload {
    pub request_type: RequestType,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    Analysis,
    CodeReview,
    Explanation,
    Suggestion,
    Validation,
    Translation,
    Summary,
    Custom,
}

impl RequestPayload {
    pub fn analysis(description: &str) -> Self {
        Self {
            request_type: RequestType::Analysis,
            description: description.to_string(),
            parameters: serde_json::json!({}),
        }
    }

    pub fn code_review(code: &str, language: &str) -> Self {
        Self {
            request_type: RequestType::CodeReview,
            description: "Review this code".to_string(),
            parameters: serde_json::json!({
                "code": code,
                "language": language
            }),
        }
    }

    pub fn with_parameters(mut self, params: serde_json::Value) -> Self {
        self.parameters = params;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePayload {
    pub success: bool,
    pub content: String,
    pub suggestions: Vec<String>,
    pub confidence: Option<f32>,
}

impl ResponsePayload {
    pub fn success(content: &str) -> Self {
        Self {
            success: true,
            content: content.to_string(),
            suggestions: Vec::new(),
            confidence: None,
        }
    }

    pub fn failure(reason: &str) -> Self {
        Self {
            success: false,
            content: reason.to_string(),
            suggestions: Vec::new(),
            confidence: None,
        }
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallPayload {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub tool_name: String,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPayload {
    pub context_key: String,
    pub context_value: serde_json::Value,
    pub operation: ContextOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextOperation {
    Set,
    Update,
    Delete,
    Append,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusPayload {
    pub agent_status: AgentStatus,
    pub current_task: Option<String>,
    pub progress: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Processing,
    WaitingForInput,
    Error,
    Offline,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub model_used: Option<String>,
    pub tokens_used: Option<u32>,
    pub processing_time_ms: Option<u64>,
    pub extra: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message() {
        let msg = CollabMessage::chat("session-1", "agent-1", "Hello!");
        assert_eq!(msg.session_id, "session-1");
        assert_eq!(msg.sender_id, "agent-1");
        assert!(msg.is_broadcast());
        assert!(matches!(msg.content, MessageContent::Text(_)));
    }

    #[test]
    fn test_direct_message() {
        let msg = CollabMessage::direct("session-1", "agent-1", "agent-2", "Private message");
        assert!(!msg.is_broadcast());
        assert!(msg.is_for("agent-2"));
        assert!(!msg.is_for("agent-3"));
    }

    #[test]
    fn test_request_response() {
        let request = RequestPayload::analysis("Analyze this code");
        let req_msg = CollabMessage::request("session-1", "agent-1", request);

        let response = ResponsePayload::success("Analysis complete").with_confidence(0.95);
        let res_msg = CollabMessage::response("session-1", "agent-2", req_msg.id, response);

        assert_eq!(res_msg.in_reply_to, Some(req_msg.id));
    }

    #[test]
    fn test_message_priority() {
        let msg = CollabMessage::chat("session-1", "agent-1", "Urgent!")
            .with_priority(MessagePriority::Urgent);
        assert_eq!(msg.priority, MessagePriority::Urgent);
    }
}
