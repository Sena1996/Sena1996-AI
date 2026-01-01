use async_trait::async_trait;

use crate::error::Result;
use crate::streaming::CompletionStream;
use crate::types::{CompletionRequest, CompletionResponse, HealthCheck};

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;

    fn models(&self) -> &[String];

    fn default_model(&self) -> &str;

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        let _ = request;
        Err(crate::Error::provider("streaming not supported"))
    }

    async fn health(&self) -> HealthCheck;

    fn supports_streaming(&self) -> bool {
        false
    }

    fn max_tokens(&self) -> u32 {
        4096
    }

    fn priority(&self) -> u8 {
        100
    }
}

#[async_trait]
pub trait ProviderRegistry: Send + Sync {
    fn register(&self, provider: Box<dyn Provider>);

    fn get(&self, name: &str) -> Option<&dyn Provider>;

    fn list(&self) -> Vec<&str>;

    fn primary(&self) -> Option<&dyn Provider>;

    fn failover_chain(&self) -> Vec<&dyn Provider>;

    async fn route(&self, request: CompletionRequest) -> Result<CompletionResponse>;
}

pub trait ProviderBuilder: Send + Sync {
    type Provider: Provider;

    fn name() -> &'static str;

    fn build(api_key: &str) -> Result<Self::Provider>;

    fn build_with_config(api_key: &str, config: &crate::config::ProviderConfig) -> Result<Self::Provider>;
}
