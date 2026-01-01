use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use parking_lot::RwLock;
use sena_core::{Embedder, Error, Result};
use std::sync::Arc;

pub struct FastEmbedder {
    model: Arc<RwLock<TextEmbedding>>,
    model_name: String,
    dimension: usize,
    batch_size: usize,
}

impl FastEmbedder {
    pub fn new(config: &sena_core::config::EmbeddingsConfig) -> Result<Self> {
        let model_enum = match config.model.as_str() {
            "BAAI/bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "BAAI/bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            "BAAI/bge-large-en-v1.5" => EmbeddingModel::BGELargeENV15,
            _ => EmbeddingModel::BGESmallENV15,
        };

        let options = InitOptions::new(model_enum).with_show_download_progress(false);

        let model = TextEmbedding::try_new(options)
            .map_err(|e| Error::embedding(format!("failed to load model: {}", e)))?;

        Ok(Self {
            model: Arc::new(RwLock::new(model)),
            model_name: config.model.clone(),
            dimension: config.dimension,
            batch_size: config.batch_size,
        })
    }

    pub fn default_model() -> Result<Self> {
        Self::new(&sena_core::config::EmbeddingsConfig::default())
    }
}

#[async_trait]
impl Embedder for FastEmbedder {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let texts_owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        let model = self.model.clone();

        tokio::task::spawn_blocking(move || {
            let model_guard = model.read();
            model_guard
                .embed(texts_owned, None)
                .map_err(|e| Error::embedding(format!("embedding failed: {}", e)))
        })
        .await
        .map_err(|e| Error::internal(format!("task join failed: {}", e)))?
    }

    fn max_batch_size(&self) -> usize {
        self.batch_size
    }

    fn max_tokens_per_text(&self) -> usize {
        512
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_embedding() {
        let embedder = FastEmbedder::default_model().unwrap();
        let texts = vec!["Hello, world!"];
        let embeddings = embedder.embed(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), 384);
    }
}
