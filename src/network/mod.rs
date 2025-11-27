pub mod protocol;
pub mod peer;
pub mod tcp;
pub mod discovery;
pub mod auth;
pub mod tls;

pub use protocol::{NetworkCommand, NetworkMessage, RemoteSession, SharedPath, DEFAULT_PORT, MDNS_SERVICE_TYPE, PROTOCOL_VERSION};
pub use peer::{Peer, PeerRegistry};
pub use tcp::{NetworkServer, NetworkClient, ClientConnection, Connection, ConnectionId};
pub use discovery::{NetworkDiscovery, DiscoveredPeer, discover_once};
pub use auth::{AuthToken, AuthTokenStore, AuthChallenge, DEFAULT_TOKEN_EXPIRY};
pub use tls::{TlsConfig, ensure_certificates};

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub enabled: bool,
    pub port: u16,
    pub auto_start: bool,
    pub discovery_enabled: bool,
    pub tls_enabled: bool,
    pub max_connections: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: DEFAULT_PORT,
            auto_start: false,
            discovery_enabled: true,
            tls_enabled: true,
            max_connections: 50,
        }
    }
}

pub struct NetworkManager {
    config: NetworkConfig,
    #[allow(dead_code)]
    data_dir: PathBuf,
    peer_registry: Arc<RwLock<PeerRegistry>>,
    token_store: Arc<RwLock<AuthTokenStore>>,
    server: Option<Arc<NetworkServer>>,
    discovery: Option<Arc<RwLock<NetworkDiscovery>>>,
    tls_config: TlsConfig,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig, data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let peer_registry = Arc::new(RwLock::new(
            PeerRegistry::load(data_dir.join("peers.json"))?
        ));

        let token_store = Arc::new(RwLock::new(
            AuthTokenStore::load(data_dir.join("tokens.json"))?
        ));

        let tls_config = TlsConfig::new(data_dir.join("tls"));

        Ok(Self {
            config,
            data_dir,
            peer_registry,
            token_store,
            server: None,
            discovery: None,
            tls_config,
        })
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let registry = self.peer_registry.read().await;
        let peer_name = registry.local_peer_name.clone();
        let peer_id = registry.local_peer_id.clone();
        drop(registry);

        if self.config.tls_enabled {
            ensure_certificates(&self.tls_config, &peer_name)?;
        }

        let server = Arc::new(NetworkServer::new(self.config.port, self.peer_registry.clone()));
        server.start().await?;
        self.server = Some(server);

        if self.config.discovery_enabled {
            let mut discovery = NetworkDiscovery::new(peer_id, peer_name, self.config.port);
            discovery.start()?;
            self.discovery = Some(Arc::new(RwLock::new(discovery)));
        }

        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(ref server) = self.server {
            server.stop().await;
        }
        self.server = None;

        if let Some(ref discovery) = self.discovery {
            discovery.write().await.stop();
        }
        self.discovery = None;
    }

    pub async fn is_running(&self) -> bool {
        if let Some(ref server) = self.server {
            server.is_running().await
        } else {
            false
        }
    }

    pub async fn status(&self) -> NetworkStatus {
        let running = self.is_running().await;
        let peer_count = self.peer_registry.read().await.peer_count();
        let authorized_count = self.peer_registry.read().await.authorized_count();

        let discovered_count = if let Some(ref discovery) = self.discovery {
            discovery.read().await.peer_count().await
        } else {
            0
        };

        let connection_count = if let Some(ref server) = self.server {
            server.get_connections().await.len()
        } else {
            0
        };

        NetworkStatus {
            running,
            port: self.config.port,
            peer_count,
            authorized_count,
            discovered_count,
            connection_count,
            tls_enabled: self.config.tls_enabled,
            discovery_enabled: self.config.discovery_enabled,
        }
    }

    pub async fn discover_peers(&self, timeout_secs: u64) -> Result<Vec<DiscoveredPeer>, String> {
        discover_once(timeout_secs).await
    }

    pub async fn add_peer(&self, address: &str, port: u16, name: Option<&str>) -> Result<Peer, String> {
        let peer_id = uuid::Uuid::new_v4().to_string();
        let peer_name = name.unwrap_or("Unknown").to_string();
        let peer = Peer::new(&peer_id, &peer_name, address, port);

        self.peer_registry.write().await.add_peer(peer.clone())?;
        Ok(peer)
    }

    pub async fn remove_peer(&self, peer_id: &str) -> Result<(), String> {
        self.peer_registry.write().await.remove_peer(peer_id)
    }

    pub async fn get_peers(&self) -> Vec<Peer> {
        self.peer_registry.read().await.get_all_peers().into_iter().cloned().collect()
    }

    pub async fn get_authorized_peers(&self) -> Vec<Peer> {
        self.peer_registry.read().await.get_authorized_peers().into_iter().cloned().collect()
    }

    pub async fn create_auth_token(&self, peer_id: Option<&str>, expires_in: i64) -> Result<AuthToken, String> {
        let mut store = self.token_store.write().await;
        if let Some(id) = peer_id {
            store.create_token_for_peer(id, expires_in)
        } else {
            store.create_token(expires_in)
        }
    }

    pub async fn validate_token(&self, token: &str, peer_id: &str) -> Result<bool, String> {
        self.token_store.write().await.validate_token(token, peer_id)
    }

    pub async fn authorize_peer(&self, peer_id: &str) -> Result<AuthToken, String> {
        let token = self.create_auth_token(Some(peer_id), DEFAULT_TOKEN_EXPIRY).await?;
        self.peer_registry.write().await.authorize_peer(peer_id, &token.token)?;
        Ok(token)
    }

    pub async fn connect_to_peer(&self, address: &str, port: u16) -> Result<ClientConnection, String> {
        let client = NetworkClient::new(self.peer_registry.clone());
        client.connect(address, port).await
    }

    pub async fn connect_and_auth(&self, address: &str, port: u16, token: &str) -> Result<ClientConnection, String> {
        let client = NetworkClient::new(self.peer_registry.clone());
        client.connect_and_auth(address, port, token).await
    }

    pub async fn get_all_sessions(&self) -> Vec<RemoteSession> {
        if let Some(ref server) = self.server {
            server.get_all_sessions().await
        } else {
            Vec::new()
        }
    }

    pub async fn announce_session(&self, session_id: &str, session_name: &str, role: &str, working_dir: &str) {
        if let Some(ref server) = self.server {
            let registry = self.peer_registry.read().await;
            let session = RemoteSession {
                peer_id: registry.local_peer_id.clone(),
                peer_name: registry.local_peer_name.clone(),
                peer_addr: format!("localhost:{}", self.config.port),
                session_id: session_id.to_string(),
                session_name: session_name.to_string(),
                role: role.to_string(),
                working_dir: working_dir.to_string(),
                last_seen: chrono::Utc::now().timestamp(),
            };
            server.add_local_session(session).await;
        }
    }

    pub async fn end_session(&self, session_id: &str) {
        if let Some(ref server) = self.server {
            server.remove_local_session(session_id).await;
        }
    }

    pub async fn broadcast_message(&self, content: &str, from_session: &str) {
        if let Some(ref server) = self.server {
            let registry = self.peer_registry.read().await;
            let msg = NetworkMessage::broadcast(&registry.local_peer_id, from_session, content);
            server.broadcast(msg).await;
        }
    }

    pub async fn get_local_peer_id(&self) -> String {
        self.peer_registry.read().await.local_peer_id.clone()
    }

    pub async fn get_local_peer_name(&self) -> String {
        self.peer_registry.read().await.local_peer_name.clone()
    }

    pub async fn set_local_peer_name(&self, name: &str) -> Result<(), String> {
        self.peer_registry.write().await.set_local_peer_name(name)
    }

    pub fn get_certificate_fingerprint(&self) -> Result<String, String> {
        self.tls_config.get_certificate_fingerprint()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub running: bool,
    pub port: u16,
    pub peer_count: usize,
    pub authorized_count: usize,
    pub discovered_count: usize,
    pub connection_count: usize,
    pub tls_enabled: bool,
    pub discovery_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_network_manager_creation() {
        let dir = temp_dir().join("sena_network_test");
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config, dir.clone());
        assert!(manager.is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn test_network_status() {
        let dir = temp_dir().join("sena_network_status_test");
        let config = NetworkConfig { enabled: false, ..Default::default() };
        let manager = NetworkManager::new(config, dir.clone()).unwrap();
        let status = manager.status().await;
        assert!(!status.running);
        let _ = std::fs::remove_dir_all(dir);
    }
}
