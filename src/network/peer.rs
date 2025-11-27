use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub authorized: bool,
    pub auth_token: Option<String>,
    pub public_key: Option<String>,
    pub last_seen: i64,
    pub created_at: i64,
}

impl Peer {
    pub fn new(id: &str, name: &str, address: &str, port: u16) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            address: address.to_string(),
            port,
            authorized: false,
            auth_token: None,
            public_key: None,
            last_seen: now,
            created_at: now,
        }
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.address, self.port)
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now().timestamp();
    }

    pub fn authorize(&mut self, token: &str) {
        self.authorized = true;
        self.auth_token = Some(token.to_string());
        self.update_last_seen();
    }

    pub fn revoke(&mut self) {
        self.authorized = false;
        self.auth_token = None;
    }

    pub fn is_online(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.last_seen < 300
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerRegistry {
    pub local_peer_id: String,
    pub local_peer_name: String,
    pub peers: HashMap<String, Peer>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl PeerRegistry {
    pub fn new(file_path: PathBuf) -> Self {
        let local_peer_id = Self::generate_peer_id();
        let local_peer_name = Self::get_default_peer_name();

        Self {
            local_peer_id,
            local_peer_name,
            peers: HashMap::new(),
            file_path,
        }
    }

    fn get_default_peer_name() -> String {
        let config = crate::config::SenaConfig::global();
        let user_prefix = &config.user.prefix;

        if user_prefix != "SENA" {
            format!("{} Instance", user_prefix)
        } else {
            whoami::fallible::hostname().unwrap_or_else(|_| "SENA Instance".to_string())
        }
    }

    pub fn load(file_path: PathBuf) -> Result<Self, String> {
        if !file_path.exists() {
            let mut registry = Self::new(file_path.clone());
            registry.file_path = file_path;
            return Ok(registry);
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read peer registry: {}", e))?;

        let mut registry: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse peer registry: {}", e))?;

        registry.file_path = file_path;
        Ok(registry)
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize peer registry: {}", e))?;

        fs::write(&self.file_path, content)
            .map_err(|e| format!("Failed to write peer registry: {}", e))
    }

    fn generate_peer_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub fn add_peer(&mut self, peer: Peer) -> Result<(), String> {
        if self.peers.contains_key(&peer.id) {
            return Err(format!("Peer {} already exists", peer.id));
        }
        self.peers.insert(peer.id.clone(), peer);
        self.save()
    }

    pub fn update_peer(&mut self, peer: Peer) -> Result<(), String> {
        self.peers.insert(peer.id.clone(), peer);
        self.save()
    }

    pub fn remove_peer(&mut self, peer_id: &str) -> Result<(), String> {
        self.peers
            .remove(peer_id)
            .ok_or_else(|| format!("Peer {} not found", peer_id))?;
        self.save()
    }

    pub fn get_peer(&self, peer_id: &str) -> Option<&Peer> {
        self.peers.get(peer_id)
    }

    pub fn get_peer_mut(&mut self, peer_id: &str) -> Option<&mut Peer> {
        self.peers.get_mut(peer_id)
    }

    pub fn get_peer_by_address(&self, address: &str, port: u16) -> Option<&Peer> {
        self.peers
            .values()
            .find(|p| p.address == address && p.port == port)
    }

    pub fn get_authorized_peers(&self) -> Vec<&Peer> {
        self.peers.values().filter(|p| p.authorized).collect()
    }

    pub fn get_online_peers(&self) -> Vec<&Peer> {
        self.peers
            .values()
            .filter(|p| p.authorized && p.is_online())
            .collect()
    }

    pub fn get_all_peers(&self) -> Vec<&Peer> {
        self.peers.values().collect()
    }

    pub fn authorize_peer(&mut self, peer_id: &str, token: &str) -> Result<(), String> {
        let peer = self
            .peers
            .get_mut(peer_id)
            .ok_or_else(|| format!("Peer {} not found", peer_id))?;
        peer.authorize(token);
        self.save()
    }

    pub fn revoke_peer(&mut self, peer_id: &str) -> Result<(), String> {
        let peer = self
            .peers
            .get_mut(peer_id)
            .ok_or_else(|| format!("Peer {} not found", peer_id))?;
        peer.revoke();
        self.save()
    }

    pub fn update_peer_last_seen(&mut self, peer_id: &str) -> Result<(), String> {
        let peer = self
            .peers
            .get_mut(peer_id)
            .ok_or_else(|| format!("Peer {} not found", peer_id))?;
        peer.update_last_seen();
        self.save()
    }

    pub fn generate_auth_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes)
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn authorized_count(&self) -> usize {
        self.peers.values().filter(|p| p.authorized).count()
    }

    pub fn set_local_peer_name(&mut self, name: &str) -> Result<(), String> {
        self.local_peer_name = name.to_string();
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_peer_creation() {
        let peer = Peer::new("test-id", "Test Peer", "192.168.1.100", 9876);
        assert_eq!(peer.id, "test-id");
        assert_eq!(peer.name, "Test Peer");
        assert!(!peer.authorized);
    }

    #[test]
    fn test_peer_authorization() {
        let mut peer = Peer::new("test-id", "Test Peer", "192.168.1.100", 9876);
        peer.authorize("test-token");
        assert!(peer.authorized);
        assert_eq!(peer.auth_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_peer_registry() {
        let path = temp_dir().join("test_peers.json");
        let mut registry = PeerRegistry::new(path);

        let peer = Peer::new("peer1", "Test", "192.168.1.1", 9876);
        registry.add_peer(peer).unwrap();

        assert_eq!(registry.peer_count(), 1);
        assert!(registry.get_peer("peer1").is_some());
    }

    #[test]
    fn test_auth_token_generation() {
        let token1 = PeerRegistry::generate_auth_token();
        let token2 = PeerRegistry::generate_auth_token();
        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
    }
}
