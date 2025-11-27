pub mod config;
pub mod error;
pub mod provider;
pub mod router;
pub mod types;

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

pub use config::ProviderConfig;
pub use error::{ProviderError, Result};
pub use provider::AIProvider;
pub use router::ProviderRouter;
pub use types::*;

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
