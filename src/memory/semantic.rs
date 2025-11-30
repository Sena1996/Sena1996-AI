use super::MemoryEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub dimensions: usize,
    pub values: Vec<f32>,
}

impl EmbeddingVector {
    pub fn new(values: Vec<f32>) -> Self {
        Self {
            dimensions: values.len(),
            values,
        }
    }

    pub fn zeros(dimensions: usize) -> Self {
        Self {
            dimensions,
            values: vec![0.0; dimensions],
        }
    }

    pub fn cosine_similarity(&self, other: &EmbeddingVector) -> f32 {
        if self.dimensions != other.dimensions {
            return 0.0;
        }

        let dot_product: f32 = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.values.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    pub fn euclidean_distance(&self, other: &EmbeddingVector) -> f32 {
        if self.dimensions != other.dimensions {
            return f32::MAX;
        }

        self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    pub fn normalize(&mut self) {
        let norm: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut self.values {
                *v /= norm;
            }
        }
    }
}

pub trait TextEmbedder {
    fn embed(&self, text: &str) -> EmbeddingVector;
    fn dimensions(&self) -> usize;
}

pub struct SimpleHashEmbedder {
    dimensions: usize,
}

impl SimpleHashEmbedder {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl Default for SimpleHashEmbedder {
    fn default() -> Self {
        Self::new(128)
    }
}

impl TextEmbedder for SimpleHashEmbedder {
    fn embed(&self, text: &str) -> EmbeddingVector {
        let mut values = vec![0.0f32; self.dimensions];

        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            let hash = simple_hash(word);
            let idx = (hash as usize) % self.dimensions;

            let position_weight = 1.0 / (1.0 + i as f32 * 0.1);
            let word_weight = 1.0 / (1.0 + word.len() as f32 * 0.05);

            values[idx] += position_weight * word_weight;

            for (j, c) in word.chars().enumerate() {
                let char_idx = ((hash as usize) + (c as usize) + j) % self.dimensions;
                values[char_idx] += 0.1 * position_weight;
            }
        }

        let bigrams = generate_bigrams(&text_lower);
        for bigram in bigrams {
            let hash = simple_hash(&bigram);
            let idx = (hash as usize) % self.dimensions;
            values[idx] += 0.5;
        }

        let mut embedding = EmbeddingVector::new(values);
        embedding.normalize();
        embedding
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for c in s.chars() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u64);
    }
    hash
}

fn generate_bigrams(text: &str) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut bigrams = Vec::new();

    for window in words.windows(2) {
        bigrams.push(format!("{}_{}", window[0], window[1]));
    }

    bigrams
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemoryEntry {
    pub memory_id: String,
    pub embedding: EmbeddingVector,
}

pub struct SemanticMemoryIndex {
    entries: HashMap<String, SemanticMemoryEntry>,
    embedder: SimpleHashEmbedder,
}

impl SemanticMemoryIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            embedder: SimpleHashEmbedder::default(),
        }
    }

    pub fn with_dimensions(dimensions: usize) -> Self {
        Self {
            entries: HashMap::new(),
            embedder: SimpleHashEmbedder::new(dimensions),
        }
    }

    pub fn index_entry(&mut self, entry: &MemoryEntry) {
        let embedding = self.embedder.embed(&entry.content);

        let semantic_entry = SemanticMemoryEntry {
            memory_id: entry.id.clone(),
            embedding,
        };

        self.entries.insert(entry.id.clone(), semantic_entry);
    }

    pub fn remove_entry(&mut self, memory_id: &str) {
        self.entries.remove(memory_id);
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SemanticSearchResult> {
        let query_embedding = self.embedder.embed(query);

        let mut results: Vec<SemanticSearchResult> = self
            .entries
            .values()
            .map(|entry| {
                let similarity = query_embedding.cosine_similarity(&entry.embedding);
                SemanticSearchResult {
                    memory_id: entry.memory_id.clone(),
                    similarity,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.into_iter().take(limit).collect()
    }

    pub fn find_similar(&self, memory_id: &str, limit: usize) -> Vec<SemanticSearchResult> {
        let entry = match self.entries.get(memory_id) {
            Some(e) => e,
            None => return Vec::new(),
        };

        let mut results: Vec<SemanticSearchResult> = self
            .entries
            .values()
            .filter(|e| e.memory_id != memory_id)
            .map(|other| {
                let similarity = entry.embedding.cosine_similarity(&other.embedding);
                SemanticSearchResult {
                    memory_id: other.memory_id.clone(),
                    similarity,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.into_iter().take(limit).collect()
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for SemanticMemoryIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub memory_id: String,
    pub similarity: f32,
}

pub struct HybridSearch {
    keyword_weight: f32,
    semantic_weight: f32,
}

impl HybridSearch {
    pub fn new(keyword_weight: f32, semantic_weight: f32) -> Self {
        let total = keyword_weight + semantic_weight;
        Self {
            keyword_weight: keyword_weight / total,
            semantic_weight: semantic_weight / total,
        }
    }

    pub fn balanced() -> Self {
        Self::new(0.5, 0.5)
    }

    pub fn keyword_focused() -> Self {
        Self::new(0.7, 0.3)
    }

    pub fn semantic_focused() -> Self {
        Self::new(0.3, 0.7)
    }

    pub fn combine_scores(&self, keyword_score: f64, semantic_similarity: f32) -> f64 {
        (self.keyword_weight as f64 * keyword_score)
            + (self.semantic_weight as f64 * semantic_similarity as f64)
    }
}

impl Default for HybridSearch {
    fn default() -> Self {
        Self::balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryType;

    #[test]
    fn test_embedding_vector_creation() {
        let vec = EmbeddingVector::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(vec.dimensions, 3);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let vec1 = EmbeddingVector::new(vec![1.0, 0.0, 0.0]);
        let vec2 = EmbeddingVector::new(vec![1.0, 0.0, 0.0]);

        let similarity = vec1.cosine_similarity(&vec2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let vec1 = EmbeddingVector::new(vec![1.0, 0.0]);
        let vec2 = EmbeddingVector::new(vec![0.0, 1.0]);

        let similarity = vec1.cosine_similarity(&vec2);
        assert!(similarity.abs() < 0.001);
    }

    #[test]
    fn test_simple_hash_embedder() {
        let embedder = SimpleHashEmbedder::new(64);
        let embedding = embedder.embed("hello world");

        assert_eq!(embedding.dimensions, 64);
    }

    #[test]
    fn test_similar_texts_have_similar_embeddings() {
        let embedder = SimpleHashEmbedder::new(128);

        let emb1 = embedder.embed("rust programming language");
        let emb2 = embedder.embed("rust programming");
        let emb3 = embedder.embed("python scripting language");

        let sim_rust = emb1.cosine_similarity(&emb2);
        let sim_different = emb1.cosine_similarity(&emb3);

        assert!(sim_rust > sim_different);
    }

    #[test]
    fn test_semantic_index() {
        let mut index = SemanticMemoryIndex::new();

        let entry1 = MemoryEntry::new("Rust is a systems programming language", MemoryType::Fact);
        let entry2 = MemoryEntry::new("Python is great for scripting", MemoryType::Fact);

        index.index_entry(&entry1);
        index.index_entry(&entry2);

        assert_eq!(index.count(), 2);
    }

    #[test]
    fn test_semantic_search() {
        let mut index = SemanticMemoryIndex::new();

        let entry1 = MemoryEntry::new("Rust programming language", MemoryType::Fact);
        let entry2 = MemoryEntry::new("Python scripting", MemoryType::Fact);
        let entry3 = MemoryEntry::new("Rust compiler and cargo", MemoryType::Fact);

        index.index_entry(&entry1);
        index.index_entry(&entry2);
        index.index_entry(&entry3);

        let results = index.search("rust", 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_hybrid_search_weights() {
        let hybrid = HybridSearch::new(0.6, 0.4);

        let combined = hybrid.combine_scores(0.8, 0.5);
        let expected = 0.6 * 0.8 + 0.4 * 0.5;

        assert!((combined - expected).abs() < 0.001);
    }
}
