use super::{MemoryEntry, MemoryStore, MemoryType};

pub struct MemorySearch<'a> {
    store: &'a MemoryStore,
    query: Option<String>,
    memory_type: Option<MemoryType>,
    tags: Vec<String>,
    min_importance: Option<f64>,
    limit: usize,
}

impl<'a> MemorySearch<'a> {
    pub fn new(store: &'a MemoryStore) -> Self {
        Self {
            store,
            query: None,
            memory_type: None,
            tags: Vec::new(),
            min_importance: None,
            limit: 100,
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn of_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    pub fn min_importance(mut self, importance: f64) -> Self {
        self.min_importance = Some(importance);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn execute(&self) -> Vec<&MemoryEntry> {
        let mut results: Vec<&MemoryEntry> = self.store.all();

        if let Some(ref query) = self.query {
            results.retain(|e| e.matches_query(query));
        }

        if let Some(ref mt) = self.memory_type {
            results.retain(|e| &e.memory_type == mt);
        }

        if !self.tags.is_empty() {
            results.retain(|e| {
                self.tags.iter().any(|tag| {
                    e.tags
                        .iter()
                        .any(|t| t.to_lowercase() == tag.to_lowercase())
                })
            });
        }

        if let Some(min_imp) = self.min_importance {
            results.retain(|e| e.importance >= min_imp);
        }

        if let Some(ref query) = self.query {
            results.sort_by(|a, b| {
                b.relevance_score(query)
                    .partial_cmp(&a.relevance_score(query))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        results.into_iter().take(self.limit).collect()
    }
}

pub struct SearchResult {
    pub entry: MemoryEntry,
    pub score: f64,
    pub matched_on: Vec<String>,
}

impl SearchResult {
    pub fn from_entry(entry: &MemoryEntry, query: &str) -> Self {
        let mut matched_on = Vec::new();

        if entry.content.to_lowercase().contains(&query.to_lowercase()) {
            matched_on.push("content".to_string());
        }

        for tag in &entry.tags {
            if tag.to_lowercase().contains(&query.to_lowercase()) {
                matched_on.push(format!("tag:{}", tag));
            }
        }

        Self {
            entry: entry.clone(),
            score: entry.relevance_score(query),
            matched_on,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_builder() {
        let mut store = MemoryStore::new();
        store.add(
            MemoryEntry::new("Rust is great", MemoryType::Preference)
                .with_tags(vec!["programming".to_string()])
                .with_importance(0.9),
        );
        store.add(
            MemoryEntry::new("Python is useful", MemoryType::Preference)
                .with_tags(vec!["programming".to_string()])
                .with_importance(0.5),
        );
        store.add(MemoryEntry::new("Coffee is good", MemoryType::Preference).with_importance(0.3));

        let search = MemorySearch::new(&store)
            .of_type(MemoryType::Preference)
            .min_importance(0.5);
        let results = search.execute();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_with_query() {
        let mut store = MemoryStore::new();
        store.add(MemoryEntry::new("Rust programming", MemoryType::Fact));
        store.add(MemoryEntry::new("Python scripting", MemoryType::Fact));

        let search = MemorySearch::new(&store).query("rust");
        let results = search.execute();

        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("Rust"));
    }

    #[test]
    fn test_search_with_tags() {
        let mut store = MemoryStore::new();
        store.add(MemoryEntry::new("Item 1", MemoryType::Fact).with_tags(vec!["important".to_string()]));
        store.add(MemoryEntry::new("Item 2", MemoryType::Fact).with_tags(vec!["trivial".to_string()]));

        let search = MemorySearch::new(&store).with_tag("important");
        let results = search.execute();

        assert_eq!(results.len(), 1);
    }
}
