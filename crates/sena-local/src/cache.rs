use async_trait::async_trait;
use moka::future::Cache;
use sena_core::EmbeddingCache;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CacheConfigBuilder {
    pub max_capacity: u64,
    pub ttl_secs: u64,
    pub tti_secs: Option<u64>,
}

impl Default for CacheConfigBuilder {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            ttl_secs: 3600,
            tti_secs: None,
        }
    }
}

impl CacheConfigBuilder {
    pub fn max_capacity(mut self, capacity: u64) -> Self {
        self.max_capacity = capacity;
        self
    }

    pub fn ttl(mut self, secs: u64) -> Self {
        self.ttl_secs = secs;
        self
    }

    pub fn time_to_idle(mut self, secs: u64) -> Self {
        self.tti_secs = Some(secs);
        self
    }
}

pub struct EmbeddingCacheImpl {
    cache: Cache<String, Vec<f32>>,
    ttl: Duration,
    hits: AtomicUsize,
    misses: AtomicUsize,
    evictions: AtomicUsize,
}

impl EmbeddingCacheImpl {
    pub fn new(config: &sena_core::config::CacheConfig) -> Self {
        let ttl = Duration::from_secs(config.ttl_secs);
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(ttl)
            .build();

        Self {
            cache,
            ttl,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            evictions: AtomicUsize::new(0),
        }
    }

    pub fn with_capacity(max_capacity: u64, ttl_secs: u64) -> Self {
        let ttl = Duration::from_secs(ttl_secs);
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();

        Self {
            cache,
            ttl,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            evictions: AtomicUsize::new(0),
        }
    }

    pub fn from_builder(builder: CacheConfigBuilder) -> Self {
        let ttl = Duration::from_secs(builder.ttl_secs);
        let mut cache_builder = Cache::builder()
            .max_capacity(builder.max_capacity)
            .time_to_live(ttl);

        if let Some(tti_secs) = builder.tti_secs {
            cache_builder = cache_builder.time_to_idle(Duration::from_secs(tti_secs));
        }

        Self {
            cache: cache_builder.build(),
            ttl,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            evictions: AtomicUsize::new(0),
        }
    }

    pub fn builder() -> CacheConfigBuilder {
        CacheConfigBuilder::default()
    }

    fn hash_key(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            size: self.cache.entry_count() as usize,
            hit_rate: self.hit_rate(),
            ttl_secs: self.ttl.as_secs(),
        }
    }

    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    pub async fn set_with_ttl(&self, text: &str, embedding: Vec<f32>, _ttl: Duration) {
        let key = Self::hash_key(text);
        self.cache.insert(key, embedding).await;
    }

    pub async fn get_or_insert_with<F>(&self, text: &str, f: F) -> Vec<f32>
    where
        F: std::future::Future<Output = Vec<f32>>,
    {
        let key = Self::hash_key(text);

        if let Some(v) = self.cache.get(&key).await {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return v;
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        let embedding = f.await;
        self.cache.insert(key, embedding.clone()).await;
        embedding
    }

    pub async fn remove(&self, text: &str) -> bool {
        let key = Self::hash_key(text);
        let existed = self.cache.get(&key).await.is_some();
        self.cache.invalidate(&key).await;
        if existed {
            self.evictions.fetch_add(1, Ordering::Relaxed);
        }
        existed
    }

    pub async fn cleanup(&self) {
        self.cache.run_pending_tasks().await;
    }

    pub fn weighted_size(&self) -> u64 {
        self.cache.weighted_size()
    }
}

impl Default for EmbeddingCacheImpl {
    fn default() -> Self {
        Self::new(&sena_core::config::CacheConfig::default())
    }
}

pub struct TtlCacheWrapper {
    inner: EmbeddingCacheImpl,
    default_ttl: Duration,
}

impl TtlCacheWrapper {
    pub fn new(cache: EmbeddingCacheImpl, default_ttl: Duration) -> Self {
        Self {
            inner: cache,
            default_ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<f32>> {
        self.inner.get(key).await
    }

    pub async fn set(&self, key: &str, value: Vec<f32>) {
        self.inner.set(key, value).await;
    }

    pub async fn set_with_ttl(&self, key: &str, value: Vec<f32>, ttl: Duration) {
        self.inner.set_with_ttl(key, value, ttl).await;
    }

    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    pub fn stats(&self) -> CacheStats {
        self.inner.stats()
    }
}

#[async_trait]
impl EmbeddingCache for EmbeddingCacheImpl {
    async fn get(&self, text: &str) -> Option<Vec<f32>> {
        let key = Self::hash_key(text);
        match self.cache.get(&key).await {
            Some(v) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(v)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    async fn set(&self, text: &str, embedding: Vec<f32>) {
        let key = Self::hash_key(text);
        self.cache.insert(key, embedding).await;
    }

    async fn get_batch(&self, texts: &[&str]) -> Vec<Option<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.get(text).await);
        }
        results
    }

    async fn set_batch(&self, items: Vec<(&str, Vec<f32>)>) {
        for (text, embedding) in items {
            self.set(text, embedding).await;
        }
    }

    async fn invalidate(&self, text: &str) {
        let key = Self::hash_key(text);
        self.cache.invalidate(&key).await;
    }

    async fn clear(&self) {
        self.cache.invalidate_all();
    }

    fn capacity(&self) -> usize {
        self.cache.policy().max_capacity().unwrap_or(0) as usize
    }

    fn len(&self) -> usize {
        self.cache.entry_count() as usize
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub size: usize,
    pub hit_rate: f64,
    pub ttl_secs: u64,
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache() {
        let cache = EmbeddingCacheImpl::with_capacity(100, 3600);

        assert!(cache.get("test").await.is_none());

        cache.set("test", vec![1.0, 2.0, 3.0]).await;

        let result = cache.get("test").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![1.0, 2.0, 3.0]);
    }

    #[tokio::test]
    async fn test_stats() {
        let cache = EmbeddingCacheImpl::with_capacity(100, 3600);

        cache.get("miss1").await;
        cache.get("miss2").await;
        cache.set("hit", vec![1.0]).await;
        cache.get("hit").await;

        let stats = cache.stats();
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.hits, 1);
    }
}
