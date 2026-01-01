use crate::crdt::{NodeId, VectorClock};
use crate::sync::SyncServer;
use futures::{SinkExt, StreamExt};
use sena_core::{Result, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    Subscribe { session_id: SessionId },
    Unsubscribe { session_id: SessionId },
    Operation { session_id: SessionId, op_type: String, payload: Vec<u8>, clock: VectorClock },
    Ack { operation_id: String },
    SyncRequest { session_id: SessionId, since_clock: VectorClock },
    SyncResponse { session_id: SessionId, operations: Vec<WsOperation> },
    Heartbeat { timestamp: u64 },
    Error { code: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsOperation {
    pub id: String,
    pub op_type: String,
    pub payload: Vec<u8>,
    pub clock: VectorClock,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WsClientState {
    pub node_id: NodeId,
    pub subscribed_sessions: Vec<SessionId>,
    pub last_clock: HashMap<SessionId, VectorClock>,
    pub connected_at: std::time::Instant,
    pub last_heartbeat: std::time::Instant,
}

pub struct WebSocketServer {
    _node_id: NodeId,
    sync_server: Arc<SyncServer>,
    clients: Arc<RwLock<HashMap<NodeId, mpsc::Sender<WsMessage>>>>,
    operation_log: Arc<RwLock<HashMap<SessionId, Vec<WsOperation>>>>,
    broadcast_tx: broadcast::Sender<(SessionId, WsMessage)>,
}

impl WebSocketServer {
    pub fn new(sync_server: Arc<SyncServer>) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);

        Self {
            _node_id: uuid::Uuid::new_v4(),
            sync_server,
            clients: Arc::new(RwLock::new(HashMap::new())),
            operation_log: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    pub async fn start(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| sena_core::Error::network(format!("bind failed: {}", e)))?;

        tracing::info!(addr = addr, "websocket server started");

        let clients = self.clients.clone();
        let sync_server = self.sync_server.clone();
        let operation_log = self.operation_log.clone();
        let broadcast_tx = self.broadcast_tx.clone();

        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                tracing::debug!(addr = %addr, "new connection");
                let clients = clients.clone();
                let sync_server = sync_server.clone();
                let operation_log = operation_log.clone();
                let broadcast_tx = broadcast_tx.clone();

                tokio::spawn(async move {
                    if let Err(e) = Self::handle_connection(
                        stream,
                        clients,
                        sync_server,
                        operation_log,
                        broadcast_tx,
                    ).await {
                        tracing::error!(error = %e, "connection error");
                    }
                });
            }
        });

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        clients: Arc<RwLock<HashMap<NodeId, mpsc::Sender<WsMessage>>>>,
        sync_server: Arc<SyncServer>,
        operation_log: Arc<RwLock<HashMap<SessionId, Vec<WsOperation>>>>,
        broadcast_tx: broadcast::Sender<(SessionId, WsMessage)>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| sena_core::Error::network(format!("websocket handshake failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();
        let node_id = uuid::Uuid::new_v4();
        let (tx, mut rx) = mpsc::channel::<WsMessage>(256);

        let mut state = WsClientState {
            node_id,
            subscribed_sessions: Vec::new(),
            last_clock: HashMap::new(),
            connected_at: std::time::Instant::now(),
            last_heartbeat: std::time::Instant::now(),
        };

        clients.write().await.insert(node_id, tx.clone());

        let mut broadcast_rx = broadcast_tx.subscribe();
        let subscribed_sessions = Arc::new(RwLock::new(Vec::<SessionId>::new()));
        let subscribed_sessions_clone = subscribed_sessions.clone();

        let write_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        let json = serde_json::to_string(&msg).unwrap_or_default();
                        if write.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Ok((session_id, msg)) = broadcast_rx.recv() => {
                        let sessions = subscribed_sessions_clone.read().await;
                        if sessions.contains(&session_id) {
                            let json = serde_json::to_string(&msg).unwrap_or_default();
                            if write.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        });

        while let Some(result) = read.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                        Self::handle_message(
                            &mut state,
                            msg,
                            &sync_server,
                            &operation_log,
                            &broadcast_tx,
                            &tx,
                            &subscribed_sessions,
                        ).await?;
                    }
                }
                Ok(Message::Ping(_)) => {
                    let _ = tx.send(WsMessage::Heartbeat {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    }).await;
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }

        for session_id in &state.subscribed_sessions {
            let _ = sync_server.leave_session(*session_id, node_id).await;
        }
        clients.write().await.remove(&node_id);
        write_task.abort();

        Ok(())
    }

    async fn handle_message(
        state: &mut WsClientState,
        msg: WsMessage,
        sync_server: &Arc<SyncServer>,
        operation_log: &Arc<RwLock<HashMap<SessionId, Vec<WsOperation>>>>,
        broadcast_tx: &broadcast::Sender<(SessionId, WsMessage)>,
        client_tx: &mpsc::Sender<WsMessage>,
        subscribed_sessions: &Arc<RwLock<Vec<SessionId>>>,
    ) -> Result<()> {
        match msg {
            WsMessage::Subscribe { session_id } => {
                sync_server.join_session(session_id, state.node_id).await?;
                state.subscribed_sessions.push(session_id);
                subscribed_sessions.write().await.push(session_id);
                state.last_clock.insert(session_id, VectorClock::new());
                tracing::debug!(session = ?session_id, node = ?state.node_id, "client subscribed");
            }

            WsMessage::Unsubscribe { session_id } => {
                sync_server.leave_session(session_id, state.node_id).await?;
                state.subscribed_sessions.retain(|s| *s != session_id);
                subscribed_sessions.write().await.retain(|s| *s != session_id);
                state.last_clock.remove(&session_id);
                tracing::debug!(session = ?session_id, node = ?state.node_id, "client unsubscribed");
            }

            WsMessage::Operation { session_id, op_type, payload, clock } => {
                if !state.subscribed_sessions.contains(&session_id) {
                    let _ = client_tx.send(WsMessage::Error {
                        code: "NOT_SUBSCRIBED".to_string(),
                        message: "not subscribed to session".to_string(),
                    }).await;
                    return Ok(());
                }

                let operation = WsOperation {
                    id: uuid::Uuid::new_v4().to_string(),
                    op_type: op_type.clone(),
                    payload: payload.clone(),
                    clock: clock.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                operation_log
                    .write()
                    .await
                    .entry(session_id)
                    .or_default()
                    .push(operation.clone());

                state.last_clock.insert(session_id, clock.clone());

                let _ = broadcast_tx.send((session_id, WsMessage::Operation {
                    session_id,
                    op_type,
                    payload,
                    clock,
                }));

                let _ = client_tx.send(WsMessage::Ack {
                    operation_id: operation.id,
                }).await;
            }

            WsMessage::SyncRequest { session_id, since_clock } => {
                let ops = operation_log.read().await;
                let session_ops = ops.get(&session_id).cloned().unwrap_or_default();

                let filtered_ops: Vec<_> = session_ops
                    .into_iter()
                    .filter(|op| !since_clock.dominates(&op.clock))
                    .collect();

                let _ = client_tx.send(WsMessage::SyncResponse {
                    session_id,
                    operations: filtered_ops,
                }).await;
            }

            WsMessage::Heartbeat { .. } => {
                state.last_heartbeat = std::time::Instant::now();
            }

            _ => {}
        }

        Ok(())
    }

    pub async fn broadcast_to_session(&self, session_id: SessionId, msg: WsMessage) {
        let _ = self.broadcast_tx.send((session_id, msg));
    }

    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    pub async fn session_operations(&self, session_id: &SessionId) -> Vec<WsOperation> {
        self.operation_log
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self::new(Arc::new(SyncServer::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Subscribe { session_id: SessionId::new() };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        match parsed {
            WsMessage::Subscribe { .. } => {}
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_ws_operation() {
        let op = WsOperation {
            id: "test-123".to_string(),
            op_type: "insert".to_string(),
            payload: vec![1, 2, 3],
            clock: VectorClock::new(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&op).unwrap();
        let parsed: WsOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test-123");
    }
}
