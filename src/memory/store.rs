use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::{MemoryEntry, MemoryError, MemoryResult, MemoryType};

pub struct MemoryStore {
    entries: HashMap<String, MemoryEntry>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn load(dir: &Path) -> MemoryResult<Self> {
        let file_path = dir.join("memories.json");

        if !file_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&file_path)?;
        let entries: Vec<MemoryEntry> =
            serde_json::from_str(&content).map_err(|e| MemoryError::SerializationError(e.to_string()))?;

        let mut store = Self::new();
        for entry in entries {
            store.entries.insert(entry.id.clone(), entry);
        }

        Ok(store)
    }

    pub fn save(&self, dir: &Path) -> MemoryResult<()> {
        let file_path = dir.join("memories.json");
        let entries: Vec<&MemoryEntry> = self.entries.values().collect();
        let content =
            serde_json::to_string_pretty(&entries).map_err(|e| MemoryError::SerializationError(e.to_string()))?;
        fs::write(&file_path, content)?;
        Ok(())
    }

    pub fn add(&mut self, entry: MemoryEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn get(&self, id: &str) -> Option<&MemoryEntry> {
        self.entries.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut MemoryEntry> {
        self.entries.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<MemoryEntry> {
        self.entries.remove(id)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn all(&self) -> Vec<&MemoryEntry> {
        self.entries.values().collect()
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let mut results: Vec<_> = self
            .entries
            .values()
            .filter(|e| e.matches_query(query))
            .collect();

        results.sort_by(|a, b| {
            b.relevance_score(query)
                .partial_cmp(&a.relevance_score(query))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    pub fn search_by_type(&self, memory_type: &MemoryType) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| &e.memory_type == memory_type)
            .collect()
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&MemoryEntry> {
        let tag_lower = tag.to_lowercase();
        self.entries
            .values()
            .filter(|e| e.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    pub fn recent(&self, limit: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        entries.into_iter().take(limit).collect()
    }

    pub fn important(&self, limit: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.into_iter().take(limit).collect()
    }

    pub fn frequently_accessed(&self, limit: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        entries.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_add_get() {
        let mut store = MemoryStore::new();
        let entry = MemoryEntry::new("Test content", MemoryType::Fact);
        let id = entry.id.clone();

        store.add(entry);

        assert!(store.get(&id).is_some());
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_store_search() {
        let mut store = MemoryStore::new();
        store.add(MemoryEntry::new("Rust programming", MemoryType::Fact));
        store.add(MemoryEntry::new("Python scripting", MemoryType::Fact));

        let results = store.search("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_store_search_by_type() {
        let mut store = MemoryStore::new();
        store.add(MemoryEntry::new("Preference 1", MemoryType::Preference));
        store.add(MemoryEntry::new("Fact 1", MemoryType::Fact));
        store.add(MemoryEntry::new("Preference 2", MemoryType::Preference));

        let results = store.search_by_type(&MemoryType::Preference);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_store_remove() {
        let mut store = MemoryStore::new();
        let entry = MemoryEntry::new("To remove", MemoryType::Fact);
        let id = entry.id.clone();

        store.add(entry);
        assert_eq!(store.count(), 1);

        store.remove(&id);
        assert_eq!(store.count(), 0);
    }
}
