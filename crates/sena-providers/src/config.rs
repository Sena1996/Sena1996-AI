use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_id: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl ProviderConfig {
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            enabled: true,
            api_key: None,
            api_key_env: None,
            base_url: None,
            default_model: None,
            max_tokens: None,
            temperature: None,
            timeout_secs: None,
            extra: HashMap::new(),
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_api_key_env(mut self, env_var: impl Into<String>) -> Self {
        self.api_key_env = Some(env_var.into());
        self
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    pub fn get_api_key(&self) -> Option<String> {
        if let Some(key) = &self.api_key {
            return Some(key.clone());
        }

        if let Some(env_var) = &self.api_key_env {
            return env::var(env_var).ok();
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    #[serde(default)]
    pub cost_optimization: bool,
}

impl ProvidersConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_provider(&mut self, config: ProviderConfig) {
        self.providers.insert(config.provider_id.clone(), config);
    }

    pub fn get_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    pub fn enabled_providers(&self) -> Vec<&ProviderConfig> {
        self.providers
            .values()
            .filter(|p| p.enabled)
            .collect()
    }

    pub fn load_from_file(path: &PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn save_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }

    pub fn config_path() -> PathBuf {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sena")
            .join("providers.toml")
    }

    pub fn load_or_default() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match Self::load_from_file(&path) {
                Ok(config) => config,
                Err(e) => {
                    tracing::warn!(
                        "Failed to load config from {}: {}. Using defaults.",
                        path.display(),
                        e
                    );
                    Self::default_config()
                }
            }
        } else {
            Self::default_config()
        }
    }

    pub fn set_default_provider(&mut self, provider_id: &str) -> bool {
        if self.providers.contains_key(provider_id) {
            self.default_provider = Some(provider_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn default_config() -> Self {
        let mut config = Self::new();

        config.add_provider(
            ProviderConfig::new("claude")
                .with_api_key_env("ANTHROPIC_API_KEY")
                .with_default_model("claude-sonnet-4-5-20250929"),
        );

        config.add_provider(
            ProviderConfig::new("openai")
                .with_api_key_env("OPENAI_API_KEY")
                .with_default_model("gpt-4.1"),
        );

        config.add_provider(
            ProviderConfig::new("gemini")
                .with_api_key_env("GOOGLE_API_KEY")
                .with_default_model("gemini-2.5-flash"),
        );

        config.add_provider(
            ProviderConfig::new("ollama")
                .with_base_url("http://localhost:11434")
                .with_default_model("llama3.2"),
        );

        config.add_provider(
            ProviderConfig::new("mistral")
                .with_api_key_env("MISTRAL_API_KEY")
                .with_default_model("mistral-large-latest"),
        );

        config.default_provider = Some("claude".to_string());
        config.fallback_chain = vec![
            "openai".to_string(),
            "gemini".to_string(),
            "ollama".to_string(),
        ];

        config
    }
}
