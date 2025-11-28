use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use tokio::sync::RwLock;

use sena_collab::CollabOrchestrator;
use sena_providers::{config::ProvidersConfig, ChatRequest, Message, ProviderRouter};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub default_model: String,
    pub has_api_key: bool,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct Capabilities {
    pub streaming: bool,
    pub tools: bool,
    pub vision: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_length: usize,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_streaming: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatResponseDto {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub content: String,
    pub usage: UsageDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageDto {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionDto {
    pub id: String,
    pub name: String,
    pub state: String,
    pub created_at: String,
    pub participants: Vec<ParticipantDto>,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParticipantDto {
    pub agent_id: String,
    pub provider: String,
    pub model: String,
    pub is_host: bool,
    pub status: String,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthDto {
    pub status: String,
    pub score: u32,
    pub version: String,
    pub uptime: u64,
    pub providers: ProvidersStatusDto,
    pub sessions: SessionsStatusDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvidersStatusDto {
    pub total: usize,
    pub connected: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionsStatusDto {
    pub active: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSessionDto {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub working_on: Option<String>,
    pub working_directory: String,
    pub joined_at: u64,
    pub last_heartbeat: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct CliSessionsFile {
    #[allow(dead_code)]
    version: String,
    sessions: std::collections::HashMap<String, CliSessionData>,
}

#[derive(Debug, Clone, Deserialize)]
struct CliSessionData {
    id: String,
    name: String,
    role: String,
    status: String,
    working_on: Option<String>,
    working_directory: String,
    joined_at: u64,
    last_heartbeat: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubMessageDto {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: u64,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendMessageResult {
    pub success: bool,
    pub message_id: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HubIdentityDto {
    pub hub_id: String,
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub short_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredPeerDto {
    pub hub_id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectedPeerDto {
    pub hub_id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub is_online: bool,
    pub session_count: usize,
    pub connected_since: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionRequestDto {
    pub request_id: String,
    pub from_hub_id: String,
    pub from_hub_name: String,
    pub from_address: String,
    pub message: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederatedSessionDto {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HubIdentityFile {
    hub_id: String,
    name: String,
    hostname: String,
    port: u16,
    created_at: u64,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeersFile {
    version: String,
    connected_hubs: std::collections::HashMap<String, ConnectedHubData>,
    pending_requests: Vec<PendingRequestData>,
    #[allow(dead_code)]
    last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectedHubData {
    hub_id: String,
    name: String,
    address: String,
    port: u16,
    #[allow(dead_code)]
    auth_token: String,
    connected_at: u64,
    last_seen: u64,
    session_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingRequestData {
    request_id: String,
    from_hub_id: String,
    from_hub_name: String,
    from_address: String,
    #[allow(dead_code)]
    from_port: u16,
    message: Option<String>,
    created_at: u64,
    expires_at: u64,
}

pub struct AppState {
    pub config: RwLock<ProvidersConfig>,
    pub orchestrator: Arc<RwLock<CollabOrchestrator>>,
    pub start_time: Instant,
}

impl AppState {
    pub fn new() -> Self {
        let config = ProvidersConfig::load_or_default();
        let orchestrator = Arc::new(RwLock::new(CollabOrchestrator::new(100)));
        Self {
            config: RwLock::new(config),
            orchestrator,
            start_time: Instant::now(),
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
async fn get_providers(state: State<'_, AppState>) -> Result<Vec<ProviderInfo>, String> {
    let config = state.config.read().await;

    let providers: Vec<ProviderInfo> = config
        .providers
        .iter()
        .map(|(id, cfg)| {
            let has_key = cfg.get_api_key().is_some() || id == "ollama";
            ProviderInfo {
                id: id.clone(),
                name: get_provider_display_name(id),
                status: if has_key {
                    "connected".to_string()
                } else {
                    "disconnected".to_string()
                },
                default_model: cfg.default_model.clone().unwrap_or_default(),
                has_api_key: has_key,
                capabilities: Capabilities {
                    streaming: true,
                    tools: id != "ollama",
                    vision: id != "ollama",
                },
            }
        })
        .collect();

    Ok(providers)
}

#[tauri::command]
async fn get_provider_status(state: State<'_, AppState>) -> Result<Vec<ProviderInfo>, String> {
    get_providers(state).await
}

#[tauri::command]
async fn get_models(
    state: State<'_, AppState>,
    provider_id: Option<String>,
) -> Result<Vec<ModelInfo>, String> {
    let config = state.config.read().await;
    let router =
        ProviderRouter::from_config(&config).map_err(|e| format!("Router error: {}", e))?;

    let models: Vec<ModelInfo> = if let Some(pid) = provider_id {
        router
            .get_provider(&pid)
            .map(|p| {
                p.available_models()
                    .iter()
                    .map(|m| ModelInfo {
                        id: m.id.clone(),
                        name: m.name.clone(),
                        provider: m.provider.clone(),
                        context_length: m.context_length,
                        supports_vision: m.supports_vision,
                        supports_tools: m.supports_tools,
                        supports_streaming: m.supports_streaming,
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        router
            .all_models()
            .iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                provider: m.provider.clone(),
                context_length: m.context_length,
                supports_vision: m.supports_vision,
                supports_tools: m.supports_tools,
                supports_streaming: m.supports_streaming,
            })
            .collect()
    };

    Ok(models)
}

#[tauri::command]
async fn send_chat(
    state: State<'_, AppState>,
    message: String,
    provider: Option<String>,
    model: Option<String>,
) -> Result<ChatResponseDto, String> {
    if message.trim().is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    let config = state.config.read().await;
    let router =
        ProviderRouter::from_config(&config).map_err(|e| format!("Router error: {}", e))?;

    let mut request = ChatRequest::new(vec![Message::user(&message)]);

    if let Some(m) = &model {
        request = request.with_model(m.clone());
    }

    let response = if let Some(provider_id) = provider {
        let target_provider = router
            .get_provider(&provider_id)
            .ok_or_else(|| format!("Provider not found: {}", provider_id))?;
        target_provider.chat(request).await
    } else {
        router.chat_with_fallback(request).await
    };

    match response {
        Ok(resp) => Ok(ChatResponseDto {
            id: resp.id,
            provider: resp.provider,
            model: resp.model,
            content: resp.content,
            usage: UsageDto {
                prompt_tokens: resp.usage.prompt_tokens,
                completion_tokens: resp.usage.completion_tokens,
                total_tokens: resp.usage.total_tokens,
            },
        }),
        Err(e) => Err(format!("Chat error: {}", e)),
    }
}

#[tauri::command]
async fn set_default_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    if config.set_default_provider(&provider_id) {
        let path = ProvidersConfig::config_path();
        config
            .save_to_file(&path)
            .map_err(|e| format!("Save error: {}", e))?;
        Ok(())
    } else {
        Err(format!("Unknown provider: {}", provider_id))
    }
}

#[tauri::command]
async fn test_provider(state: State<'_, AppState>, provider_id: String) -> Result<bool, String> {
    let config = state.config.read().await;
    let router =
        ProviderRouter::from_config(&config).map_err(|e| format!("Router error: {}", e))?;

    let provider = router
        .get_provider(&provider_id)
        .ok_or_else(|| format!("Provider not found: {}", provider_id))?;

    let test_request =
        ChatRequest::new(vec![Message::user("Say 'OK' if you can hear me.")]).with_max_tokens(10);

    match provider.chat(test_request).await {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Test failed: {}", e)),
    }
}

#[tauri::command]
async fn create_session(
    state: State<'_, AppState>,
    name: String,
    host_provider: String,
) -> Result<SessionDto, String> {
    if name.trim().is_empty() {
        return Err("Session name cannot be empty".to_string());
    }

    if name.len() > 200 {
        return Err("Session name too long (max 200 characters)".to_string());
    }

    let config = state.config.read().await;
    let router =
        ProviderRouter::from_config(&config).map_err(|e| format!("Router error: {}", e))?;

    let mut orchestrator = state.orchestrator.write().await;

    for provider in router.available_providers() {
        orchestrator.register_provider(Arc::clone(provider));
    }

    let session_id = orchestrator
        .create_session(&name, &host_provider)
        .await
        .map_err(|e| format!("Create session error: {}", e))?;

    let summary = orchestrator
        .get_session_summary(&session_id)
        .await
        .map_err(|e| format!("Get summary error: {}", e))?;

    Ok(SessionDto {
        id: summary.session_id,
        name: summary.name,
        state: format!("{:?}", summary.state),
        created_at: summary.created_at.to_rfc3339(),
        participants: summary
            .participants
            .iter()
            .map(|p| ParticipantDto {
                agent_id: p.agent_id.clone(),
                provider: p.provider.clone(),
                model: p.model.clone(),
                is_host: p.is_host,
                status: format!("{:?}", p.status),
                message_count: p.message_count,
            })
            .collect(),
        message_count: summary.message_count,
    })
}

#[tauri::command]
async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionDto>, String> {
    let orchestrator = state.orchestrator.read().await;
    let summaries = orchestrator.list_all_sessions().await;

    let sessions: Vec<SessionDto> = summaries
        .iter()
        .map(|summary| SessionDto {
            id: summary.session_id.clone(),
            name: summary.name.clone(),
            state: format!("{:?}", summary.state),
            created_at: summary.created_at.to_rfc3339(),
            participants: summary
                .participants
                .iter()
                .map(|p| ParticipantDto {
                    agent_id: p.agent_id.clone(),
                    provider: p.provider.clone(),
                    model: p.model.clone(),
                    is_host: p.is_host,
                    status: format!("{:?}", p.status),
                    message_count: p.message_count,
                })
                .collect(),
            message_count: summary.message_count,
        })
        .collect();

    Ok(sessions)
}

#[tauri::command]
async fn get_health(state: State<'_, AppState>) -> Result<HealthDto, String> {
    let config = state.config.read().await;
    let orchestrator = state.orchestrator.read().await;

    let connected = config
        .providers
        .iter()
        .filter(|(id, cfg)| cfg.get_api_key().is_some() || *id == "ollama")
        .count();

    let active_sessions = orchestrator.list_active_sessions().await;

    let health_score = calculate_health_score(connected, config.providers.len());

    Ok(HealthDto {
        status: if health_score >= 80 {
            "healthy".to_string()
        } else if health_score >= 50 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        },
        score: health_score,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: state.uptime_secs(),
        providers: ProvidersStatusDto {
            total: config.providers.len(),
            connected,
        },
        sessions: SessionsStatusDto {
            active: active_sessions.len(),
            total: active_sessions.len(),
        },
    })
}

fn calculate_health_score(connected: usize, total: usize) -> u32 {
    if total == 0 {
        return 50;
    }
    let provider_score = (connected as f32 / total as f32 * 100.0) as u32;
    provider_score.min(100)
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn get_provider_display_name(id: &str) -> String {
    match id {
        "claude" => "Claude (Anthropic)".to_string(),
        "openai" => "OpenAI".to_string(),
        "gemini" => "Google Gemini".to_string(),
        "ollama" => "Ollama (Local)".to_string(),
        "mistral" => "Mistral AI".to_string(),
        _ => id.to_string(),
    }
}

#[tauri::command]
async fn list_cli_sessions() -> Result<Vec<CliSessionDto>, String> {
    let sessions_path = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".claude")
        .join("hub")
        .join("sessions.json");

    if !sessions_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&sessions_path)
        .map_err(|e| format!("Cannot read sessions file: {}", e))?;

    let data: CliSessionsFile = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse sessions file: {}", e))?;

    let sessions: Vec<CliSessionDto> = data
        .sessions
        .into_values()
        .map(|s| CliSessionDto {
            id: s.id,
            name: s.name,
            role: s.role,
            status: s.status,
            working_on: s.working_on,
            working_directory: s.working_directory,
            joined_at: s.joined_at,
            last_heartbeat: s.last_heartbeat,
        })
        .collect();

    Ok(sessions)
}

#[tauri::command]
async fn send_message_to_session(
    target_session: String,
    message: String,
) -> Result<SendMessageResult, String> {
    let sessions_path = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".claude")
        .join("hub")
        .join("sessions.json");

    let sessions_content = std::fs::read_to_string(&sessions_path)
        .map_err(|e| format!("Cannot read sessions: {}", e))?;

    let sessions_data: CliSessionsFile = serde_json::from_str(&sessions_content)
        .map_err(|e| format!("Cannot parse sessions: {}", e))?;

    let target = sessions_data
        .sessions
        .values()
        .find(|s| s.name.to_lowercase() == target_session.to_lowercase() || s.id == target_session)
        .ok_or_else(|| format!("Session '{}' not found", target_session))?;

    let target_id = target.id.clone();
    let target_name = target.name.clone();

    let messages_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".claude")
        .join("hub")
        .join("messages");

    std::fs::create_dir_all(&messages_dir)
        .map_err(|e| format!("Cannot create messages dir: {}", e))?;

    let message_file = messages_dir.join(format!("{}.json", target_id));

    let mut messages: Vec<HubMessageDto> = if message_file.exists() {
        let content = std::fs::read_to_string(&message_file)
            .map_err(|e| format!("Cannot read messages: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let message_id = format!("hub-{}-{}", target_id, timestamp);

    let new_message = HubMessageDto {
        id: message_id.clone(),
        from: "hub".to_string(),
        to: target_id.clone(),
        content: message,
        message_type: "Direct".to_string(),
        timestamp,
        read: false,
    };

    messages.push(new_message);

    let json = serde_json::to_string_pretty(&messages)
        .map_err(|e| format!("Cannot serialize: {}", e))?;

    std::fs::write(&message_file, json)
        .map_err(|e| format!("Cannot write messages: {}", e))?;

    Ok(SendMessageResult {
        success: true,
        message_id,
        target: target_name,
    })
}

#[tauri::command]
async fn get_all_messages() -> Result<Vec<HubMessageDto>, String> {
    let messages_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".claude")
        .join("hub")
        .join("messages");

    if !messages_dir.exists() {
        return Ok(Vec::new());
    }

    let mut all_messages: Vec<HubMessageDto> = Vec::new();

    let entries = std::fs::read_dir(&messages_dir)
        .map_err(|e| format!("Cannot read messages dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(messages) = serde_json::from_str::<Vec<HubMessageDto>>(&content) {
                    all_messages.extend(messages);
                }
            }
        }
    }

    all_messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(all_messages)
}

#[tauri::command]
async fn broadcast_message(message: String) -> Result<SendMessageResult, String> {
    let messages_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".claude")
        .join("hub")
        .join("messages");

    std::fs::create_dir_all(&messages_dir)
        .map_err(|e| format!("Cannot create messages dir: {}", e))?;

    let broadcast_file = messages_dir.join("broadcast.json");

    let mut messages: Vec<HubMessageDto> = if broadcast_file.exists() {
        let content = std::fs::read_to_string(&broadcast_file)
            .map_err(|e| format!("Cannot read broadcast: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let message_id = format!("hub-all-{}", timestamp);

    let new_message = HubMessageDto {
        id: message_id.clone(),
        from: "hub".to_string(),
        to: "all".to_string(),
        content: message,
        message_type: "Broadcast".to_string(),
        timestamp,
        read: false,
    };

    messages.push(new_message);

    let json = serde_json::to_string_pretty(&messages)
        .map_err(|e| format!("Cannot serialize: {}", e))?;

    std::fs::write(&broadcast_file, json)
        .map_err(|e| format!("Cannot write broadcast: {}", e))?;

    Ok(SendMessageResult {
        success: true,
        message_id,
        target: "all".to_string(),
    })
}

#[tauri::command]
async fn clear_message_history() -> Result<(), String> {
    let messages_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".claude")
        .join("hub")
        .join("messages");

    if messages_dir.exists() {
        let entries = std::fs::read_dir(&messages_dir)
            .map_err(|e| format!("Cannot read messages dir: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("Cannot remove file: {}", e))?;
            }
        }
    }

    Ok(())
}

fn get_hub_dir() -> Result<std::path::PathBuf, String> {
    dirs::home_dir()
        .ok_or("Cannot find home directory".to_string())
        .map(|h| h.join(".claude").join("hub"))
}

fn ensure_hub_identity() -> Result<HubIdentityFile, String> {
    let hub_dir = get_hub_dir()?;
    let identity_file = hub_dir.join("identity.json");

    if identity_file.exists() {
        let content = std::fs::read_to_string(&identity_file)
            .map_err(|e| format!("Cannot read identity: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse identity: {}", e))
    } else {
        std::fs::create_dir_all(&hub_dir)
            .map_err(|e| format!("Cannot create hub dir: {}", e))?;

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "SENA Hub".to_string());

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let identity = HubIdentityFile {
            hub_id: uuid::Uuid::new_v4().to_string(),
            name: hostname.clone(),
            hostname,
            port: 9876,
            created_at: timestamp,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let json = serde_json::to_string_pretty(&identity)
            .map_err(|e| format!("Cannot serialize identity: {}", e))?;

        std::fs::write(&identity_file, json)
            .map_err(|e| format!("Cannot write identity: {}", e))?;

        Ok(identity)
    }
}

#[tauri::command]
async fn get_hub_identity() -> Result<HubIdentityDto, String> {
    let identity = ensure_hub_identity()?;

    Ok(HubIdentityDto {
        hub_id: identity.hub_id.clone(),
        name: identity.name,
        hostname: identity.hostname,
        port: identity.port,
        short_id: identity.hub_id.chars().take(8).collect(),
    })
}

#[tauri::command]
async fn set_hub_name(name: String) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Hub name cannot be empty".to_string());
    }

    if name.len() > 50 {
        return Err("Hub name too long (max 50 characters)".to_string());
    }

    let hub_dir = get_hub_dir()?;
    let identity_file = hub_dir.join("identity.json");

    let mut identity = ensure_hub_identity()?;
    identity.name = name;

    let json = serde_json::to_string_pretty(&identity)
        .map_err(|e| format!("Cannot serialize identity: {}", e))?;

    std::fs::write(&identity_file, json)
        .map_err(|e| format!("Cannot write identity: {}", e))
}

#[tauri::command]
async fn get_connected_peers() -> Result<Vec<ConnectedPeerDto>, String> {
    let hub_dir = get_hub_dir()?;
    let peers_file = hub_dir.join("peers.json");

    if !peers_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&peers_file)
        .map_err(|e| format!("Cannot read peers file: {}", e))?;

    let data: PeersFile = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse peers file: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let peers: Vec<ConnectedPeerDto> = data.connected_hubs.into_values()
        .map(|h| {
            let connected_secs = now.saturating_sub(h.connected_at);
            let connected_since = if connected_secs < 60 {
                format!("{}s", connected_secs)
            } else if connected_secs < 3600 {
                format!("{}m", connected_secs / 60)
            } else if connected_secs < 86400 {
                format!("{}h", connected_secs / 3600)
            } else {
                format!("{}d", connected_secs / 86400)
            };

            ConnectedPeerDto {
                hub_id: h.hub_id,
                name: h.name,
                address: h.address,
                port: h.port,
                is_online: now.saturating_sub(h.last_seen) < 60,
                session_count: h.session_count,
                connected_since,
            }
        })
        .collect();

    Ok(peers)
}

#[tauri::command]
async fn get_pending_requests() -> Result<Vec<ConnectionRequestDto>, String> {
    let hub_dir = get_hub_dir()?;
    let peers_file = hub_dir.join("peers.json");

    if !peers_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&peers_file)
        .map_err(|e| format!("Cannot read peers file: {}", e))?;

    let data: PeersFile = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse peers file: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let requests: Vec<ConnectionRequestDto> = data.pending_requests
        .into_iter()
        .filter(|r| r.expires_at > now)
        .map(|r| ConnectionRequestDto {
            request_id: r.request_id,
            from_hub_id: r.from_hub_id,
            from_hub_name: r.from_hub_name,
            from_address: r.from_address,
            message: r.message,
            created_at: r.created_at,
        })
        .collect();

    Ok(requests)
}

#[tauri::command]
async fn approve_peer_request(request_id: String) -> Result<ConnectedPeerDto, String> {
    let hub_dir = get_hub_dir()?;
    let peers_file = hub_dir.join("peers.json");

    if !peers_file.exists() {
        return Err("No peers file found".to_string());
    }

    let content = std::fs::read_to_string(&peers_file)
        .map_err(|e| format!("Cannot read peers file: {}", e))?;

    let mut data: PeersFile = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse peers file: {}", e))?;

    let request = data.pending_requests.iter()
        .find(|r| r.request_id == request_id)
        .ok_or_else(|| format!("Request {} not found", request_id))?
        .clone();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if request.expires_at < now {
        data.pending_requests.retain(|r| r.request_id != request_id);
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Cannot serialize: {}", e))?;
        std::fs::write(&peers_file, json)
            .map_err(|e| format!("Cannot write: {}", e))?;
        return Err("Request has expired".to_string());
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let token_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let auth_token = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &token_bytes,
    );

    let connected_hub = ConnectedHubData {
        hub_id: request.from_hub_id.clone(),
        name: request.from_hub_name.clone(),
        address: request.from_address.clone(),
        port: request.from_port,
        auth_token,
        connected_at: now,
        last_seen: now,
        session_count: 0,
    };

    data.connected_hubs.insert(request.from_hub_id.clone(), connected_hub.clone());
    data.pending_requests.retain(|r| r.request_id != request_id);
    data.last_updated = now;

    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Cannot serialize: {}", e))?;

    std::fs::write(&peers_file, json)
        .map_err(|e| format!("Cannot write: {}", e))?;

    Ok(ConnectedPeerDto {
        hub_id: connected_hub.hub_id,
        name: connected_hub.name,
        address: connected_hub.address,
        port: connected_hub.port,
        is_online: true,
        session_count: 0,
        connected_since: "0s".to_string(),
    })
}

#[tauri::command]
async fn reject_peer_request(request_id: String) -> Result<(), String> {
    let hub_dir = get_hub_dir()?;
    let peers_file = hub_dir.join("peers.json");

    if !peers_file.exists() {
        return Err("No peers file found".to_string());
    }

    let content = std::fs::read_to_string(&peers_file)
        .map_err(|e| format!("Cannot read peers file: {}", e))?;

    let mut data: PeersFile = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse peers file: {}", e))?;

    let exists = data.pending_requests.iter().any(|r| r.request_id == request_id);
    if !exists {
        return Err(format!("Request {} not found", request_id));
    }

    data.pending_requests.retain(|r| r.request_id != request_id);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    data.last_updated = now;

    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Cannot serialize: {}", e))?;

    std::fs::write(&peers_file, json)
        .map_err(|e| format!("Cannot write: {}", e))
}

#[tauri::command]
async fn disconnect_peer(hub_id: String) -> Result<(), String> {
    let hub_dir = get_hub_dir()?;
    let peers_file = hub_dir.join("peers.json");

    if !peers_file.exists() {
        return Err("No peers file found".to_string());
    }

    let content = std::fs::read_to_string(&peers_file)
        .map_err(|e| format!("Cannot read peers file: {}", e))?;

    let mut data: PeersFile = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse peers file: {}", e))?;

    if !data.connected_hubs.contains_key(&hub_id) {
        return Err(format!("Hub {} not connected", hub_id));
    }

    data.connected_hubs.remove(&hub_id);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    data.last_updated = now;

    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Cannot serialize: {}", e))?;

    std::fs::write(&peers_file, json)
        .map_err(|e| format!("Cannot write: {}", e))
}

#[tauri::command]
async fn get_federated_sessions() -> Result<Vec<FederatedSessionDto>, String> {
    let hub_dir = get_hub_dir()?;
    let identity = ensure_hub_identity()?;

    let mut all_sessions: Vec<FederatedSessionDto> = Vec::new();

    let sessions_path = hub_dir.join("sessions.json");
    if sessions_path.exists() {
        let content = std::fs::read_to_string(&sessions_path)
            .map_err(|e| format!("Cannot read sessions: {}", e))?;

        if let Ok(data) = serde_json::from_str::<CliSessionsFile>(&content) {
            for session in data.sessions.into_values() {
                all_sessions.push(FederatedSessionDto {
                    hub_id: identity.hub_id.clone(),
                    hub_name: identity.name.clone(),
                    session_id: session.id,
                    session_name: session.name,
                    role: session.role,
                    status: session.status,
                    working_on: session.working_on,
                    working_directory: session.working_directory,
                    is_local: true,
                });
            }
        }
    }

    Ok(all_sessions)
}

#[tauri::command]
async fn get_hub_passkey() -> Result<String, String> {
    let hub_dir = get_hub_dir()?;
    let passkey_file = hub_dir.join("passkey.txt");

    if !passkey_file.exists() {
        return Err("No passkey generated".to_string());
    }

    std::fs::read_to_string(&passkey_file)
        .map_err(|e| format!("Cannot read passkey: {}", e))
}

#[tauri::command]
async fn generate_hub_passkey() -> Result<String, String> {
    use rand::Rng;

    let hub_dir = get_hub_dir()?;
    let passkey_file = hub_dir.join("passkey.txt");

    let mut rng = rand::thread_rng();
    let passkey_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let passkey = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &passkey_bytes,
    );

    std::fs::write(&passkey_file, &passkey)
        .map_err(|e| format!("Cannot write passkey: {}", e))?;

    Ok(passkey)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_providers,
            get_provider_status,
            get_models,
            send_chat,
            set_default_provider,
            test_provider,
            create_session,
            list_sessions,
            list_cli_sessions,
            send_message_to_session,
            get_all_messages,
            broadcast_message,
            clear_message_history,
            get_health,
            get_version,
            get_hub_identity,
            set_hub_name,
            get_connected_peers,
            get_pending_requests,
            approve_peer_request,
            reject_peer_request,
            disconnect_peer,
            get_federated_sessions,
            get_hub_passkey,
            generate_hub_passkey,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
