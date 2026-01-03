use async_trait::async_trait;
use sena_core::Result;
use std::io::{self, Write};

use super::{AuthMethod, OAuthConfig};

#[async_trait]
pub trait ProviderAuth: Send + Sync {
    fn provider_name(&self) -> &str;
    fn preferred_auth_method(&self) -> AuthMethod;
    fn oauth_config(&self) -> Option<OAuthConfig>;
    fn api_key_url(&self) -> &str;
    fn api_key_env_var(&self) -> &str;

    async fn validate_api_key(&self, key: &str) -> Result<bool>;

    async fn prompt_for_api_key(&self) -> Result<String> {
        println!("\n🔑 API Key required for {}", self.provider_name());
        println!("Get your API key at: {}", self.api_key_url());
        println!("Or set environment variable: {}\n", self.api_key_env_var());

        print!("Enter API key: ");
        io::stdout().flush()?;

        let mut key = String::new();
        io::stdin().read_line(&mut key)?;

        Ok(key.trim().to_string())
    }
}

pub struct AnthropicAuth;

impl AnthropicAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for AnthropicAuth {
    fn provider_name(&self) -> &str {
        "Anthropic (Claude)"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://console.anthropic.com/settings/keys"
    }

    fn api_key_env_var(&self) -> &str {
        "ANTHROPIC_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .body(r#"{"model":"claude-3-haiku-20240307","max_tokens":1,"messages":[{"role":"user","content":"hi"}]}"#)
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success() || r.status().as_u16() == 400),
            Err(_) => Ok(false),
        }
    }
}

impl Default for AnthropicAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OpenAIAuth;

impl OpenAIAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for OpenAIAuth {
    fn provider_name(&self) -> &str {
        "OpenAI (GPT)"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://platform.openai.com/api-keys"
    }

    fn api_key_env_var(&self) -> &str {
        "OPENAI_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for OpenAIAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GeminiAuth {
    oauth_config: OAuthConfig,
}

impl GeminiAuth {
    pub fn new() -> Self {
        Self {
            oauth_config: OAuthConfig::new("gemini")
                .with_client_id(&std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap_or_default())
                .with_client_secret(&std::env::var("GOOGLE_OAUTH_CLIENT_SECRET").unwrap_or_default())
                .with_auth_url("https://accounts.google.com/o/oauth2/v2/auth")
                .with_token_url("https://oauth2.googleapis.com/token")
                .with_scopes(vec![
                    "openid",
                    "profile",
                    "email",
                    "https://www.googleapis.com/auth/generative-language.retriever",
                ]),
        }
    }
}

#[async_trait]
impl ProviderAuth for GeminiAuth {
    fn provider_name(&self) -> &str {
        "Google (Gemini)"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        Some(self.oauth_config.clone())
    }

    fn api_key_url(&self) -> &str {
        "https://aistudio.google.com/app/apikey"
    }

    fn api_key_env_var(&self) -> &str {
        "GEMINI_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://generativelanguage.googleapis.com/v1/models")
            .header("x-goog-api-key", key)
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for GeminiAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GroqAuth;

impl GroqAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for GroqAuth {
    fn provider_name(&self) -> &str {
        "Groq"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://console.groq.com/keys"
    }

    fn api_key_env_var(&self) -> &str {
        "GROQ_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.groq.com/openai/v1/models")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for GroqAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DeepSeekAuth;

impl DeepSeekAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for DeepSeekAuth {
    fn provider_name(&self) -> &str {
        "DeepSeek"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://platform.deepseek.com/api_keys"
    }

    fn api_key_env_var(&self) -> &str {
        "DEEPSEEK_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.deepseek.com/v1/models")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for DeepSeekAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MistralAuth;

impl MistralAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for MistralAuth {
    fn provider_name(&self) -> &str {
        "Mistral AI"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://console.mistral.ai/api-keys"
    }

    fn api_key_env_var(&self) -> &str {
        "MISTRAL_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.mistral.ai/v1/models")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for MistralAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CohereAuth;

impl CohereAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for CohereAuth {
    fn provider_name(&self) -> &str {
        "Cohere"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://dashboard.cohere.com/api-keys"
    }

    fn api_key_env_var(&self) -> &str {
        "COHERE_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.cohere.ai/v1/models")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for CohereAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct XaiAuth;

impl XaiAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for XaiAuth {
    fn provider_name(&self) -> &str {
        "xAI (Grok)"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://console.x.ai/api-keys"
    }

    fn api_key_env_var(&self) -> &str {
        "XAI_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.x.ai/v1/models")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for XaiAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TogetherAuth;

impl TogetherAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for TogetherAuth {
    fn provider_name(&self) -> &str {
        "Together AI"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://api.together.xyz/settings/api-keys"
    }

    fn api_key_env_var(&self) -> &str {
        "TOGETHER_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.together.xyz/v1/models")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for TogetherAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HuggingFaceAuth;

impl HuggingFaceAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for HuggingFaceAuth {
    fn provider_name(&self) -> &str {
        "Hugging Face"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://huggingface.co/settings/tokens"
    }

    fn api_key_env_var(&self) -> &str {
        "HUGGINGFACE_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .get("https://huggingface.co/api/whoami-v2")
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for HuggingFaceAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PerplexityAuth;

impl PerplexityAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for PerplexityAuth {
    fn provider_name(&self) -> &str {
        "Perplexity"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://www.perplexity.ai/settings/api"
    }

    fn api_key_env_var(&self) -> &str {
        "PERPLEXITY_API_KEY"
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool> {
        if key.is_empty() {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.perplexity.ai/chat/completions")
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .body(r#"{"model":"llama-3.1-sonar-small-128k-chat","messages":[{"role":"user","content":"hi"}],"max_tokens":1}"#)
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success() || r.status().as_u16() == 400),
            Err(_) => Ok(false),
        }
    }
}

impl Default for PerplexityAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OllamaAuth;

impl OllamaAuth {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderAuth for OllamaAuth {
    fn provider_name(&self) -> &str {
        "Ollama (Local)"
    }

    fn preferred_auth_method(&self) -> AuthMethod {
        AuthMethod::None
    }

    fn oauth_config(&self) -> Option<OAuthConfig> {
        None
    }

    fn api_key_url(&self) -> &str {
        "https://ollama.ai/download"
    }

    fn api_key_env_var(&self) -> &str {
        "OLLAMA_HOST"
    }

    async fn validate_api_key(&self, _key: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        let host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let response = client
            .get(format!("{}/api/tags", host))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Default for OllamaAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HuggingFaceAuthOAuth {
    oauth_config: OAuthConfig,
}

impl HuggingFaceAuthOAuth {
    pub fn new() -> Self {
        Self {
            oauth_config: OAuthConfig::new("huggingface")
                .with_client_id(&std::env::var("HUGGINGFACE_OAUTH_CLIENT_ID").unwrap_or_default())
                .with_client_secret(&std::env::var("HUGGINGFACE_OAUTH_CLIENT_SECRET").unwrap_or_default())
                .with_auth_url("https://huggingface.co/oauth/authorize")
                .with_token_url("https://huggingface.co/oauth/token")
                .with_scopes(vec![
                    "openid",
                    "profile",
                    "inference-api",
                ]),
        }
    }

    pub fn oauth_config(&self) -> &OAuthConfig {
        &self.oauth_config
    }
}

pub struct ClaudeAuthOAuth {
    oauth_config: OAuthConfig,
}

impl ClaudeAuthOAuth {
    pub fn new() -> Self {
        Self {
            oauth_config: OAuthConfig::new("claude")
                .with_client_id("9d1c250a-e61b-44d9-88ed-5944d1962f5e")
                .with_auth_url("https://console.anthropic.com/oauth/authorize")
                .with_token_url("https://console.anthropic.com/oauth/token")
                .with_scopes(vec!["openid", "profile", "email"]),
        }
    }

    pub fn oauth_config(&self) -> &OAuthConfig {
        &self.oauth_config
    }
}
