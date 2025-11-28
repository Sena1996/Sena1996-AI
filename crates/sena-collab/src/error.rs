use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollabError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Agent unavailable: {0}")]
    AgentUnavailable(String),

    #[error("Message delivery failed: {0}")]
    MessageDeliveryFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Session already exists: {0}")]
    SessionAlreadyExists(String),

    #[error("Session limit reached: maximum {0} sessions allowed")]
    SessionLimitReached(usize),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Provider error: {0}")]
    ProviderError(#[from] sena_providers::ProviderError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Config error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, CollabError>;
