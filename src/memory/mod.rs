use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

pub mod search;
pub mod semantic;
pub mod store;

pub use search::MemorySearch;
pub use semantic::{
    EmbeddingVector, HybridSearch, SemanticMemoryIndex, SemanticSearchResult, SimpleHashEmbedder,
    TextEmbedder,
};
pub use store::MemoryStore;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory not found: {0}")]
    NotFound(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type MemoryResult<T> = Result<T, MemoryError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryType {
    Preference,
    Fact,
    Project,
    Context,
    Conversation,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_count: u64,
    pub importance: f64,
}

impl MemoryEntry {
    pub fn new(content: impl Into<String>, memory_type: MemoryType) -> Self {
        let content = content.into();
        let id = format!(
            "mem_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        Self {
            id,
            content,
            memory_type,
            tags: Vec::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            access_count: 0,
            importance: 0.5,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.content.to_lowercase().contains(&query_lower)
            || self
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
    }

    pub fn relevance_score(&self, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let content_lower = self.content.to_lowercase();

        let mut score = 0.0;

        if content_lower.contains(&query_lower) {
            score += 0.5;
            if content_lower.starts_with(&query_lower) {
                score += 0.2;
            }
        }

        let tag_matches = self
            .tags
            .iter()
            .filter(|t| t.to_lowercase().contains(&query_lower))
            .count();
        score += tag_matches as f64 * 0.1;

        score += self.importance * 0.2;

        let age_days = (Utc::now() - self.created_at).num_days() as f64;
        let recency_factor = 1.0 / (1.0 + age_days / 30.0);
        score += recency_factor * 0.1;

        score.min(1.0)
    }
}

pub struct PersistentMemory {
    store: MemoryStore,
    memory_dir: PathBuf,
}

impl PersistentMemory {
    pub fn new() -> MemoryResult<Self> {
        let memory_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sena")
            .join("memory");

        if !memory_dir.exists() {
            fs::create_dir_all(&memory_dir)?;
        }

        let store = MemoryStore::load(&memory_dir)?;

        Ok(Self { store, memory_dir })
    }

    pub fn with_dir(memory_dir: PathBuf) -> MemoryResult<Self> {
        if !memory_dir.exists() {
            fs::create_dir_all(&memory_dir)?;
        }

        let store = MemoryStore::load(&memory_dir)?;

        Ok(Self { store, memory_dir })
    }

    pub fn add(&mut self, entry: MemoryEntry) -> MemoryResult<String> {
        let id = entry.id.clone();
        self.store.add(entry);
        self.save()?;
        Ok(id)
    }

    pub fn add_quick(&mut self, content: &str, memory_type: MemoryType) -> MemoryResult<String> {
        let entry = MemoryEntry::new(content, memory_type);
        self.add(entry)
    }

    pub fn get(&mut self, id: &str) -> Option<&mut MemoryEntry> {
        self.store.get_mut(id)
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.store.search(query)
    }

    pub fn search_by_type(&self, memory_type: &MemoryType) -> Vec<&MemoryEntry> {
        self.store.search_by_type(memory_type)
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&MemoryEntry> {
        self.store.search_by_tag(tag)
    }

    pub fn remove(&mut self, id: &str) -> MemoryResult<Option<MemoryEntry>> {
        let entry = self.store.remove(id);
        self.save()?;
        Ok(entry)
    }

    pub fn clear(&mut self) -> MemoryResult<()> {
        self.store.clear();
        self.save()
    }

    pub fn count(&self) -> usize {
        self.store.count()
    }

    pub fn all(&self) -> Vec<&MemoryEntry> {
        self.store.all()
    }

    pub fn recent(&self, limit: usize) -> Vec<&MemoryEntry> {
        self.store.recent(limit)
    }

    pub fn important(&self, limit: usize) -> Vec<&MemoryEntry> {
        self.store.important(limit)
    }

    pub fn get_context_for_query(&self, query: &str, max_entries: usize) -> String {
        let relevant = self.search(query);
        let entries: Vec<_> = relevant.into_iter().take(max_entries).collect();

        if entries.is_empty() {
            return String::new();
        }

        let mut context = String::from("Relevant memories:\n");
        for entry in entries {
            context.push_str(&format!("- [{:?}] {}\n", entry.memory_type, entry.content));
        }

        context
    }

    fn save(&self) -> MemoryResult<()> {
        self.store.save(&self.memory_dir)
    }

    pub fn stats(&self) -> MemoryStats {
        let all = self.store.all();

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut total_access = 0u64;

        for entry in &all {
            let type_key = format!("{:?}", entry.memory_type);
            *by_type.entry(type_key).or_insert(0) += 1;
            total_access += entry.access_count;
        }

        MemoryStats {
            total_entries: all.len(),
            by_type,
            total_access_count: total_access,
            avg_importance: if all.is_empty() {
                0.0
            } else {
                all.iter().map(|e| e.importance).sum::<f64>() / all.len() as f64
            },
        }
    }
}

impl Default for PersistentMemory {
    fn default() -> Self {
        Self::new().expect("Failed to create memory")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub by_type: HashMap<String, usize>,
    pub total_access_count: u64,
    pub avg_importance: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry::new("Test content", MemoryType::Fact)
            .with_tags(vec!["test".to_string()])
            .with_importance(0.8);

        assert!(entry.content.contains("Test"));
        assert_eq!(entry.tags.len(), 1);
        assert!((entry.importance - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_memory_entry_matches() {
        let entry = MemoryEntry::new("User prefers Rust", MemoryType::Preference);
        assert!(entry.matches_query("rust"));
        assert!(entry.matches_query("RUST"));
        assert!(!entry.matches_query("python"));
    }

    #[test]
    fn test_relevance_score() {
        let entry = MemoryEntry::new("User prefers Rust programming", MemoryType::Preference)
            .with_importance(0.9);

        let score = entry.relevance_score("rust");
        assert!(score > 0.0);
    }
}
