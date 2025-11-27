use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    provider::{AIProvider, ChatStream},
    ChatRequest, ChatResponse, Message, MessageContent, ModelInfo, ProviderCapabilities,
    ProviderConfig, ProviderError, ProviderStatus, Result, Role, StreamChunk, Usage,
    FinishReason, ToolCall, ToolCallFunction,
};

const MISTRAL_API_URL: &str = "https://api.mistral.ai/v1/chat/completions";

pub struct MistralProvider {
    client: Client,
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    status: ProviderStatus,
}

impl MistralProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config.get_api_key()
            .ok_or_else(|| ProviderError::NotConfigured("MISTRAL_API_KEY not set".into()))?;

        let client = Client::builder()
            .default_headers(Self::build_headers(&api_key)?)
            .timeout(std::time::Duration::from_secs(config.timeout_secs.unwrap_or(120)))
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
            auth_value.parse().map_err(|_| ProviderError::AuthenticationFailed("Invalid API key format".into()))?,
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().map_err(|_| ProviderError::Unknown("Invalid content type".into()))?,
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
                    id: "mistral-large-latest".into(),
                    name: "Mistral Large".into(),
                    provider: "mistral".into(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "mistral-medium-latest".into(),
                    name: "Mistral Medium".into(),
                    provider: "mistral".into(),
                    context_length: 32000,
                    supports_vision: false,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "mistral-small-latest".into(),
                    name: "Mistral Small".into(),
                    provider: "mistral".into(),
                    context_length: 32000,
                    supports_vision: false,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "codestral-latest".into(),
                    name: "Codestral (Code)".into(),
                    provider: "mistral".into(),
                    context_length: 32000,
                    supports_vision: false,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "open-mistral-nemo".into(),
                    name: "Mistral Nemo (Open)".into(),
                    provider: "mistral".into(),
                    context_length: 128000,
                    supports_vision: false,
                    supports_tools: true,
                    supports_streaming: true,
                },
            ],
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<MistralMessage> {
        messages
            .iter()
            .map(|msg| MistralMessage {
                role: Self::convert_role(&msg.role),
                content: Self::convert_content(&msg.content),
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

    fn convert_content(content: &MessageContent) -> MistralContent {
        match content {
            MessageContent::Text(text) => MistralContent::Text(text.clone()),
            MessageContent::Parts(parts) => {
                let mistral_parts: Vec<MistralContentPart> = parts
                    .iter()
                    .map(|part| match part {
                        crate::ContentPart::Text { text } => {
                            MistralContentPart::Text { text: text.clone() }
                        }
                        crate::ContentPart::ImageUrl { image_url } => {
                            MistralContentPart::ImageUrl {
                                image_url: MistralImageUrl {
                                    url: image_url.url.clone(),
                                },
                            }
                        }
                    })
                    .collect();
                MistralContent::Parts(mistral_parts)
            }
        }
    }

    fn parse_finish_reason(reason: &str) -> FinishReason {
        match reason {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "tool_calls" | "function_call" => FinishReason::ToolCalls,
            "model_length" => FinishReason::Length,
            _ => FinishReason::Stop,
        }
    }
}

#[async_trait]
impl AIProvider for MistralProvider {
    fn provider_id(&self) -> &str {
        "mistral"
    }

    fn display_name(&self) -> &str {
        "Mistral AI"
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn default_model(&self) -> &str {
        self.config
            .default_model
            .as_deref()
            .unwrap_or("mistral-large-latest")
    }

    fn available_models(&self) -> &[ModelInfo] {
        &self.capabilities.models
    }

    fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request.model.as_deref().unwrap_or_else(|| self.default_model());
        let messages = self.convert_messages(&request.messages);

        let mistral_request = MistralRequest {
            model: model.into(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: false,
        };

        let response = self
            .client
            .post(MISTRAL_API_URL)
            .json(&mistral_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(ProviderError::AuthenticationFailed(error_text)),
                429 => Err(ProviderError::RateLimited { retry_after_secs: 60 }),
                _ => Err(ProviderError::RequestFailed(format!("{}: {}", status, error_text))),
            };
        }

        let mistral_response: MistralResponse = response.json().await?;
        let choice = mistral_response
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
            id: mistral_response.id,
            provider: "mistral".into(),
            model: mistral_response.model,
            content: choice.message.content.clone().unwrap_or_default(),
            role: Role::Assistant,
            tool_calls,
            usage: Usage {
                prompt_tokens: mistral_response.usage.prompt_tokens,
                completion_tokens: mistral_response.usage.completion_tokens,
                total_tokens: mistral_response.usage.total_tokens,
            },
            created_at: chrono::Utc::now(),
            finish_reason: choice
                .finish_reason
                .as_ref()
                .map(|r| Self::parse_finish_reason(r)),
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream> {
        let model = request.model.as_deref().unwrap_or_else(|| self.default_model());
        let messages = self.convert_messages(&request.messages);

        let mistral_request = MistralRequest {
            model: model.into(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: true,
        };

        let response = self
            .client
            .post(MISTRAL_API_URL)
            .json(&mistral_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(ProviderError::AuthenticationFailed(error_text)),
                429 => Err(ProviderError::RateLimited { retry_after_secs: 60 }),
                _ => Err(ProviderError::RequestFailed(format!("{}: {}", status, error_text))),
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
                    provider: "mistral".into(),
                    model: model.into(),
                    delta: String::new(),
                    is_final: true,
                    usage: None,
                    finish_reason: Some(FinishReason::Stop),
                });
            }

            if let Ok(event) = serde_json::from_str::<MistralStreamResponse>(data) {
                if let Some(choice) = event.choices.first() {
                    let delta = choice.delta.content.clone().unwrap_or_default();
                    let is_final = choice.finish_reason.is_some();
                    let finish_reason = choice
                        .finish_reason
                        .as_ref()
                        .map(|r| MistralProvider::parse_finish_reason(r));

                    return Ok(StreamChunk {
                        id: event.id,
                        provider: "mistral".into(),
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
        provider: "mistral".into(),
        model: model.into(),
        delta: String::new(),
        is_final: false,
        usage: None,
        finish_reason: None,
    })
}

#[derive(Debug, Serialize)]
struct MistralRequest {
    model: String,
    messages: Vec<MistralMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct MistralMessage {
    role: String,
    content: MistralContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum MistralContent {
    Text(String),
    Parts(Vec<MistralContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum MistralContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: MistralImageUrl },
}

#[derive(Debug, Serialize)]
struct MistralImageUrl {
    url: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralResponse {
    id: String,
    model: String,
    choices: Vec<MistralChoice>,
    usage: MistralUsage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralChoice {
    message: MistralResponseMessage,
    finish_reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<MistralToolCall>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: MistralFunctionCall,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralFunctionCall {
    name: String,
    arguments: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralStreamResponse {
    id: String,
    choices: Vec<MistralStreamChoice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralStreamChoice {
    delta: MistralStreamDelta,
    finish_reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MistralStreamDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_capabilities() {
        let caps = MistralProvider::build_capabilities();
        assert!(caps.streaming);
        assert!(caps.tool_use);
        assert!(!caps.models.is_empty());
    }

    #[test]
    fn test_parse_finish_reason() {
        assert_eq!(MistralProvider::parse_finish_reason("stop"), FinishReason::Stop);
        assert_eq!(MistralProvider::parse_finish_reason("length"), FinishReason::Length);
        assert_eq!(MistralProvider::parse_finish_reason("tool_calls"), FinishReason::ToolCalls);
        assert_eq!(MistralProvider::parse_finish_reason("model_length"), FinishReason::Length);
    }

    #[test]
    fn test_convert_role() {
        assert_eq!(MistralProvider::convert_role(&Role::System), "system");
        assert_eq!(MistralProvider::convert_role(&Role::User), "user");
        assert_eq!(MistralProvider::convert_role(&Role::Assistant), "assistant");
        assert_eq!(MistralProvider::convert_role(&Role::Tool), "tool");
    }
}
