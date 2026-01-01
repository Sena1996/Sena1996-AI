use async_trait::async_trait;

use crate::error::Result;

#[async_trait]
pub trait Embedder: Send + Sync {
    fn model_name(&self) -> &str;

    fn dimension(&self) -> usize;

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    async fn embed_single(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed(&[text]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::Error::embedding("No embedding returned"))
    }

    fn max_batch_size(&self) -> usize {
        32
    }

    fn max_tokens_per_text(&self) -> usize {
        512
    }
}

#[async_trait]
pub trait EmbeddingCache: Send + Sync {
    async fn get(&self, text: &str) -> Option<Vec<f32>>;

    async fn set(&self, text: &str, embedding: Vec<f32>);

    async fn get_batch(&self, texts: &[&str]) -> Vec<Option<Vec<f32>>>;

    async fn set_batch(&self, items: Vec<(&str, Vec<f32>)>);

    async fn invalidate(&self, text: &str);

    async fn clear(&self);

    fn capacity(&self) -> usize;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait TextProcessor: Send + Sync {
    fn truncate(&self, text: &str, max_tokens: usize) -> String;

    fn chunk(&self, text: &str, chunk_size: usize, overlap: usize) -> Vec<String>;

    fn normalize(&self, text: &str) -> String;

    fn estimate_tokens(&self, text: &str) -> usize;
}
