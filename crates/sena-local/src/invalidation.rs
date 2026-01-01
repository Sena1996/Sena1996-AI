use sena_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub source_files: Vec<PathBuf>,
    pub version: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u32,
}

#[derive(Debug, Clone)]
pub struct InvalidationContext {
    pub modified_files: Vec<PathBuf>,
    pub invalidated_tags: Vec<String>,
    pub force_refresh: bool,
    pub current_version: String,
}

impl Default for InvalidationContext {
    fn default() -> Self {
        Self {
            modified_files: Vec::new(),
            invalidated_tags: Vec::new(),
            force_refresh: false,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InvalidationConfig {
    pub watch_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub debounce_ms: u64,
    pub max_tracked_files: usize,
}

impl Default for InvalidationConfig {
    fn default() -> Self {
        Self {
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.ts".to_string(),
                "**/*.py".to_string(),
                "**/*.go".to_string(),
            ],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
            debounce_ms: 500,
            max_tracked_files: 10_000,
        }
    }
}

pub struct FileTracker {
    file_mtimes: Arc<RwLock<HashMap<PathBuf, SystemTime>>>,
    entry_files: Arc<RwLock<HashMap<String, HashSet<PathBuf>>>>,
    file_entries: Arc<RwLock<HashMap<PathBuf, HashSet<String>>>>,
    _config: InvalidationConfig,
}

impl FileTracker {
    pub fn new(config: InvalidationConfig) -> Self {
        Self {
            file_mtimes: Arc::new(RwLock::new(HashMap::new())),
            entry_files: Arc::new(RwLock::new(HashMap::new())),
            file_entries: Arc::new(RwLock::new(HashMap::new())),
            _config: config,
        }
    }

    pub async fn track(&self, entry_key: &str, files: &[PathBuf]) -> Result<()> {
        let mut entry_files = self.entry_files.write().await;
        let mut file_entries = self.file_entries.write().await;
        let mut mtimes = self.file_mtimes.write().await;

        let file_set: HashSet<PathBuf> = files.iter().cloned().collect();
        entry_files.insert(entry_key.to_string(), file_set.clone());

        for file in files {
            file_entries
                .entry(file.clone())
                .or_default()
                .insert(entry_key.to_string());

            if !mtimes.contains_key(file) {
                if let Ok(metadata) = tokio::fs::metadata(file).await {
                    if let Ok(mtime) = metadata.modified() {
                        mtimes.insert(file.clone(), mtime);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn untrack(&self, entry_key: &str) {
        let mut entry_files = self.entry_files.write().await;
        let mut file_entries = self.file_entries.write().await;

        if let Some(files) = entry_files.remove(entry_key) {
            for file in files {
                if let Some(entries) = file_entries.get_mut(&file) {
                    entries.remove(entry_key);
                    if entries.is_empty() {
                        file_entries.remove(&file);
                    }
                }
            }
        }
    }

    pub async fn check_modified(&self) -> Vec<PathBuf> {
        let mut modified = Vec::new();
        let mut mtimes = self.file_mtimes.write().await;

        for (path, old_mtime) in mtimes.iter_mut() {
            if let Ok(metadata) = tokio::fs::metadata(path).await {
                if let Ok(new_mtime) = metadata.modified() {
                    if new_mtime > *old_mtime {
                        modified.push(path.clone());
                        *old_mtime = new_mtime;
                    }
                }
            }
        }

        modified
    }

    pub async fn entries_for_files(&self, files: &[PathBuf]) -> HashSet<String> {
        let file_entries = self.file_entries.read().await;
        let mut entries = HashSet::new();

        for file in files {
            if let Some(file_entry_set) = file_entries.get(file) {
                entries.extend(file_entry_set.iter().cloned());
            }
        }

        entries
    }

    pub async fn tracked_file_count(&self) -> usize {
        self.file_mtimes.read().await.len()
    }

    pub async fn tracked_entry_count(&self) -> usize {
        self.entry_files.read().await.len()
    }
}

impl Default for FileTracker {
    fn default() -> Self {
        Self::new(InvalidationConfig::default())
    }
}

pub struct CacheInvalidator {
    file_tracker: FileTracker,
    _version: String,
    invalidation_callbacks: Arc<RwLock<Vec<Box<dyn Fn(&str) + Send + Sync>>>>,
}

impl CacheInvalidator {
    pub fn new(config: InvalidationConfig) -> Self {
        Self {
            file_tracker: FileTracker::new(config),
            _version: env!("CARGO_PKG_VERSION").to_string(),
            invalidation_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn file_tracker(&self) -> &FileTracker {
        &self.file_tracker
    }

    pub async fn register_entry(&self, key: &str, files: &[PathBuf]) -> Result<()> {
        self.file_tracker.track(key, files).await
    }

    pub async fn check_and_invalidate(&self) -> Vec<String> {
        let modified_files = self.file_tracker.check_modified().await;

        if modified_files.is_empty() {
            return Vec::new();
        }

        let entries_to_invalidate = self.file_tracker.entries_for_files(&modified_files).await;
        let callbacks = self.invalidation_callbacks.read().await;

        for entry in &entries_to_invalidate {
            for callback in callbacks.iter() {
                callback(entry);
            }
            self.file_tracker.untrack(entry).await;
        }

        entries_to_invalidate.into_iter().collect()
    }

    pub fn should_invalidate(&self, entry: &CacheEntry, context: &InvalidationContext) -> bool {
        if context.force_refresh {
            return true;
        }

        if entry.version != context.current_version {
            return true;
        }

        for source_file in &entry.source_files {
            if context.modified_files.contains(source_file) {
                return true;
            }
        }

        for tag in &context.invalidated_tags {
            if entry.key.contains(tag) {
                return true;
            }
        }

        false
    }

    pub async fn on_invalidate<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.invalidation_callbacks
            .write()
            .await
            .push(Box::new(callback));
    }

    pub async fn invalidate_for_path(&self, path: &Path) -> Result<usize> {
        let entries = self.file_tracker.entries_for_files(&[path.to_path_buf()]).await;
        let count = entries.len();

        let callbacks = self.invalidation_callbacks.read().await;
        for entry in &entries {
            for callback in callbacks.iter() {
                callback(entry);
            }
            self.file_tracker.untrack(entry).await;
        }

        Ok(count)
    }

    pub async fn is_valid(&self, key: &str) -> bool {
        let entry_files = self.file_tracker.entry_files.read().await;

        if let Some(files) = entry_files.get(key) {
            let mtimes = self.file_tracker.file_mtimes.read().await;

            for file in files {
                if let Ok(metadata) = tokio::fs::metadata(file).await {
                    if let Ok(current_mtime) = metadata.modified() {
                        if let Some(tracked_mtime) = mtimes.get(file) {
                            if current_mtime > *tracked_mtime {
                                return false;
                            }
                        }
                    }
                } else {
                    return false;
                }
            }
            true
        } else {
            true
        }
    }

    pub async fn start_watcher(&self, check_interval_ms: u64) {
        let file_tracker = self.file_tracker.file_mtimes.clone();
        let entry_files = self.file_tracker.entry_files.clone();
        let file_entries = self.file_tracker.file_entries.clone();
        let callbacks = self.invalidation_callbacks.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(check_interval_ms)
            );

            loop {
                interval.tick().await;

                let mut modified = Vec::new();
                {
                    let mut mtimes = file_tracker.write().await;
                    for (path, old_mtime) in mtimes.iter_mut() {
                        if let Ok(metadata) = tokio::fs::metadata(path).await {
                            if let Ok(new_mtime) = metadata.modified() {
                                if new_mtime > *old_mtime {
                                    modified.push(path.clone());
                                    *old_mtime = new_mtime;
                                }
                            }
                        }
                    }
                }

                if !modified.is_empty() {
                    let entries_to_invalidate = {
                        let file_entries_map = file_entries.read().await;
                        let mut entries = HashSet::new();
                        for file in &modified {
                            if let Some(file_entry_set) = file_entries_map.get(file) {
                                entries.extend(file_entry_set.iter().cloned());
                            }
                        }
                        entries
                    };

                    let cbs = callbacks.read().await;
                    for entry in &entries_to_invalidate {
                        for callback in cbs.iter() {
                            callback(entry);
                        }
                    }

                    let mut ef = entry_files.write().await;
                    let mut fe = file_entries.write().await;
                    for entry in entries_to_invalidate {
                        if let Some(files) = ef.remove(&entry) {
                            for file in files {
                                if let Some(entries) = fe.get_mut(&file) {
                                    entries.remove(&entry);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

impl Default for CacheInvalidator {
    fn default() -> Self {
        Self::new(InvalidationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_tracker() {
        let tracker = FileTracker::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        tracker.track("entry1", &[path.clone()]).await.unwrap();

        assert_eq!(tracker.tracked_file_count().await, 1);
        assert_eq!(tracker.tracked_entry_count().await, 1);

        let entries = tracker.entries_for_files(&[path.clone()]).await;
        assert!(entries.contains("entry1"));
    }

    #[tokio::test]
    async fn test_invalidation_context() {
        let invalidator = CacheInvalidator::default();
        let entry = CacheEntry {
            key: "test_key".to_string(),
            source_files: vec![PathBuf::from("/tmp/test.rs")],
            version: "0.0.0".to_string(),
            created_at: 0,
            last_accessed: 0,
            access_count: 0,
        };

        let context = InvalidationContext {
            modified_files: vec![PathBuf::from("/tmp/test.rs")],
            ..Default::default()
        };

        assert!(invalidator.should_invalidate(&entry, &context));
    }

    #[tokio::test]
    async fn test_version_invalidation() {
        let invalidator = CacheInvalidator::default();
        let entry = CacheEntry {
            key: "test_key".to_string(),
            source_files: vec![],
            version: "old_version".to_string(),
            created_at: 0,
            last_accessed: 0,
            access_count: 0,
        };

        let context = InvalidationContext::default();
        assert!(invalidator.should_invalidate(&entry, &context));
    }
}
