use regex::Regex;
use sena_core::{Error, Result};
use std::path::{Path, PathBuf};

pub struct Sanitizer {
    max_input_length: usize,
    injection_patterns: Vec<Regex>,
}

impl Sanitizer {
    pub fn new() -> Self {
        Self {
            max_input_length: 100_000,
            injection_patterns: Self::compile_injection_patterns(),
        }
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_input_length = max_length;
        self
    }

    fn compile_injection_patterns() -> Vec<Regex> {
        let patterns = [
            r"(?i)\bignore\s+(previous|all|above)\s+(instructions?|prompts?)\b",
            r"(?i)\byou\s+are\s+now\b",
            r"(?i)\bact\s+as\s+(if|a)\b",
            r"(?i)\bforget\s+(everything|all)\b",
            r"(?i)\bpretend\s+(you|to)\b",
            r"(?i)\bdisregard\s+(previous|all)\b",
            r"(?i)\boverride\s+(your|the)\b",
            r"(?i)\bnew\s+instructions?\b",
            r"(?i)\bsystem\s*:\s*",
            r"(?i)\[system\]",
            r"(?i)```system",
            r"(?i)<\s*system\s*>",
        ];

        patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect()
    }

    pub fn check_injection(&self, input: &str) -> Result<()> {
        for pattern in &self.injection_patterns {
            if pattern.is_match(input) {
                return Err(Error::validation("potential prompt injection detected"));
            }
        }
        Ok(())
    }

    pub fn sanitize_input(&self, input: &str) -> Result<String> {
        if input.len() > self.max_input_length {
            return Err(Error::validation(format!(
                "input exceeds maximum length of {} bytes",
                self.max_input_length
            )));
        }

        self.check_injection(input)?;

        let sanitized = input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect::<String>();

        Ok(sanitized.trim().to_string())
    }

    pub fn sanitize_path(&self, path: &str) -> Result<PathBuf> {
        let path = Path::new(path);

        if path.components().any(|c| {
            matches!(c, std::path::Component::ParentDir)
        }) {
            return Err(Error::validation("path traversal not allowed"));
        }

        let normalized: PathBuf = path.components().collect();

        if normalized.to_string_lossy().contains('\0') {
            return Err(Error::validation("null bytes not allowed in path"));
        }

        Ok(normalized)
    }

    pub fn sanitize_api_key(&self, key: &str) -> Result<String> {
        let trimmed = key.trim();

        if trimmed.is_empty() {
            return Err(Error::validation("API key cannot be empty"));
        }

        if trimmed.len() < 20 {
            return Err(Error::validation("API key too short"));
        }

        if trimmed.len() > 200 {
            return Err(Error::validation("API key too long"));
        }

        if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(Error::validation("API key contains invalid characters"));
        }

        Ok(trimmed.to_string())
    }

    pub fn sanitize_model_name(&self, name: &str) -> Result<String> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(Error::validation("model name cannot be empty"));
        }

        if trimmed.len() > 100 {
            return Err(Error::validation("model name too long"));
        }

        if !trimmed.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/'
        }) {
            return Err(Error::validation("model name contains invalid characters"));
        }

        Ok(trimmed.to_string())
    }
}

impl Default for Sanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_detection() {
        let sanitizer = Sanitizer::new();
        assert!(sanitizer.check_injection("ignore previous instructions").is_err());
        assert!(sanitizer.check_injection("Hello, how are you?").is_ok());
    }

    #[test]
    fn test_path_sanitization() {
        let sanitizer = Sanitizer::new();
        assert!(sanitizer.sanitize_path("../../../etc/passwd").is_err());
        assert!(sanitizer.sanitize_path("data/file.txt").is_ok());
    }
}
