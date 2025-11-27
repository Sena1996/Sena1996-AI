use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};

use super::peer::PeerRegistry;
use super::protocol::{NetworkCommand, NetworkMessage, RemoteSession, PROTOCOL_VERSION};

pub type ConnectionId = String;
type MessageHandler = Arc<RwLock<Option<mpsc::Sender<(ConnectionId, NetworkMessage)>>>>;

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub peer_id: Option<String>,
    pub peer_name: Option<String>,
    pub address: SocketAddr,
    pub authenticated: bool,
    pub sender: mpsc::Sender<NetworkMessage>,
}

pub struct NetworkServer {
    port: u16,
    peer_registry: Arc<RwLock<PeerRegistry>>,
    connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
    sessions: Arc<RwLock<Vec<RemoteSession>>>,
    local_sessions: Arc<RwLock<Vec<RemoteSession>>>,
    running: Arc<RwLock<bool>>,
    message_handler: MessageHandler,
}

impl NetworkServer {
    pub fn new(port: u16, peer_registry: Arc<RwLock<PeerRegistry>>) -> Self {
        Self {
            port,
            peer_registry,
            connections: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(Vec::new())),
            local_sessions: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            message_handler: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        *self.running.write().await = true;

        let connections = self.connections.clone();
        let peer_registry = self.peer_registry.clone();
        let sessions = self.sessions.clone();
        let local_sessions = self.local_sessions.clone();
        let running = self.running.clone();
        let message_handler = self.message_handler.clone();

        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let conn_id = uuid::Uuid::new_v4().to_string();
                        let connections = connections.clone();
                        let peer_registry = peer_registry.clone();
                        let sessions = sessions.clone();
                        let local_sessions = local_sessions.clone();
                        let message_handler = message_handler.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                conn_id,
                                stream,
                                addr,
                                connections,
                                peer_registry,
                                sessions,
                                local_sessions,
                                message_handler,
                            )
                            .await
                            {
                                eprintln!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        if *running.read().await {
                            eprintln!("Accept error: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        *self.running.write().await = false;
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_connection(
        conn_id: ConnectionId,
        stream: TcpStream,
        addr: SocketAddr,
        connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
        peer_registry: Arc<RwLock<PeerRegistry>>,
        sessions: Arc<RwLock<Vec<RemoteSession>>>,
        local_sessions: Arc<RwLock<Vec<RemoteSession>>>,
        message_handler: MessageHandler,
    ) -> Result<(), String> {
        let (tx, mut rx) = mpsc::channel::<NetworkMessage>(32);

        let connection = Connection {
            id: conn_id.clone(),
            peer_id: None,
            peer_name: None,
            address: addr,
            authenticated: false,
            sender: tx,
        };

        connections
            .write()
            .await
            .insert(conn_id.clone(), connection);

        let stream = Arc::new(tokio::sync::Mutex::new(stream));
        let stream_writer = stream.clone();

        let write_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(bytes) = msg.to_bytes() {
                    let mut stream = stream_writer.lock().await;
                    if stream.write_all(&bytes).await.is_err() {
                        break;
                    }
                }
            }
        });

        let mut buffer = vec![0u8; 65536];
        let mut data = Vec::new();

        loop {
            let n = {
                let mut stream_guard = stream.lock().await;
                match stream_guard.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                }
            };

            data.extend_from_slice(&buffer[..n]);

            while data.len() >= 4 {
                let msg_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

                if data.len() < 4 + msg_len {
                    break;
                }

                let msg_bytes: Vec<u8> = data.drain(..4 + msg_len).collect();

                if let Ok(msg) = NetworkMessage::from_bytes(&msg_bytes) {
                    let msg_clone = msg.clone();
                    let response = Self::process_message(
                        &conn_id,
                        msg,
                        connections.clone(),
                        peer_registry.clone(),
                        sessions.clone(),
                        local_sessions.clone(),
                    )
                    .await;

                    if let Some(response) = response {
                        if let Some(conn) = connections.read().await.get(&conn_id) {
                            let _ = conn.sender.send(response).await;
                        }
                    }

                    if let Some(handler) = message_handler.read().await.as_ref() {
                        let _ = handler.send((conn_id.clone(), msg_clone.clone())).await;
                    }

                    if matches!(msg_clone.command, NetworkCommand::Disconnect) {
                        break;
                    }
                }
            }
        }

        connections.write().await.remove(&conn_id);
        write_task.abort();

        Ok(())
    }

    async fn process_message(
        conn_id: &str,
        msg: NetworkMessage,
        connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
        peer_registry: Arc<RwLock<PeerRegistry>>,
        sessions: Arc<RwLock<Vec<RemoteSession>>>,
        local_sessions: Arc<RwLock<Vec<RemoteSession>>>,
    ) -> Option<NetworkMessage> {
        match msg.command {
            NetworkCommand::Ping => Some(NetworkMessage::pong()),

            NetworkCommand::Handshake {
                peer_id,
                peer_name,
                version: _,
            } => {
                let registry = peer_registry.read().await;
                let local_id = registry.local_peer_id.clone();
                let local_name = registry.local_peer_name.clone();
                drop(registry);

                if let Some(conn) = connections.write().await.get_mut(conn_id) {
                    conn.peer_id = Some(peer_id.clone());
                    conn.peer_name = Some(peer_name.clone());
                }

                Some(NetworkMessage::handshake_ack(
                    &local_id,
                    &local_name,
                    PROTOCOL_VERSION,
                ))
            }

            NetworkCommand::AuthRequest { token } => {
                let registry = peer_registry.read().await;

                let mut authorized = false;
                if let Some(conn) = connections.read().await.get(conn_id) {
                    if let Some(ref peer_id) = conn.peer_id {
                        if let Some(peer) = registry.get_peer(peer_id) {
                            if peer.auth_token.as_deref() == Some(&token) {
                                authorized = true;
                            }
                        }
                    }
                }
                drop(registry);

                if authorized {
                    if let Some(conn) = connections.write().await.get_mut(conn_id) {
                        conn.authenticated = true;
                    }
                    Some(NetworkMessage::auth_response(true, "Authorized"))
                } else {
                    Some(NetworkMessage::auth_response(false, "Invalid token"))
                }
            }

            NetworkCommand::Who => {
                let all_sessions: Vec<RemoteSession> = {
                    let remote = sessions.read().await;
                    let local = local_sessions.read().await;
                    remote.iter().chain(local.iter()).cloned().collect()
                };
                Some(NetworkMessage::who_response(all_sessions))
            }

            NetworkCommand::SessionAnnounce {
                session_id,
                session_name,
                role,
                working_dir,
            } => {
                if let Some(conn) = connections.read().await.get(conn_id) {
                    if conn.authenticated {
                        let session = RemoteSession {
                            peer_id: conn.peer_id.clone().unwrap_or_default(),
                            peer_name: conn.peer_name.clone().unwrap_or_default(),
                            peer_addr: conn.address.to_string(),
                            session_id,
                            session_name,
                            role,
                            working_dir,
                            last_seen: chrono::Utc::now().timestamp(),
                        };

                        let mut sessions = sessions.write().await;
                        sessions.retain(|s| s.session_id != session.session_id);
                        sessions.push(session);
                    }
                }
                None
            }

            NetworkCommand::SessionEnd { session_id } => {
                sessions
                    .write()
                    .await
                    .retain(|s| s.session_id != session_id);
                None
            }

            NetworkCommand::Message { .. } => {
                Some(NetworkMessage::new(NetworkCommand::MessageAck {
                    message_id: msg.id,
                }))
            }

            _ => None,
        }
    }

    pub async fn add_local_session(&self, session: RemoteSession) {
        let mut sessions = self.local_sessions.write().await;
        sessions.retain(|s| s.session_id != session.session_id);
        sessions.push(session);
    }

    pub async fn remove_local_session(&self, session_id: &str) {
        self.local_sessions
            .write()
            .await
            .retain(|s| s.session_id != session_id);
    }

    pub async fn get_all_sessions(&self) -> Vec<RemoteSession> {
        let remote = self.sessions.read().await;
        let local = self.local_sessions.read().await;
        remote.iter().chain(local.iter()).cloned().collect()
    }

    pub async fn get_connections(&self) -> Vec<(ConnectionId, SocketAddr, bool)> {
        self.connections
            .read()
            .await
            .iter()
            .map(|(id, conn)| (id.clone(), conn.address, conn.authenticated))
            .collect()
    }

    pub async fn send_to_connection(
        &self,
        conn_id: &str,
        msg: NetworkMessage,
    ) -> Result<(), String> {
        let connections = self.connections.read().await;
        let conn = connections
            .get(conn_id)
            .ok_or_else(|| format!("Connection {} not found", conn_id))?;
        conn.sender
            .send(msg)
            .await
            .map_err(|e| format!("Failed to send: {}", e))
    }

    pub async fn broadcast(&self, msg: NetworkMessage) {
        let connections = self.connections.read().await;
        for conn in connections.values() {
            if conn.authenticated {
                let _ = conn.sender.send(msg.clone()).await;
            }
        }
    }

    pub fn set_message_handler(&self, handler: mpsc::Sender<(ConnectionId, NetworkMessage)>) {
        let message_handler = self.message_handler.clone();
        tokio::spawn(async move {
            *message_handler.write().await = Some(handler);
        });
    }
}

pub struct NetworkClient {
    peer_registry: Arc<RwLock<PeerRegistry>>,
}

impl NetworkClient {
    pub fn new(peer_registry: Arc<RwLock<PeerRegistry>>) -> Self {
        Self { peer_registry }
    }

    pub async fn connect(&self, address: &str, port: u16) -> Result<ClientConnection, String> {
        let addr = format!("{}:{}", address, port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

        let registry = self.peer_registry.read().await;
        let local_id = registry.local_peer_id.clone();
        let local_name = registry.local_peer_name.clone();
        drop(registry);

        let mut client = ClientConnection::new(stream, local_id, local_name);
        client.handshake().await?;

        Ok(client)
    }

    pub async fn connect_and_auth(
        &self,
        address: &str,
        port: u16,
        token: &str,
    ) -> Result<ClientConnection, String> {
        let mut client = self.connect(address, port).await?;
        client.authenticate(token).await?;
        Ok(client)
    }
}

pub struct ClientConnection {
    stream: TcpStream,
    local_peer_id: String,
    local_peer_name: String,
    remote_peer_id: Option<String>,
    remote_peer_name: Option<String>,
    authenticated: bool,
}

impl ClientConnection {
    fn new(stream: TcpStream, local_peer_id: String, local_peer_name: String) -> Self {
        Self {
            stream,
            local_peer_id,
            local_peer_name,
            remote_peer_id: None,
            remote_peer_name: None,
            authenticated: false,
        }
    }

    async fn send(&mut self, msg: NetworkMessage) -> Result<(), String> {
        let bytes = msg.to_bytes()?;
        self.stream
            .write_all(&bytes)
            .await
            .map_err(|e| format!("Failed to send: {}", e))
    }

    async fn receive(&mut self) -> Result<NetworkMessage, String> {
        let mut len_buf = [0u8; 4];
        self.stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| format!("Failed to read length: {}", e))?;

        let len = u32::from_be_bytes(len_buf) as usize;
        let mut msg_buf = vec![0u8; len];
        self.stream
            .read_exact(&mut msg_buf)
            .await
            .map_err(|e| format!("Failed to read message: {}", e))?;

        let mut full_buf = len_buf.to_vec();
        full_buf.extend(msg_buf);

        NetworkMessage::from_bytes(&full_buf)
    }

    pub async fn handshake(&mut self) -> Result<(), String> {
        let msg =
            NetworkMessage::handshake(&self.local_peer_id, &self.local_peer_name, PROTOCOL_VERSION);
        self.send(msg).await?;

        let response = self.receive().await?;
        if let NetworkCommand::HandshakeAck {
            peer_id,
            peer_name,
            version: _,
        } = response.command
        {
            self.remote_peer_id = Some(peer_id);
            self.remote_peer_name = Some(peer_name);
            Ok(())
        } else {
            Err("Invalid handshake response".to_string())
        }
    }

    pub async fn authenticate(&mut self, token: &str) -> Result<(), String> {
        let msg = NetworkMessage::auth_request(token);
        self.send(msg).await?;

        let response = self.receive().await?;
        if let NetworkCommand::AuthResponse { success, message } = response.command {
            if success {
                self.authenticated = true;
                Ok(())
            } else {
                Err(message)
            }
        } else {
            Err("Invalid auth response".to_string())
        }
    }

    pub async fn ping(&mut self) -> Result<bool, String> {
        self.send(NetworkMessage::ping()).await?;
        let response = self.receive().await?;
        Ok(matches!(response.command, NetworkCommand::Pong))
    }

    pub async fn who(&mut self) -> Result<Vec<RemoteSession>, String> {
        self.send(NetworkMessage::who()).await?;
        let response = self.receive().await?;
        if let NetworkCommand::WhoResponse { sessions } = response.command {
            Ok(sessions)
        } else {
            Err("Invalid who response".to_string())
        }
    }

    pub async fn send_message(&mut self, to_session: &str, content: &str) -> Result<(), String> {
        let msg = NetworkMessage::message(
            &self.local_peer_id,
            "",
            self.remote_peer_id.as_deref().unwrap_or(""),
            to_session,
            content,
        );
        self.send(msg).await
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        self.send(NetworkMessage::disconnect()).await
    }

    pub fn remote_peer_id(&self) -> Option<&str> {
        self.remote_peer_id.as_deref()
    }

    pub fn remote_peer_name(&self) -> Option<&str> {
        self.remote_peer_name.as_deref()
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_server_creation() {
        let registry = Arc::new(RwLock::new(PeerRegistry::new(std::path::PathBuf::from(
            "/tmp/test_peers.json",
        ))));
        let server = NetworkServer::new(0, registry);
        assert!(!server.is_running().await);
    }
}
