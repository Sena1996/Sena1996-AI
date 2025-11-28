//! Peer Manager for Cross-Hub Communication
//!
//! Manages discovered hubs, connection requests, and trusted peers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::identity::{ConnectedHub, ConnectionRequest, DiscoveredHub, HubIdentity};
use super::session::Session;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSession {
    pub hub_id: String,
    pub hub_name: String,
    pub session_id: String,
    pub session_name: String,
    pub role: String,
    pub status: String,
    pub working_on: Option<String>,
    pub working_directory: String,
}

impl RemoteSession {
    pub fn full_address(&self) -> String {
        format!("{}:{}", self.hub_name, self.session_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PeerManagerData {
    version: String,
    connected_hubs: HashMap<String, ConnectedHub>,
    pending_requests: Vec<ConnectionRequest>,
    remote_sessions: HashMap<String, Vec<RemoteSession>>,
    last_updated: u64,
}

pub struct PeerManager {
    identity: HubIdentity,
    connected_hubs: HashMap<String, ConnectedHub>,
    pending_requests: Vec<ConnectionRequest>,
    remote_sessions: HashMap<String, Vec<RemoteSession>>,
    discovered_hubs: Vec<DiscoveredHub>,
    peers_file: PathBuf,
}

impl PeerManager {
    pub fn new(identity: HubIdentity, hub_dir: &Path) -> Self {
        Self {
            identity,
            connected_hubs: HashMap::new(),
            pending_requests: Vec::new(),
            remote_sessions: HashMap::new(),
            discovered_hubs: Vec::new(),
            peers_file: hub_dir.join("peers.json"),
        }
    }

    pub fn identity(&self) -> &HubIdentity {
        &self.identity
    }

    pub fn set_hub_name(&mut self, name: &str) -> Result<(), String> {
        self.identity.set_name(name);
        let identity_file = self.peers_file.parent()
            .map(|p| p.join("identity.json"))
            .ok_or("Cannot determine identity file path")?;
        self.identity.save(&identity_file)?;
        Ok(())
    }

    pub fn add_discovered_hub(&mut self, hub: DiscoveredHub) {
        if hub.hub_id == self.identity.hub_id {
            return;
        }

        if let Some(existing) = self.discovered_hubs.iter_mut()
            .find(|h| h.hub_id == hub.hub_id)
        {
            existing.address = hub.address;
            existing.port = hub.port;
            existing.discovered_at = hub.discovered_at;
        } else {
            self.discovered_hubs.push(hub);
        }
    }

    pub fn get_discovered_hubs(&self) -> Vec<&DiscoveredHub> {
        self.discovered_hubs.iter()
            .filter(|h| !self.connected_hubs.contains_key(&h.hub_id))
            .collect()
    }

    pub fn clear_discovered(&mut self) {
        self.discovered_hubs.clear();
    }

    pub fn create_connection_request(&self, message: Option<String>) -> ConnectionRequest {
        ConnectionRequest::new(&self.identity, "0.0.0.0", message)
    }

    pub fn add_pending_request(&mut self, request: ConnectionRequest) -> Result<(), String> {
        if request.from_hub_id == self.identity.hub_id {
            return Err("Cannot add request from self".to_string());
        }

        self.pending_requests.retain(|r| r.from_hub_id != request.from_hub_id);
        self.pending_requests.push(request);
        self.cleanup_expired_requests();
        self.save()
    }

    pub fn get_pending_requests(&self) -> Vec<&ConnectionRequest> {
        self.pending_requests.iter()
            .filter(|r| !r.is_expired())
            .collect()
    }

    pub fn approve_request(&mut self, request_id: &str) -> Result<ConnectedHub, String> {
        let request = self.pending_requests.iter()
            .find(|r| r.request_id == request_id)
            .ok_or_else(|| format!("Request {} not found", request_id))?
            .clone();

        if request.is_expired() {
            self.pending_requests.retain(|r| r.request_id != request_id);
            return Err("Request has expired".to_string());
        }

        let auth_token = Self::generate_auth_token();
        let connected_hub = ConnectedHub::new(
            &request.from_hub_id,
            &request.from_hub_name,
            &request.from_address,
            request.from_port,
            &auth_token,
        );

        self.connected_hubs.insert(request.from_hub_id.clone(), connected_hub.clone());
        self.pending_requests.retain(|r| r.request_id != request_id);
        self.save()?;

        Ok(connected_hub)
    }

    pub fn reject_request(&mut self, request_id: &str) -> Result<(), String> {
        let exists = self.pending_requests.iter().any(|r| r.request_id == request_id);
        if !exists {
            return Err(format!("Request {} not found", request_id));
        }

        self.pending_requests.retain(|r| r.request_id != request_id);
        self.save()
    }

    pub fn add_connected_hub(&mut self, hub: ConnectedHub) -> Result<(), String> {
        self.connected_hubs.insert(hub.hub_id.clone(), hub);
        self.save()
    }

    pub fn get_connected_hubs(&self) -> Vec<&ConnectedHub> {
        self.connected_hubs.values().collect()
    }

    pub fn get_connected_hub(&self, hub_id: &str) -> Option<&ConnectedHub> {
        self.connected_hubs.get(hub_id)
    }

    pub fn get_hub_by_name(&self, name: &str) -> Option<&ConnectedHub> {
        let name_lower = name.to_lowercase();
        self.connected_hubs.values()
            .find(|h| h.name.to_lowercase() == name_lower)
    }

    pub fn disconnect_hub(&mut self, hub_id: &str) -> Result<(), String> {
        self.connected_hubs.remove(hub_id)
            .ok_or_else(|| format!("Hub {} not connected", hub_id))?;
        self.remote_sessions.remove(hub_id);
        self.save()
    }

    pub fn update_hub_last_seen(&mut self, hub_id: &str) -> Result<(), String> {
        let hub = self.connected_hubs.get_mut(hub_id)
            .ok_or_else(|| format!("Hub {} not connected", hub_id))?;
        hub.update_last_seen();
        self.save()
    }

    pub fn update_remote_sessions(&mut self, hub_id: &str, sessions: Vec<RemoteSession>) {
        if let Some(hub) = self.connected_hubs.get_mut(hub_id) {
            hub.session_count = sessions.len();
        }
        self.remote_sessions.insert(hub_id.to_string(), sessions);
    }

    pub fn get_remote_sessions(&self, hub_id: Option<&str>) -> Vec<&RemoteSession> {
        match hub_id {
            Some(id) => self.remote_sessions.get(id)
                .map(|s| s.iter().collect())
                .unwrap_or_default(),
            None => self.remote_sessions.values()
                .flat_map(|s| s.iter())
                .collect(),
        }
    }

    pub fn get_all_sessions(&self, local_sessions: &[Session]) -> Vec<FederatedSession> {
        let mut all_sessions = Vec::new();

        for session in local_sessions {
            all_sessions.push(FederatedSession {
                hub_id: self.identity.hub_id.clone(),
                hub_name: self.identity.name.clone(),
                session_id: session.id.clone(),
                session_name: session.name.clone(),
                role: session.role.name().to_string(),
                status: format!("{:?}", session.status),
                working_on: session.working_on.clone(),
                working_directory: session.working_directory.clone().unwrap_or_default(),
                is_local: true,
            });
        }

        for remote in self.remote_sessions.values().flat_map(|s| s.iter()) {
            all_sessions.push(FederatedSession {
                hub_id: remote.hub_id.clone(),
                hub_name: remote.hub_name.clone(),
                session_id: remote.session_id.clone(),
                session_name: remote.session_name.clone(),
                role: remote.role.clone(),
                status: remote.status.clone(),
                working_on: remote.working_on.clone(),
                working_directory: remote.working_directory.clone(),
                is_local: false,
            });
        }

        all_sessions
    }

    pub fn resolve_session(&self, target: &str, local_sessions: &[Session]) -> Option<ResolvedTarget> {
        if let Some((hub_name, session_name)) = target.split_once(':') {
            if hub_name.to_lowercase() == self.identity.name.to_lowercase() {
                return self.resolve_local_session(session_name, local_sessions);
            }

            if let Some(hub) = self.get_hub_by_name(hub_name) {
                if let Some(sessions) = self.remote_sessions.get(&hub.hub_id) {
                    let session_lower = session_name.to_lowercase();
                    if let Some(session) = sessions.iter()
                        .find(|s| s.session_name.to_lowercase() == session_lower)
                    {
                        return Some(ResolvedTarget::Remote {
                            hub_id: hub.hub_id.clone(),
                            hub_name: hub.name.clone(),
                            hub_address: hub.socket_address(),
                            session_id: session.session_id.clone(),
                            session_name: session.session_name.clone(),
                        });
                    }
                }
            }
            return None;
        }

        self.resolve_local_session(target, local_sessions)
    }

    fn resolve_local_session(&self, name: &str, local_sessions: &[Session]) -> Option<ResolvedTarget> {
        let name_lower = name.to_lowercase();
        local_sessions.iter()
            .find(|s| s.name.to_lowercase() == name_lower || s.id == name)
            .map(|s| ResolvedTarget::Local {
                session_id: s.id.clone(),
                session_name: s.name.clone(),
            })
    }

    fn cleanup_expired_requests(&mut self) {
        self.pending_requests.retain(|r| !r.is_expired());
    }

    fn generate_auth_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes)
    }

    pub fn save(&self) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let data = PeerManagerData {
            version: crate::VERSION.to_string(),
            connected_hubs: self.connected_hubs.clone(),
            pending_requests: self.pending_requests.clone(),
            remote_sessions: self.remote_sessions.clone(),
            last_updated: timestamp,
        };

        if let Some(parent) = self.peers_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create peers directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Cannot serialize peers: {}", e))?;

        fs::write(&self.peers_file, json)
            .map_err(|e| format!("Cannot write peers file: {}", e))
    }

    pub fn load(&mut self) -> Result<(), String> {
        if !self.peers_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.peers_file)
            .map_err(|e| format!("Cannot read peers file: {}", e))?;

        let data: PeerManagerData = serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse peers file: {}", e))?;

        self.connected_hubs = data.connected_hubs;
        self.pending_requests = data.pending_requests;
        self.remote_sessions = data.remote_sessions;

        self.cleanup_expired_requests();

        Ok(())
    }

    pub fn connected_count(&self) -> usize {
        self.connected_hubs.len()
    }

    pub fn online_count(&self) -> usize {
        self.connected_hubs.values().filter(|h| h.is_online()).count()
    }

    pub fn pending_count(&self) -> usize {
        self.get_pending_requests().len()
    }

    pub fn remote_session_count(&self) -> usize {
        self.remote_sessions.values().map(|s| s.len()).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedSession {
    pub hub_id: String,
    pub hub_name: String,
    pub session_id: String,
    pub session_name: String,
    pub role: String,
    pub status: String,
    pub working_on: Option<String>,
    pub working_directory: String,
    pub is_local: bool,
}

impl FederatedSession {
    pub fn full_address(&self) -> String {
        format!("{}:{}", self.hub_name, self.session_name)
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedTarget {
    Local {
        session_id: String,
        session_name: String,
    },
    Remote {
        hub_id: String,
        hub_name: String,
        hub_address: String,
        session_id: String,
        session_name: String,
    },
    LocalBroadcast,
    RemoteBroadcast {
        hub_id: String,
        hub_name: String,
        hub_address: String,
    },
    GlobalBroadcast,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    fn create_test_identity() -> HubIdentity {
        HubIdentity::create_new()
    }

    #[test]
    fn test_peer_manager_creation() {
        let identity = create_test_identity();
        let hub_dir = temp_dir().join("test_hub");
        let manager = PeerManager::new(identity.clone(), &hub_dir);
        assert_eq!(manager.identity().hub_id, identity.hub_id);
    }

    #[test]
    fn test_discovered_hub_deduplication() {
        let identity = create_test_identity();
        let hub_dir = temp_dir().join("test_hub2");
        let mut manager = PeerManager::new(identity, &hub_dir);

        let hub = DiscoveredHub::new("hub1", "Test Hub", "192.168.1.100", 9876, "1.0");
        manager.add_discovered_hub(hub.clone());
        manager.add_discovered_hub(hub);

        assert_eq!(manager.get_discovered_hubs().len(), 1);
    }

    #[test]
    fn test_resolve_local_session() {
        let identity = create_test_identity();
        let hub_dir = temp_dir().join("test_hub3");
        let manager = PeerManager::new(identity, &hub_dir);

        let sessions = vec![
            Session::new(crate::hub::session::SessionRole::Android, Some("Android".to_string())),
        ];

        let resolved = manager.resolve_session("Android", &sessions);
        assert!(matches!(resolved, Some(ResolvedTarget::Local { .. })));
    }
}
