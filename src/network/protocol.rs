use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkCommand {
    Ping,
    Pong,

    Handshake {
        peer_id: String,
        peer_name: String,
        version: String,
    },
    HandshakeAck {
        peer_id: String,
        peer_name: String,
        version: String,
    },

    AuthRequest {
        token: String,
    },
    AuthResponse {
        success: bool,
        message: String,
    },

    SessionAnnounce {
        session_id: String,
        session_name: String,
        role: String,
        working_dir: String,
    },
    SessionEnd {
        session_id: String,
    },

    Who,
    WhoResponse {
        sessions: Vec<RemoteSession>,
    },

    Message {
        from_peer: String,
        from_session: String,
        to_peer: String,
        to_session: String,
        content: String,
        timestamp: i64,
    },
    MessageAck {
        message_id: String,
    },

    Broadcast {
        from_peer: String,
        from_session: String,
        content: String,
        timestamp: i64,
    },

    ShareInfo {
        shared_paths: Vec<SharedPath>,
    },
    ShareRequest {
        path: String,
    },

    Disconnect,

    Error {
        code: u32,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSession {
    pub peer_id: String,
    pub peer_name: String,
    pub peer_addr: String,
    pub session_id: String,
    pub session_name: String,
    pub role: String,
    pub working_dir: String,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedPath {
    pub path: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub id: String,
    pub command: NetworkCommand,
    pub timestamp: i64,
}

impl NetworkMessage {
    pub fn new(command: NetworkCommand) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            command,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn ping() -> Self {
        Self::new(NetworkCommand::Ping)
    }

    pub fn pong() -> Self {
        Self::new(NetworkCommand::Pong)
    }

    pub fn handshake(peer_id: &str, peer_name: &str, version: &str) -> Self {
        Self::new(NetworkCommand::Handshake {
            peer_id: peer_id.to_string(),
            peer_name: peer_name.to_string(),
            version: version.to_string(),
        })
    }

    pub fn handshake_ack(peer_id: &str, peer_name: &str, version: &str) -> Self {
        Self::new(NetworkCommand::HandshakeAck {
            peer_id: peer_id.to_string(),
            peer_name: peer_name.to_string(),
            version: version.to_string(),
        })
    }

    pub fn auth_request(token: &str) -> Self {
        Self::new(NetworkCommand::AuthRequest {
            token: token.to_string(),
        })
    }

    pub fn auth_response(success: bool, message: &str) -> Self {
        Self::new(NetworkCommand::AuthResponse {
            success,
            message: message.to_string(),
        })
    }

    pub fn who() -> Self {
        Self::new(NetworkCommand::Who)
    }

    pub fn who_response(sessions: Vec<RemoteSession>) -> Self {
        Self::new(NetworkCommand::WhoResponse { sessions })
    }

    pub fn message(
        from_peer: &str,
        from_session: &str,
        to_peer: &str,
        to_session: &str,
        content: &str,
    ) -> Self {
        Self::new(NetworkCommand::Message {
            from_peer: from_peer.to_string(),
            from_session: from_session.to_string(),
            to_peer: to_peer.to_string(),
            to_session: to_session.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub fn broadcast(from_peer: &str, from_session: &str, content: &str) -> Self {
        Self::new(NetworkCommand::Broadcast {
            from_peer: from_peer.to_string(),
            from_session: from_session.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub fn error(code: u32, message: &str) -> Self {
        Self::new(NetworkCommand::Error {
            code,
            message: message.to_string(),
        })
    }

    pub fn disconnect() -> Self {
        Self::new(NetworkCommand::Disconnect)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let json =
            serde_json::to_string(self).map_err(|e| format!("Serialization failed: {}", e))?;
        let mut bytes = (json.len() as u32).to_be_bytes().to_vec();
        bytes.extend(json.as_bytes());
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 4 {
            return Err("Message too short".to_string());
        }
        let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        if bytes.len() < 4 + len {
            return Err("Incomplete message".to_string());
        }
        let json =
            std::str::from_utf8(&bytes[4..4 + len]).map_err(|e| format!("Invalid UTF-8: {}", e))?;
        serde_json::from_str(json).map_err(|e| format!("Deserialization failed: {}", e))
    }
}

pub const DEFAULT_PORT: u16 = 9876;
pub const MDNS_SERVICE_TYPE: &str = "_sena._tcp.local.";
pub const PROTOCOL_VERSION: &str = "1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = NetworkMessage::ping();
        let bytes = msg.to_bytes().unwrap();
        let decoded = NetworkMessage::from_bytes(&bytes).unwrap();
        assert!(matches!(decoded.command, NetworkCommand::Ping));
    }

    #[test]
    fn test_handshake_message() {
        let msg = NetworkMessage::handshake("peer1", "Test Peer", "1.0");
        let bytes = msg.to_bytes().unwrap();
        let decoded = NetworkMessage::from_bytes(&bytes).unwrap();
        if let NetworkCommand::Handshake {
            peer_id,
            peer_name,
            version,
        } = decoded.command
        {
            assert_eq!(peer_id, "peer1");
            assert_eq!(peer_name, "Test Peer");
            assert_eq!(version, "1.0");
        } else {
            panic!("Wrong command type");
        }
    }
}
