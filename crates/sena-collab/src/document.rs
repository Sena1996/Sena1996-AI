use crate::crdt::{LWWMap, NodeId, ORSet, VectorClock};
use crate::sync::{SyncClient, SyncMessage, SyncServer};
use sena_core::{Message, Result, SessionId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    pub session_id: SessionId,
    pub messages: Vec<Message>,
    pub metadata: LWWMap<String, String>,
    pub participants: ORSet<NodeId>,
    pub clock: VectorClock,
}

impl DocumentState {
    pub fn new(session_id: SessionId, node_id: NodeId) -> Self {
        Self {
            session_id,
            messages: Vec::new(),
            metadata: LWWMap::new(node_id),
            participants: ORSet::new(),
            clock: VectorClock::new(),
        }
    }

    pub fn add_message(&mut self, message: Message, node_id: NodeId) {
        self.messages.push(message);
        self.clock.increment(node_id);
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(&key.to_string())
    }

    pub fn add_participant(&mut self, node_id: NodeId) {
        self.participants.add(node_id, node_id);
    }

    pub fn remove_participant(&mut self, node_id: &NodeId) {
        self.participants.remove(node_id);
    }

    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    pub fn merge(&mut self, other: &DocumentState) {
        for msg in &other.messages {
            if !self.messages.iter().any(|m| {
                m.content == msg.content && m.role == msg.role
            }) {
                self.messages.push(msg.clone());
            }
        }

        self.metadata.merge(&other.metadata);
        self.participants.merge(&other.participants);
        self.clock.merge(&other.clock);
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| sena_core::Error::internal(format!("serialization failed: {}", e)))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| sena_core::Error::internal(format!("deserialization failed: {}", e)))
    }
}

pub struct DocumentSynchronizer {
    node_id: NodeId,
    state: Arc<RwLock<DocumentState>>,
    sync_client: SyncClient,
}

impl DocumentSynchronizer {
    pub fn new(session_id: SessionId, sync_server: Arc<SyncServer>) -> Self {
        let node_id = uuid::Uuid::new_v4();
        let state = DocumentState::new(session_id, node_id);

        Self {
            node_id,
            state: Arc::new(RwLock::new(state)),
            sync_client: SyncClient::new(sync_server),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub async fn join(&mut self) -> Result<()> {
        let session_id = self.state.read().await.session_id;
        self.sync_client.join(session_id).await?;

        {
            let mut state = self.state.write().await;
            state.add_participant(self.node_id);
        }

        self.broadcast_state().await
    }

    pub async fn leave(&mut self) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.remove_participant(&self.node_id);
        }

        self.broadcast_state().await?;
        self.sync_client.leave().await
    }

    pub async fn add_message(&mut self, message: Message) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.add_message(message, self.node_id);
        }

        self.broadcast_state().await
    }

    pub async fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.set_metadata(key, value);
            state.clock.increment(self.node_id);
        }

        self.broadcast_state().await
    }

    pub async fn get_state(&self) -> DocumentState {
        self.state.read().await.clone()
    }

    pub async fn get_messages(&self) -> Vec<Message> {
        self.state.read().await.messages.clone()
    }

    async fn broadcast_state(&mut self) -> Result<()> {
        let state = self.state.read().await;
        let payload = state.to_bytes()?;
        drop(state);

        self.sync_client.send_update(payload).await
    }

    pub async fn process_updates(&mut self) -> Result<u32> {
        let mut processed = 0;

        while let Some(msg) = self.sync_client.try_recv() {
            match msg {
                SyncMessage::StateUpdate { session_id, payload, .. } => {
                    let current_session = self.state.read().await.session_id;
                    if session_id == current_session {
                        if let Ok(remote_state) = DocumentState::from_bytes(&payload) {
                            let mut state = self.state.write().await;
                            state.merge(&remote_state);
                            processed += 1;
                        }
                    }
                }
                SyncMessage::Join { session_id, node_id } => {
                    let current_session = self.state.read().await.session_id;
                    if session_id == current_session {
                        let mut state = self.state.write().await;
                        state.add_participant(node_id);
                        processed += 1;
                    }
                }
                SyncMessage::Leave { session_id, node_id } => {
                    let current_session = self.state.read().await.session_id;
                    if session_id == current_session {
                        let mut state = self.state.write().await;
                        state.remove_participant(&node_id);
                        processed += 1;
                    }
                }
                _ => {}
            }
        }

        Ok(processed)
    }

    pub async fn sync_loop(&mut self) -> ! {
        loop {
            let _ = self.process_updates().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

pub struct CollaborativeSession {
    synchronizer: DocumentSynchronizer,
}

impl CollaborativeSession {
    pub fn new(session_id: SessionId, sync_server: Arc<SyncServer>) -> Self {
        Self {
            synchronizer: DocumentSynchronizer::new(session_id, sync_server),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.synchronizer.join().await
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.synchronizer.leave().await
    }

    pub async fn add_message(&mut self, message: Message) -> Result<()> {
        self.synchronizer.add_message(message).await
    }

    pub async fn get_messages(&self) -> Vec<Message> {
        self.synchronizer.get_messages().await
    }

    pub async fn get_state(&self) -> DocumentState {
        self.synchronizer.get_state().await
    }

    pub async fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        self.synchronizer.set_metadata(key, value).await
    }

    pub async fn process_updates(&mut self) -> Result<u32> {
        self.synchronizer.process_updates().await
    }

    pub fn node_id(&self) -> NodeId {
        self.synchronizer.node_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_document_state_merge() {
        let session_id = SessionId::new();
        let n1 = uuid::Uuid::new_v4();
        let n2 = uuid::Uuid::new_v4();

        let mut state1 = DocumentState::new(session_id, n1);
        let mut state2 = DocumentState::new(session_id, n2);

        state1.add_message(Message::user("Hello from node 1"), n1);
        state2.add_message(Message::user("Hello from node 2"), n2);

        state1.merge(&state2);

        assert_eq!(state1.messages.len(), 2);
    }

    #[tokio::test]
    async fn test_document_synchronizer() {
        let session_id = SessionId::new();
        let server = Arc::new(SyncServer::new());

        let mut sync1 = DocumentSynchronizer::new(session_id, server.clone());
        let mut sync2 = DocumentSynchronizer::new(session_id, server.clone());

        sync1.join().await.unwrap();
        sync2.join().await.unwrap();

        sync1.add_message(Message::user("Test message")).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let processed = sync2.process_updates().await.unwrap();
        assert!(processed > 0);
    }
}
