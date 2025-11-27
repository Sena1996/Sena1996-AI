use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub peer_id: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub used: bool,
    pub used_at: Option<i64>,
    pub used_by: Option<String>,
}

impl AuthToken {
    pub fn new(expires_in_seconds: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            token: Self::generate_token(),
            peer_id: None,
            created_at: now,
            expires_at: now + expires_in_seconds,
            used: false,
            used_at: None,
            used_by: None,
        }
    }

    pub fn for_peer(peer_id: &str, expires_in_seconds: i64) -> Self {
        let mut token = Self::new(expires_in_seconds);
        token.peer_id = Some(peer_id.to_string());
        token
    }

    fn generate_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes)
    }

    pub fn is_valid(&self) -> bool {
        !self.used && chrono::Utc::now().timestamp() < self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() >= self.expires_at
    }

    pub fn mark_used(&mut self, used_by: &str) {
        self.used = true;
        self.used_at = Some(chrono::Utc::now().timestamp());
        self.used_by = Some(used_by.to_string());
    }

    pub fn remaining_seconds(&self) -> i64 {
        let now = chrono::Utc::now().timestamp();
        (self.expires_at - now).max(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthTokenStore {
    pub tokens: HashMap<String, AuthToken>,
    #[serde(skip)]
    file_path: PathBuf,
}

impl AuthTokenStore {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            tokens: HashMap::new(),
            file_path,
        }
    }

    pub fn load(file_path: PathBuf) -> Result<Self, String> {
        if !file_path.exists() {
            let mut store = Self::new(file_path.clone());
            store.file_path = file_path;
            return Ok(store);
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read token store: {}", e))?;

        let mut store: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse token store: {}", e))?;

        store.file_path = file_path;
        store.cleanup_expired();
        Ok(store)
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize token store: {}", e))?;

        fs::write(&self.file_path, content)
            .map_err(|e| format!("Failed to write token store: {}", e))
    }

    pub fn create_token(&mut self, expires_in_seconds: i64) -> Result<AuthToken, String> {
        let token = AuthToken::new(expires_in_seconds);
        self.tokens.insert(token.token.clone(), token.clone());
        self.save()?;
        Ok(token)
    }

    pub fn create_token_for_peer(&mut self, peer_id: &str, expires_in_seconds: i64) -> Result<AuthToken, String> {
        let token = AuthToken::for_peer(peer_id, expires_in_seconds);
        self.tokens.insert(token.token.clone(), token.clone());
        self.save()?;
        Ok(token)
    }

    pub fn validate_token(&mut self, token_str: &str, peer_id: &str) -> Result<bool, String> {
        let token = self.tokens.get_mut(token_str)
            .ok_or_else(|| "Token not found".to_string())?;

        if !token.is_valid() {
            return Ok(false);
        }

        if let Some(ref expected_peer) = token.peer_id {
            if expected_peer != peer_id {
                return Ok(false);
            }
        }

        token.mark_used(peer_id);
        self.save()?;
        Ok(true)
    }

    pub fn get_token(&self, token_str: &str) -> Option<&AuthToken> {
        self.tokens.get(token_str)
    }

    pub fn revoke_token(&mut self, token_str: &str) -> Result<(), String> {
        self.tokens.remove(token_str)
            .ok_or_else(|| "Token not found".to_string())?;
        self.save()
    }

    pub fn cleanup_expired(&mut self) {
        self.tokens.retain(|_, t| !t.is_expired());
    }

    pub fn get_active_tokens(&self) -> Vec<&AuthToken> {
        self.tokens.values()
            .filter(|t| t.is_valid())
            .collect()
    }

    pub fn get_used_tokens(&self) -> Vec<&AuthToken> {
        self.tokens.values()
            .filter(|t| t.used)
            .collect()
    }

    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    pub fn active_token_count(&self) -> usize {
        self.tokens.values().filter(|t| t.is_valid()).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub challenge: String,
    pub peer_id: String,
    pub created_at: i64,
    pub expires_at: i64,
}

impl AuthChallenge {
    pub fn new(peer_id: &str, expires_in_seconds: i64) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        let challenge = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes);

        let now = chrono::Utc::now().timestamp();
        Self {
            challenge,
            peer_id: peer_id.to_string(),
            created_at: now,
            expires_at: now + expires_in_seconds,
        }
    }

    pub fn is_valid(&self) -> bool {
        chrono::Utc::now().timestamp() < self.expires_at
    }

    pub fn verify_response(&self, response: &str, shared_secret: &str) -> bool {
        let expected = self.compute_response(shared_secret);
        response == expected
    }

    pub fn compute_response(&self, shared_secret: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.challenge.as_bytes());
        hasher.update(shared_secret.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

pub const DEFAULT_TOKEN_EXPIRY: i64 = 300;
pub const DEFAULT_CHALLENGE_EXPIRY: i64 = 60;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_creation() {
        let token = AuthToken::new(300);
        assert!(token.is_valid());
        assert!(!token.used);
    }

    #[test]
    fn test_auth_token_expiry() {
        let token = AuthToken::new(-1);
        assert!(!token.is_valid());
        assert!(token.is_expired());
    }

    #[test]
    fn test_auth_token_mark_used() {
        let mut token = AuthToken::new(300);
        token.mark_used("peer-123");
        assert!(token.used);
        assert_eq!(token.used_by, Some("peer-123".to_string()));
        assert!(!token.is_valid());
    }

    #[test]
    fn test_auth_challenge() {
        let challenge = AuthChallenge::new("peer-123", 60);
        assert!(challenge.is_valid());

        let secret = "shared-secret";
        let response = challenge.compute_response(secret);
        assert!(challenge.verify_response(&response, secret));
        assert!(!challenge.verify_response("wrong", secret));
    }

    #[test]
    fn test_token_store() {
        let path = std::path::PathBuf::from("/tmp/test_tokens.json");
        let mut store = AuthTokenStore::new(path);

        let token = store.create_token(300).unwrap();
        assert!(store.get_token(&token.token).is_some());
        assert_eq!(store.active_token_count(), 1);
    }
}
