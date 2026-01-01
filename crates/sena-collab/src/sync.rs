use crate::crdt::{NodeId, VectorClock};
use sena_core::{Result, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

const BROADCAST_CAPACITY: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    Join { session_id: SessionId, node_id: NodeId },
    Leave { session_id: SessionId, node_id: NodeId },
    StateUpdate { session_id: SessionId, clock: VectorClock, payload: Vec<u8> },
    StateRequest { session_id: SessionId, from_clock: VectorClock },
    StateResponse { session_id: SessionId, clock: VectorClock, payload: Vec<u8> },
    Ping { node_id: NodeId },
    Pong { node_id: NodeId },
}

#[derive(Debug, Clone)]
pub struct SyncPeer {
    pub node_id: NodeId,
    pub session_id: SessionId,
    pub clock: VectorClock,
    pub last_seen: std::time::Instant,
}

pub struct SyncServer {
    node_id: NodeId,
    peers: Arc<RwLock<HashMap<NodeId, SyncPeer>>>,
    sessions: Arc<RwLock<HashMap<SessionId, Vec<NodeId>>>>,
    broadcast_tx: broadcast::Sender<SyncMessage>,
    command_tx: mpsc::Sender<SyncCommand>,
}

#[derive(Debug)]
pub enum SyncCommand {
    AddPeer { peer: SyncPeer },
    RemovePeer { node_id: NodeId },
    BroadcastToSession { session_id: SessionId, message: SyncMessage },
    UpdateClock { node_id: NodeId, clock: VectorClock },
}

impl SyncServer {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let (command_tx, command_rx) = mpsc::channel(256);
        let peers = Arc::new(RwLock::new(HashMap::new()));
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let node_id = Uuid::new_v4();

        let server = Self {
            node_id,
            peers: peers.clone(),
            sessions: sessions.clone(),
            broadcast_tx: broadcast_tx.clone(),
            command_tx,
        };

        tokio::spawn(Self::run_command_loop(
            command_rx,
            peers,
            sessions,
            broadcast_tx,
        ));

        server
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SyncMessage> {
        self.broadcast_tx.subscribe()
    }

    pub async fn join_session(&self, session_id: SessionId, node_id: NodeId) -> Result<()> {
        let peer = SyncPeer {
            node_id,
            session_id,
            clock: VectorClock::new(),
            last_seen: std::time::Instant::now(),
        };

        self.command_tx
            .send(SyncCommand::AddPeer { peer })
            .await
            .map_err(|_| sena_core::Error::internal("sync command channel closed"))?;

        self.broadcast(SyncMessage::Join { session_id, node_id }).await
    }

    pub async fn leave_session(&self, session_id: SessionId, node_id: NodeId) -> Result<()> {
        self.command_tx
            .send(SyncCommand::RemovePeer { node_id })
            .await
            .map_err(|_| sena_core::Error::internal("sync command channel closed"))?;

        self.broadcast(SyncMessage::Leave { session_id, node_id }).await
    }

    pub async fn broadcast(&self, message: SyncMessage) -> Result<()> {
        let _ = self.broadcast_tx.send(message);
        Ok(())
    }

    pub async fn broadcast_to_session(
        &self,
        session_id: SessionId,
        message: SyncMessage,
    ) -> Result<()> {
        self.command_tx
            .send(SyncCommand::BroadcastToSession { session_id, message })
            .await
            .map_err(|_| sena_core::Error::internal("sync command channel closed"))
    }

    pub async fn send_state_update(
        &self,
        session_id: SessionId,
        clock: VectorClock,
        payload: Vec<u8>,
    ) -> Result<()> {
        self.broadcast(SyncMessage::StateUpdate {
            session_id,
            clock,
            payload,
        }).await
    }

    pub async fn request_state(&self, session_id: SessionId, from_clock: VectorClock) -> Result<()> {
        self.broadcast(SyncMessage::StateRequest {
            session_id,
            from_clock,
        }).await
    }

    pub async fn peers(&self) -> Vec<SyncPeer> {
        self.peers.read().await.values().cloned().collect()
    }

    pub async fn session_peers(&self, session_id: &SessionId) -> Vec<NodeId> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    async fn run_command_loop(
        mut command_rx: mpsc::Receiver<SyncCommand>,
        peers: Arc<RwLock<HashMap<NodeId, SyncPeer>>>,
        sessions: Arc<RwLock<HashMap<SessionId, Vec<NodeId>>>>,
        broadcast_tx: broadcast::Sender<SyncMessage>,
    ) {
        while let Some(command) = command_rx.recv().await {
            match command {
                SyncCommand::AddPeer { peer } => {
                    let session_id = peer.session_id;
                    let node_id = peer.node_id;
                    peers.write().await.insert(node_id, peer);
                    sessions
                        .write()
                        .await
                        .entry(session_id)
                        .or_default()
                        .push(node_id);
                }
                SyncCommand::RemovePeer { node_id } => {
                    if let Some(peer) = peers.write().await.remove(&node_id) {
                        if let Some(session_peers) =
                            sessions.write().await.get_mut(&peer.session_id)
                        {
                            session_peers.retain(|id| *id != node_id);
                        }
                    }
                }
                SyncCommand::BroadcastToSession { session_id, message } => {
                    let session_nodes = sessions
                        .read()
                        .await
                        .get(&session_id)
                        .cloned()
                        .unwrap_or_default();

                    if !session_nodes.is_empty() {
                        let _ = broadcast_tx.send(message);
                    }
                }
                SyncCommand::UpdateClock { node_id, clock } => {
                    if let Some(peer) = peers.write().await.get_mut(&node_id) {
                        peer.clock.merge(&clock);
                        peer.last_seen = std::time::Instant::now();
                    }
                }
            }
        }
    }
}

impl Default for SyncServer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SyncClient {
    node_id: NodeId,
    session_id: Option<SessionId>,
    clock: VectorClock,
    server: Arc<SyncServer>,
    message_rx: broadcast::Receiver<SyncMessage>,
}

impl SyncClient {
    pub fn new(server: Arc<SyncServer>) -> Self {
        let node_id = Uuid::new_v4();
        let message_rx = server.subscribe();
        Self {
            node_id,
            session_id: None,
            clock: VectorClock::new(),
            server,
            message_rx,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn session_id(&self) -> Option<SessionId> {
        self.session_id
    }

    pub fn clock(&self) -> &VectorClock {
        &self.clock
    }

    pub async fn join(&mut self, session_id: SessionId) -> Result<()> {
        self.session_id = Some(session_id);
        self.server.join_session(session_id, self.node_id).await
    }

    pub async fn leave(&mut self) -> Result<()> {
        if let Some(session_id) = self.session_id.take() {
            self.server.leave_session(session_id, self.node_id).await?;
        }
        Ok(())
    }

    pub async fn send_update(&mut self, payload: Vec<u8>) -> Result<()> {
        let session_id = self.session_id
            .ok_or_else(|| sena_core::Error::validation("not joined to a session"))?;

        self.clock.increment(self.node_id);

        self.server
            .send_state_update(session_id, self.clock.clone(), payload)
            .await
    }

    pub async fn recv(&mut self) -> Option<SyncMessage> {
        self.message_rx.recv().await.ok()
    }

    pub fn try_recv(&mut self) -> Option<SyncMessage> {
        self.message_rx.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_server_join() {
        let server = Arc::new(SyncServer::new());
        let session_id = SessionId::new();
        let node_id = Uuid::new_v4();

        server.join_session(session_id, node_id).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let peers = server.session_peers(&session_id).await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0], node_id);
    }

    #[tokio::test]
    async fn test_sync_client_join_leave() {
        let server = Arc::new(SyncServer::new());
        let mut client = SyncClient::new(server.clone());
        let session_id = SessionId::new();

        client.join(session_id).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert_eq!(client.session_id(), Some(session_id));

        client.leave().await.unwrap();
        assert_eq!(client.session_id(), None);
    }
}
