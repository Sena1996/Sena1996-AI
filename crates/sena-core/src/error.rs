use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Provider,
    Embedding,
    VectorStore,
    Session,
    Security,
    Config,
    Network,
    Timeout,
    RateLimit,
    Validation,
    NotFound,
    Internal,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider => write!(f, "provider"),
            Self::Embedding => write!(f, "embedding"),
            Self::VectorStore => write!(f, "vector_store"),
            Self::Session => write!(f, "session"),
            Self::Security => write!(f, "security"),
            Self::Config => write!(f, "config"),
            Self::Network => write!(f, "network"),
            Self::Timeout => write!(f, "timeout"),
            Self::RateLimit => write!(f, "rate_limit"),
            Self::Validation => write!(f, "validation"),
            Self::NotFound => write!(f, "not_found"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn provider(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Provider, message)
    }

    pub fn embedding(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Embedding, message)
    }

    pub fn vector_store(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::VectorStore, message)
    }

    pub fn session(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Session, message)
    }

    pub fn security(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Security, message)
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Security, message)
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Config, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Network, message)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Timeout, message)
    }

    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RateLimit, message)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Validation, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::Network | ErrorKind::Timeout | ErrorKind::RateLimit
        )
    }

    pub fn is_transient(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::Network | ErrorKind::Timeout | ErrorKind::RateLimit | ErrorKind::Provider
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorKind::Internal, err.to_string()).with_source(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorKind::Validation, err.to_string()).with_source(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
