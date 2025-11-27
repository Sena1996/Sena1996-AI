use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::protocol::MDNS_SERVICE_TYPE;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub peer_name: String,
    pub address: String,
    pub port: u16,
    pub discovered_at: i64,
}

pub struct NetworkDiscovery {
    service_daemon: Option<ServiceDaemon>,
    discovered_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    local_peer_id: String,
    local_peer_name: String,
    port: u16,
    running: Arc<RwLock<bool>>,
}

impl NetworkDiscovery {
    pub fn new(local_peer_id: String, local_peer_name: String, port: u16) -> Self {
        Self {
            service_daemon: None,
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            local_peer_id,
            local_peer_name,
            port,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let daemon =
            ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

        self.register_service(&daemon)?;
        self.start_browsing(&daemon)?;

        self.service_daemon = Some(daemon);
        let running = self.running.clone();
        tokio::spawn(async move {
            *running.write().await = true;
        });

        Ok(())
    }

    fn register_service(&self, daemon: &ServiceDaemon) -> Result<(), String> {
        let host_name = format!(
            "{}.local.",
            self.local_peer_name.replace(' ', "-").to_lowercase()
        );
        let instance_name = format!("sena-{}", &self.local_peer_id[..8]);

        let mut properties = HashMap::new();
        properties.insert("peer_id".to_string(), self.local_peer_id.clone());
        properties.insert("peer_name".to_string(), self.local_peer_name.clone());
        properties.insert("version".to_string(), crate::VERSION.to_string());

        let service_info = ServiceInfo::new(
            MDNS_SERVICE_TYPE,
            &instance_name,
            &host_name,
            "",
            self.port,
            properties,
        )
        .map_err(|e| format!("Failed to create service info: {}", e))?;

        daemon
            .register(service_info)
            .map_err(|e| format!("Failed to register service: {}", e))?;

        Ok(())
    }

    fn start_browsing(&self, daemon: &ServiceDaemon) -> Result<(), String> {
        let receiver = daemon
            .browse(MDNS_SERVICE_TYPE)
            .map_err(|e| format!("Failed to start browsing: {}", e))?;

        let discovered_peers = self.discovered_peers.clone();
        let local_peer_id = self.local_peer_id.clone();
        let _running = self.running.clone();

        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        if let Some(peer_id) = info.get_properties().get("peer_id") {
                            let peer_id_str = peer_id.val_str();
                            if peer_id_str != local_peer_id {
                                let peer = DiscoveredPeer {
                                    peer_id: peer_id_str.to_string(),
                                    peer_name: info
                                        .get_properties()
                                        .get("peer_name")
                                        .map(|p| p.val_str().to_string())
                                        .unwrap_or_else(|| "Unknown".to_string()),
                                    address: info
                                        .get_addresses()
                                        .iter()
                                        .next()
                                        .map(|a| a.to_string())
                                        .unwrap_or_default(),
                                    port: info.get_port(),
                                    discovered_at: chrono::Utc::now().timestamp(),
                                };

                                let peers = discovered_peers.clone();
                                let peer_clone = peer.clone();
                                tokio::spawn(async move {
                                    peers
                                        .write()
                                        .await
                                        .insert(peer_clone.peer_id.clone(), peer_clone);
                                });
                            }
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, full_name) => {
                        let peers = discovered_peers.clone();
                        tokio::spawn(async move {
                            let mut peers = peers.write().await;
                            peers.retain(|_, p| !full_name.contains(&p.peer_id[..8]));
                        });
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(daemon) = self.service_daemon.take() {
            let _ = daemon.shutdown();
        }
        let running = self.running.clone();
        tokio::spawn(async move {
            *running.write().await = false;
        });
    }

    pub async fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    pub async fn get_peer(&self, peer_id: &str) -> Option<DiscoveredPeer> {
        self.discovered_peers.read().await.get(peer_id).cloned()
    }

    pub async fn clear_stale_peers(&self, max_age_seconds: i64) {
        let now = chrono::Utc::now().timestamp();
        self.discovered_peers
            .write()
            .await
            .retain(|_, p| now - p.discovered_at < max_age_seconds);
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn peer_count(&self) -> usize {
        self.discovered_peers.read().await.len()
    }
}

pub async fn discover_once(timeout_secs: u64) -> Result<Vec<DiscoveredPeer>, String> {
    let daemon =
        ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

    let receiver = daemon
        .browse(MDNS_SERVICE_TYPE)
        .map_err(|e| format!("Failed to start browsing: {}", e))?;

    let mut peers = HashMap::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);

    while std::time::Instant::now() < deadline {
        if let Ok(ServiceEvent::ServiceResolved(info)) =
            receiver.recv_timeout(Duration::from_millis(100))
        {
            if let Some(peer_id) = info.get_properties().get("peer_id") {
                let peer = DiscoveredPeer {
                    peer_id: peer_id.val_str().to_string(),
                    peer_name: info
                        .get_properties()
                        .get("peer_name")
                        .map(|p| p.val_str().to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    address: info
                        .get_addresses()
                        .iter()
                        .next()
                        .map(|a| a.to_string())
                        .unwrap_or_default(),
                    port: info.get_port(),
                    discovered_at: chrono::Utc::now().timestamp(),
                };
                peers.insert(peer.peer_id.clone(), peer);
            }
        }
    }

    let _ = daemon.shutdown();
    Ok(peers.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_peer_creation() {
        let peer = DiscoveredPeer {
            peer_id: "test-id".to_string(),
            peer_name: "Test".to_string(),
            address: "192.168.1.1".to_string(),
            port: 9876,
            discovered_at: chrono::Utc::now().timestamp(),
        };
        assert_eq!(peer.peer_id, "test-id");
    }

    #[test]
    fn test_network_discovery_creation() {
        let discovery =
            NetworkDiscovery::new("peer-123".to_string(), "Test Peer".to_string(), 9876);
        assert_eq!(discovery.port, 9876);
    }
}
