use async_trait::async_trait;
use reqwest::Client;
use sena_core::{
    CompletionRequest, CompletionResponse, Error, HealthCheck, Message, MessageRole,
    Provider, Result, Usage,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_URL: &str = "https://api-inference.huggingface.co/models";

pub struct HuggingFaceProvider {
    client: Client,
    api_key: String,
    models: Vec<String>,
    default_model: String,
}

impl HuggingFaceProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
            models: vec![
                "meta-llama/Llama-3.2-3B-Instruct".to_string(),
                "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
                "HuggingFaceH4/zephyr-7b-beta".to_string(),
                "microsoft/Phi-3-mini-4k-instruct".to_string(),
            ],
            default_model: "meta-llama/Llama-3.2-3B-Instruct".to_string(),
        }
    }

    fn format_prompt(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    prompt.push_str(&format!("<|system|>\n{}\n", msg.content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("<|user|>\n{}\n", msg.content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!("<|assistant|>\n{}\n", msg.content));
                }
            }
        }

        prompt.push_str("<|assistant|>\n");
        prompt
    }
}

#[async_trait]
impl Provider for HuggingFaceProvider {
    fn name(&self) -> &str {
        "huggingface"
    }

    fn models(&self) -> &[String] {
        &self.models
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let model = request.model.as_deref().unwrap_or(&self.default_model);
        let prompt = self.format_prompt(&request.messages);

        let api_request = ApiRequest {
            inputs: prompt.clone(),
            parameters: Parameters {
                max_new_tokens: request.max_tokens.unwrap_or(1024),
                temperature: request.temperature.unwrap_or(0.7),
                top_p: request.top_p,
                return_full_text: false,
            },
        };

        let url = format!("{}/{}", API_URL, model);

        let response = self
            .client
            .post(&url)
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

        let api_response: Vec<ApiResponse> = response
            .json()
            .await
            .map_err(|e| Error::provider(format!("failed to parse response: {}", e)))?;

        let content = api_response
            .into_iter()
            .next()
            .map(|r| r.generated_text)
            .unwrap_or_default();

        let prompt_tokens = (prompt.len() / 4) as u32;
        let completion_tokens = (content.len() / 4) as u32;

        Ok(CompletionResponse {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            model: model.to_string(),
            usage: Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            finish_reason: Some("stop".to_string()),
            metadata: Default::default(),
        })
    }

    async fn health(&self) -> HealthCheck {
        let start = std::time::Instant::now();
        let url = format!("{}/{}", API_URL, self.default_model);

        let result = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "inputs": "Hi",
                "parameters": {"max_new_tokens": 1}
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
        false
    }

    fn max_tokens(&self) -> u32 {
        4096
    }

    fn priority(&self) -> u8 {
        50
    }
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    inputs: String,
    parameters: Parameters,
}

#[derive(Debug, Serialize)]
struct Parameters {
    max_new_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    return_full_text: bool,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    generated_text: String,
}
