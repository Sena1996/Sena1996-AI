pub mod config;
pub mod error;
pub mod streaming;
pub mod traits;
pub mod types;

pub use config::SenaConfig;
pub use error::{Error, ErrorKind, Result};
pub use streaming::{
    BoxStream, ChunkCollector, CompletionStream, StreamBuffer, StreamChunk, StreamEvent,
    StreamEventType, StreamResult,
};
pub use traits::{
    EmbeddingCache, Embedder, Filter, FilterCondition, KeyValueStore, MessageStore, Provider,
    ProviderBuilder, ProviderRegistry, Session, SessionCreateConfig, SessionManager, SessionState,
    SessionSync, TextProcessor, VectorStore,
};
pub use types::{
    CircuitState, CompletionRequest, CompletionResponse, HealthCheck, HealthStatus, Message,
    MessageRole, Query, QueryResponse, QueryType, SearchResult, SessionId, Usage, VectorPoint,
};
