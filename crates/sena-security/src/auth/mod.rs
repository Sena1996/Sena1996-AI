mod credentials;
mod oauth;
mod provider_auth;
mod token_store;

pub use credentials::{AuthCredentials, AuthMethod, CredentialStatus};
pub use oauth::{OAuthClient, OAuthConfig, OAuthToken};
pub use provider_auth::{
    AnthropicAuth, CohereAuth, DeepSeekAuth, GeminiAuth, GroqAuth, HuggingFaceAuth,
    MistralAuth, OllamaAuth, OpenAIAuth, PerplexityAuth, ProviderAuth, TogetherAuth, XaiAuth,
};
pub use token_store::TokenStore;

use sena_core::Result;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AuthManager {
    token_store: Arc<TokenStore>,
    providers: HashMap<String, Arc<dyn ProviderAuth>>,
    oauth_client: OAuthClient,
}

impl AuthManager {
    pub fn new() -> Result<Self> {
        let token_store = Arc::new(TokenStore::new()?);
        let oauth_client = OAuthClient::new();

        let mut providers: HashMap<String, Arc<dyn ProviderAuth>> = HashMap::new();

        providers.insert("anthropic".to_string(), Arc::new(AnthropicAuth::new()));
        providers.insert("openai".to_string(), Arc::new(OpenAIAuth::new()));
        providers.insert("gemini".to_string(), Arc::new(GeminiAuth::new()));
        providers.insert("groq".to_string(), Arc::new(GroqAuth::new()));
        providers.insert("deepseek".to_string(), Arc::new(DeepSeekAuth::new()));
        providers.insert("mistral".to_string(), Arc::new(MistralAuth::new()));
        providers.insert("cohere".to_string(), Arc::new(CohereAuth::new()));
        providers.insert("xai".to_string(), Arc::new(XaiAuth::new()));
        providers.insert("together".to_string(), Arc::new(TogetherAuth::new()));
        providers.insert("huggingface".to_string(), Arc::new(HuggingFaceAuth::new()));
        providers.insert("perplexity".to_string(), Arc::new(PerplexityAuth::new()));
        providers.insert("ollama".to_string(), Arc::new(OllamaAuth::new()));

        Ok(Self {
            token_store,
            providers,
            oauth_client,
        })
    }

    pub fn supported_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_provider_auth(&self, provider: &str) -> Option<Arc<dyn ProviderAuth>> {
        self.providers.get(provider).cloned()
    }

    pub async fn login(&self, provider: &str) -> Result<AuthCredentials> {
        let provider_auth = self.providers.get(provider)
            .ok_or_else(|| sena_core::Error::validation(format!("Unknown provider: {}", provider)))?;

        let auth_method = provider_auth.preferred_auth_method();

        let credentials = match auth_method {
            AuthMethod::OAuth => {
                let config = provider_auth.oauth_config()
                    .ok_or_else(|| sena_core::Error::validation("OAuth not supported for this provider"))?;

                let token = self.oauth_client.authenticate(&config).await?;

                AuthCredentials {
                    provider: provider.to_string(),
                    method: AuthMethod::OAuth,
                    api_key: None,
                    oauth_token: Some(token),
                    status: CredentialStatus::Valid,
                }
            }
            AuthMethod::ApiKey => {
                if let Some(key) = self.token_store.get_api_key(provider)? {
                    AuthCredentials {
                        provider: provider.to_string(),
                        method: AuthMethod::ApiKey,
                        api_key: Some(key),
                        oauth_token: None,
                        status: CredentialStatus::Valid,
                    }
                } else {
                    let key = provider_auth.prompt_for_api_key().await?;
                    self.token_store.store_api_key(provider, &key)?;

                    AuthCredentials {
                        provider: provider.to_string(),
                        method: AuthMethod::ApiKey,
                        api_key: Some(key),
                        oauth_token: None,
                        status: CredentialStatus::Valid,
                    }
                }
            }
            AuthMethod::None => {
                AuthCredentials {
                    provider: provider.to_string(),
                    method: AuthMethod::None,
                    api_key: None,
                    oauth_token: None,
                    status: CredentialStatus::Valid,
                }
            }
        };

        if let Some(ref token) = credentials.oauth_token {
            self.token_store.store_oauth_token(provider, token)?;
        }

        Ok(credentials)
    }

    pub async fn login_with_key(&self, provider: &str, api_key: &str) -> Result<AuthCredentials> {
        let provider_auth = self.providers.get(provider)
            .ok_or_else(|| sena_core::Error::validation(format!("Unknown provider: {}", provider)))?;

        let valid = provider_auth.validate_api_key(api_key).await?;

        if !valid {
            return Err(sena_core::Error::auth("Invalid API key"));
        }

        self.token_store.store_api_key(provider, api_key)?;

        Ok(AuthCredentials {
            provider: provider.to_string(),
            method: AuthMethod::ApiKey,
            api_key: Some(api_key.to_string()),
            oauth_token: None,
            status: CredentialStatus::Valid,
        })
    }

    pub async fn logout(&self, provider: &str) -> Result<()> {
        self.token_store.remove_credentials(provider)
    }

    pub fn get_provider_auth_method(&self, provider: &str) -> Option<AuthMethod> {
        self.providers.get(provider).map(|p| p.preferred_auth_method())
    }

    pub fn get_api_key_env_var(&self, provider: &str) -> Option<String> {
        self.providers.get(provider).map(|p| p.api_key_env_var().to_string())
    }

    pub fn get_api_key_url(&self, provider: &str) -> Option<String> {
        self.providers.get(provider).map(|p| p.api_key_url().to_string())
    }

    pub fn get_credentials(&self, provider: &str) -> Result<Option<AuthCredentials>> {
        if let Some(key) = self.token_store.get_api_key(provider)? {
            return Ok(Some(AuthCredentials {
                provider: provider.to_string(),
                method: AuthMethod::ApiKey,
                api_key: Some(key),
                oauth_token: None,
                status: CredentialStatus::Valid,
            }));
        }

        if let Some(token) = self.token_store.get_oauth_token(provider)? {
            let status = if token.is_expired() {
                CredentialStatus::Expired
            } else {
                CredentialStatus::Valid
            };

            return Ok(Some(AuthCredentials {
                provider: provider.to_string(),
                method: AuthMethod::OAuth,
                api_key: None,
                oauth_token: Some(token),
                status,
            }));
        }

        Ok(None)
    }

    pub async fn refresh_if_needed(&self, provider: &str) -> Result<AuthCredentials> {
        let credentials = self.get_credentials(provider)?
            .ok_or_else(|| sena_core::Error::auth(format!("Not logged in to {}", provider)))?;

        if credentials.status == CredentialStatus::Expired {
            if let Some(token) = credentials.oauth_token {
                if let Some(refresh_token) = token.refresh_token {
                    let provider_auth = self.providers.get(provider).unwrap();
                    let config = provider_auth.oauth_config().unwrap();
                    let new_token = self.oauth_client.refresh_token(&config, &refresh_token).await?;
                    self.token_store.store_oauth_token(provider, &new_token)?;

                    return Ok(AuthCredentials {
                        provider: provider.to_string(),
                        method: AuthMethod::OAuth,
                        api_key: None,
                        oauth_token: Some(new_token),
                        status: CredentialStatus::Valid,
                    });
                }
            }
            return Err(sena_core::Error::auth("Token expired and cannot be refreshed"));
        }

        Ok(credentials)
    }

    pub fn get_api_key_for_provider(&self, provider: &str) -> Result<Option<String>> {
        if let Some(creds) = self.get_credentials(provider)? {
            if let Some(key) = creds.api_key {
                return Ok(Some(key));
            }
            if let Some(token) = creds.oauth_token {
                return Ok(Some(token.access_token));
            }
        }

        if let Ok(key) = std::env::var(format!("{}_API_KEY", provider.to_uppercase())) {
            return Ok(Some(key));
        }

        Ok(None)
    }

    pub fn list_authenticated(&self) -> Result<Vec<(String, CredentialStatus)>> {
        let mut result = Vec::new();

        for provider in self.providers.keys() {
            if let Some(creds) = self.get_credentials(provider)? {
                result.push((provider.clone(), creds.status));
            }
        }

        Ok(result)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new().expect("Failed to create AuthManager")
    }
}
