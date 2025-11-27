use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    config::ProvidersConfig,
    provider::{AIProvider, ChatStream},
    ChatRequest, ChatResponse, ModelInfo, ProviderError, ProviderStatus, Result,
};

#[cfg(feature = "claude")]
use crate::claude::ClaudeProvider;

#[cfg(feature = "openai")]
use crate::openai::OpenAIProvider;

#[cfg(feature = "gemini")]
use crate::gemini::GeminiProvider;

#[cfg(feature = "ollama")]
use crate::ollama::OllamaProvider;

#[cfg(feature = "mistral")]
use crate::mistral::MistralProvider;

pub struct ProviderRouter {
    providers: HashMap<String, Arc<dyn AIProvider>>,
    default_provider: Option<String>,
    fallback_chain: Vec<String>,
}

impl ProviderRouter {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
            fallback_chain: Vec::new(),
        }
    }

    pub fn from_config(config: &ProvidersConfig) -> Result<Self> {
        let mut router = Self::new();

        for (provider_id, provider_config) in &config.providers {
            if !provider_config.enabled {
                continue;
            }

            let provider_result: Result<Arc<dyn AIProvider>> = match provider_id.as_str() {
                #[cfg(feature = "claude")]
                "claude" => ClaudeProvider::new(provider_config.clone())
                    .map(|p| Arc::new(p) as Arc<dyn AIProvider>),

                #[cfg(feature = "openai")]
                "openai" => OpenAIProvider::new(provider_config.clone())
                    .map(|p| Arc::new(p) as Arc<dyn AIProvider>),

                #[cfg(feature = "gemini")]
                "gemini" => GeminiProvider::new(provider_config.clone())
                    .map(|p| Arc::new(p) as Arc<dyn AIProvider>),

                #[cfg(feature = "ollama")]
                "ollama" => OllamaProvider::new(provider_config.clone())
                    .map(|p| Arc::new(p) as Arc<dyn AIProvider>),

                #[cfg(feature = "mistral")]
                "mistral" => MistralProvider::new(provider_config.clone())
                    .map(|p| Arc::new(p) as Arc<dyn AIProvider>),

                _ => continue,
            };

            if let Ok(provider) = provider_result {
                router.register_provider(provider);
            }
        }

        if let Some(default) = &config.default_provider {
            router.default_provider = Some(default.clone());
        }

        router.fallback_chain = config.fallback_chain.clone();

        Ok(router)
    }

    pub fn register_provider(&mut self, provider: Arc<dyn AIProvider>) {
        let id = provider.provider_id().to_string();
        self.providers.insert(id, provider);
    }

    pub fn set_default_provider(&mut self, provider_id: &str) -> Result<()> {
        if !self.providers.contains_key(provider_id) {
            return Err(ProviderError::NotConfigured(provider_id.into()));
        }
        self.default_provider = Some(provider_id.to_string());
        Ok(())
    }

    pub fn set_fallback_chain(&mut self, chain: Vec<String>) {
        self.fallback_chain = chain;
    }

    pub fn get_provider(&self, provider_id: &str) -> Option<&Arc<dyn AIProvider>> {
        self.providers.get(provider_id)
    }

    pub fn default_provider(&self) -> Option<&Arc<dyn AIProvider>> {
        self.default_provider
            .as_ref()
            .and_then(|id| self.providers.get(id))
    }

    pub fn available_providers(&self) -> Vec<&Arc<dyn AIProvider>> {
        self.providers.values().collect()
    }

    pub fn provider_status(&self) -> HashMap<String, ProviderStatus> {
        self.providers
            .iter()
            .map(|(id, provider)| (id.clone(), provider.status()))
            .collect()
    }

    pub fn all_models(&self) -> Vec<ModelInfo> {
        self.providers
            .values()
            .flat_map(|provider| provider.available_models().to_vec())
            .collect()
    }

    pub fn find_model(&self, model_id: &str) -> Option<(&Arc<dyn AIProvider>, &ModelInfo)> {
        for provider in self.providers.values() {
            if let Some(model) = provider
                .available_models()
                .iter()
                .find(|m| m.id == model_id)
            {
                return Some((provider, model));
            }
        }
        None
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let provider = self.resolve_provider(&request)?;
        provider.chat(request).await
    }

    pub async fn chat_with_fallback(&self, request: ChatRequest) -> Result<ChatResponse> {
        let primary_provider = self.resolve_provider(&request);

        if let Ok(provider) = primary_provider {
            match provider.chat(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if !self.should_fallback(&e) {
                        return Err(e);
                    }
                }
            }
        }

        for provider_id in &self.fallback_chain {
            if let Some(provider) = self.providers.get(provider_id) {
                if provider.status() != ProviderStatus::Connected {
                    continue;
                }

                match provider.chat(request.clone()).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        if !self.should_fallback(&e) {
                            return Err(e);
                        }
                    }
                }
            }
        }

        Err(ProviderError::Unavailable(
            "All providers failed or unavailable".into(),
        ))
    }

    pub async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream> {
        let provider = self.resolve_provider(&request)?;
        provider.chat_stream(request).await
    }

    fn resolve_provider(&self, request: &ChatRequest) -> Result<&Arc<dyn AIProvider>> {
        if let Some(model) = &request.model {
            if let Some((provider, _)) = self.find_model(model) {
                return Ok(provider);
            }
        }

        self.default_provider()
            .ok_or_else(|| ProviderError::NotConfigured("No default provider configured".into()))
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::RateLimited { .. }
                | ProviderError::Unavailable(_)
                | ProviderError::Timeout(_)
                | ProviderError::NetworkError(_)
        )
    }
}

impl Default for ProviderRouter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RouterBuilder {
    router: ProviderRouter,
}

impl RouterBuilder {
    pub fn new() -> Self {
        Self {
            router: ProviderRouter::new(),
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn AIProvider>) -> Self {
        self.router.register_provider(provider);
        self
    }

    pub fn with_default(mut self, provider_id: &str) -> Self {
        self.router.default_provider = Some(provider_id.to_string());
        self
    }

    pub fn with_fallback_chain(mut self, chain: Vec<String>) -> Self {
        self.router.fallback_chain = chain;
        self
    }

    pub fn build(self) -> ProviderRouter {
        self.router
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProvidersConfig;

    #[test]
    fn test_router_new() {
        let router = ProviderRouter::new();
        assert!(router.providers.is_empty());
        assert!(router.default_provider.is_none());
        assert!(router.fallback_chain.is_empty());
    }

    #[test]
    fn test_router_builder() {
        let router = RouterBuilder::new()
            .with_default("claude")
            .with_fallback_chain(vec!["openai".into(), "ollama".into()])
            .build();

        assert_eq!(router.default_provider, Some("claude".into()));
        assert_eq!(router.fallback_chain.len(), 2);
    }

    #[test]
    fn test_should_fallback() {
        let router = ProviderRouter::new();

        assert!(router.should_fallback(&ProviderError::RateLimited {
            retry_after_secs: 60
        }));
        assert!(router.should_fallback(&ProviderError::Unavailable("test".into())));
        assert!(router.should_fallback(&ProviderError::Timeout(30)));
        assert!(router.should_fallback(&ProviderError::NetworkError("test".into())));

        assert!(!router.should_fallback(&ProviderError::AuthenticationFailed("test".into())));
        assert!(!router.should_fallback(&ProviderError::InvalidResponse("test".into())));
    }

    #[test]
    fn test_default_config() {
        let config = ProvidersConfig::default_config();
        assert!(config.providers.contains_key("claude"));
        assert!(config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("gemini"));
        assert!(config.providers.contains_key("ollama"));
        assert!(config.providers.contains_key("mistral"));
    }
}
