use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    metadata::{ollama_metadata, ProviderMetadata},
    provider::{AIProvider, ChatStream},
    ChatRequest, ChatResponse, FinishReason, Message, MessageContent, ModelInfo,
    ProviderCapabilities, ProviderConfig, ProviderError, ProviderStatus, Result, Role, StreamChunk,
    Usage,
};

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

pub struct OllamaProvider {
    client: Client,
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    status: ProviderStatus,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(
                config.timeout_secs.unwrap_or(300),
            ))
            .build()
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_OLLAMA_URL.into());

        let capabilities = Self::build_capabilities();

        Ok(Self {
            client,
            config,
            capabilities,
            status: ProviderStatus::Connected,
            base_url,
        })
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
                    id: "llama3.2".into(),
                    name: "Llama 3.2".into(),
                    provider: "ollama".into(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "llama3.2:70b".into(),
                    name: "Llama 3.2 70B".into(),
                    provider: "ollama".into(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "mistral".into(),
                    name: "Mistral".into(),
                    provider: "ollama".into(),
                    context_length: 32000,
                    supports_vision: false,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "codellama".into(),
                    name: "Code Llama".into(),
                    provider: "ollama".into(),
                    context_length: 16000,
                    supports_vision: false,
                    supports_tools: false,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "deepseek-r1".into(),
                    name: "DeepSeek R1".into(),
                    provider: "ollama".into(),
                    context_length: 64000,
                    supports_vision: false,
                    supports_tools: true,
                    supports_streaming: true,
                },
                ModelInfo {
                    id: "qwen2.5".into(),
                    name: "Qwen 2.5".into(),
                    provider: "ollama".into(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_tools: true,
                    supports_streaming: true,
                },
            ],
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/api/chat", self.base_url)
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|msg| OllamaMessage {
                role: Self::convert_role(&msg.role),
                content: Self::extract_text(&msg.content),
                images: Self::extract_images(&msg.content),
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

    fn extract_text(content: &MessageContent) -> String {
        match content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Parts(parts) => parts
                .iter()
                .filter_map(|part| {
                    if let crate::ContentPart::Text { text } = part {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    fn extract_images(content: &MessageContent) -> Option<Vec<String>> {
        match content {
            MessageContent::Text(_) => None,
            MessageContent::Parts(parts) => {
                let images: Vec<String> = parts
                    .iter()
                    .filter_map(|part| {
                        if let crate::ContentPart::ImageUrl { image_url } = part {
                            Some(image_url.url.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                if images.is_empty() {
                    None
                } else {
                    Some(images)
                }
            }
        }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    fn provider_id(&self) -> &str {
        "ollama"
    }

    fn display_name(&self) -> &str {
        "Ollama (Local)"
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn default_model(&self) -> &str {
        self.config.default_model.as_deref().unwrap_or("llama3.2")
    }

    fn available_models(&self) -> &[ModelInfo] {
        &self.capabilities.models
    }

    fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    fn provider_metadata(&self) -> ProviderMetadata {
        ollama_metadata()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| self.default_model());
        let messages = self.convert_messages(&request.messages);

        let ollama_request = OllamaRequest {
            model: model.into(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                num_predict: request.max_tokens.map(|t| t as i32),
                stop: request.stop,
            }),
        };

        let response = self
            .client
            .post(self.chat_url())
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    ProviderError::Unavailable(format!("Ollama not running at {}", self.base_url))
                } else {
                    ProviderError::NetworkError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return Err(ProviderError::RequestFailed(format!(
                "{}: {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "ollama".into(),
            model: ollama_response.model,
            content: ollama_response.message.content,
            role: Role::Assistant,
            tool_calls: None,
            usage: Usage {
                prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
                completion_tokens: ollama_response.eval_count.unwrap_or(0),
                total_tokens: ollama_response.prompt_eval_count.unwrap_or(0)
                    + ollama_response.eval_count.unwrap_or(0),
            },
            created_at: chrono::Utc::now(),
            finish_reason: if ollama_response.done {
                Some(FinishReason::Stop)
            } else {
                None
            },
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream> {
        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| self.default_model());
        let messages = self.convert_messages(&request.messages);

        let ollama_request = OllamaRequest {
            model: model.into(),
            messages,
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                num_predict: request.max_tokens.map(|t| t as i32),
                stop: request.stop,
            }),
        };

        let response = self
            .client
            .post(self.chat_url())
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    ProviderError::Unavailable(format!("Ollama not running at {}", self.base_url))
                } else {
                    ProviderError::NetworkError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return Err(ProviderError::RequestFailed(format!(
                "{}: {}",
                status, error_text
            )));
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
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(response) = serde_json::from_str::<OllamaStreamResponse>(trimmed) {
            return Ok(StreamChunk {
                id: String::new(),
                provider: "ollama".into(),
                model: model.into(),
                delta: response.message.content,
                is_final: response.done,
                usage: if response.done {
                    Some(Usage {
                        prompt_tokens: response.prompt_eval_count.unwrap_or(0),
                        completion_tokens: response.eval_count.unwrap_or(0),
                        total_tokens: response.prompt_eval_count.unwrap_or(0)
                            + response.eval_count.unwrap_or(0),
                    })
                } else {
                    None
                },
                finish_reason: if response.done {
                    Some(FinishReason::Stop)
                } else {
                    None
                },
            });
        }
    }

    Ok(StreamChunk {
        id: String::new(),
        provider: "ollama".into(),
        model: model.into(),
        delta: String::new(),
        is_final: false,
        usage: None,
        finish_reason: None,
    })
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    message: OllamaResponseMessage,
    done: bool,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    role: String,
    content: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OllamaStreamResponse {
    model: String,
    message: OllamaResponseMessage,
    done: bool,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_capabilities() {
        let caps = OllamaProvider::build_capabilities();
        assert!(caps.streaming);
        assert!(caps.tool_use);
        assert!(caps.vision);
        assert!(caps.embeddings);
        assert!(!caps.models.is_empty());
    }

    #[test]
    fn test_convert_role() {
        assert_eq!(OllamaProvider::convert_role(&Role::System), "system");
        assert_eq!(OllamaProvider::convert_role(&Role::User), "user");
        assert_eq!(OllamaProvider::convert_role(&Role::Assistant), "assistant");
        assert_eq!(OllamaProvider::convert_role(&Role::Tool), "tool");
    }

    #[test]
    fn test_extract_text() {
        let content = MessageContent::Text("Hello".into());
        assert_eq!(OllamaProvider::extract_text(&content), "Hello");
    }
}
