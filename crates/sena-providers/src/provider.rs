use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::{
    ChatRequest, ChatResponse, ModelInfo, ProviderCapabilities,
    ProviderStatus, Result, StreamChunk,
};

pub type ChatStream = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn provider_id(&self) -> &str;

    fn display_name(&self) -> &str;

    fn capabilities(&self) -> &ProviderCapabilities;

    fn default_model(&self) -> &str;

    fn available_models(&self) -> &[ModelInfo];

    fn status(&self) -> ProviderStatus;

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream>;

    fn supports_streaming(&self) -> bool {
        self.capabilities().streaming
    }

    fn supports_tools(&self) -> bool {
        self.capabilities().tool_use
    }

    fn supports_vision(&self) -> bool {
        self.capabilities().vision
    }

    fn max_context_tokens(&self) -> usize {
        self.capabilities().max_context_tokens
    }
}
