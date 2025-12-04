use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    metadata::{openai_metadata, ProviderMetadata},
    provider::{AIProvider, ChatStream},
    ChatRequest, ChatResponse, FinishReason, Message, MessageContent, ModelInfo,
    ProviderCapabilities, ProviderConfig, ProviderError, ProviderStatus, Result, Role, StreamChunk,
    ToolCall, ToolCallFunction, Usage,
};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAIProvider {
    client: Client,
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    status: ProviderStatus,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config
            .get_api_key()
            .ok_or_else(|| ProviderError::NotConfigured("OPENAI_API_KEY not set".into()))?;

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
        let auth_value = format!("Bearer {}", api_key);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            auth_value.parse().map_err(|_| {
                ProviderError::AuthenticationFailed("Invalid API key format".into())
            })?,
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
            embeddings: true,
            max_context_tokens: 128000,
            models: vec![
                ModelInfo {
                    id: "gpt-4.1".into(),
                    name: "GPT-4.1".into(),
                    provider: "openai".into(),
                    context_length: 1047576,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "gpt-4.1-mini".into(),
                    name: "GPT-4.1 Mini".into(),
                    provider: "openai".into(),
                    context_length: 1047576,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "gpt-4.1-nano".into(),
                    name: "GPT-4.1 Nano".into(),
                    provider: "openai".into(),
                    context_length: 1047576,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "o4-mini".into(),
                    name: "o4-mini (Reasoning)".into(),
                    provider: "openai".into(),
                    context_length: 200000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "gpt-4o".into(),
                    name: "GPT-4o".into(),
                    provider: "openai".into(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
            ],
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: Self::convert_role(&msg.role),
                content: Self::convert_content(&msg.content),
                name: msg.name.clone(),
            })
            .collect()
    }

    fn convert_role(role: &Role) -> String {
        match role {
            Role::System => "system".into(),
            Role::User => "user".into(),
            Role::Assistant => "assistant".into(),
            Role::Tool => "tool".into(),
        }
    }

    fn convert_content(content: &MessageContent) -> OpenAIContent {
        match content {
            MessageContent::Text(text) => OpenAIContent::Text(text.clone()),
            MessageContent::Parts(parts) => {
                let openai_parts: Vec<OpenAIContentPart> = parts
                    .iter()
                    .map(|part| match part {
                        crate::ContentPart::Text { text } => {
                            OpenAIContentPart::Text { text: text.clone() }
                        }
                        crate::ContentPart::ImageUrl { image_url } => OpenAIContentPart::ImageUrl {
                            image_url: OpenAIImageUrl {
                                url: image_url.url.clone(),
                                detail: image_url.detail.clone(),
                            },
                        },
                    })
                    .collect();
                OpenAIContent::Parts(openai_parts)
            }
        }
    }

    fn parse_finish_reason(reason: &str) -> FinishReason {
        match reason {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "tool_calls" | "function_call" => FinishReason::ToolCalls,
            "content_filter" => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn provider_id(&self) -> &str {
        "openai"
    }

    fn display_name(&self) -> &str {
        "OpenAI"
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn default_model(&self) -> &str {
        self.config.default_model.as_deref().unwrap_or("gpt-4.1")
    }

    fn available_models(&self) -> &[ModelInfo] {
        &self.capabilities.models
    }

    fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    fn provider_metadata(&self) -> ProviderMetadata {
        openai_metadata()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| self.default_model());
        let messages = self.convert_messages(&request.messages);

        let openai_request = OpenAIRequest {
            model: model.into(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stop: request.stop,
            stream: false,
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .json(&openai_request)
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

        let openai_response: OpenAIResponse = response.json().await?;
        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| ProviderError::InvalidResponse("No choices in response".into()))?;

        let tool_calls = choice.message.tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(|tc| ToolCall {
                    id: tc.id.clone(),
                    call_type: tc.call_type.clone(),
                    function: ToolCallFunction {
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    },
                })
                .collect()
        });

        Ok(ChatResponse {
            id: openai_response.id,
            provider: "openai".into(),
            model: openai_response.model,
            content: choice.message.content.clone().unwrap_or_default(),
            role: Role::Assistant,
            tool_calls,
            usage: Usage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            created_at: chrono::Utc::now(),
            finish_reason: choice
                .finish_reason
                .as_ref()
                .map(|r| Self::parse_finish_reason(r)),
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream> {
        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| self.default_model());
        let messages = self.convert_messages(&request.messages);

        let openai_request = OpenAIRequest {
            model: model.into(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stop: request.stop,
            stream: true,
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .json(&openai_request)
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
                    provider: "openai".into(),
                    model: model.into(),
                    delta: String::new(),
                    is_final: true,
                    usage: None,
                    finish_reason: Some(FinishReason::Stop),
                });
            }

            if let Ok(event) = serde_json::from_str::<OpenAIStreamResponse>(data) {
                if let Some(choice) = event.choices.first() {
                    let delta = choice.delta.content.clone().unwrap_or_default();
                    let is_final = choice.finish_reason.is_some();
                    let finish_reason = choice
                        .finish_reason
                        .as_ref()
                        .map(|r| OpenAIProvider::parse_finish_reason(r));

                    return Ok(StreamChunk {
                        id: event.id,
                        provider: "openai".into(),
                        model: model.into(),
                        delta,
                        is_final,
                        usage: None,
                        finish_reason,
                    });
                }
            }
        }
    }

    Ok(StreamChunk {
        id: String::new(),
        provider: "openai".into(),
        model: model.into(),
        delta: String::new(),
        is_final: false,
        usage: None,
        finish_reason: None,
    })
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: OpenAIContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum OpenAIContent {
    Text(String),
    Parts(Vec<OpenAIContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAIImageUrl },
}

#[derive(Debug, Serialize)]
struct OpenAIImageUrl {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    id: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_capabilities() {
        let caps = OpenAIProvider::build_capabilities();
        assert!(caps.streaming);
        assert!(caps.tool_use);
        assert!(caps.vision);
        assert!(caps.embeddings);
        assert!(!caps.models.is_empty());
    }

    #[test]
    fn test_parse_finish_reason() {
        assert_eq!(
            OpenAIProvider::parse_finish_reason("stop"),
            FinishReason::Stop
        );
        assert_eq!(
            OpenAIProvider::parse_finish_reason("length"),
            FinishReason::Length
        );
        assert_eq!(
            OpenAIProvider::parse_finish_reason("tool_calls"),
            FinishReason::ToolCalls
        );
        assert_eq!(
            OpenAIProvider::parse_finish_reason("content_filter"),
            FinishReason::ContentFilter
        );
    }

    #[test]
    fn test_convert_role() {
        assert_eq!(OpenAIProvider::convert_role(&Role::System), "system");
        assert_eq!(OpenAIProvider::convert_role(&Role::User), "user");
        assert_eq!(OpenAIProvider::convert_role(&Role::Assistant), "assistant");
        assert_eq!(OpenAIProvider::convert_role(&Role::Tool), "tool");
    }
}
