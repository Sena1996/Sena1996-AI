//! Offline-First Synchronization Engine
//! 100% offline functionality with local-first data and conflict-free sync

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::crdt::{CRDT, ValueEntry};
use crate::base::component::{BaseComponent, ComponentMetrics, ComponentState, ComponentStatus};

/// Represents a data change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub timestamp: f64,
    pub operation: String,  // create, update, delete
    pub collection: String,
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub author: String,
    pub vector_clock: HashMap<String, u64>,
}

/// Offline sync metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineSyncMetrics {
    pub writes: u64,
    pub reads: u64,
    pub syncs: u64,
    pub conflicts_resolved: u64,
    pub changes_applied: u64,
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub applied: u64,
    pub conflicts: u64,
    pub total: u64,
}

/// Persisted data format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedData {
    version: String,
    author_id: String,
    data: HashMap<String, ValueEntry>,
    vector_clock: HashMap<String, u64>,
    tombstones: Vec<String>,
    last_updated: String,
}

/// Offline-First Synchronization Engine
pub struct OfflineSync {
    state: ComponentState,
    author_id: String,
    crdt: RwLock<CRDT>,
    change_log: RwLock<Vec<Change>>,
    pending_changes: RwLock<Vec<Change>>,
    last_sync: RwLock<Option<DateTime<Utc>>>,
    sync_in_progress: RwLock<bool>,
    metrics: RwLock<OfflineSyncMetrics>,
    storage_dir: PathBuf,
    config: SyncConfig,
}

/// Sync configuration
#[derive(Debug, Clone)]
struct SyncConfig {
    auto_sync: bool,
    sync_interval_seconds: u64,
    max_change_log_size: usize,
}

impl OfflineSync {
    /// Create a new offline sync engine
    pub fn new(author_id: Option<&str>) -> Self {
        let storage_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("data")
            .join("offline");

        let _ = fs::create_dir_all(&storage_dir);

        let author = author_id
            .map(|s| s.to_string())
            .unwrap_or_else(Self::generate_author_id);

        Self {
            state: ComponentState::new("OfflineSync"),
            author_id: author.clone(),
            crdt: RwLock::new(CRDT::new(&author)),
            change_log: RwLock::new(Vec::new()),
            pending_changes: RwLock::new(Vec::new()),
            last_sync: RwLock::new(None),
            sync_in_progress: RwLock::new(false),
            metrics: RwLock::new(OfflineSyncMetrics {
                writes: 0,
                reads: 0,
                syncs: 0,
                conflicts_resolved: 0,
                changes_applied: 0,
            }),
            storage_dir,
            config: SyncConfig {
                auto_sync: false,
                sync_interval_seconds: 300,
                max_change_log_size: 10000,
            },
        }
    }

    /// Generate unique author ID
    fn generate_author_id() -> String {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        format!("{}-{}", hostname, timestamp)
    }

    /// Set a value (works offline)
    pub fn set(&self, key: &str, value: serde_json::Value) {
        let change = {
            let mut crdt = self.crdt.write().unwrap();
            crdt.set(key, value)
        };

        // Add to logs
        {
            let mut log = self.change_log.write().unwrap();
            log.push(change.clone());
        }
        {
            let mut pending = self.pending_changes.write().unwrap();
            pending.push(change.clone());
        }

        // Persist
        self.save_local_data();
        self.append_to_change_log(&change);

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.writes += 1;
        }
    }

    /// Get a value (works offline)
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        let mut metrics = self.metrics.write().unwrap();
        metrics.reads += 1;
        drop(metrics);

        let crdt = self.crdt.read().unwrap();
        crdt.get(key)
    }

    /// Delete a value (works offline)
    pub fn delete(&self, key: &str) {
        let change = {
            let mut crdt = self.crdt.write().unwrap();
            crdt.delete(key)
        };

        // Add to logs
        {
            let mut log = self.change_log.write().unwrap();
            log.push(change.clone());
        }
        {
            let mut pending = self.pending_changes.write().unwrap();
            pending.push(change.clone());
        }

        // Persist
        self.save_local_data();
        self.append_to_change_log(&change);

        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.writes += 1;
        }
    }

    /// Get all data
    pub fn get_all(&self) -> HashMap<String, serde_json::Value> {
        let crdt = self.crdt.read().unwrap();
        crdt.get_all()
    }

    /// Get changes pending sync
    pub fn get_pending_changes(&self) -> Vec<Change> {
        let pending = self.pending_changes.read().unwrap();
        pending.clone()
    }

    /// Apply changes from remote
    pub fn apply_remote_changes(&self, remote_changes: Vec<Change>) -> SyncResult {
        let mut applied = 0;
        let mut conflicts = 0;

        for change in &remote_changes {
            let was_applied = {
                let mut crdt = self.crdt.write().unwrap();
                crdt.merge(change)
            };

            if was_applied {
                applied += 1;
                {
                    let mut metrics = self.metrics.write().unwrap();
                    metrics.changes_applied += 1;
                }

                // Add to change log
                {
                    let mut log = self.change_log.write().unwrap();
                    log.push(change.clone());
                }
            } else {
                conflicts += 1;
                {
                    let mut metrics = self.metrics.write().unwrap();
                    metrics.conflicts_resolved += 1;
                }
            }
        }

        // Persist if changes applied
        if applied > 0 {
            self.save_local_data();
        }

        // Clear pending
        {
            let mut pending = self.pending_changes.write().unwrap();
            pending.clear();
        }

        // Update sync time
        {
            let mut last_sync = self.last_sync.write().unwrap();
            *last_sync = Some(Utc::now());
        }

        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.syncs += 1;
        }

        SyncResult {
            applied,
            conflicts,
            total: remote_changes.len() as u64,
        }
    }

    /// Check if online (placeholder)
    pub fn is_online(&self) -> bool {
        false // For now, assume offline
    }

    /// Check if sync is needed
    pub fn needs_sync(&self) -> bool {
        let pending = self.pending_changes.read().unwrap();
        if pending.is_empty() {
            return false;
        }

        let last_sync = self.last_sync.read().unwrap();
        if last_sync.is_none() {
            return true;
        }

        let interval = chrono::Duration::seconds(self.config.sync_interval_seconds as i64);
        Utc::now() - last_sync.unwrap() > interval
    }

    /// Get sync status
    pub fn get_sync_status(&self) -> serde_json::Value {
        let pending = self.pending_changes.read().unwrap();
        let last_sync = self.last_sync.read().unwrap();
        let in_progress = self.sync_in_progress.read().unwrap();

        serde_json::json!({
            "author_id": self.author_id,
            "last_sync": last_sync.map(|t| t.to_rfc3339()),
            "pending_changes": pending.len(),
            "sync_in_progress": *in_progress,
            "online": self.is_online(),
            "needs_sync": self.needs_sync(),
        })
    }

    /// Get statistics
    pub fn get_stats(&self) -> serde_json::Value {
        let crdt = self.crdt.read().unwrap();
        let log = self.change_log.read().unwrap();
        let pending = self.pending_changes.read().unwrap();
        let last_sync = self.last_sync.read().unwrap();
        let metrics = self.metrics.read().unwrap();

        serde_json::json!({
            "author_id": self.author_id,
            "total_changes": log.len(),
            "pending_changes": pending.len(),
            "total_keys": crdt.get_data().len(),
            "tombstones": crdt.get_tombstones().len(),
            "last_sync": last_sync.map(|t| t.to_rfc3339()),
            "writes": metrics.writes,
            "reads": metrics.reads,
            "syncs": metrics.syncs,
            "conflicts_resolved": metrics.conflicts_resolved,
            "changes_applied": metrics.changes_applied,
        })
    }

    // File paths
    fn data_file(&self) -> PathBuf {
        self.storage_dir.join("local_data.json")
    }

    fn change_log_file(&self) -> PathBuf {
        self.storage_dir.join("change_log.jsonl")
    }

    // Persistence
    fn save_local_data(&self) {
        let crdt = self.crdt.read().unwrap();

        let data = PersistedData {
            version: "7.0.0".to_string(),
            author_id: self.author_id.clone(),
            data: crdt.get_data().clone(),
            vector_clock: crdt.get_vector_clock().clone(),
            tombstones: crdt.get_tombstones().iter().cloned().collect(),
            last_updated: Utc::now().to_rfc3339(),
        };

        // Atomic write
        let temp_file = self.data_file().with_extension("tmp");
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            if fs::write(&temp_file, &json).is_ok() {
                let _ = fs::rename(&temp_file, self.data_file());
            }
        }
    }

    fn load_local_data(&self) {
        if let Ok(content) = fs::read_to_string(self.data_file()) {
            if let Ok(data) = serde_json::from_str::<PersistedData>(&content) {
                let mut crdt = self.crdt.write().unwrap();
                crdt.restore(
                    data.data,
                    data.vector_clock,
                    data.tombstones.into_iter().collect(),
                );
            }
        }
    }

    fn append_to_change_log(&self, change: &Change) {
        if let Ok(json) = serde_json::to_string(change) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.change_log_file())
            {
                let _ = writeln!(file, "{}", json);
            }
        }
    }

    fn load_change_log(&self) {
        if let Ok(file) = File::open(self.change_log_file()) {
            let reader = BufReader::new(file);
            let mut log = self.change_log.write().unwrap();

            for line in reader.lines().flatten() {
                if let Ok(change) = serde_json::from_str::<Change>(&line) {
                    log.push(change);
                }
            }
        }
    }
}

impl BaseComponent for OfflineSync {
    fn name(&self) -> &str {
        &self.state.name
    }

    fn initialize(&mut self) -> Result<(), String> {
        self.load_local_data();
        self.load_change_log();
        self.state.mark_initialized();
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), String> {
        self.save_local_data();
        Ok(())
    }

    fn get_status(&self) -> ComponentStatus {
        ComponentStatus {
            name: self.state.name.clone(),
            initialized: self.state.initialized,
            healthy: true,
            details: {
                let mut details = HashMap::new();
                details.insert("stats".to_string(), self.get_stats());
                details
            },
        }
    }

    fn is_initialized(&self) -> bool {
        self.state.initialized
    }

    fn get_metrics(&self) -> ComponentMetrics {
        self.state.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_sync_creation() {
        let sync = OfflineSync::new(Some("test-author"));
        assert!(!sync.is_initialized());
    }

    #[test]
    fn test_set_and_get() {
        let sync = OfflineSync::new(Some("test-author"));
        sync.set("key1", serde_json::json!("value1"));

        let value = sync.get("key1");
        assert_eq!(value, Some(serde_json::json!("value1")));
    }

    #[test]
    fn test_delete() {
        let sync = OfflineSync::new(Some("test-author"));
        sync.set("key1", serde_json::json!("value1"));
        sync.delete("key1");

        assert!(sync.get("key1").is_none());
    }

    #[test]
    fn test_pending_changes() {
        let sync = OfflineSync::new(Some("test-author"));
        sync.set("key1", serde_json::json!("value1"));

        let pending = sync.get_pending_changes();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_get_all() {
        let sync = OfflineSync::new(Some("test-author"));
        sync.set("key1", serde_json::json!("value1"));
        sync.set("key2", serde_json::json!("value2"));

        let all = sync.get_all();
        assert_eq!(all.len(), 2);
    }
}
