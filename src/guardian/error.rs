use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuardianError {
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Execution blocked: {0}")]
    ExecutionBlocked(String),

    #[error("Hallucination detected with confidence {confidence:.2}")]
    HallucinationDetected { confidence: f64 },

    #[error("Sandbox error: {0}")]
    SandboxError(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type GuardianResult<T> = Result<T, GuardianError>;
