pub mod error;
pub mod types;
pub mod provider;
pub mod config;
pub mod router;

#[cfg(feature = "claude")]
pub mod claude;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "gemini")]
pub mod gemini;

#[cfg(feature = "ollama")]
pub mod ollama;

#[cfg(feature = "mistral")]
pub mod mistral;

pub use error::{ProviderError, Result};
pub use types::*;
pub use provider::AIProvider;
pub use config::ProviderConfig;
pub use router::ProviderRouter;

#[cfg(feature = "claude")]
pub use claude::ClaudeProvider;

#[cfg(feature = "openai")]
pub use openai::OpenAIProvider;

#[cfg(feature = "gemini")]
pub use gemini::GeminiProvider;

#[cfg(feature = "ollama")]
pub use ollama::OllamaProvider;

#[cfg(feature = "mistral")]
pub use mistral::MistralProvider;
