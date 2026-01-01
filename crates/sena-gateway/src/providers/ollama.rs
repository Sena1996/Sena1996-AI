use async_trait::async_trait;
use reqwest::Client;
use sena_core::{
    CompletionRequest, CompletionResponse, Error, HealthCheck, Message, MessageRole, Provider,
    Result, Usage,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_API_URL: &str = "http://localhost:11434";

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    models: Vec<String>,
    default_model: String,
    timeout: Duration,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_API_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self::with_base_url_and_timeout(base_url, Duration::from_secs(120))
    }

    pub fn with_base_url_and_timeout(base_url: impl Into<String>, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            base_url: base_url.into(),
            models: vec![
                "llama3.2".to_string(),
                "llama3.1".to_string(),
                "codellama".to_string(),
                "mistral".to_string(),
                "mixtral".to_string(),
                "phi3".to_string(),
                "gemma2".to_string(),
                "qwen2.5".to_string(),
            ],
            default_model: "llama3.2".to_string(),
            timeout,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn list_available_models(&self) -> Result<Vec<OllamaModelInfo>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("failed to list models: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::provider(format!(
                "ollama returned status {}",
                response.status()
            )));
        }

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| Error::provider(format!("failed to parse response: {}", e)))?;

        Ok(tags.models)
    }

    pub async fn pull_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "name": model, "stream": false }))
            .timeout(Duration::from_secs(3600))
            .send()
            .await
            .map_err(|e| Error::network(format!("failed to pull model: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::provider(format!(
                "failed to pull model {}: {}",
                model,
                response.status()
            )));
        }

        Ok(())
    }

    pub async fn delete_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/delete", self.base_url);

        let response = self
            .client
            .delete(&url)
            .json(&serde_json::json!({ "name": model }))
            .send()
            .await
            .map_err(|e| Error::network(format!("failed to delete model: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::provider(format!(
                "failed to delete model {}: {}",
                model,
                response.status()
            )));
        }

        Ok(())
    }

    pub async fn is_model_available(&self, model: &str) -> bool {
        self.list_available_models()
            .await
            .map(|models| models.iter().any(|m| m.name.starts_with(model)))
            .unwrap_or(false)
    }

    pub async fn generate_embedding(&self, model: &str, prompt: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "model": model,
                "prompt": prompt
            }))
            .send()
            .await
            .map_err(|e| Error::network(format!("embedding request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::provider(format!(
                "embedding failed with status {}",
                response.status()
            )));
        }

        let data: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| Error::provider(format!("failed to parse embedding: {}", e)))?;

        Ok(data.embedding)
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

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
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
            stream: false,
            options: Some(ApiOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                num_predict: request.max_tokens.map(|t| t as i32),
                stop: if request.stop.is_empty() {
                    None
                } else {
                    Some(request.stop.clone())
                },
            }),
        };

        let url = format!("{}/api/chat", self.base_url);
        let response = self
            .client
            .post(&url)
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

        let prompt_tokens = api_response.prompt_eval_count.unwrap_or(0);
        let completion_tokens = api_response.eval_count.unwrap_or(0);

        Ok(CompletionResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            content: api_response.message.content,
            model: api_response.model,
            usage: Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            finish_reason: if api_response.done {
                Some("stop".to_string())
            } else {
                None
            },
            metadata: Default::default(),
        })
    }

    async fn health(&self) -> HealthCheck {
        let start = std::time::Instant::now();

        let url = format!("{}/api/tags", self.base_url);
        let result = self.client.get(&url).send().await;

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
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ApiOptions>,
}

#[derive(Debug, Serialize)]
struct ApiOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    model: String,
    message: ApiMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub size: u64,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub modified_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}
