use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider not configured: {0}")]
    NotConfigured(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Rate limited: retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Context length exceeded: {used} tokens used, {max} max")]
    ContextLengthExceeded { used: usize, max: usize },

    #[error("Provider unavailable: {0}")]
    Unavailable(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::Timeout(30)
        } else if err.is_connect() {
            ProviderError::NetworkError(err.to_string())
        } else {
            ProviderError::RequestFailed(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ProviderError>;
