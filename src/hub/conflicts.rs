//! Conflict Detection System
//!
//! Detects when multiple sessions are editing the same files

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use super::state::HubState;

/// File conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConflict {
    pub file_path: String,
    pub sessions: Vec<String>,
    pub other_session: String,  // For single conflict reporting
    pub detected_at: u64,
    pub severity: ConflictSeverity,
}

/// Conflict severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Warning,   // Same file, different sections likely OK
    High,      // Same file, likely conflict
    Critical,  // Same exact location
}

impl ConflictSeverity {
    pub fn emoji(&self) -> &'static str {
        match self {
            ConflictSeverity::Warning => "⚠️",
            ConflictSeverity::High => "🔶",
            ConflictSeverity::Critical => "🔴",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ConflictSeverity::Warning => "Warning",
            ConflictSeverity::High => "High",
            ConflictSeverity::Critical => "Critical",
        }
    }
}

/// File lock for exclusive access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLock {
    pub file_path: String,
    pub session_id: String,
    pub locked_at: u64,
    pub expires_at: u64,
}

impl FileLock {
    /// Create a new file lock
    pub fn new(file_path: &str, session_id: &str, duration_secs: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            file_path: file_path.to_string(),
            session_id: session_id.to_string(),
            locked_at: now,
            expires_at: now + duration_secs,
        }
    }

    /// Check if lock is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        now > self.expires_at
    }
}

/// Conflict Detector
pub struct ConflictDetector {
    locks: HashMap<String, FileLock>,
    conflict_history: Vec<FileConflict>,
}

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            conflict_history: Vec::new(),
        }
    }

    /// Check if a file would cause a conflict
    pub fn check_file(&mut self, file_path: &str, session_id: &str, state: &HubState) -> Option<FileConflict> {
        // Get all sessions working on this file
        let working_sessions = state.who_is_working_on(file_path);

        // Filter out current session
        let others: Vec<String> = working_sessions
            .into_iter()
            .filter(|s| s != session_id)
            .collect();

        if others.is_empty() {
            return None;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let conflict = FileConflict {
            file_path: file_path.to_string(),
            sessions: {
                let mut all = others.clone();
                all.push(session_id.to_string());
                all
            },
            other_session: others.first().cloned().unwrap_or_default(),
            detected_at: now,
            severity: if others.len() > 1 {
                ConflictSeverity::Critical
            } else {
                ConflictSeverity::High
            },
        };

        self.conflict_history.push(conflict.clone());

        Some(conflict)
    }

    /// Get all current conflicts from state
    pub fn get_all(&self, state: &HubState) -> Vec<FileConflict> {
        let working_sessions = state.get_working_sessions();

        // Group by file path
        let mut file_sessions: HashMap<String, Vec<String>> = HashMap::new();

        for (session_id, file_path) in working_sessions {
            file_sessions
                .entry(file_path.clone())
                .or_default()
                .push(session_id.clone());
        }

        // Find conflicts (multiple sessions on same file)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        file_sessions
            .into_iter()
            .filter(|(_, sessions)| sessions.len() > 1)
            .map(|(file_path, sessions)| FileConflict {
                file_path: file_path.clone(),
                other_session: sessions.first().cloned().unwrap_or_default(),
                sessions,
                detected_at: now,
                severity: ConflictSeverity::High,
            })
            .collect()
    }

    /// Try to acquire a lock on a file
    pub fn acquire_lock(&mut self, file_path: &str, session_id: &str, duration_secs: u64) -> Result<FileLock, String> {
        // Clean up expired locks
        self.cleanup_expired_locks();

        // Check if already locked
        if let Some(existing) = self.locks.get(file_path) {
            if existing.session_id != session_id {
                return Err(format!(
                    "File {} is locked by session {}",
                    file_path, existing.session_id
                ));
            }
            // Same session, extend lock
        }

        let lock = FileLock::new(file_path, session_id, duration_secs);
        self.locks.insert(file_path.to_string(), lock.clone());

        Ok(lock)
    }

    /// Release a lock
    pub fn release_lock(&mut self, file_path: &str, session_id: &str) -> Result<(), String> {
        if let Some(lock) = self.locks.get(file_path) {
            if lock.session_id != session_id {
                return Err("Cannot release lock owned by another session".to_string());
            }
        }

        self.locks.remove(file_path);
        Ok(())
    }

    /// Check if a file is locked
    pub fn is_locked(&self, file_path: &str) -> bool {
        if let Some(lock) = self.locks.get(file_path) {
            !lock.is_expired()
        } else {
            false
        }
    }

    /// Get lock holder for a file
    pub fn get_lock_holder(&self, file_path: &str) -> Option<String> {
        self.locks.get(file_path).map(|l| l.session_id.clone())
    }

    /// Clean up expired locks
    pub fn cleanup_expired_locks(&mut self) {
        let expired: Vec<String> = self.locks
            .iter()
            .filter(|(_, lock)| lock.is_expired())
            .map(|(path, _)| path.clone())
            .collect();

        for path in expired {
            self.locks.remove(&path);
        }
    }

    /// Get conflict history
    pub fn get_history(&self, limit: usize) -> Vec<FileConflict> {
        self.conflict_history
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear conflict history
    pub fn clear_history(&mut self) {
        self.conflict_history.clear();
    }

    /// Get all active locks
    pub fn get_locks(&self) -> Vec<&FileLock> {
        self.locks.values().filter(|l| !l.is_expired()).collect()
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_severity() {
        assert_eq!(ConflictSeverity::Critical.emoji(), "🔴");
        assert_eq!(ConflictSeverity::Warning.name(), "Warning");
    }

    #[test]
    fn test_file_lock_creation() {
        let lock = FileLock::new("/src/main.rs", "session-1", 300);
        assert_eq!(lock.file_path, "/src/main.rs");
        assert!(!lock.is_expired());
    }

    #[test]
    fn test_conflict_detector_creation() {
        let detector = ConflictDetector::new();
        assert!(detector.get_locks().is_empty());
    }

    #[test]
    fn test_acquire_and_release_lock() {
        let mut detector = ConflictDetector::new();

        let lock = detector.acquire_lock("/test.rs", "session-1", 300);
        assert!(lock.is_ok());
        assert!(detector.is_locked("/test.rs"));

        let release = detector.release_lock("/test.rs", "session-1");
        assert!(release.is_ok());
        assert!(!detector.is_locked("/test.rs"));
    }
}
