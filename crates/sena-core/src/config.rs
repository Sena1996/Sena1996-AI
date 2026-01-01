use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenaConfig {
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub local: LocalConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub observe: ObserveConfig,
}

impl Default for SenaConfig {
    fn default() -> Self {
        Self {
            providers: ProvidersConfig::default(),
            local: LocalConfig::default(),
            security: SecurityConfig::default(),
            session: SessionConfig::default(),
            observe: ObserveConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default = "default_timeout_secs")]
    pub default_timeout_secs: u64,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub failover_chain: Vec<String>,
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: default_timeout_secs(),
            providers: HashMap::new(),
            failover_chain: Vec::new(),
        }
    }
}

fn default_timeout_secs() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub max_retries: u8,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
    #[serde(default = "default_half_open_timeout_secs")]
    pub half_open_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            success_threshold: default_success_threshold(),
            half_open_timeout_secs: default_half_open_timeout_secs(),
        }
    }
}

fn default_failure_threshold() -> u32 {
    5
}

fn default_success_threshold() -> u32 {
    2
}

fn default_half_open_timeout_secs() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    #[serde(default)]
    pub embeddings: EmbeddingsConfig,
    #[serde(default)]
    pub vector_store: VectorStoreConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            embeddings: EmbeddingsConfig::default(),
            vector_store: VectorStoreConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_dimension")]
    pub dimension: usize,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            dimension: default_dimension(),
            batch_size: default_batch_size(),
        }
    }
}

fn default_model() -> String {
    "BAAI/bge-small-en-v1.5".to_string()
}

fn default_dimension() -> usize {
    384
}

fn default_batch_size() -> usize {
    32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    #[serde(default = "default_qdrant_url")]
    pub url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_collection")]
    pub default_collection: String,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            url: default_qdrant_url(),
            api_key: None,
            default_collection: default_collection(),
        }
    }
}

fn default_qdrant_url() -> String {
    "http://localhost:6334".to_string()
}

fn default_collection() -> String {
    "sena_embeddings".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_max_capacity")]
    pub max_capacity: u64,
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: default_max_capacity(),
            ttl_secs: default_ttl_secs(),
        }
    }
}

fn default_max_capacity() -> u64 {
    10_000
}

fn default_ttl_secs() -> u64 {
    3600
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_keyring_service")]
    pub keyring_service: String,
    #[serde(default)]
    pub encryption_enabled: bool,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            keyring_service: default_keyring_service(),
            encryption_enabled: false,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

fn default_keyring_service() -> String {
    "sena".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: default_requests_per_minute(),
            burst_size: default_burst_size(),
        }
    }
}

fn default_requests_per_minute() -> u32 {
    60
}

fn default_burst_size() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,
    #[serde(default = "default_session_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default)]
    pub persistence_path: Option<PathBuf>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: default_max_sessions(),
            ttl_secs: default_session_ttl_secs(),
            persistence_path: None,
        }
    }
}

fn default_max_sessions() -> usize {
    100
}

fn default_session_ttl_secs() -> u64 {
    86400
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserveConfig {
    #[serde(default)]
    pub metrics_enabled: bool,
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for ObserveConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: false,
            metrics_port: default_metrics_port(),
            log_level: default_log_level(),
        }
    }
}

fn default_metrics_port() -> u16 {
    9090
}

fn default_log_level() -> String {
    "info".to_string()
}

impl SenaConfig {
    pub fn provider_timeout(&self, provider: &str) -> Duration {
        self.providers
            .providers
            .get(provider)
            .and_then(|p| p.timeout_secs)
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(self.providers.default_timeout_secs))
    }
}
