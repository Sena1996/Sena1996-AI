use serde::Serialize;
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
            get_health,
            get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
