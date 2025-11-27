use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    provider::{AIProvider, ChatStream},
    ChatRequest, ChatResponse, Message, MessageContent, ModelInfo, ProviderCapabilities,
    ProviderConfig, ProviderError, ProviderStatus, Result, Role, StreamChunk, Usage,
    FinishReason,
};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GeminiProvider {
    client: Client,
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    status: ProviderStatus,
    api_key: String,
}

impl GeminiProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config.get_api_key()
            .ok_or_else(|| ProviderError::NotConfigured("GOOGLE_API_KEY not set".into()))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs.unwrap_or(120)))
            .build()
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let capabilities = Self::build_capabilities();

        Ok(Self {
            client,
            config,
            capabilities,
            status: ProviderStatus::Connected,
            api_key,
        })
    }

    fn build_capabilities() -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tool_use: true,
            vision: true,
            embeddings: true,
            max_context_tokens: 1000000,
            models: vec![
                ModelInfo {
                    id: "gemini-2.5-flash".into(),
                    name: "Gemini 2.5 Flash".into(),
                    provider: "gemini".into(),
                    context_length: 1048576,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "gemini-2.5-pro".into(),
                    name: "Gemini 2.5 Pro".into(),
                    provider: "gemini".into(),
                    context_length: 1048576,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "gemini-2.5-flash-lite".into(),
                    name: "Gemini 2.5 Flash-Lite".into(),
                    provider: "gemini".into(),
                    context_length: 1048576,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "gemini-2.0-flash".into(),
                    name: "Gemini 2.0 Flash".into(),
                    provider: "gemini".into(),
                    context_length: 1048576,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
            ],
        }
    }

    fn build_url(&self, model: &str, stream: bool) -> String {
        let action = if stream { "streamGenerateContent" } else { "generateContent" };
        format!(
            "{}/{}:{}?key={}",
            GEMINI_API_BASE, model, action, self.api_key
        )
    }

    fn convert_messages(&self, messages: &[Message]) -> (Option<GeminiSystemInstruction>, Vec<GeminiContent>) {
        let mut system_instruction = None;
        let mut contents = Vec::with_capacity(messages.len());

        for message in messages {
            match message.role {
                Role::System => {
                    if let Some(text) = message.content.as_text() {
                        system_instruction = Some(GeminiSystemInstruction {
                            parts: vec![GeminiPart::Text { text: text.to_string() }],
                        });
                    }
                }
                Role::User | Role::Assistant => {
                    contents.push(GeminiContent {
                        role: match message.role {
                            Role::User => "user".into(),
                            Role::Assistant => "model".into(),
                            _ => continue,
                        },
                        parts: self.convert_content(&message.content),
                    });
                }
                Role::Tool => {}
            }
        }

        (system_instruction, contents)
    }

    fn convert_content(&self, content: &MessageContent) -> Vec<GeminiPart> {
        match content {
            MessageContent::Text(text) => vec![GeminiPart::Text { text: text.clone() }],
            MessageContent::Parts(parts) => {
                parts
                    .iter()
                    .map(|part| match part {
                        crate::ContentPart::Text { text } => {
                            GeminiPart::Text { text: text.clone() }
                        }
                        crate::ContentPart::ImageUrl { image_url } => {
                            GeminiPart::InlineData {
                                inline_data: GeminiInlineData {
                                    mime_type: "image/jpeg".into(),
                                    data: image_url.url.clone(),
                                },
                            }
                        }
                    })
                    .collect()
            }
        }
    }

    fn parse_finish_reason(reason: &str) -> FinishReason {
        match reason {
            "STOP" => FinishReason::Stop,
            "MAX_TOKENS" => FinishReason::Length,
            "SAFETY" => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        }
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    fn provider_id(&self) -> &str {
        "gemini"
    }

    fn display_name(&self) -> &str {
        "Google Gemini"
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn default_model(&self) -> &str {
        self.config
            .default_model
            .as_deref()
            .unwrap_or("gemini-2.5-flash")
    }

    fn available_models(&self) -> &[ModelInfo] {
        &self.capabilities.models
    }

    fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request.model.as_deref().unwrap_or_else(|| self.default_model());
        let (system_instruction, contents) = self.convert_messages(&request.messages);

        let gemini_request = GeminiRequest {
            contents,
            system_instruction,
            generation_config: Some(GeminiGenerationConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
                top_p: request.top_p,
                stop_sequences: request.stop,
            }),
        };

        let url = self.build_url(model, false);
        let response = self
            .client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 | 403 => Err(ProviderError::AuthenticationFailed(error_text)),
                429 => Err(ProviderError::RateLimited { retry_after_secs: 60 }),
                _ => Err(ProviderError::RequestFailed(format!("{}: {}", status, error_text))),
            };
        }

        let gemini_response: GeminiResponse = response.json().await?;
        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| ProviderError::InvalidResponse("No candidates in response".into()))?;

        let content = candidate
            .content
            .parts
            .iter()
            .filter_map(|part| {
                if let GeminiPart::Text { text } = part {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        let usage = gemini_response.usage_metadata.map(|u| Usage {
            prompt_tokens: u.prompt_token_count,
            completion_tokens: u.candidates_token_count,
            total_tokens: u.total_token_count,
        }).unwrap_or_default();

        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "gemini".into(),
            model: model.into(),
            content,
            role: Role::Assistant,
            tool_calls: None,
            usage,
            created_at: chrono::Utc::now(),
            finish_reason: candidate
                .finish_reason
                .as_ref()
                .map(|r| Self::parse_finish_reason(r)),
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream> {
        let model = request.model.as_deref().unwrap_or_else(|| self.default_model());
        let (system_instruction, contents) = self.convert_messages(&request.messages);

        let gemini_request = GeminiRequest {
            contents,
            system_instruction,
            generation_config: Some(GeminiGenerationConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
                top_p: request.top_p,
                stop_sequences: request.stop,
            }),
        };

        let url = self.build_url(model, true);
        let response = self
            .client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 | 403 => Err(ProviderError::AuthenticationFailed(error_text)),
                429 => Err(ProviderError::RateLimited { retry_after_secs: 60 }),
                _ => Err(ProviderError::RequestFailed(format!("{}: {}", status, error_text))),
            };
        }

        let model_clone = model.to_string();
        let stream = response.bytes_stream().map(move |chunk| {
            let chunk = chunk.map_err(|e| ProviderError::StreamingError(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);

            parse_stream_chunk(&text, &model_clone)
        });

        Ok(Box::pin(stream))
    }
}

fn parse_stream_chunk(text: &str, model: &str) -> Result<StreamChunk> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "[" || trimmed == "]" || trimmed == "," {
        return Ok(StreamChunk {
            id: String::new(),
            provider: "gemini".into(),
            model: model.into(),
            delta: String::new(),
            is_final: false,
            usage: None,
            finish_reason: None,
        });
    }

    let json_text = trimmed.trim_start_matches('[').trim_start_matches(',');

    if let Ok(response) = serde_json::from_str::<GeminiStreamResponse>(json_text) {
        if let Some(candidate) = response.candidates.first() {
            let delta = candidate
                .content
                .parts
                .iter()
                .filter_map(|part| {
                    if let GeminiPart::Text { text } = part {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");

            let is_final = candidate.finish_reason.is_some();
            let finish_reason = candidate
                .finish_reason
                .as_ref()
                .map(|r| GeminiProvider::parse_finish_reason(r));

            return Ok(StreamChunk {
                id: String::new(),
                provider: "gemini".into(),
                model: model.into(),
                delta,
                is_final,
                usage: None,
                finish_reason,
            });
        }
    }

    Ok(StreamChunk {
        id: String::new(),
        provider: "gemini".into(),
        model: model.into(),
        delta: String::new(),
        is_final: false,
        usage: None,
        finish_reason: None,
    })
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    InlineData { inline_data: GeminiInlineData },
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: GeminiResponseContent,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
}

#[derive(Debug, Deserialize)]
struct GeminiStreamResponse {
    candidates: Vec<GeminiCandidate>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_capabilities() {
        let caps = GeminiProvider::build_capabilities();
        assert!(caps.streaming);
        assert!(caps.tool_use);
        assert!(caps.vision);
        assert!(caps.embeddings);
        assert_eq!(caps.max_context_tokens, 1000000);
        assert!(!caps.models.is_empty());
    }

    #[test]
    fn test_parse_finish_reason() {
        assert_eq!(GeminiProvider::parse_finish_reason("STOP"), FinishReason::Stop);
        assert_eq!(GeminiProvider::parse_finish_reason("MAX_TOKENS"), FinishReason::Length);
        assert_eq!(GeminiProvider::parse_finish_reason("SAFETY"), FinishReason::ContentFilter);
    }
}
