use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    metadata::{claude_metadata, ProviderMetadata},
    provider::{AIProvider, ChatStream},
    ChatRequest, ChatResponse, FinishReason, Message, MessageContent, ModelInfo,
    ProviderCapabilities, ProviderConfig, ProviderError, ProviderStatus, Result, Role, StreamChunk,
    Usage,
};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_API_VERSION: &str = "2023-06-01";

pub struct ClaudeProvider {
    client: Client,
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    status: ProviderStatus,
}

impl ClaudeProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config
            .get_api_key()
            .ok_or_else(|| ProviderError::NotConfigured("ANTHROPIC_API_KEY not set".into()))?;

        let client = Client::builder()
            .default_headers(Self::build_headers(&api_key)?)
            .timeout(std::time::Duration::from_secs(
                config.timeout_secs.unwrap_or(120),
            ))
            .build()
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let capabilities = Self::build_capabilities();

        Ok(Self {
            client,
            config,
            capabilities,
            status: ProviderStatus::Connected,
        })
    }

    fn build_headers(api_key: &str) -> Result<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            api_key.parse().map_err(|_| {
                ProviderError::AuthenticationFailed("Invalid API key format".into())
            })?,
        );
        headers.insert(
            "anthropic-version",
            CLAUDE_API_VERSION
                .parse()
                .map_err(|_| ProviderError::Unknown("Invalid version header".into()))?,
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json"
                .parse()
                .map_err(|_| ProviderError::Unknown("Invalid content type".into()))?,
        );
        Ok(headers)
    }

    fn build_capabilities() -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tool_use: true,
            vision: true,
            embeddings: false,
            max_context_tokens: 200000,
            models: vec![
                ModelInfo {
                    id: "claude-sonnet-4-5-20250929".into(),
                    name: "Claude Sonnet 4.5".into(),
                    provider: "claude".into(),
                    context_length: 200000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "claude-3-5-haiku-20241022".into(),
                    name: "Claude 3.5 Haiku".into(),
                    provider: "claude".into(),
                    context_length: 200000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
            ],
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<ClaudeMessage>) {
        let mut system_prompt = None;
        let mut claude_messages = Vec::with_capacity(messages.len());

        for message in messages {
            match message.role {
                Role::System => {
                    if let Some(text) = message.content.as_text() {
                        system_prompt = Some(text.to_string());
                    }
                }
                Role::User | Role::Assistant => {
                    claude_messages.push(ClaudeMessage {
                        role: match message.role {
                            Role::User => "user".into(),
                            Role::Assistant => "assistant".into(),
                            _ => continue,
                        },
                        content: self.convert_content(&message.content),
                    });
                }
                Role::Tool => {}
            }
        }

        (system_prompt, claude_messages)
    }

    fn convert_content(&self, content: &MessageContent) -> ClaudeContent {
        match content {
            MessageContent::Text(text) => ClaudeContent::Text(text.clone()),
            MessageContent::Parts(parts) => {
                let claude_parts: Vec<ClaudeContentPart> = parts
                    .iter()
                    .map(|part| match part {
                        crate::ContentPart::Text { text } => {
                            ClaudeContentPart::Text { text: text.clone() }
                        }
                        crate::ContentPart::ImageUrl { image_url } => ClaudeContentPart::Image {
                            source: ImageSource {
                                source_type: "url".into(),
                                url: Some(image_url.url.clone()),
                                media_type: None,
                                data: None,
                            },
                        },
                    })
                    .collect();
                ClaudeContent::Parts(claude_parts)
            }
        }
    }

    fn parse_finish_reason(stop_reason: &str) -> FinishReason {
        match stop_reason {
            "end_turn" | "stop_sequence" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            "tool_use" => FinishReason::ToolCalls,
            _ => FinishReason::Stop,
        }
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    fn provider_id(&self) -> &str {
        "claude"
    }

    fn display_name(&self) -> &str {
        "Anthropic Claude"
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn default_model(&self) -> &str {
        self.config
            .default_model
            .as_deref()
            .unwrap_or("claude-sonnet-4-5-20250929")
    }

    fn available_models(&self) -> &[ModelInfo] {
        &self.capabilities.models
    }

    fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    fn provider_metadata(&self) -> ProviderMetadata {
        claude_metadata()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| self.default_model());
        let (system, messages) = self.convert_messages(&request.messages);

        let claude_request = ClaudeRequest {
            model: model.into(),
            messages,
            system,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop,
            stream: false,
        };

        let response = self
            .client
            .post(CLAUDE_API_URL)
            .json(&claude_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(ProviderError::AuthenticationFailed(error_text)),
                429 => Err(ProviderError::RateLimited {
                    retry_after_secs: 60,
                }),
                _ => Err(ProviderError::RequestFailed(format!(
                    "{}: {}",
                    status, error_text
                ))),
            };
        }

        let claude_response: ClaudeResponse = response.json().await?;

        let content = claude_response
            .content
            .iter()
            .filter_map(|block| {
                if let ClaudeContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(ChatResponse {
            id: claude_response.id,
            provider: "claude".into(),
            model: claude_response.model,
            content,
            role: Role::Assistant,
            tool_calls: None,
            usage: Usage {
                prompt_tokens: claude_response.usage.input_tokens,
                completion_tokens: claude_response.usage.output_tokens,
                total_tokens: claude_response.usage.input_tokens
                    + claude_response.usage.output_tokens,
            },
            created_at: chrono::Utc::now(),
            finish_reason: claude_response
                .stop_reason
                .map(|r| Self::parse_finish_reason(&r)),
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream> {
        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| self.default_model());
        let (system, messages) = self.convert_messages(&request.messages);

        let claude_request = ClaudeRequest {
            model: model.into(),
            messages,
            system,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop,
            stream: true,
        };

        let response = self
            .client
            .post(CLAUDE_API_URL)
            .json(&claude_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(ProviderError::AuthenticationFailed(error_text)),
                429 => Err(ProviderError::RateLimited {
                    retry_after_secs: 60,
                }),
                _ => Err(ProviderError::RequestFailed(format!(
                    "{}: {}",
                    status, error_text
                ))),
            };
        }

        let model_clone = model.to_string();
        let stream = response.bytes_stream().map(move |chunk| {
            let chunk = chunk.map_err(|e| ProviderError::StreamingError(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);

            parse_sse_chunk(&text, &model_clone)
        });

        Ok(Box::pin(stream))
    }
}

fn parse_sse_chunk(text: &str, model: &str) -> Result<StreamChunk> {
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Ok(StreamChunk {
                    id: String::new(),
                    provider: "claude".into(),
                    model: model.into(),
                    delta: String::new(),
                    is_final: true,
                    usage: None,
                    finish_reason: Some(FinishReason::Stop),
                });
            }

            if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(data) {
                return match event {
                    ClaudeStreamEvent::ContentBlockDelta { delta, .. } => {
                        if let ClaudeDelta::TextDelta { text } = delta {
                            Ok(StreamChunk {
                                id: String::new(),
                                provider: "claude".into(),
                                model: model.into(),
                                delta: text,
                                is_final: false,
                                usage: None,
                                finish_reason: None,
                            })
                        } else {
                            Ok(StreamChunk {
                                id: String::new(),
                                provider: "claude".into(),
                                model: model.into(),
                                delta: String::new(),
                                is_final: false,
                                usage: None,
                                finish_reason: None,
                            })
                        }
                    }
                    ClaudeStreamEvent::MessageStop => Ok(StreamChunk {
                        id: String::new(),
                        provider: "claude".into(),
                        model: model.into(),
                        delta: String::new(),
                        is_final: true,
                        usage: None,
                        finish_reason: Some(FinishReason::Stop),
                    }),
                    _ => Ok(StreamChunk {
                        id: String::new(),
                        provider: "claude".into(),
                        model: model.into(),
                        delta: String::new(),
                        is_final: false,
                        usage: None,
                        finish_reason: None,
                    }),
                };
            }
        }
    }

    Ok(StreamChunk {
        id: String::new(),
        provider: "claude".into(),
        model: model.into(),
        delta: String::new(),
        is_final: false,
        usage: None,
        finish_reason: None,
    })
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: ClaudeContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ClaudeContent {
    Text(String),
    Parts(Vec<ClaudeContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ClaudeContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

#[derive(Debug, Serialize)]
struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    model: String,
    content: Vec<ClaudeContentBlock>,
    stop_reason: Option<String>,
    usage: ClaudeUsage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: serde_json::Value },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: serde_json::Value,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: u32, delta: ClaudeDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: serde_json::Value,
        usage: Option<ClaudeUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: serde_json::Value },
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_capabilities() {
        let caps = ClaudeProvider::build_capabilities();
        assert!(caps.streaming);
        assert!(caps.tool_use);
        assert!(caps.vision);
        assert_eq!(caps.max_context_tokens, 200000);
        assert!(!caps.models.is_empty());
    }

    #[test]
    fn test_parse_finish_reason() {
        assert_eq!(
            ClaudeProvider::parse_finish_reason("end_turn"),
            FinishReason::Stop
        );
        assert_eq!(
            ClaudeProvider::parse_finish_reason("max_tokens"),
            FinishReason::Length
        );
        assert_eq!(
            ClaudeProvider::parse_finish_reason("tool_use"),
            FinishReason::ToolCalls
        );
    }
}
