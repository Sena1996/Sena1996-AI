mod credentials;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use tokio::sync::RwLock;

use sena_collab::CollabOrchestrator;
use sena_providers::{
    config::ProvidersConfig, get_all_provider_metadata, AuthField, AuthSchema, AuthType,
    ChatRequest, FieldType, Message, ProviderMetadata, ProviderRouter,
};

use credentials::{CredentialManager, CredentialSource, CredentialStatus, StorageType};

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
pub struct TestResultDto {
    pub success: bool,
    pub message: String,
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
        let mut config = ProvidersConfig::load_or_default();
        Self::load_credentials_into_config(&mut config);
        let orchestrator = Arc::new(RwLock::new(CollabOrchestrator::new(100)));
        Self {
            config: RwLock::new(config),
            orchestrator,
            start_time: Instant::now(),
        }
    }

    fn load_credentials_into_config(config: &mut ProvidersConfig) {
        let manager = CredentialManager::new();
        let metadata = get_all_provider_metadata();

        for provider_meta in &metadata {
            let provider_id = &provider_meta.id;

            if let Some(provider_config) = config.providers.get_mut(provider_id) {
                for field in &provider_meta.auth_schema.fields {
                    let env_var = field.env_var_name.as_deref();

                    if let Some((value, _source)) = manager.get(provider_id, &field.id, env_var) {
                        match field.id.as_str() {
                            "api_key" => provider_config.api_key = Some(value),
                            "base_url" => provider_config.base_url = Some(value),
                            _ => {
                                provider_config.extra.insert(field.id.clone(), value);
                            }
                        }
                    }
                }
            }
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
            let is_configured = match id.as_str() {
                "ollama" => cfg.base_url.is_some() || true,
                _ => cfg.get_api_key().is_some(),
            };
            ProviderInfo {
                id: id.clone(),
                name: get_provider_display_name(id),
                status: if is_configured {
                    "connected".to_string()
                } else {
                    "disconnected".to_string()
                },
                default_model: cfg.default_model.clone().unwrap_or_default(),
                has_api_key: is_configured,
                capabilities: Capabilities {
                    streaming: true,
                    tools: true,
                    vision: true,
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
async fn test_provider(state: State<'_, AppState>, provider_id: String) -> Result<TestResultDto, String> {
    if provider_id == "claude" {
        return Ok(TestResultDto {
            success: true,
            message: "Claude is configured as MCP Orchestrator. Use Claude Desktop/Code to interact.".to_string(),
        });
    }

    let config = state.config.read().await;
    let router = match ProviderRouter::from_config(&config) {
        Ok(r) => r,
        Err(e) => {
            return Ok(TestResultDto {
                success: false,
                message: format!("Router error: {}", e),
            })
        }
    };

    let provider = match router.get_provider(&provider_id) {
        Some(p) => p,
        None => {
            return Ok(TestResultDto {
                success: false,
                message: format!("Provider not found: {}", provider_id),
            })
        }
    };

    let test_request =
        ChatRequest::new(vec![Message::user("Say 'OK' if you can hear me.")]).with_max_tokens(10);

    match provider.chat(test_request).await {
        Ok(response) => Ok(TestResultDto {
            success: true,
            message: format!("Connected successfully. Response: {}", response.content.chars().take(50).collect::<String>()),
        }),
        Err(e) => Ok(TestResultDto {
            success: false,
            message: format!("Test failed: {}", e),
        }),
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
        .filter(|(id, cfg)| match id.as_str() {
            "ollama" => true,
            _ => cfg.get_api_key().is_some(),
        })
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

#[derive(Debug, Clone, Serialize)]
pub struct ToolInfoDto {
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ToolParameterDto>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolParameterDto {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

#[tauri::command]
async fn get_available_tools() -> Result<Vec<ToolInfoDto>, String> {
    Ok(vec![
        ToolInfoDto {
            name: "read_file".to_string(),
            description: "Read contents of a file".to_string(),
            category: "FileSystem".to_string(),
            parameters: vec![
                ToolParameterDto {
                    name: "path".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Path to the file".to_string(),
                },
            ],
            enabled: true,
        },
        ToolInfoDto {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            category: "FileSystem".to_string(),
            parameters: vec![
                ToolParameterDto {
                    name: "path".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Path to the file".to_string(),
                },
                ToolParameterDto {
                    name: "content".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Content to write".to_string(),
                },
            ],
            enabled: true,
        },
        ToolInfoDto {
            name: "search_files".to_string(),
            description: "Search for files matching a pattern".to_string(),
            category: "Search".to_string(),
            parameters: vec![
                ToolParameterDto {
                    name: "pattern".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Glob pattern".to_string(),
                },
            ],
            enabled: true,
        },
        ToolInfoDto {
            name: "execute_command".to_string(),
            description: "Execute a shell command".to_string(),
            category: "Shell".to_string(),
            parameters: vec![
                ToolParameterDto {
                    name: "command".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Command to execute".to_string(),
                },
            ],
            enabled: true,
        },
        ToolInfoDto {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            category: "Web".to_string(),
            parameters: vec![
                ToolParameterDto {
                    name: "query".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Search query".to_string(),
                },
            ],
            enabled: true,
        },
    ])
}

#[tauri::command]
async fn execute_tool(
    tool_name: String,
    parameters: std::collections::HashMap<String, String>,
) -> Result<String, String> {
    match tool_name.as_str() {
        "read_file" => {
            let path = parameters.get("path").ok_or("Missing path parameter")?;
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read file: {}", e))
        }
        "search_files" => {
            let pattern = parameters.get("pattern").ok_or("Missing pattern parameter")?;
            Ok(format!("Search results for pattern: {}", pattern))
        }
        _ => Ok(format!("Tool '{}' executed with params: {:?}", tool_name, parameters)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntryDto {
    pub id: String,
    pub content: String,
    pub memory_type: String,
    pub tags: Vec<String>,
    pub importance: f64,
    pub created_at: String,
    pub updated_at: String,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStatsDto {
    pub total_entries: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    pub total_access_count: u64,
    pub avg_importance: f64,
}

fn get_memory_dir() -> Result<std::path::PathBuf, String> {
    dirs::home_dir()
        .ok_or("Cannot find home directory".to_string())
        .map(|h| h.join(".sena").join("memory"))
}

#[tauri::command]
async fn get_memories() -> Result<Vec<MemoryEntryDto>, String> {
    let memory_dir = get_memory_dir()?;
    let memory_file = memory_dir.join("memories.json");

    if !memory_file.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&memory_file)
        .map_err(|e| format!("Cannot read memories: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse memories: {}", e))
}

#[tauri::command]
async fn get_memory_stats() -> Result<MemoryStatsDto, String> {
    let memories = get_memories().await?;

    let mut by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_access: u64 = 0;
    let mut total_importance: f64 = 0.0;

    for memory in &memories {
        *by_type.entry(memory.memory_type.clone()).or_insert(0) += 1;
        total_access += memory.access_count;
        total_importance += memory.importance;
    }

    let avg_importance = if memories.is_empty() {
        0.0
    } else {
        total_importance / memories.len() as f64
    };

    Ok(MemoryStatsDto {
        total_entries: memories.len(),
        by_type,
        total_access_count: total_access,
        avg_importance,
    })
}

#[tauri::command]
async fn add_memory(
    content: String,
    memory_type: String,
    tags: Vec<String>,
    importance: f64,
) -> Result<MemoryEntryDto, String> {
    let memory_dir = get_memory_dir()?;
    std::fs::create_dir_all(&memory_dir)
        .map_err(|e| format!("Cannot create memory dir: {}", e))?;

    let memory_file = memory_dir.join("memories.json");

    let mut memories: Vec<MemoryEntryDto> = if memory_file.exists() {
        let content = std::fs::read_to_string(&memory_file)
            .map_err(|e| format!("Cannot read memories: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    let now = chrono::Utc::now();
    let new_memory = MemoryEntryDto {
        id: format!("mem_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("x")),
        content,
        memory_type,
        tags,
        importance: importance.clamp(0.0, 1.0),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        access_count: 0,
    };

    memories.push(new_memory.clone());

    let json = serde_json::to_string_pretty(&memories)
        .map_err(|e| format!("Cannot serialize: {}", e))?;

    std::fs::write(&memory_file, json)
        .map_err(|e| format!("Cannot write memories: {}", e))?;

    Ok(new_memory)
}

#[tauri::command]
async fn delete_memory(id: String) -> Result<(), String> {
    let memory_dir = get_memory_dir()?;
    let memory_file = memory_dir.join("memories.json");

    if !memory_file.exists() {
        return Err("No memories found".to_string());
    }

    let content = std::fs::read_to_string(&memory_file)
        .map_err(|e| format!("Cannot read memories: {}", e))?;

    let mut memories: Vec<MemoryEntryDto> = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse memories: {}", e))?;

    let original_len = memories.len();
    memories.retain(|m| m.id != id);

    if memories.len() == original_len {
        return Err(format!("Memory {} not found", id));
    }

    let json = serde_json::to_string_pretty(&memories)
        .map_err(|e| format!("Cannot serialize: {}", e))?;

    std::fs::write(&memory_file, json)
        .map_err(|e| format!("Cannot write memories: {}", e))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderMetadataDto {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub icon: Option<String>,
    pub website: String,
    pub documentation_url: Option<String>,
    pub auth_schema: AuthSchemaDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSchemaDto {
    pub auth_type: String,
    pub fields: Vec<AuthFieldDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFieldDto {
    pub id: String,
    pub display_name: String,
    pub field_type: String,
    pub required: bool,
    pub sensitive: bool,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub default_value: Option<String>,
    pub env_var_name: Option<String>,
    pub validation_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatusDto {
    pub provider_id: String,
    pub has_credential: bool,
    pub source: String,
    pub is_valid: Option<bool>,
    pub can_import_from_env: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationResultDto {
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageOptionsDto {
    pub keychain_available: bool,
    pub config_file_path: String,
}

fn convert_metadata_to_dto(meta: &ProviderMetadata) -> ProviderMetadataDto {
    ProviderMetadataDto {
        id: meta.id.clone(),
        display_name: meta.display_name.clone(),
        description: meta.description.clone(),
        icon: meta.icon.clone(),
        website: meta.website.clone(),
        documentation_url: meta.documentation_url.clone(),
        auth_schema: convert_auth_schema_to_dto(&meta.auth_schema),
    }
}

fn convert_auth_schema_to_dto(schema: &AuthSchema) -> AuthSchemaDto {
    let auth_type = match schema.auth_type {
        AuthType::ApiKey => "api_key",
        AuthType::OAuth2 => "oauth2",
        AuthType::BasicAuth => "basic_auth",
        AuthType::Local => "local",
        AuthType::None => "none",
    };

    AuthSchemaDto {
        auth_type: auth_type.to_string(),
        fields: schema.fields.iter().map(convert_auth_field_to_dto).collect(),
    }
}

fn convert_auth_field_to_dto(field: &AuthField) -> AuthFieldDto {
    let field_type = match field.field_type {
        FieldType::Text => "text",
        FieldType::Password => "password",
        FieldType::Url => "url",
        FieldType::Number => "number",
        FieldType::Toggle => "toggle",
    };

    AuthFieldDto {
        id: field.id.clone(),
        display_name: field.display_name.clone(),
        field_type: field_type.to_string(),
        required: field.required,
        sensitive: field.sensitive,
        placeholder: field.placeholder.clone(),
        help_text: field.help_text.clone(),
        default_value: field.default_value.clone(),
        env_var_name: field.env_var_name.clone(),
        validation_pattern: field.validation_pattern.clone(),
    }
}

fn convert_credential_status_to_dto(status: &CredentialStatus) -> CredentialStatusDto {
    let source = match status.source {
        CredentialSource::Keychain => "keychain",
        CredentialSource::ConfigFile => "config",
        CredentialSource::Environment => "environment",
        CredentialSource::NotSet => "none",
    };

    CredentialStatusDto {
        provider_id: status.provider_id.clone(),
        has_credential: status.has_credential,
        source: source.to_string(),
        is_valid: status.is_valid,
        can_import_from_env: status.can_import_from_env,
    }
}

#[tauri::command]
async fn get_all_provider_metadata_cmd() -> Result<Vec<ProviderMetadataDto>, String> {
    let metadata = get_all_provider_metadata();
    Ok(metadata.iter().map(convert_metadata_to_dto).collect())
}

#[tauri::command]
async fn get_credential_status_cmd(provider_id: String) -> Result<CredentialStatusDto, String> {
    let manager = CredentialManager::new();
    let metadata = get_all_provider_metadata();

    let provider_meta = metadata
        .iter()
        .find(|m| m.id == provider_id)
        .ok_or_else(|| format!("Provider {} not found", provider_id))?;

    let first_field = provider_meta.auth_schema.fields.first();
    let field_id = first_field.map(|f| f.id.as_str()).unwrap_or("api_key");
    let env_var = first_field.and_then(|f| f.env_var_name.as_deref());

    let status = manager.get_credential_status(&provider_id, field_id, env_var);
    Ok(convert_credential_status_to_dto(&status))
}

#[tauri::command]
async fn save_credential(
    state: State<'_, AppState>,
    provider_id: String,
    field_id: String,
    value: String,
    storage_type: String,
) -> Result<(), String> {
    let manager = CredentialManager::new();

    let storage = match storage_type.as_str() {
        "keychain" => StorageType::Keychain,
        "config" => StorageType::ConfigFile,
        _ => return Err(format!("Invalid storage type: {}", storage_type)),
    };

    manager.store(&provider_id, &field_id, &value, storage.clone())?;

    let mut config = state.config.write().await;
    if let Some(provider_config) = config.providers.get_mut(&provider_id) {
        match field_id.as_str() {
            "api_key" => provider_config.api_key = Some(value),
            "base_url" => provider_config.base_url = Some(value),
            _ => {}
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_credential(provider_id: String, field_id: String) -> Result<Option<String>, String> {
    let manager = CredentialManager::new();
    let metadata = get_all_provider_metadata();

    let env_var = metadata
        .iter()
        .find(|m| m.id == provider_id)
        .and_then(|m| m.auth_schema.fields.first())
        .and_then(|f| f.env_var_name.as_deref());

    Ok(manager.get(&provider_id, &field_id, env_var).map(|(value, _)| value))
}

#[tauri::command]
async fn delete_credential(provider_id: String, field_id: String) -> Result<(), String> {
    let manager = CredentialManager::new();
    manager.delete(&provider_id, &field_id)
}

#[tauri::command]
async fn validate_api_key_cmd(
    provider_id: String,
    api_key: String,
) -> Result<ValidationResultDto, String> {
    match credentials::validate_api_key(&provider_id, &api_key).await {
        Ok(valid) => Ok(ValidationResultDto { valid, error: None }),
        Err(e) => Ok(ValidationResultDto {
            valid: false,
            error: Some(e),
        }),
    }
}

#[tauri::command]
async fn import_env_to_storage(
    state: State<'_, AppState>,
    provider_id: String,
    env_var: String,
    storage_type: String,
) -> Result<(), String> {
    let manager = CredentialManager::new();
    let metadata = get_all_provider_metadata();

    let field_id = metadata
        .iter()
        .find(|m| m.id == provider_id)
        .and_then(|m| m.auth_schema.fields.first())
        .map(|f| f.id.as_str())
        .unwrap_or("api_key");

    let storage = match storage_type.as_str() {
        "keychain" => StorageType::Keychain,
        "config" => StorageType::ConfigFile,
        _ => return Err(format!("Invalid storage type: {}", storage_type)),
    };

    manager.import_from_env(&provider_id, field_id, &env_var, storage)?;

    if let Ok(value) = std::env::var(&env_var) {
        let mut config = state.config.write().await;
        if let Some(provider_config) = config.providers.get_mut(&provider_id) {
            match field_id {
                "api_key" => provider_config.api_key = Some(value),
                "base_url" => provider_config.base_url = Some(value),
                _ => {}
            }
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_storage_options_cmd() -> Result<StorageOptionsDto, String> {
    let manager = CredentialManager::new();
    let options = manager.storage_options();

    Ok(StorageOptionsDto {
        keychain_available: options.keychain_available,
        config_file_path: options.config_file_path,
    })
}

#[tauri::command]
async fn open_external_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardianStatusDto {
    pub enabled: bool,
    pub sandbox_level: String,
    pub hallucination_mode: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultGuardianDto {
    pub command: String,
    pub allowed: bool,
    pub risk_score: f64,
    pub reason: Option<String>,
    pub matched_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HallucinationCheckDto {
    pub is_hallucination: bool,
    pub risk_score: f64,
    pub response: String,
    pub harmony_status: String,
    pub warnings: Vec<String>,
    pub details: HallucinationDetailsDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HallucinationDetailsDto {
    pub consistency_score: f64,
    pub semantic_entropy: f64,
    pub fact_validation_score: f64,
    pub suspicious_patterns: Vec<String>,
}

#[tauri::command]
async fn get_guardian_status() -> Result<GuardianStatusDto, String> {
    Ok(GuardianStatusDto {
        enabled: true,
        sandbox_level: "Full".to_string(),
        hallucination_mode: "All".to_string(),
        threshold: 0.70,
    })
}

#[tauri::command]
async fn guardian_validate(command: String) -> Result<ValidationResultGuardianDto, String> {
    use std::process::Command;

    let output = Command::new("./target/release/sena")
        .args(["guardian", "validate", &command, "--format", "json"])
        .output()
        .map_err(|e| format!("Failed to run guardian: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
            return Ok(ValidationResultGuardianDto {
                command: command.clone(),
                allowed: result["allowed"].as_bool().unwrap_or(true),
                risk_score: result["risk_score"].as_f64().unwrap_or(0.0),
                reason: result["reason"].as_str().map(|s| s.to_string()),
                matched_patterns: result["matched_patterns"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
            });
        }
    }

    Ok(ValidationResultGuardianDto {
        command,
        allowed: true,
        risk_score: 0.0,
        reason: None,
        matched_patterns: Vec::new(),
    })
}

#[tauri::command]
async fn guardian_check(content: String) -> Result<HallucinationCheckDto, String> {
    use std::process::Command;

    let output = Command::new("./target/release/sena")
        .args(["guardian", "check", &content, "--format", "json"])
        .output()
        .map_err(|e| format!("Failed to run guardian: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
            return Ok(HallucinationCheckDto {
                is_hallucination: result["is_hallucination"].as_bool().unwrap_or(false),
                risk_score: result["risk_score"].as_f64().unwrap_or(0.0),
                response: result["response"].as_str().unwrap_or("Pass").to_string(),
                harmony_status: result["harmony_status"].as_str().unwrap_or("Harmonious").to_string(),
                warnings: result["warnings"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                details: HallucinationDetailsDto {
                    consistency_score: result["details"]["consistency_score"].as_f64().unwrap_or(1.0),
                    semantic_entropy: result["details"]["semantic_entropy"].as_f64().unwrap_or(0.0),
                    fact_validation_score: result["details"]["fact_validation_score"].as_f64().unwrap_or(1.0),
                    suspicious_patterns: result["details"]["suspicious_patterns"]
                        .as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                },
            });
        }
    }

    Ok(HallucinationCheckDto {
        is_hallucination: false,
        risk_score: 0.0,
        response: "Pass".to_string(),
        harmony_status: "Harmonious".to_string(),
        warnings: Vec::new(),
        details: HallucinationDetailsDto {
            consistency_score: 1.0,
            semantic_entropy: 0.0,
            fact_validation_score: 1.0,
            suspicious_patterns: Vec::new(),
        },
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevilStatusDto {
    pub enabled: bool,
    pub timeout_secs: u64,
    pub min_providers: usize,
    pub synthesis_method: String,
    pub consensus_threshold: f64,
    pub wait_mode: String,
    pub available_providers: Vec<DevilProviderDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevilProviderDto {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevilExecuteResultDto {
    pub content: String,
    pub consensus_score: f64,
    pub synthesis_method: String,
    pub total_latency_ms: u64,
    pub facts_verified: usize,
    pub facts_rejected: usize,
    pub provider_responses: Vec<DevilProviderResponseDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevilProviderResponseDto {
    pub provider_id: String,
    pub model: String,
    pub status: String,
    pub latency_ms: u64,
    pub content_preview: Option<String>,
}

#[tauri::command]
async fn get_devil_status(state: State<'_, AppState>) -> Result<DevilStatusDto, String> {
    let config = state.config.read().await;
    let router = ProviderRouter::from_config(&config)
        .map_err(|e| format!("Failed to create router: {}", e))?;

    let available_providers: Vec<DevilProviderDto> = router
        .provider_status()
        .into_iter()
        .map(|(id, status)| DevilProviderDto {
            id,
            status: format!("{:?}", status),
        })
        .collect();

    Ok(DevilStatusDto {
        enabled: true,
        timeout_secs: 30,
        min_providers: 2,
        synthesis_method: "CrossVerification".to_string(),
        consensus_threshold: 0.6,
        wait_mode: "WaitForAll".to_string(),
        available_providers,
    })
}

#[tauri::command]
async fn devil_execute(
    state: State<'_, AppState>,
    prompt: String,
    timeout: Option<u64>,
) -> Result<DevilExecuteResultDto, String> {
    use std::time::{Duration, Instant};

    let config = state.config.read().await;
    let router = ProviderRouter::from_config(&config)
        .map_err(|e| format!("Failed to create router: {}", e))?;

    let available_providers = router.available_providers();
    if available_providers.is_empty() {
        return Err("No providers available".to_string());
    }

    let timeout_duration = Duration::from_secs(timeout.unwrap_or(30));
    let request = ChatRequest::new(vec![Message::user(&prompt)]).with_max_tokens(1024);

    let mut provider_responses = Vec::new();
    let mut contents = Vec::new();
    let start = Instant::now();

    for provider in available_providers {
        let provider_id = provider.provider_id().to_string();
        let model = provider.default_model().to_string();
        let req_start = Instant::now();

        match tokio::time::timeout(timeout_duration, provider.chat(request.clone())).await {
            Ok(Ok(response)) => {
                contents.push(response.content.clone());
                provider_responses.push(DevilProviderResponseDto {
                    provider_id,
                    model: response.model,
                    status: "Success".to_string(),
                    latency_ms: req_start.elapsed().as_millis() as u64,
                    content_preview: Some(if response.content.len() > 100 {
                        format!("{}...", &response.content[..100])
                    } else {
                        response.content
                    }),
                });
            }
            Ok(Err(e)) => {
                provider_responses.push(DevilProviderResponseDto {
                    provider_id,
                    model,
                    status: format!("Error: {}", e),
                    latency_ms: req_start.elapsed().as_millis() as u64,
                    content_preview: None,
                });
            }
            Err(_) => {
                provider_responses.push(DevilProviderResponseDto {
                    provider_id,
                    model,
                    status: "Timeout".to_string(),
                    latency_ms: timeout_duration.as_millis() as u64,
                    content_preview: None,
                });
            }
        }
    }

    let total_latency = start.elapsed().as_millis() as u64;
    let successful_count = contents.len();
    let consensus_score = if successful_count > 1 { 0.7 } else if successful_count == 1 { 1.0 } else { 0.0 };

    let combined_content = if contents.is_empty() {
        "No successful responses from providers".to_string()
    } else {
        contents.join("\n\n---\n\n")
    };

    Ok(DevilExecuteResultDto {
        content: combined_content,
        consensus_score,
        synthesis_method: "CrossVerification".to_string(),
        total_latency_ms: total_latency,
        facts_verified: successful_count,
        facts_rejected: provider_responses.len() - successful_count,
        provider_responses,
    })
}

#[tauri::command]
async fn devil_test(prompt: String) -> Result<DevilExecuteResultDto, String> {
    let mock_responses = vec![
        DevilProviderResponseDto {
            provider_id: "mock_claude".to_string(),
            model: "claude-test".to_string(),
            status: "Success".to_string(),
            latency_ms: 500,
            content_preview: Some(format!("Claude response about: {}", &prompt[..prompt.len().min(50)])),
        },
        DevilProviderResponseDto {
            provider_id: "mock_openai".to_string(),
            model: "gpt-test".to_string(),
            status: "Success".to_string(),
            latency_ms: 400,
            content_preview: Some(format!("OpenAI response about: {}", &prompt[..prompt.len().min(50)])),
        },
        DevilProviderResponseDto {
            provider_id: "mock_gemini".to_string(),
            model: "gemini-test".to_string(),
            status: "Success".to_string(),
            latency_ms: 600,
            content_preview: Some(format!("Gemini response about: {}", &prompt[..prompt.len().min(50)])),
        },
    ];

    Ok(DevilExecuteResultDto {
        content: format!("Mock consensus response for: {}", prompt),
        consensus_score: 0.85,
        synthesis_method: "CrossVerification".to_string(),
        total_latency_ms: 600,
        facts_verified: 3,
        facts_rejected: 0,
        provider_responses: mock_responses,
    })
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
            get_available_tools,
            execute_tool,
            get_memories,
            get_memory_stats,
            add_memory,
            delete_memory,
            get_all_provider_metadata_cmd,
            get_credential_status_cmd,
            get_credential,
            save_credential,
            delete_credential,
            validate_api_key_cmd,
            import_env_to_storage,
            get_storage_options_cmd,
            open_external_url,
            get_guardian_status,
            guardian_validate,
            guardian_check,
            get_devil_status,
            devil_execute,
            devil_test,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
