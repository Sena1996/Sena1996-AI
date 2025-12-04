use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevilError {
    #[error("No providers available")]
    NoProviders,

    #[error("All providers failed: {0}")]
    AllProvidersFailed(String),

    #[error("Consensus failed: insufficient agreement")]
    ConsensusFailure,

    #[error("Synthesis failed: {0}")]
    SynthesisError(String),

    #[error("Timeout: only {completed}/{total} providers responded")]
    PartialTimeout { completed: usize, total: usize },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type DevilResult<T> = Result<T, DevilError>;
