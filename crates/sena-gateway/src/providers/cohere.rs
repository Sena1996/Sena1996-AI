use async_trait::async_trait;
use reqwest::Client;
use sena_core::{
    CompletionRequest, CompletionResponse, Error, HealthCheck, Message, MessageRole,
    Provider, Result, Usage,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_URL: &str = "https://api.cohere.com/v2/chat";

pub struct CohereProvider {
    client: Client,
    api_key: String,
    models: Vec<String>,
    default_model: String,
}

impl CohereProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
            models: vec![
                "command-r-plus".to_string(),
                "command-r".to_string(),
                "command".to_string(),
                "command-light".to_string(),
            ],
            default_model: "command-r-plus".to_string(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<ApiMessage>) {
        let mut system = None;
        let mut api_messages = Vec::new();

        for m in messages {
            match m.role {
                MessageRole::System => {
                    system = Some(m.content.clone());
                }
                MessageRole::User => {
                    api_messages.push(ApiMessage {
                        role: "user".to_string(),
                        content: m.content.clone(),
                    });
                }
                MessageRole::Assistant => {
                    api_messages.push(ApiMessage {
                        role: "assistant".to_string(),
                        content: m.content.clone(),
                    });
                }
            }
        }

        (system, api_messages)
    }
}

#[async_trait]
impl Provider for CohereProvider {
    fn name(&self) -> &str {
        "cohere"
    }

    fn models(&self) -> &[String] {
        &self.models
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let model = request.model.as_deref().unwrap_or(&self.default_model);
        let (system, messages) = self.convert_messages(&request.messages);

        let api_request = ApiRequest {
            model: model.to_string(),
            messages,
            system,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            p: request.top_p,
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

        let content = api_response
            .message
            .content
            .into_iter()
            .filter_map(|c| if c.content_type == "text" { Some(c.text) } else { None })
            .collect::<Vec<_>>()
            .join("");

        Ok(CompletionResponse {
            id: api_response.id,
            content,
            model: model.to_string(),
            usage: Usage {
                prompt_tokens: api_response.usage.billed_units.input_tokens,
                completion_tokens: api_response.usage.billed_units.output_tokens,
                total_tokens: api_response.usage.billed_units.input_tokens
                    + api_response.usage.billed_units.output_tokens,
            },
            finish_reason: Some(api_response.finish_reason),
            metadata: Default::default(),
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
                "model": "command-light",
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
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    p: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    id: String,
    message: ResponseMessage,
    finish_reason: String,
    usage: ApiUsage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Vec<ContentBlock>,
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
    billed_units: BilledUnits,
}

#[derive(Debug, Deserialize)]
struct BilledUnits {
    input_tokens: u32,
    output_tokens: u32,
}
