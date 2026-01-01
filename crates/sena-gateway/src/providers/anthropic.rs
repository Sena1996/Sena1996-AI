use async_trait::async_trait;
use reqwest::Client;
use sena_core::{
    CompletionRequest, CompletionResponse, CompletionStream, Error, HealthCheck, Message,
    MessageRole, Provider, Result, Usage,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::streaming::create_anthropic_stream;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    models: Vec<String>,
    default_model: String,
    timeout: Duration,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_timeout(api_key, Duration::from_secs(60))
    }

    pub fn with_timeout(api_key: impl Into<String>, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
            models: vec![
                "claude-sonnet-4-20250514".to_string(),
                "claude-opus-4-20250514".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
            ],
            default_model: "claude-sonnet-4-20250514".to_string(),
            timeout,
        }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<ApiMessage> {
        messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| ApiMessage {
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "user".to_string(),
                },
                content: m.content.clone(),
            })
            .collect()
    }

    fn extract_system(&self, messages: &[Message]) -> Option<String> {
        messages
            .iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| m.content.clone())
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn models(&self) -> &[String] {
        &self.models
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let model = request.model.as_deref().unwrap_or(&self.default_model);
        let max_tokens = request.max_tokens.unwrap_or(4096);

        let api_request = ApiRequest {
            model: model.to_string(),
            max_tokens,
            messages: self.convert_messages(&request.messages),
            system: self.extract_system(&request.messages),
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: if request.stop.is_empty() {
                None
            } else {
                Some(request.stop.clone())
            },
            stream: false,
        };

        let response = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&api_request)
            .send()
            .await
            .map_err(|e| Error::network(format!("request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::provider(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| Error::provider(format!("failed to parse response: {}", e)))?;

        let content = api_response
            .content
            .into_iter()
            .filter_map(|c| if c.content_type == "text" { Some(c.text) } else { None })
            .collect::<Vec<_>>()
            .join("");

        Ok(CompletionResponse {
            id: api_response.id,
            content,
            model: api_response.model,
            usage: Usage {
                prompt_tokens: api_response.usage.input_tokens,
                completion_tokens: api_response.usage.output_tokens,
                total_tokens: api_response.usage.input_tokens + api_response.usage.output_tokens,
            },
            finish_reason: api_response.stop_reason,
            metadata: Default::default(),
        })
    }

    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let model = request.model.as_deref().unwrap_or(&self.default_model);
        let max_tokens = request.max_tokens.unwrap_or(4096);

        let api_request = ApiRequest {
            model: model.to_string(),
            max_tokens,
            messages: self.convert_messages(&request.messages),
            system: self.extract_system(&request.messages),
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: if request.stop.is_empty() {
                None
            } else {
                Some(request.stop.clone())
            },
            stream: true,
        };

        let response = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&api_request)
            .send()
            .await
            .map_err(|e| Error::network(format!("request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::provider(format!("API error {}: {}", status, error_text)));
        }

        let byte_stream = response.bytes_stream();
        Ok(create_anthropic_stream(byte_stream))
    }

    async fn health(&self) -> HealthCheck {
        let start = std::time::Instant::now();

        let result = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-3-5-haiku-20241022",
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "Hi"}]
            }))
            .send()
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) if response.status().is_success() => {
                HealthCheck::healthy().with_latency(latency_ms)
            }
            Ok(response) => HealthCheck::degraded(format!("status: {}", response.status())),
            Err(e) => HealthCheck::unhealthy(format!("error: {}", e)),
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn max_tokens(&self) -> u32 {
        200_000
    }

    fn priority(&self) -> u8 {
        100
    }
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: ApiUsage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    input_tokens: u32,
    output_tokens: u32,
}
