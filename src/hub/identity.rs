//! Hub Identity Management
//!
//! Provides persistent, machine-based identity for SENA Hub instances
//! Enables cross-hub discovery and communication

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::network::protocol::DEFAULT_PORT;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubIdentity {
    pub hub_id: String,
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub created_at: u64,
    pub version: String,
}

impl HubIdentity {
    pub fn load_or_create(identity_file: &PathBuf) -> Result<Self, String> {
        if identity_file.exists() {
            Self::load(identity_file)
        } else {
            let identity = Self::create_new();
            identity.save(identity_file)?;
            Ok(identity)
        }
    }

    pub fn create_new() -> Self {
        let hostname = Self::get_hostname();
        let hub_id = uuid::Uuid::new_v4().to_string();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            hub_id,
            name: hostname.clone(),
            hostname,
            port: DEFAULT_PORT,
            created_at: timestamp,
            version: crate::VERSION.to_string(),
        }
    }

    pub fn load(file_path: &PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Cannot read identity file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse identity file: {}", e))
    }

    pub fn save(&self, file_path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create identity directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Cannot serialize identity: {}", e))?;

        fs::write(file_path, json)
            .map_err(|e| format!("Cannot write identity file: {}", e))
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn short_id(&self) -> String {
        self.hub_id.chars().take(8).collect()
    }

    fn get_hostname() -> String {
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "SENA Hub".to_string())
    }

    pub fn display_name(&self) -> String {
        if self.name == self.hostname {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.hostname)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredHub {
    pub hub_id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub version: String,
    pub discovered_at: u64,
}

impl DiscoveredHub {
    pub fn new(hub_id: &str, name: &str, address: &str, port: u16, version: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            hub_id: hub_id.to_string(),
            name: name.to_string(),
            address: address.to_string(),
            port,
            version: version.to_string(),
            discovered_at: timestamp,
        }
    }

    pub fn socket_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub request_id: String,
    pub from_hub_id: String,
    pub from_hub_name: String,
    pub from_address: String,
    pub from_port: u16,
    pub message: Option<String>,
    pub created_at: u64,
    pub expires_at: u64,
}

impl ConnectionRequest {
    pub fn new(from_hub: &HubIdentity, from_address: &str, message: Option<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            from_hub_id: from_hub.hub_id.clone(),
            from_hub_name: from_hub.name.clone(),
            from_address: from_address.to_string(),
            from_port: from_hub.port,
            message,
            created_at: timestamp,
            expires_at: timestamp + 300,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        now > self.expires_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedHub {
    pub hub_id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub auth_token: String,
    pub connected_at: u64,
    pub last_seen: u64,
    pub session_count: usize,
}

impl ConnectedHub {
    pub fn new(hub_id: &str, name: &str, address: &str, port: u16, auth_token: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            hub_id: hub_id.to_string(),
            name: name.to_string(),
            address: address.to_string(),
            port,
            auth_token: auth_token.to_string(),
            connected_at: timestamp,
            last_seen: timestamp,
            session_count: 0,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    pub fn is_online(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        now.saturating_sub(self.last_seen) < 60
    }

    pub fn socket_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn connected_duration(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let secs = now.saturating_sub(self.connected_at);

        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else if secs < 86400 {
            format!("{}h", secs / 3600)
        } else {
            format!("{}d", secs / 86400)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_identity_creation() {
        let identity = HubIdentity::create_new();
        assert!(!identity.hub_id.is_empty());
        assert!(!identity.name.is_empty());
        assert_eq!(identity.port, DEFAULT_PORT);
    }

    #[test]
    fn test_short_id() {
        let identity = HubIdentity::create_new();
        assert_eq!(identity.short_id().len(), 8);
    }

    #[test]
    fn test_connection_request_expiry() {
        let identity = HubIdentity::create_new();
        let request = ConnectionRequest::new(&identity, "192.168.1.100", None);
        assert!(!request.is_expired());
    }

    #[test]
    fn test_connected_hub_online_status() {
        let hub = ConnectedHub::new("test-id", "Test Hub", "192.168.1.100", 9876, "token123");
        assert!(hub.is_online());
    }
}
