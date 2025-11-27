//! SENA Interface Definitions
//! Defines contracts (traits) that components must implement

use serde_json::Value;
use std::collections::HashMap;

/// Result type for verification operations
pub type VerificationResult = (bool, String);

/// Result type for validation operations with confidence
pub type ValidationResult = (bool, f64);

/// Interface for verification components
pub trait IVerifier: Send + Sync {
    /// Verify content
    ///
    /// # Arguments
    /// * `content` - Content to verify
    /// * `context` - Optional context dictionary
    ///
    /// # Returns
    /// Tuple of (allowed: bool, reason: String)
    fn verify(&self, content: &str, context: Option<&HashMap<String, Value>>)
        -> VerificationResult;
}

/// Interface for storage components
pub trait IStorage: Send + Sync {
    /// Store a key-value pair
    fn store(&mut self, key: &str, value: Value);

    /// Retrieve value by key
    fn retrieve(&self, key: &str) -> Option<Value>;

    /// Check if key exists
    fn exists(&self, key: &str) -> bool;

    /// Delete a key
    fn delete(&mut self, key: &str);

    /// Get all keys
    fn keys(&self) -> Vec<String>;
}

/// Execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub errors: Vec<String>,
    pub execution_time_ms: u64,
    pub exit_code: Option<i32>,
}

/// Interface for code execution components
pub trait IExecutor: Send + Sync {
    /// Execute code safely
    ///
    /// # Arguments
    /// * `code` - Code to execute
    /// * `language` - Programming language
    /// * `timeout_ms` - Maximum execution time in milliseconds
    ///
    /// # Returns
    /// ExecutionResult with status, output, errors, etc.
    fn execute(&self, code: &str, language: &str, timeout_ms: u64) -> ExecutionResult;
}

/// Permission status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionStatus {
    Allowed,
    Denied,
    AskUser,
}

/// Interface for permission management
pub trait IPermissionManager: Send + Sync {
    /// Check permission for resource/action
    ///
    /// # Returns
    /// Permission status
    fn check_permission(&self, resource: &str, action: &str) -> PermissionStatus;

    /// Grant permission
    fn grant_permission(&mut self, resource: &str, action: &str);

    /// Deny permission
    fn deny_permission(&mut self, resource: &str, action: &str);

    /// Persist permissions to storage
    fn save_permissions(&self) -> Result<(), String>;
}

/// Code pattern for memory
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodePattern {
    pub id: String,
    pub language: String,
    pub pattern_type: String,
    pub code: String,
    pub description: String,
    pub tags: Vec<String>,
    pub usage_count: u32,
    pub created_at: String,
}

/// Interface for codebase pattern memory
pub trait ICodebaseMemory: Send + Sync {
    /// Add a code pattern
    fn add_pattern(&mut self, pattern: CodePattern);

    /// Find similar patterns
    fn find_similar_pattern(&self, code: &str, threshold: f64) -> Vec<CodePattern>;

    /// Get patterns for a specific language
    fn get_patterns_by_language(&self, language: &str) -> Vec<CodePattern>;
}

/// Research result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchResult {
    pub query: String,
    pub findings: Vec<String>,
    pub confidence: f64,
    pub sources: Vec<String>,
    pub timestamp: String,
}

/// Interface for research intelligence
pub trait IResearchSystem: Send + Sync {
    /// Perform research on a query
    ///
    /// # Returns
    /// ResearchResult with findings, confidence, sources, etc.
    fn research(&self, query: &str, context: Option<&HashMap<String, Value>>) -> ResearchResult;

    /// Validate a claim with evidence
    ///
    /// # Returns
    /// Tuple of (valid: bool, confidence: f64)
    fn validate_claim(&self, claim: &str, evidence: &[String]) -> ValidationResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            success: true,
            output: "Hello".to_string(),
            errors: vec![],
            execution_time_ms: 100,
            exit_code: Some(0),
        };
        assert!(result.success);
    }

    #[test]
    fn test_permission_status() {
        assert_eq!(PermissionStatus::Allowed, PermissionStatus::Allowed);
        assert_ne!(PermissionStatus::Allowed, PermissionStatus::Denied);
    }

    #[test]
    fn test_code_pattern() {
        let pattern = CodePattern {
            id: "test-1".to_string(),
            language: "rust".to_string(),
            pattern_type: "function".to_string(),
            code: "fn test() {}".to_string(),
            description: "Test function".to_string(),
            tags: vec!["test".to_string()],
            usage_count: 0,
            created_at: "2024-01-01".to_string(),
        };
        assert_eq!(pattern.language, "rust");
    }
}
