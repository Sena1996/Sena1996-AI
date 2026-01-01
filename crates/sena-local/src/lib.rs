mod cache;
mod chunking;
mod complexity;
mod compression;
mod consensus;
mod context;
mod embeddings;
mod hallucination;
mod intent;
mod invalidation;
mod persistent;
mod storage;
mod textrank;
mod tokenizer;
mod vector;

pub use cache::{CacheConfigBuilder, CacheStats, EmbeddingCacheImpl, TtlCacheWrapper};
pub use chunking::{ChunkConfig, TextChunk, TextChunker};
pub use complexity::{ComplexityLevel, ComplexityResult, ComplexityScorer};
pub use compression::{CompressionConfig, CompressionResult, PromptCompressor};
pub use consensus::{
    ConsensusConfig, ConsensusResult, Discrepancy, DiscrepancySeverity, MultiModelConsensus,
    ProviderResponse, VotingStrategy,
};
pub use context::{ContextConfig, ContextMessage, ContextStats, ContextWindow};
pub use embeddings::FastEmbedder;
pub use hallucination::{
    ControllerConfig, DetectionResult, FactCheckResult, FactChecker, HallucinationController,
    HallucinationDetector, HallucinationIssue, HallucinationType, RiskLevel, VerificationResult,
};
pub use intent::{Intent, IntentDetector, IntentResult, ModelTier};
pub use invalidation::{
    CacheEntry, CacheInvalidator, FileTracker, InvalidationConfig, InvalidationContext,
};
pub use persistent::{SqliteMessageStore, SqliteSessionManager};
pub use storage::SqliteStore;
pub use textrank::TextRankSummarizer;
pub use tokenizer::{count_tokens, estimate_tokens, TokenBudget, Tokenizer, TokenizerType};
pub use vector::QdrantStore;

use sena_core::{CompletionRequest, Embedder, EmbeddingCache, Provider, Result, VectorStore};
use std::sync::Arc;

pub struct LocalEngine {
    embedder: Arc<dyn Embedder>,
    cache: Arc<dyn EmbeddingCache>,
    vector_store: Option<Arc<dyn VectorStore>>,
    summarizer: TextRankSummarizer,
    chunker: TextChunker,
    intent_detector: IntentDetector,
    complexity_scorer: ComplexityScorer,
    hallucination_detector: HallucinationDetector,
    hallucination_controller: HallucinationController,
    compressor: PromptCompressor,
    context_window: ContextWindow,
    tokenizer: Tokenizer,
    cache_invalidator: CacheInvalidator,
}

impl LocalEngine {
    pub fn new(
        embedder: Arc<dyn Embedder>,
        cache: Arc<dyn EmbeddingCache>,
        vector_store: Option<Arc<dyn VectorStore>>,
    ) -> Self {
        Self {
            embedder,
            cache,
            vector_store,
            summarizer: TextRankSummarizer::new(),
            chunker: TextChunker::default(),
            intent_detector: IntentDetector::new(),
            complexity_scorer: ComplexityScorer::new(),
            hallucination_detector: HallucinationDetector::new(),
            hallucination_controller: HallucinationController::default(),
            compressor: PromptCompressor::default(),
            context_window: ContextWindow::default(),
            tokenizer: Tokenizer::new(TokenizerType::Cl100k),
            cache_invalidator: CacheInvalidator::new(InvalidationConfig::default()),
        }
    }

    pub fn with_chunk_config(mut self, config: ChunkConfig) -> Self {
        self.chunker = TextChunker::new(config);
        self
    }

    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = vec![None; texts.len()];
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();

        for (i, text) in texts.iter().enumerate() {
            if let Some(cached) = self.cache.get(text).await {
                results[i] = Some(cached);
            } else {
                uncached_indices.push(i);
                uncached_texts.push(*text);
            }
        }

        if !uncached_texts.is_empty() {
            let new_embeddings = self.embedder.embed(&uncached_texts).await?;

            for (idx, embedding) in uncached_indices.into_iter().zip(new_embeddings) {
                self.cache.set(texts[idx], embedding.clone()).await;
                results[idx] = Some(embedding);
            }
        }

        Ok(results.into_iter().map(|r| r.unwrap()).collect())
    }

    pub async fn embed_single(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(cached) = self.cache.get(text).await {
            return Ok(cached);
        }

        let embedding = self.embedder.embed_single(text).await?;
        self.cache.set(text, embedding.clone()).await;
        Ok(embedding)
    }

    pub fn summarize(&self, text: &str, num_sentences: usize) -> Result<String> {
        self.summarizer.summarize(text, num_sentences)
    }

    pub fn extract_keywords(&self, text: &str, num_keywords: usize) -> Vec<String> {
        self.summarizer.extract_keywords(text, num_keywords)
    }

    pub async fn store(&self, collection: &str, id: &str, text: &str) -> Result<()> {
        let store = self
            .vector_store
            .as_ref()
            .ok_or_else(|| sena_core::Error::config("vector store not configured"))?;

        let embedding = self.embed_single(text).await?;

        let point = sena_core::VectorPoint::new(id, embedding)
            .with_payload("text", serde_json::Value::String(text.to_string()));

        store.upsert(collection, vec![point]).await
    }

    pub async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<sena_core::SearchResult>> {
        let store = self
            .vector_store
            .as_ref()
            .ok_or_else(|| sena_core::Error::config("vector store not configured"))?;

        let query_embedding = self.embed_single(query).await?;
        store.search(collection, query_embedding, limit).await
    }

    pub fn embedder(&self) -> &dyn Embedder {
        self.embedder.as_ref()
    }

    pub fn cache(&self) -> &dyn EmbeddingCache {
        self.cache.as_ref()
    }

    pub fn vector_store(&self) -> Option<&dyn VectorStore> {
        self.vector_store.as_ref().map(|s| s.as_ref())
    }

    pub fn detect_intent(&self, text: &str) -> IntentResult {
        self.intent_detector.detect(text)
    }

    pub fn score_complexity(&self, text: &str) -> ComplexityResult {
        self.complexity_scorer.score(text)
    }

    pub fn validate_response(&self, response: &str, context: Option<&str>) -> DetectionResult {
        self.hallucination_detector.detect(response, context)
    }

    pub fn should_use_premium_model(&self, text: &str) -> bool {
        self.complexity_scorer.should_use_premium_model(text)
    }

    pub async fn embed_smart(&self, text: &str) -> Result<Vec<f32>> {
        if !self.chunker.needs_chunking(text) {
            return self.embed_single(text).await;
        }

        let chunks = self.chunker.chunk(text);
        let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embed(&chunk_texts).await?;

        let weights: Vec<f32> = chunks.iter().map(|c| c.token_estimate as f32).collect();
        Ok(TextChunker::weighted_average_embeddings(&embeddings, &weights))
    }

    pub async fn store_smart(&self, collection: &str, id: &str, text: &str) -> Result<()> {
        let store = self
            .vector_store
            .as_ref()
            .ok_or_else(|| sena_core::Error::config("vector store not configured"))?;

        if !self.chunker.needs_chunking(text) {
            return self.store(collection, id, text).await;
        }

        let chunks = self.chunker.chunk(text);
        let mut points = Vec::with_capacity(chunks.len());

        for chunk in &chunks {
            let embedding = self.embed_single(&chunk.content).await?;
            let chunk_id = format!("{}_{}", id, chunk.index);
            let point = sena_core::VectorPoint::new(&chunk_id, embedding)
                .with_payload("text", serde_json::Value::String(chunk.content.clone()))
                .with_payload("parent_id", serde_json::Value::String(id.to_string()))
                .with_payload("chunk_index", serde_json::Value::Number(chunk.index.into()))
                .with_payload("start_char", serde_json::Value::Number(chunk.start_char.into()))
                .with_payload("end_char", serde_json::Value::Number(chunk.end_char.into()));
            points.push(point);
        }

        store.upsert(collection, points).await
    }

    pub fn analyze_query(&self, query: &str) -> QueryAnalysis {
        let intent = self.detect_intent(query);
        let complexity = self.score_complexity(query);
        let recommended_model = complexity.recommended_model.clone();
        let max_tokens = complexity.suggested_max_tokens;
        let requires_ai = intent.requires_ai;

        QueryAnalysis {
            intent,
            complexity,
            recommended_model,
            max_tokens,
            requires_ai,
        }
    }

    pub fn chunker(&self) -> &TextChunker {
        &self.chunker
    }

    pub fn intent_detector(&self) -> &IntentDetector {
        &self.intent_detector
    }

    pub fn complexity_scorer(&self) -> &ComplexityScorer {
        &self.complexity_scorer
    }

    pub fn hallucination_detector(&self) -> &HallucinationDetector {
        &self.hallucination_detector
    }

    pub fn compressor(&self) -> &PromptCompressor {
        &self.compressor
    }

    pub fn compress_prompt(&self, text: &str) -> Result<CompressionResult> {
        self.compressor.compress(text)
    }

    pub fn compress_to_limit(&self, text: &str, max_tokens: u32) -> Result<CompressionResult> {
        self.compressor.compress_to_token_limit(text, max_tokens)
    }

    pub fn compress_messages(&self, messages: &[(String, String)]) -> Result<Vec<(String, String)>> {
        self.compressor.compress_conversation(messages)
    }

    pub fn hallucination_controller(&self) -> &HallucinationController {
        &self.hallucination_controller
    }

    pub async fn verify_response_with_consensus(
        &self,
        request: &CompletionRequest,
        providers: &[Arc<dyn Provider>],
    ) -> Result<VerificationResult> {
        self.hallucination_controller.verify_with_consensus(request, providers).await
    }

    pub fn context_window(&self) -> &ContextWindow {
        &self.context_window
    }

    pub fn context_window_mut(&mut self) -> &mut ContextWindow {
        &mut self.context_window
    }

    pub fn add_to_context(&mut self, role: &str, content: &str) {
        match role {
            "user" => self.context_window.add_user_message(content),
            "assistant" => self.context_window.add_assistant_message(content),
            _ => self.context_window.add_user_message(content),
        }
    }

    pub fn get_context_messages(&self) -> Vec<ContextMessage> {
        self.context_window.messages()
    }

    pub fn context_stats(&self) -> ContextStats {
        self.context_window.stats()
    }

    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    pub fn count_tokens(&self, text: &str) -> u32 {
        self.tokenizer.count_tokens(text)
    }

    pub fn count_message_tokens(&self, messages: &[(String, String)]) -> u32 {
        self.tokenizer.count_messages(messages)
    }

    pub fn cache_invalidator(&self) -> &CacheInvalidator {
        &self.cache_invalidator
    }

    pub async fn invalidate_for_file(&self, path: &std::path::Path) -> Result<usize> {
        self.cache_invalidator.invalidate_for_path(path).await
    }

    pub async fn check_cache_validity(&self, key: &str) -> bool {
        self.cache_invalidator.is_valid(key).await
    }

    pub fn with_context_config(mut self, config: ContextConfig) -> Self {
        self.context_window = ContextWindow::new(config);
        self
    }

    pub fn with_tokenizer_type(mut self, tokenizer_type: TokenizerType) -> Self {
        self.tokenizer = Tokenizer::new(tokenizer_type);
        self
    }

    pub fn with_invalidation_config(mut self, config: InvalidationConfig) -> Self {
        self.cache_invalidator = CacheInvalidator::new(config);
        self
    }

    pub fn with_hallucination_config(mut self, config: ControllerConfig) -> Self {
        self.hallucination_controller = HallucinationController::new(config);
        self
    }
}

#[derive(Debug, Clone)]
pub struct QueryAnalysis {
    pub intent: IntentResult,
    pub complexity: ComplexityResult,
    pub recommended_model: String,
    pub max_tokens: u32,
    pub requires_ai: bool,
}
