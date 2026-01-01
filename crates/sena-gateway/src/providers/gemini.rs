use async_trait::async_trait;
use reqwest::Client;
use sena_core::{
    CompletionRequest, CompletionResponse, Error, HealthCheck, Message, MessageRole, Provider,
    Result, Usage,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    models: Vec<String>,
    default_model: String,
    timeout: Duration,
}

impl GeminiProvider {
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
                "gemini-2.0-flash-exp".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
                "gemini-1.5-flash-8b".to_string(),
            ],
            default_model: "gemini-2.0-flash-exp".to_string(),
            timeout,
        }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    fn convert_messages(&self, messages: &[Message]) -> (Option<SystemInstruction>, Vec<Content>) {
        let system = messages
            .iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| SystemInstruction {
                parts: vec![Part {
                    text: m.content.clone(),
                }],
            });

        let contents: Vec<Content> = messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| Content {
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "model".to_string(),
                    MessageRole::System => "user".to_string(),
                },
                parts: vec![Part {
                    text: m.content.clone(),
                }],
            })
            .collect();

        (system, contents)
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn models(&self) -> &[String] {
        &self.models
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let model = request.model.as_deref().unwrap_or(&self.default_model);
        let (system_instruction, contents) = self.convert_messages(&request.messages);

        let generation_config = GenerationConfig {
            max_output_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: if request.stop.is_empty() {
                None
            } else {
                Some(request.stop.clone())
            },
        };

        let api_request = ApiRequest {
            contents,
            system_instruction,
            generation_config: Some(generation_config),
        };

        let url = format!(
            "{}/{}:generateContent?key={}",
            API_URL, model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
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

        let candidate = api_response
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| Error::provider("no candidates in response"))?;

        let content = candidate
            .content
            .parts
            .into_iter()
            .map(|p| p.text)
            .collect::<Vec<_>>()
            .join("");

        let usage = api_response.usage_metadata.unwrap_or_default();

        Ok(CompletionResponse {
            id: format!("gemini-{}", uuid::Uuid::new_v4()),
            content,
            model: model.to_string(),
            usage: Usage {
                prompt_tokens: usage.prompt_token_count,
                completion_tokens: usage.candidates_token_count,
                total_tokens: usage.total_token_count,
            },
            finish_reason: candidate.finish_reason,
            metadata: Default::default(),
        })
    }

    async fn health(&self) -> HealthCheck {
        let start = std::time::Instant::now();

        let url = format!(
            "{}/{}:generateContent?key={}",
            API_URL, "gemini-1.5-flash-8b", self.api_key
        );

        let result = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "contents": [{"role": "user", "parts": [{"text": "Hi"}]}],
                "generationConfig": {"maxOutputTokens": 1}
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
        2_000_000
    }

    fn priority(&self) -> u8 {
        85
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
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
struct ApiResponse {
    candidates: Vec<Candidate>,
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    content: Content,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UsageMetadata {
    #[serde(default)]
    prompt_token_count: u32,
    #[serde(default)]
    candidates_token_count: u32,
    #[serde(default)]
    total_token_count: u32,
}
