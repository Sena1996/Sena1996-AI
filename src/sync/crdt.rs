//! Conflict-Free Replicated Data Type (CRDT)
//! Last-Write-Wins Register implementation for offline-first sync

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;

use super::offline::Change;

/// Value entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueEntry {
    pub value: serde_json::Value,
    pub timestamp: f64,
    pub vector_clock: HashMap<String, u64>,
}

/// CRDT - Conflict-Free Replicated Data Type
/// Last-Write-Wins Register implementation
pub struct CRDT {
    author_id: String,
    data: HashMap<String, ValueEntry>,
    vector_clock: HashMap<String, u64>,
    tombstones: HashSet<String>,
}

impl CRDT {
    /// Create a new CRDT instance
    pub fn new(author_id: &str) -> Self {
        let mut vector_clock = HashMap::new();
        vector_clock.insert(author_id.to_string(), 0);

        Self {
            author_id: author_id.to_string(),
            data: HashMap::new(),
            vector_clock,
            tombstones: HashSet::new(),
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }

    /// Generate unique change ID
    fn generate_change_id(&self) -> String {
        let data = format!(
            "{}{}{}",
            self.author_id,
            Self::current_timestamp(),
            uuid::Uuid::new_v4()
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(&hasher.finalize()[..8])
    }

    /// Set a value
    pub fn set(&mut self, key: &str, value: serde_json::Value) -> Change {
        // Increment vector clock
        *self.vector_clock.entry(self.author_id.clone()).or_insert(0) += 1;

        // Remove from tombstones if present
        self.tombstones.remove(key);

        let timestamp = Self::current_timestamp();

        // Store value
        self.data.insert(
            key.to_string(),
            ValueEntry {
                value: value.clone(),
                timestamp,
                vector_clock: self.vector_clock.clone(),
            },
        );

        Change {
            id: self.generate_change_id(),
            timestamp,
            operation: "update".to_string(),
            collection: "default".to_string(),
            key: key.to_string(),
            value: Some(value),
            author: self.author_id.clone(),
            vector_clock: self.vector_clock.clone(),
        }
    }

    /// Get a value
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        if self.tombstones.contains(key) {
            return None;
        }

        self.data.get(key).map(|entry| entry.value.clone())
    }

    /// Delete a value
    pub fn delete(&mut self, key: &str) -> Change {
        // Increment vector clock
        *self.vector_clock.entry(self.author_id.clone()).or_insert(0) += 1;

        // Remove from data
        self.data.remove(key);

        // Add to tombstones
        self.tombstones.insert(key.to_string());

        let timestamp = Self::current_timestamp();

        Change {
            id: self.generate_change_id(),
            timestamp,
            operation: "delete".to_string(),
            collection: "default".to_string(),
            key: key.to_string(),
            value: None,
            author: self.author_id.clone(),
            vector_clock: self.vector_clock.clone(),
        }
    }

    /// Merge a change from another replica
    /// Returns true if change was applied
    pub fn merge(&mut self, change: &Change) -> bool {
        // Update vector clock
        for (author, clock) in &change.vector_clock {
            let current = self.vector_clock.entry(author.clone()).or_insert(0);
            *current = (*current).max(*clock);
        }

        // Handle deletion
        if change.operation == "delete" {
            self.tombstones.insert(change.key.clone());
            self.data.remove(&change.key);
            return true;
        }

        // Handle create/update
        if change.operation == "create" || change.operation == "update" {
            // Check tombstone
            if self.tombstones.contains(&change.key) {
                // Already deleted - need to compare timestamps
                // For now, we won't resurrect deleted entries
                return false;
            }

            // Check if key exists
            if let Some(existing) = self.data.get(&change.key) {
                // Conflict resolution: Last-Write-Wins
                if change.timestamp > existing.timestamp {
                    // Newer timestamp wins
                    if let Some(ref value) = change.value {
                        self.data.insert(
                            change.key.clone(),
                            ValueEntry {
                                value: value.clone(),
                                timestamp: change.timestamp,
                                vector_clock: change.vector_clock.clone(),
                            },
                        );
                        return true;
                    }
                } else if (change.timestamp - existing.timestamp).abs() < f64::EPSILON {
                    // Same timestamp, use author ID as tiebreaker
                    if change.author > self.author_id {
                        if let Some(ref value) = change.value {
                            self.data.insert(
                                change.key.clone(),
                                ValueEntry {
                                    value: value.clone(),
                                    timestamp: change.timestamp,
                                    vector_clock: change.vector_clock.clone(),
                                },
                            );
                            return true;
                        }
                    }
                }
                return false;
            }

            // New key, apply change
            if let Some(ref value) = change.value {
                self.data.insert(
                    change.key.clone(),
                    ValueEntry {
                        value: value.clone(),
                        timestamp: change.timestamp,
                        vector_clock: change.vector_clock.clone(),
                    },
                );
                return true;
            }
        }

        false
    }

    /// Get all data
    pub fn get_all(&self) -> HashMap<String, serde_json::Value> {
        self.data
            .iter()
            .filter(|(k, _)| !self.tombstones.contains(*k))
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    /// Get data for serialization
    pub fn get_data(&self) -> &HashMap<String, ValueEntry> {
        &self.data
    }

    /// Get vector clock
    pub fn get_vector_clock(&self) -> &HashMap<String, u64> {
        &self.vector_clock
    }

    /// Get tombstones
    pub fn get_tombstones(&self) -> &HashSet<String> {
        &self.tombstones
    }

    /// Restore state from persisted data
    pub fn restore(
        &mut self,
        data: HashMap<String, ValueEntry>,
        vector_clock: HashMap<String, u64>,
        tombstones: HashSet<String>,
    ) {
        self.data = data;
        self.vector_clock = vector_clock;
        self.tombstones = tombstones;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_creation() {
        let crdt = CRDT::new("author-1");
        assert!(crdt.get("key").is_none());
    }

    #[test]
    fn test_set_and_get() {
        let mut crdt = CRDT::new("author-1");
        crdt.set("key1", serde_json::json!("value1"));

        let value = crdt.get("key1");
        assert!(value.is_some());
        assert_eq!(value.unwrap(), serde_json::json!("value1"));
    }

    #[test]
    fn test_delete() {
        let mut crdt = CRDT::new("author-1");
        crdt.set("key1", serde_json::json!("value1"));
        crdt.delete("key1");

        assert!(crdt.get("key1").is_none());
    }

    #[test]
    fn test_merge_new_key() {
        let mut crdt = CRDT::new("author-1");

        let change = Change {
            id: "change-1".to_string(),
            timestamp: CRDT::current_timestamp(),
            operation: "create".to_string(),
            collection: "default".to_string(),
            key: "key1".to_string(),
            value: Some(serde_json::json!("value1")),
            author: "author-2".to_string(),
            vector_clock: HashMap::new(),
        };

        let applied = crdt.merge(&change);
        assert!(applied);
        assert_eq!(crdt.get("key1"), Some(serde_json::json!("value1")));
    }

    #[test]
    fn test_get_all() {
        let mut crdt = CRDT::new("author-1");
        crdt.set("key1", serde_json::json!("value1"));
        crdt.set("key2", serde_json::json!("value2"));

        let all = crdt.get_all();
        assert_eq!(all.len(), 2);
    }
}
