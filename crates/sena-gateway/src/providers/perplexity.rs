use async_trait::async_trait;
use reqwest::Client;
use sena_core::{
    CompletionRequest, CompletionResponse, Error, HealthCheck, Message, MessageRole,
    Provider, Result, Usage,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_URL: &str = "https://api.perplexity.ai/chat/completions";

pub struct PerplexityProvider {
    client: Client,
    api_key: String,
    models: Vec<String>,
    default_model: String,
}

impl PerplexityProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
            models: vec![
                "llama-3.1-sonar-small-128k-online".to_string(),
                "llama-3.1-sonar-large-128k-online".to_string(),
                "llama-3.1-sonar-huge-128k-online".to_string(),
                "llama-3.1-sonar-small-128k-chat".to_string(),
                "llama-3.1-sonar-large-128k-chat".to_string(),
            ],
            default_model: "llama-3.1-sonar-large-128k-online".to_string(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<ApiMessage> {
        messages
            .iter()
            .map(|m| ApiMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl Provider for PerplexityProvider {
    fn name(&self) -> &str {
        "perplexity"
    }

    fn models(&self) -> &[String] {
        &self.models
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let model = request.model.as_deref().unwrap_or(&self.default_model);

        let api_request = ApiRequest {
            model: model.to_string(),
            messages: self.convert_messages(&request.messages),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: false,
            return_citations: Some(true),
            return_images: Some(false),
        };

        let response = self
            .client
            .post(API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&api_request)
            .send()
            .await
            .map_err(|e| Error::network(format!("request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::provider(format!("API error {}: {}", status, error_text)));
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| Error::provider(format!("failed to parse response: {}", e)))?;

        let choice = api_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| Error::provider("no response choices"))?;

        let mut metadata = std::collections::HashMap::new();
        if let Some(citations) = api_response.citations {
            metadata.insert("citations".to_string(), serde_json::json!(citations));
        }

        Ok(CompletionResponse {
            id: api_response.id,
            content: choice.message.content,
            model: api_response.model,
            usage: Usage {
                prompt_tokens: api_response.usage.prompt_tokens,
                completion_tokens: api_response.usage.completion_tokens,
                total_tokens: api_response.usage.total_tokens,
            },
            finish_reason: choice.finish_reason,
            metadata,
        })
    }

    async fn health(&self) -> HealthCheck {
        let start = std::time::Instant::now();

        let result = self
            .client
            .post(API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "llama-3.1-sonar-small-128k-chat",
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
        128_000
    }

    fn priority(&self) -> u8 {
        70
    }
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_citations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_images: Option<bool>,
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
    choices: Vec<Choice>,
    usage: ApiUsage,
    #[serde(default)]
    citations: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ApiMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
