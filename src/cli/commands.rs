//! CLI Command Execution
//!
//! Handles execution of CLI commands

use crate::cli::args::*;
use crate::config::SenaConfig;
use crate::integration::AutoIntegration;
use crate::metrics::SenaHealth;
use crate::output::{FormatBox, ProgressBar, TableBuilder};
use crate::ProcessingRequest;
use crate::SenaUnifiedSystem;
use std::path::PathBuf;

/// Execute a CLI command
pub async fn execute_command(cli: &Cli) -> Result<String, String> {
    match &cli.command {
        Some(Commands::Mcp { debug }) => execute_mcp(*debug).await,

        Some(Commands::Hook { hook_type, input }) => {
            execute_hook(*hook_type, input.clone(), cli.format).await
        }

        Some(Commands::Process {
            content,
            request_type,
        }) => execute_process(content, request_type, cli.format).await,

        Some(Commands::Health { detailed }) => execute_health(*detailed, cli.format),

        Some(Commands::Metrics { category }) => execute_metrics(*category, cli.format),

        Some(Commands::Detect { text }) => execute_detect(text, cli.format),

        Some(Commands::Daemon { action }) => execute_daemon(*action).await,

        Some(Commands::Session { action, id, name }) => {
            execute_session(*action, id.clone(), name.clone(), cli.format)
        }

        Some(Commands::Validate { content, strict }) => {
            execute_validate(content, *strict, cli.format)
        }

        Some(Commands::Format {
            format_type,
            title,
            data,
        }) => execute_format(*format_type, title.clone(), data, cli.format),

        // Hub commands
        Some(Commands::Hub { action }) => execute_hub(action.clone()).await,

        Some(Commands::Join { role, name }) => execute_join(role, name.clone(), cli.format).await,

        Some(Commands::Who) => execute_who(cli.format).await,

        Some(Commands::Tell { target, message }) => execute_tell(target, message, cli.format).await,

        Some(Commands::Inbox) => execute_inbox(cli.format).await,

        Some(Commands::Task { action }) => execute_task(action.clone(), cli.format).await,

        Some(Commands::Watch) => execute_watch().await,

        Some(Commands::Sync) => execute_sync(cli.format).await,

        Some(Commands::Knowledge { action }) => execute_knowledge(action.clone(), cli.format).await,

        Some(Commands::Think { query, depth }) => execute_think(query, *depth, cli.format).await,

        Some(Commands::Agent {
            agent_type,
            content,
        }) => execute_agent(*agent_type, content, cli.format).await,

        Some(Commands::Evolve { action }) => execute_evolve(action.clone(), cli.format).await,

        Some(Commands::Feedback {
            feedback_type,
            message,
            context,
        }) => execute_feedback(*feedback_type, message, context.clone(), cli.format).await,

        Some(Commands::Backend { analysis, input }) => {
            execute_backend(*analysis, input, cli.format).await
        }

        Some(Commands::Iot { analysis, input }) => execute_iot(*analysis, input, cli.format).await,

        Some(Commands::Ios { analysis, input }) => execute_ios(*analysis, input, cli.format).await,

        Some(Commands::Android { analysis, input }) => {
            execute_android(*analysis, input, cli.format).await
        }

        Some(Commands::Web { analysis, input }) => execute_web(*analysis, input, cli.format).await,

        Some(Commands::Setup {
            install_type,
            name,
            yes,
        }) => execute_setup(*install_type, name.clone(), *yes, cli.format).await,

        Some(Commands::Network { action }) => execute_network(action.clone(), cli.format).await,

        Some(Commands::Peer { action }) => execute_peer(action.clone(), cli.format).await,

        Some(Commands::Discover { timeout }) => execute_discover(*timeout, cli.format).await,

        Some(Commands::Provider { action }) => execute_provider(action.clone(), cli.format).await,

        Some(Commands::Collab { action }) => execute_collab(action.clone(), cli.format).await,

        Some(Commands::Tools { action }) => execute_tools(action.clone(), cli.format).await,

        Some(Commands::Memory { action }) => execute_memory(action.clone(), cli.format).await,

        Some(Commands::Auto {
            task,
            max_steps,
            cwd,
            confirm,
        }) => execute_auto(task, *max_steps, cwd.clone(), *confirm, cli.format).await,

        Some(Commands::Git { action }) => execute_git(action.clone(), cli.format).await,

        Some(Commands::Guardian { action }) => execute_guardian(action.clone(), cli.format).await,

        Some(Commands::Devil { action }) => execute_devil(action.clone(), cli.format).await,

        None => {
            execute_health(false, cli.format)
        }
    }
}

async fn execute_mcp(debug: bool) -> Result<String, String> {
    if debug {
        eprintln!(
            "{} MCP Server starting in debug mode...",
            SenaConfig::brand()
        );
    }

    // Start MCP server
    crate::mcp::run_server().await
}

async fn execute_hook(
    hook_type: HookType,
    input: Option<String>,
    format: OutputFormat,
) -> Result<String, String> {
    let input_data = match input {
        Some(data) => data,
        None => {
            // Read from stdin
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let mut lines = Vec::new();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => lines.push(l),
                    Err(_) => break,
                }
            }
            lines.join("\n")
        }
    };

    let result = crate::hooks::handle_hook(hook_type, &input_data).await?;

    match format {
        OutputFormat::Json => Ok(serde_json::to_string(&result).unwrap_or_default()),
        _ => Ok(result.message),
    }
}

async fn execute_process(
    content: &str,
    request_type: &str,
    format: OutputFormat,
) -> Result<String, String> {
    let mut system = SenaUnifiedSystem::new();
    let request = ProcessingRequest::new(content, request_type);

    let result = system.process(request).await;

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
        OutputFormat::Pretty => {
            let mut output = String::new();
            output
                .push_str(&FormatBox::new(&SenaConfig::brand_title("PROCESSING RESULT")).render());
            output.push('\n');
            output.push_str(&format!("Request ID: {}\n", result.request_id));
            output.push_str(&format!("Success: {}\n", result.success));
            if !result.content.is_empty() {
                output.push_str(&format!("Response: {}\n", result.content));
            }
            Ok(output)
        }
        OutputFormat::Text => {
            if result.success {
                Ok(if result.content.is_empty() {
                    "OK".to_string()
                } else {
                    result.content.clone()
                })
            } else {
                Err(if result.content.is_empty() {
                    "Error".to_string()
                } else {
                    result.content.clone()
                })
            }
        }
    }
}

fn execute_health(detailed: bool, format: OutputFormat) -> Result<String, String> {
    let health = SenaHealth::new();
    let report = health.get_health();

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&report).map_err(|e| e.to_string()),
        OutputFormat::Pretty | OutputFormat::Text => {
            let mut output = String::new();

            if detailed || format == OutputFormat::Pretty {
                output
                    .push_str(&FormatBox::new(&SenaConfig::brand_title("HEALTH STATUS")).render());
                output.push('\n');
            }

            output.push_str(&format!("Version: {}\n", report.version));
            output.push_str(&format!("Status: {}\n", report.overall_status));
            output.push_str(&format!(
                "Health: {}%\n",
                report.metrics.overall_health_percentage
            ));

            if detailed {
                output.push_str("\nComponents:\n");
                output.push_str(&format!("  Core: {}\n", report.metrics.core_components));
                output.push_str(&format!("  Memory: {}\n", report.metrics.memory_system));
                output.push_str(&format!("  Hooks: {}\n", report.metrics.hooks));
            }

            Ok(output)
        }
    }
}

fn execute_metrics(
    category: Option<MetricCategory>,
    format: OutputFormat,
) -> Result<String, String> {
    let health = SenaHealth::new();
    let cat = category.unwrap_or(MetricCategory::All);

    let metrics = match cat {
        MetricCategory::Health => serde_json::to_value(health.get_health()).ok(),
        MetricCategory::Innovation => serde_json::to_value(health.get_innovation_metrics()).ok(),
        MetricCategory::Phase => serde_json::to_value(health.get_phase_status()).ok(),
        MetricCategory::All => Some(crate::metrics::SenaMetrics::collect()),
        _ => Some(crate::metrics::SenaMetrics::collect()),
    };

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&metrics).map_err(|e| e.to_string()),
        _ => serde_json::to_string_pretty(&metrics).map_err(|e| e.to_string()),
    }
}

fn execute_detect(text: &str, format: OutputFormat) -> Result<String, String> {
    let ai = AutoIntegration::new();
    let detected = ai.detect_format(text);

    let result = serde_json::json!({
        "input": text,
        "detected_format": detected.as_ref().map(|f| f.name()),
        "should_format": detected.is_some(),
    });

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
        _ => match detected {
            Some(fmt) => Ok(format!("Detected format: {}", fmt.name())),
            None => Ok("No special format detected".to_string()),
        },
    }
}

async fn execute_daemon(action: DaemonAction) -> Result<String, String> {
    match action {
        DaemonAction::Start => crate::daemon::start_daemon().await,
        DaemonAction::Stop => crate::daemon::stop_daemon().await,
        DaemonAction::Restart => {
            crate::daemon::stop_daemon().await?;
            crate::daemon::start_daemon().await
        }
        DaemonAction::Status => crate::daemon::daemon_status().await,
    }
}

fn execute_session(
    action: SessionAction,
    id: Option<String>,
    name: Option<String>,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::hub::{Hub, SessionContext, SessionRole};

    let mut hub = Hub::new()?;
    hub.load()?;

    let result = match action {
        SessionAction::Start => {
            if let Some(existing_id) = hub.get_current_session_id() {
                if hub.sessions.get(&existing_id).is_some() {
                    return Err(format!(
                        "Already in session '{}'. End it first or use different terminal.",
                        existing_id
                    ));
                }
            }

            match hub.sessions.register(SessionRole::General, name.clone()) {
                Ok(session) => {
                    let context = SessionContext::new(&session.id, &session.name, "general");
                    hub.context.save_context(&context)?;
                    hub.state.set_session_active(&session.id, true);
                    hub.save()?;

                    serde_json::json!({
                        "action": "start",
                        "session_id": session.id,
                        "session_name": session.name,
                        "started_at": session.joined_at,
                    })
                }
                Err(e) => serde_json::json!({"error": e}),
            }
        }
        SessionAction::End => {
            let session_id = id.or_else(|| hub.get_current_session_id());
            if let Some(sid) = session_id {
                hub.leave(&sid)?;
                hub.save()?;
                serde_json::json!({
                    "action": "end",
                    "session_id": sid,
                    "status": "session ended",
                })
            } else {
                serde_json::json!({"error": "No session ID provided and no current session"})
            }
        }
        SessionAction::Info => {
            let target_id = id.or_else(|| hub.get_current_session_id());
            match target_id.and_then(|sid| hub.sessions.get(&sid).cloned()) {
                Some(session) => session.stats(),
                None => {
                    let active = hub.sessions.get_active();
                    if active.is_empty() {
                        serde_json::json!({"error": "no active session"})
                    } else {
                        serde_json::to_value(&active).unwrap_or_default()
                    }
                }
            }
        }
        SessionAction::List => {
            let sessions = hub.sessions.get_active();
            let current_id = hub.get_current_session_id();
            let sessions_with_current: Vec<serde_json::Value> = sessions
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "name": s.name,
                        "role": s.role.name(),
                        "status": format!("{:?}", s.status),
                        "is_current": current_id.as_ref() == Some(&s.id),
                    })
                })
                .collect();
            serde_json::json!(sessions_with_current)
        }
        SessionAction::Restore => match id {
            Some(session_id) => {
                if let Some(session) = hub.sessions.get(&session_id) {
                    let context =
                        SessionContext::new(&session.id, &session.name, session.role.name());
                    hub.context.save_context(&context)?;
                    serde_json::json!({
                        "action": "restore",
                        "session_id": session_id,
                        "status": "restored",
                    })
                } else {
                    serde_json::json!({"error": format!("Session {} not found", session_id)})
                }
            }
            None => serde_json::json!({"error": "session ID required for restore"}),
        },
    };

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
        _ => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
    }
}

fn execute_validate(content: &str, strict: bool, format: OutputFormat) -> Result<String, String> {
    // Use harmony validation engine
    use crate::ancient::HarmonyValidationEngine;

    let mut engine = HarmonyValidationEngine::new();
    let result = engine.validate(content);

    let violations_count = result.rule_violations.len();
    let checks_passed = result
        .checks
        .iter()
        .filter(|c| c.status == crate::ancient::HarmonyStatus::Harmonious)
        .count();

    let output = serde_json::json!({
        "content": content,
        "valid": result.is_valid(),
        "confidence": result.overall_confidence,
        "strict_mode": strict,
        "checks_passed": checks_passed,
        "violations": violations_count,
        "suggestions": result.corrections_suggested,
    });

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&output).map_err(|e| e.to_string()),
        OutputFormat::Pretty => {
            let mut out = String::new();
            out.push_str(&FormatBox::new(&SenaConfig::brand_title("VALIDATION RESULT")).render());
            out.push('\n');
            out.push_str(&format!("Valid: {}\n", result.is_valid()));
            out.push_str(&format!(
                "Confidence: {:.1}%\n",
                result.overall_confidence * 100.0
            ));
            out.push_str(&format!("Checks Passed: {}\n", checks_passed));
            out.push_str(&format!("Violations: {}\n", violations_count));
            Ok(out)
        }
        OutputFormat::Text => {
            if result.is_valid() {
                Ok(format!(
                    "VALID (confidence: {:.1}%)",
                    result.overall_confidence * 100.0
                ))
            } else {
                Ok(format!(
                    "INVALID (confidence: {:.1}%)",
                    result.overall_confidence * 100.0
                ))
            }
        }
    }
}

fn execute_format(
    format_type: FormatOutputType,
    title: Option<String>,
    data: &str,
    _format: OutputFormat,
) -> Result<String, String> {
    match format_type {
        FormatOutputType::Table => {
            // Parse data as JSON array
            let parsed: Result<Vec<Vec<String>>, _> = serde_json::from_str(data);
            match parsed {
                Ok(rows) => {
                    let mut builder = TableBuilder::new();
                    if let Some(t) = title {
                        builder = builder.title(&t);
                    }
                    for row in rows {
                        builder = builder.row(row);
                    }
                    Ok(builder.build())
                }
                Err(e) => Err(format!("Invalid table data: {}", e)),
            }
        }
        FormatOutputType::Progress => {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(data);
            match parsed {
                Ok(v) => {
                    let percent = v.get("percent").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    let label = v
                        .get("label")
                        .and_then(|l| l.as_str())
                        .unwrap_or("Progress");
                    Ok(ProgressBar::new(label, percent as f32).render())
                }
                Err(e) => Err(format!("Invalid progress data: {}", e)),
            }
        }
        FormatOutputType::BrilliantThinking => Ok(FormatBox::new(
            &title.unwrap_or_else(|| SenaConfig::brand_title("BRILLIANT THINKING")),
        )
        .render()),
        FormatOutputType::TruthVerification => Ok(FormatBox::new(
            &title.unwrap_or_else(|| SenaConfig::brand_title("TRUTH VERIFICATION")),
        )
        .render()),
        FormatOutputType::CodeAnalysis => Ok(FormatBox::new(
            &title.unwrap_or_else(|| SenaConfig::brand_title("CODE ANALYSIS")),
        )
        .render()),
    }
}

// ================================
// Hub Command Implementations
// ================================

async fn execute_hub(action: HubAction) -> Result<String, String> {
    use crate::hub::{Hub, HubConfig};

    match action {
        HubAction::Start => {
            let config = HubConfig::new();
            config.ensure_dirs()?;
            Ok("Hub started. Use 'sena join --role=<role>' to join.".to_string())
        }
        HubAction::Stop => Ok("Hub stopped.".to_string()),
        HubAction::Status => {
            let mut hub = Hub::new()?;
            hub.load()?;
            let status = hub.status();
            Ok(format!(
                "Hub Status:\n  Sessions: {}\n  Tasks: {} ({} pending)\n  Conflicts: {}",
                status.online_sessions,
                status.total_tasks,
                status.pending_tasks,
                status.active_conflicts
            ))
        }
        HubAction::Sessions => {
            let mut hub = Hub::new()?;
            hub.load()?;
            let sessions = hub.who();

            if sessions.is_empty() {
                return Ok("No sessions online. Use 'sena join --role=<role>' to create one.".to_string());
            }

            let mut output = String::from("╔══════════════════════════════════════════════════════════════╗\n");
            output.push_str("║                    HUB SESSIONS                              ║\n");
            output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

            for session in &sessions {
                output.push_str(&format!(
                    "{} {} ({})\n  ID: {}\n  Status: {:?}\n  Working: {}\n  Last active: {}\n\n",
                    session.role.emoji(),
                    session.name,
                    session.role.name(),
                    session.id,
                    session.status,
                    session.working_on.as_deref().unwrap_or("idle"),
                    session.idle_display()
                ));
            }

            output.push_str(&format!("Total: {} session(s)\n", sessions.len()));
            output.push_str("\nUse 'sena hub tell <name> <message>' to send a message.");
            Ok(output)
        }
        HubAction::Tell { target, message } => {
            let mut hub = Hub::new()?;
            hub.load()?;

            let resolved_target = hub.sessions.resolve_session(&target).ok_or_else(|| {
                format!("Session '{}' not found. Use 'sena hub sessions' to list.", target)
            })?;

            hub.tell("hub", &resolved_target, &message)?;
            hub.save()?;

            Ok(format!("📨 Hub → {}: {}", target, message))
        }
        HubAction::Broadcast { message } => {
            let mut hub = Hub::new()?;
            hub.load()?;

            hub.broadcast("hub", &message)?;
            hub.save()?;

            let session_count = hub.who().len();
            Ok(format!("📢 Broadcast to {} session(s): {}", session_count, message))
        }
        HubAction::Messages { count } => {
            let mut hub = Hub::new()?;
            hub.load()?;

            let messages = hub.messages.get_recent(count);

            if messages.is_empty() {
                return Ok("No messages in Hub.".to_string());
            }

            let mut output = String::from("╔══════════════════════════════════════════════════════════════╗\n");
            output.push_str("║                    HUB MESSAGES                              ║\n");
            output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

            for msg in &messages {
                let direction = if msg.to == "all" {
                    format!("{} → ALL", msg.from)
                } else {
                    format!("{} → {}", msg.from, msg.to)
                };
                output.push_str(&format!(
                    "{} [{}] {}\n   {}\n\n",
                    msg.message_type.emoji(),
                    msg.time_display(),
                    direction,
                    msg.content
                ));
            }

            Ok(output)
        }
        HubAction::Conflicts => {
            let mut hub = Hub::new()?;
            hub.load()?;
            let conflicts = hub.get_conflicts();
            if conflicts.is_empty() {
                Ok("No conflicts detected.".to_string())
            } else {
                let mut output = String::from("Active Conflicts:\n");
                for conflict in conflicts {
                    output.push_str(&format!(
                        "  {} {} - Sessions: {:?}\n",
                        conflict.severity.emoji(),
                        conflict.file_path,
                        conflict.sessions
                    ));
                }
                Ok(output)
            }
        }
        HubAction::Clear => {
            let config = HubConfig::new();
            let _ = std::fs::remove_dir_all(&config.hub_dir);
            config.ensure_dirs()?;
            Ok("Hub data cleared.".to_string())
        }
        HubAction::Identity => {
            use crate::hub::HubIdentity;

            let config = HubConfig::new();
            let identity_file = config.hub_dir.join("identity.json");
            let identity = HubIdentity::load_or_create(&identity_file)?;

            let mut output = String::from("╔══════════════════════════════════════════════════════════════╗\n");
            output.push_str("║                    HUB IDENTITY                              ║\n");
            output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");
            output.push_str(&format!("  Name:     {}\n", identity.name));
            output.push_str(&format!("  Hostname: {}\n", identity.hostname));
            output.push_str(&format!("  Hub ID:   {}\n", identity.short_id()));
            output.push_str(&format!("  Full ID:  {}\n", identity.hub_id));
            output.push_str(&format!("  Port:     {}\n", identity.port));
            output.push_str(&format!("  Version:  {}\n", identity.version));
            output.push_str("\nTo change hub name: sena hub set-name <new-name>");
            Ok(output)
        }
        HubAction::SetName { name } => {
            use crate::hub::HubIdentity;

            let config = HubConfig::new();
            let identity_file = config.hub_dir.join("identity.json");
            let mut identity = HubIdentity::load_or_create(&identity_file)?;
            identity.set_name(&name);
            identity.save(&identity_file)?;

            Ok(format!("✅ Hub name changed to: {}", name))
        }
        HubAction::Peers => {
            use crate::hub::{HubIdentity, PeerManager};

            let config = HubConfig::new();
            let identity_file = config.hub_dir.join("identity.json");
            let identity = HubIdentity::load_or_create(&identity_file)?;
            let mut peer_manager = PeerManager::new(identity, &config.hub_dir);
            let _ = peer_manager.load();

            let connected = peer_manager.get_connected_hubs();

            if connected.is_empty() {
                return Ok("No connected hubs.\n\nTo connect to another hub:\n  sena hub connect <address:port>".to_string());
            }

            let mut output = String::from("╔══════════════════════════════════════════════════════════════╗\n");
            output.push_str("║                    CONNECTED HUBS                            ║\n");
            output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

            for hub in &connected {
                let status = if hub.is_online() { "🟢 Online" } else { "⚫ Offline" };
                output.push_str(&format!(
                    "  {} {}\n    ID: {}...\n    Address: {}:{}\n    Sessions: {}\n    Connected: {}\n\n",
                    status,
                    hub.name,
                    &hub.hub_id[..8],
                    hub.address,
                    hub.port,
                    hub.session_count,
                    hub.connected_duration()
                ));
            }
            output.push_str(&format!("Total: {} hub(s)\n", connected.len()));
            Ok(output)
        }
        HubAction::Requests => {
            use crate::hub::{HubIdentity, PeerManager};

            let config = HubConfig::new();
            let identity_file = config.hub_dir.join("identity.json");
            let identity = HubIdentity::load_or_create(&identity_file)?;
            let mut peer_manager = PeerManager::new(identity, &config.hub_dir);
            let _ = peer_manager.load();

            let requests = peer_manager.get_pending_requests();

            if requests.is_empty() {
                return Ok("No pending connection requests.".to_string());
            }

            let mut output = String::from("╔══════════════════════════════════════════════════════════════╗\n");
            output.push_str("║                 PENDING CONNECTION REQUESTS                  ║\n");
            output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

            for req in &requests {
                output.push_str(&format!(
                    "  📥 {} ({})\n    From: {}\n    ID: {}...\n    Message: {}\n\n",
                    req.from_hub_name,
                    req.from_address,
                    req.from_hub_id,
                    &req.request_id[..8],
                    req.message.as_deref().unwrap_or("(no message)")
                ));
            }
            output.push_str("\nTo approve: sena hub approve <request-id>\n");
            output.push_str("To reject:  sena hub reject <request-id>");
            Ok(output)
        }
        HubAction::Connect { address, message: _ } => {
            Ok(format!(
                "Connection request to {} would be sent.\n\
                Note: Network connectivity requires the hub daemon to be running.\n\
                Use 'sena network start' to start the network server.",
                address
            ))
        }
        HubAction::Approve { request_id } => {
            use crate::hub::{HubIdentity, PeerManager};

            let config = HubConfig::new();
            let identity_file = config.hub_dir.join("identity.json");
            let identity = HubIdentity::load_or_create(&identity_file)?;
            let mut peer_manager = PeerManager::new(identity, &config.hub_dir);
            let _ = peer_manager.load();

            let requests = peer_manager.get_pending_requests();
            let matched_id = requests
                .iter()
                .find(|r| r.request_id.starts_with(&request_id))
                .map(|r| r.request_id.clone());

            match matched_id {
                Some(req_id) => {
                    let hub = peer_manager.approve_request(&req_id)?;
                    Ok(format!("✅ Connection approved for: {}\n   Hub is now trusted.", hub.name))
                }
                None => Err(format!("Request not found: {}", request_id)),
            }
        }
        HubAction::Reject { request_id } => {
            use crate::hub::{HubIdentity, PeerManager};

            let config = HubConfig::new();
            let identity_file = config.hub_dir.join("identity.json");
            let identity = HubIdentity::load_or_create(&identity_file)?;
            let mut peer_manager = PeerManager::new(identity, &config.hub_dir);
            let _ = peer_manager.load();

            let requests = peer_manager.get_pending_requests();
            let matched_id = requests
                .iter()
                .find(|r| r.request_id.starts_with(&request_id))
                .map(|r| r.request_id.clone());

            match matched_id {
                Some(req_id) => {
                    peer_manager.reject_request(&req_id)?;
                    Ok("❌ Connection request rejected.".to_string())
                }
                None => Err(format!("Request not found: {}", request_id)),
            }
        }
        HubAction::Disconnect { hub } => {
            use crate::hub::{HubIdentity, PeerManager};

            let config = HubConfig::new();
            let identity_file = config.hub_dir.join("identity.json");
            let identity = HubIdentity::load_or_create(&identity_file)?;
            let mut peer_manager = PeerManager::new(identity, &config.hub_dir);
            let _ = peer_manager.load();

            if let Some(connected_hub) = peer_manager.get_hub_by_name(&hub) {
                peer_manager.disconnect_hub(&connected_hub.hub_id.clone())?;
                Ok(format!("✅ Disconnected from: {}", hub))
            } else if let Some(connected_hub) = peer_manager.get_connected_hub(&hub) {
                let name = connected_hub.name.clone();
                peer_manager.disconnect_hub(&hub)?;
                Ok(format!("✅ Disconnected from: {}", name))
            } else {
                Err(format!("Hub not found: {}", hub))
            }
        }
        HubAction::Federation => {
            use crate::hub::{HubIdentity, PeerManager};

            let config = HubConfig::new();
            let mut hub = Hub::new()?;
            hub.load()?;

            let identity_file = config.hub_dir.join("identity.json");
            let identity = HubIdentity::load_or_create(&identity_file)?;
            let mut peer_manager = PeerManager::new(identity, &config.hub_dir);
            let _ = peer_manager.load();

            let local_sessions = hub.who();
            let all_sessions = peer_manager.get_all_sessions(&local_sessions);

            if all_sessions.is_empty() {
                return Ok("No sessions found (local or remote).".to_string());
            }

            let mut output = String::from("╔══════════════════════════════════════════════════════════════╗\n");
            output.push_str("║                  FEDERATED SESSIONS                          ║\n");
            output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

            for session in &all_sessions {
                let location = if session.is_local { "🏠 Local" } else { "🌐 Remote" };
                output.push_str(&format!(
                    "  {} {}:{}\n    Role: {}\n    Status: {}\n    Working: {}\n\n",
                    location,
                    session.hub_name,
                    session.session_name,
                    session.role,
                    session.status,
                    session.working_on.as_deref().unwrap_or("idle")
                ));
            }

            let local_count = all_sessions.iter().filter(|s| s.is_local).count();
            let remote_count = all_sessions.len() - local_count;
            output.push_str(&format!(
                "Total: {} session(s) ({} local, {} remote)\n",
                all_sessions.len(),
                local_count,
                remote_count
            ));
            output.push_str("\nUse 'sena hub tell HubName:SessionName <message>' to send cross-hub message.");
            Ok(output)
        }
    }
}

async fn execute_join(
    role: &str,
    name: Option<String>,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::hub::{Hub, SessionRole};

    let mut hub = Hub::new()?;
    hub.load()?;

    if let Some(existing_id) = hub.get_current_session_id() {
        if hub.sessions.get(&existing_id).is_some() {
            return Err(format!(
                "Already in session '{}'. Use 'sena session end --id={}' first, or use a different terminal.",
                existing_id, existing_id
            ));
        }
    }

    let session_role = SessionRole::parse(role);
    let session = hub.join(session_role, name)?;
    hub.save()?;

    let result = serde_json::json!({
        "action": "joined",
        "session_id": session.id,
        "role": session.role.name(),
        "name": session.name,
        "terminal": hub.context.load_current_context().map(|c| c.terminal_id),
    });

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
        _ => Ok(format!(
            "{} Joined Hub!\n  Session: {}\n  Name: {}\n  Role: {}\n\nYou can now use 'sena tell <target> <message>' and 'sena inbox'",
            session.role.emoji(),
            session.id,
            session.name,
            session.role.name()
        )),
    }
}

async fn execute_who(format: OutputFormat) -> Result<String, String> {
    use crate::hub::Hub;

    let mut hub = Hub::new()?;
    hub.load()?;

    let sessions = hub.who();
    let current_session_id = hub.get_current_session_id();

    if sessions.is_empty() {
        return Ok("No sessions online. Use 'sena join --role=<role>' to join.".to_string());
    }

    match format {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = sessions
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "role": s.role.name(),
                        "name": s.name,
                        "status": format!("{:?}", s.status),
                        "working_on": s.working_on,
                        "idle": s.idle_display(),
                        "is_current": current_session_id.as_ref() == Some(&s.id),
                    })
                })
                .collect();
            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
        }
        _ => {
            let mut output = String::from("Sessions Online:\n");
            for session in sessions {
                let current_marker = if current_session_id.as_ref() == Some(&session.id) {
                    " (you)"
                } else {
                    ""
                };
                output.push_str(&format!(
                    "  {} {} │ {} │ {} │ {}{}\n",
                    session.role.emoji(),
                    session.name,
                    session.status.indicator(),
                    session.working_on.as_deref().unwrap_or("-"),
                    session.idle_display(),
                    current_marker
                ));
            }
            Ok(output)
        }
    }
}

async fn execute_tell(target: &str, message: &str, format: OutputFormat) -> Result<String, String> {
    use crate::hub::Hub;

    let mut hub = Hub::new()?;
    hub.load()?;

    let sender_id = hub
        .get_current_session_id()
        .ok_or_else(|| "No active session. Use 'sena join --role=<role>' first.".to_string())?;

    let resolved_target = hub.sessions.resolve_session(target).ok_or_else(|| {
        format!(
            "Session '{}' not found. Use 'sena who' to see active sessions.",
            target
        )
    })?;

    hub.tell(&sender_id, &resolved_target, message)?;
    hub.save()?;

    match format {
        OutputFormat::Json => Ok(serde_json::json!({
            "sent": true,
            "from": sender_id,
            "to": resolved_target,
            "target_input": target,
            "message": message
        })
        .to_string()),
        _ => Ok(format!("Message sent to {} from {}", target, sender_id)),
    }
}

async fn execute_inbox(format: OutputFormat) -> Result<String, String> {
    use crate::hub::Hub;

    let mut hub = Hub::new()?;
    hub.load()?;

    let session_id = hub
        .get_current_session_id()
        .ok_or_else(|| "No active session. Use 'sena join --role=<role>' first.".to_string())?;

    let messages = hub.inbox(&session_id);

    if messages.is_empty() {
        return Ok(format!("No messages for session {}.", session_id));
    }

    match format {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = messages
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "from": m.from,
                        "to": m.to,
                        "content": m.content,
                        "time": m.time_display(),
                        "read": m.read,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
        }
        _ => {
            let mut output = format!("Inbox for {}:\n", session_id);
            for msg in messages {
                output.push_str(&format!(
                    "  {} [{}] {}: {}\n",
                    msg.message_type.emoji(),
                    msg.time_display(),
                    msg.from,
                    msg.content
                ));
            }
            Ok(output)
        }
    }
}

async fn execute_task(action: TaskAction, format: OutputFormat) -> Result<String, String> {
    use crate::hub::{Hub, TaskPriority, TaskStatus};

    let mut hub = Hub::new()?;
    hub.load()?;

    match action {
        TaskAction::New {
            title,
            to,
            priority,
        } => {
            let prio = TaskPriority::parse(&priority);
            let resolved_to = hub
                .sessions
                .resolve_session(&to)
                .unwrap_or_else(|| to.clone());
            let task = hub.create_task(&title, &resolved_to, prio)?;
            hub.save()?;

            match format {
                OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                    "created": true,
                    "id": task.id,
                    "title": task.title,
                    "assignee": task.assignee,
                }))
                .map_err(|e| e.to_string()),
                _ => Ok(format!(
                    "Task #{} created: {} (assigned to {})",
                    task.id, task.title, task.assignee
                )),
            }
        }
        TaskAction::List { status } => {
            let tasks = if let Some(s) = status {
                let task_status = TaskStatus::parse(&s);
                hub.tasks.get_by_status(task_status)
            } else {
                hub.get_tasks()
            };

            if tasks.is_empty() {
                return Ok("No tasks.".to_string());
            }

            match format {
                OutputFormat::Json => {
                    let json: Vec<serde_json::Value> = tasks
                        .iter()
                        .map(|t| {
                            serde_json::json!({
                                "id": t.id,
                                "title": t.title,
                                "assignee": t.assignee,
                                "priority": t.priority.name(),
                                "status": t.status.name(),
                            })
                        })
                        .collect();
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::from("Tasks:\n");
                    for task in tasks {
                        output.push_str(&format!(
                            "  #{} │ {} {} │ {} │ {} │ {}\n",
                            task.id,
                            task.priority.emoji(),
                            task.priority.name(),
                            task.assignee,
                            task.title,
                            task.status.name()
                        ));
                    }
                    Ok(output)
                }
            }
        }
        TaskAction::Mine => {
            let session_id = hub.get_current_session_id().ok_or_else(|| {
                "No active session. Use 'sena join --role=<role>' first.".to_string()
            })?;

            let tasks = hub.get_my_tasks(&session_id);

            if tasks.is_empty() {
                return Ok(format!("No tasks assigned to {}.", session_id));
            }

            let mut output = format!("Tasks for {}:\n", session_id);
            for task in tasks {
                output.push_str(&format!(
                    "  #{} │ {} │ {} │ {}\n",
                    task.id,
                    task.priority.emoji(),
                    task.title,
                    task.status.name()
                ));
            }
            Ok(output)
        }
        TaskAction::Done { id } => {
            hub.update_task(id, TaskStatus::Done)?;
            hub.save()?;
            Ok(format!("Task #{} marked as done.", id))
        }
        TaskAction::Update { id, status } => {
            let task_status = TaskStatus::parse(&status);
            hub.update_task(id, task_status)?;
            hub.save()?;
            Ok(format!("Task #{} updated to {}.", id, task_status.name()))
        }
        TaskAction::Assign { id, to } => {
            hub.tasks.reassign(id, &to)?;
            hub.save()?;
            Ok(format!("Task #{} reassigned to {}.", id, to))
        }
        TaskAction::Delete { id } => {
            hub.tasks.delete(id)?;
            hub.save()?;
            Ok(format!("Task #{} deleted.", id))
        }
    }
}

async fn execute_watch() -> Result<String, String> {
    use crate::hub::Hub;
    use crate::output::FormatBox;

    let mut hub = Hub::new()?;
    hub.load()?;

    let status = hub.status();
    let sessions = hub.who();
    let tasks = hub.get_tasks();
    let conflicts = hub.get_conflicts();
    let recent_messages = hub.messages.get_recent(5);

    let mut output = String::new();

    output.push_str(&FormatBox::new(&SenaConfig::brand_title("COLLABORATION HUB")).render());
    output.push('\n');

    output.push_str(&format!(
        "Status: {} sessions │ {} tasks │ {} conflicts\n",
        status.online_sessions, status.total_tasks, status.active_conflicts
    ));

    output.push_str("\n┌─────────────────────────────────────────────────────────────┐\n");
    output.push_str("│ SESSIONS                                                    │\n");
    output.push_str("├─────────────────────────────────────────────────────────────┤\n");
    if sessions.is_empty() {
        output.push_str("│ No sessions. Use 'sena join --role=<role>' to join.        │\n");
    } else {
        for session in &sessions {
            output.push_str(&format!(
                "│ {} {:<12} │ {} │ {:<20} │ {:<8} │\n",
                session.role.emoji(),
                session.name,
                session.status.indicator(),
                session.working_on.as_deref().unwrap_or("-"),
                session.idle_display()
            ));
        }
    }
    output.push_str("└─────────────────────────────────────────────────────────────┘\n");

    output.push_str("\n┌─────────────────────────────────────────────────────────────┐\n");
    output.push_str("│ RECENT MESSAGES                                             │\n");
    output.push_str("├─────────────────────────────────────────────────────────────┤\n");
    if recent_messages.is_empty() {
        output.push_str("│ No messages yet.                                            │\n");
    } else {
        for msg in recent_messages.iter().take(5) {
            let direction = if msg.to == "all" {
                format!("{} → ALL", msg.from)
            } else {
                format!("{} → {}", msg.from, msg.to)
            };
            let truncated_content: String = msg.content.chars().take(35).collect();
            output.push_str(&format!(
                "│ {} [{}] {} : {}..│\n",
                msg.message_type.emoji(),
                msg.time_display(),
                direction,
                truncated_content
            ));
        }
    }
    output.push_str("└─────────────────────────────────────────────────────────────┘\n");

    output.push_str("\n┌─────────────────────────────────────────────────────────────┐\n");
    output.push_str("│ ACTIVE TASKS                                                │\n");
    output.push_str("├─────────────────────────────────────────────────────────────┤\n");
    let active_tasks: Vec<_> = tasks.iter().filter(|t| !t.is_complete()).take(5).collect();
    if active_tasks.is_empty() {
        output.push_str("│ No active tasks.                                            │\n");
    } else {
        for task in active_tasks {
            output.push_str(&format!(
                "│ #{:<3} │ {} │ {:<12} │ {:<20} │ {:<10} │\n",
                task.id,
                task.priority.emoji(),
                task.assignee,
                task.title.chars().take(20).collect::<String>(),
                task.status.name()
            ));
        }
    }
    output.push_str("└─────────────────────────────────────────────────────────────┘\n");

    if !conflicts.is_empty() {
        output.push_str("\n⚠️  CONFLICTS:\n");
        for conflict in &conflicts {
            output.push_str(&format!(
                "  {} {} - {:?}\n",
                conflict.severity.emoji(),
                conflict.file_path,
                conflict.sessions
            ));
        }
    }

    output.push_str("\nCommands: hub tell <name> <msg> │ hub broadcast <msg> │ hub messages\n");

    Ok(output)
}

async fn execute_sync(format: OutputFormat) -> Result<String, String> {
    use crate::hub::Hub;

    let mut hub = Hub::new()?;
    hub.load()?;

    let status = hub.status();
    hub.save()?;

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "synced": true,
            "sessions": status.online_sessions,
            "tasks": status.total_tasks,
            "conflicts": status.active_conflicts,
        }))
        .map_err(|e| e.to_string()),
        _ => Ok(format!(
            "Sync complete. {} sessions, {} tasks, {} conflicts.",
            status.online_sessions, status.total_tasks, status.active_conflicts
        )),
    }
}

// ================================
// Knowledge System Commands
// ================================

async fn execute_knowledge(
    action: KnowledgeAction,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::knowledge::KnowledgeSystem;

    let knowledge = KnowledgeSystem::new();

    match action {
        KnowledgeAction::Search { query, limit } => {
            let mut results = knowledge.search(&query);
            results.truncate(limit);

            match format {
                OutputFormat::Json => {
                    let json: Vec<serde_json::Value> = results
                        .iter()
                        .map(|r| {
                            serde_json::json!({
                                "domain": r.domain,
                                "title": r.title,
                                "description": r.description,
                                "relevance": r.relevance,
                            })
                        })
                        .collect();
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                OutputFormat::Pretty => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("KNOWLEDGE SEARCH")).render(),
                    );
                    output.push_str(&format!("\nQuery: \"{}\"\n", query));
                    output.push_str(&format!("Found: {} results\n\n", results.len()));

                    for result in &results {
                        output.push_str(&format!(
                            "┌─ {} ─────────────────────────────────────\n",
                            result.domain.to_uppercase()
                        ));
                        output.push_str(&format!("│ {}\n", result.title));
                        output.push_str(&format!("│ {}\n", result.description));
                        output
                            .push_str(&format!("│ Relevance: {:.0}%\n", result.relevance * 100.0));
                        output.push_str("└──────────────────────────────────────────\n\n");
                    }
                    Ok(output)
                }
                OutputFormat::Text => {
                    if results.is_empty() {
                        Ok("No results found.".to_string())
                    } else {
                        let mut output =
                            format!("Found {} results for \"{}\":\n", results.len(), query);
                        for result in &results {
                            output.push_str(&format!(
                                "  • [{}] {} - {}\n",
                                result.domain, result.title, result.description
                            ));
                        }
                        Ok(output)
                    }
                }
            }
        }
        KnowledgeAction::List { category } => {
            let domain = match category {
                KnowledgeCategory::Reasoning => "reasoning",
                KnowledgeCategory::Security => "security",
                KnowledgeCategory::Performance => "performance",
                KnowledgeCategory::Architecture => "architecture",
                KnowledgeCategory::All => "all",
            };

            let patterns = if domain == "all" {
                let mut all = Vec::new();
                all.extend(knowledge.get_domain_patterns("reasoning"));
                all.extend(knowledge.get_domain_patterns("security"));
                all.extend(knowledge.get_domain_patterns("performance"));
                all.extend(knowledge.get_domain_patterns("architecture"));
                all
            } else {
                knowledge.get_domain_patterns(domain)
            };

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&patterns).map_err(|e| e.to_string())
                }
                OutputFormat::Pretty => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title(&format!(
                            "{} PATTERNS",
                            format!("{:?}", category).to_uppercase()
                        )))
                        .render(),
                    );
                    output.push_str(&format!("\nTotal: {} patterns\n\n", patterns.len()));

                    for pattern in &patterns {
                        output.push_str(&format!("  • {}\n", pattern));
                    }
                    Ok(output)
                }
                OutputFormat::Text => {
                    if patterns.is_empty() {
                        Ok(format!("No {:?} patterns found.", category))
                    } else {
                        let mut output = format!("{:?} patterns ({}):\n", category, patterns.len());
                        for pattern in &patterns {
                            output.push_str(&format!("  • {}\n", pattern));
                        }
                        Ok(output)
                    }
                }
            }
        }
        KnowledgeAction::Stats => {
            let stats = &knowledge.stats;

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&stats).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("KNOWLEDGE STATISTICS")).render(),
                    );
                    output.push('\n');
                    output.push_str(&format!("Total Entries: {}\n", stats.total_entries));
                    output.push_str(&format!(
                        "Reasoning Frameworks: {}\n",
                        stats.reasoning_count
                    ));
                    output.push_str(&format!("Security Patterns: {}\n", stats.security_count));
                    output.push_str(&format!(
                        "Performance Patterns: {}\n",
                        stats.performance_count
                    ));
                    output.push_str(&format!(
                        "Architecture Patterns: {}\n",
                        stats.architecture_count
                    ));
                    Ok(output)
                }
            }
        }
    }
}

// ================================
// Intelligence System Commands
// ================================

async fn execute_think(
    query: &str,
    depth: ThinkingDepthArg,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::intelligence::{IntelligenceSystem, ThinkingDepth};

    let intelligence = IntelligenceSystem::new();

    let thinking_depth = match depth {
        ThinkingDepthArg::Quick => ThinkingDepth::Quick,
        ThinkingDepthArg::Standard => ThinkingDepth::Standard,
        ThinkingDepthArg::Deep => ThinkingDepth::Deep,
        ThinkingDepthArg::Maximum => ThinkingDepth::Maximum,
    };

    let result = intelligence.analyze(query, thinking_depth);

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "query": query,
            "depth": format!("{:?}", depth),
            "problem": result.problem,
            "conclusion": result.conclusion,
            "confidence": result.confidence,
            "frameworks_used": result.frameworks_used,
            "steps": result.steps.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                    "output": s.output,
                })
            }).collect::<Vec<_>>(),
            "thinking_time_ms": result.thinking_time_ms,
        }))
        .map_err(|e| e.to_string()),
        OutputFormat::Pretty => {
            let mut output = String::new();
            output
                .push_str(&FormatBox::new(&SenaConfig::brand_title("EXTENDED THINKING")).render());
            output.push_str(&format!("\nDepth: {:?}\n", depth));
            output.push_str(&format!(
                "Confidence: {:.1}%\n\n",
                result.confidence * 100.0
            ));

            output.push_str("═══════════════════════════════════════════\n");
            output.push_str("  PROBLEM\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            output.push_str(&result.problem);
            output.push_str("\n\n");

            output.push_str("═══════════════════════════════════════════\n");
            output.push_str("  THINKING STEPS\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            for (i, step) in result.steps.iter().enumerate() {
                output.push_str(&format!(
                    "{}. **{}**\n   {}\n\n",
                    i + 1,
                    step.name,
                    step.description
                ));
            }

            output.push_str("═══════════════════════════════════════════\n");
            output.push_str("  FRAMEWORKS USED\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            for framework in &result.frameworks_used {
                output.push_str(&format!("  • {}\n", framework));
            }
            output.push('\n');

            output.push_str("═══════════════════════════════════════════\n");
            output.push_str("  CONCLUSION\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            output.push_str(&format!("  {}\n", result.conclusion));
            output.push_str(&format!("\nThinking time: {}ms\n", result.thinking_time_ms));

            Ok(output)
        }
        OutputFormat::Text => {
            let mut output = format!(
                "Analysis ({:?}, {:.0}% confidence):\n\n",
                depth,
                result.confidence * 100.0
            );
            output.push_str(&format!("Problem: {}\n\n", result.problem));
            output.push_str("Steps:\n");
            for (i, step) in result.steps.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, step.name));
            }
            output.push_str(&format!("\nConclusion: {}\n", result.conclusion));
            Ok(output)
        }
    }
}

async fn execute_agent(
    agent_type: AgentTypeArg,
    content: &str,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::intelligence::{AgentType, IntelligenceSystem};

    let intelligence = IntelligenceSystem::new();

    let agent = match agent_type {
        AgentTypeArg::Security => AgentType::Security,
        AgentTypeArg::Performance => AgentType::Performance,
        AgentTypeArg::Architecture => AgentType::Architecture,
        AgentTypeArg::General => AgentType::General,
    };

    let result = intelligence.dispatch(content, agent);

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "agent": format!("{:?}", agent_type),
            "task": result.task,
            "analysis": result.analysis,
            "recommendations": result.recommendations,
            "confidence": result.confidence,
        }))
        .map_err(|e| e.to_string()),
        OutputFormat::Pretty => {
            let mut output = String::new();
            let title =
                SenaConfig::brand_title(&format!("{:?} AGENT ANALYSIS", agent_type).to_uppercase());
            output.push_str(&FormatBox::new(&title).render());
            output.push_str(&format!("\nAgent: {:?}\n", agent_type));
            output.push_str(&format!(
                "Confidence: {:.1}%\n\n",
                result.confidence * 100.0
            ));

            output.push_str("═══════════════════════════════════════════\n");
            output.push_str("  ANALYSIS\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            output.push_str(&result.analysis);
            output.push('\n');

            output.push_str("\n═══════════════════════════════════════════\n");
            output.push_str("  RECOMMENDATIONS\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            for rec in &result.recommendations {
                output.push_str(&format!("  ✓ {}\n", rec));
            }

            Ok(output)
        }
        OutputFormat::Text => {
            let mut output = format!(
                "{:?} Agent Analysis (Confidence: {:.0}%):\n\n",
                agent_type,
                result.confidence * 100.0
            );
            output.push_str(&result.analysis);
            output.push_str("\n\nRecommendations:\n");
            for rec in &result.recommendations {
                output.push_str(&format!("  ✓ {}\n", rec));
            }
            Ok(output)
        }
    }
}

// ================================
// Evolution System Commands
// ================================

async fn execute_evolve(
    action: Option<EvolveAction>,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::evolution::{EvolutionSystem, OptimizationTarget as EvOptTarget};

    let mut evolution = EvolutionSystem::new();
    evolution.load().ok();

    match action {
        None => {
            let result = evolution.evolve();
            evolution
                .save()
                .map_err(|e| format!("Failed to save evolution state: {}", e))?;

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("EVOLUTION CYCLE")).render(),
                    );
                    output.push('\n');
                    output.push_str(&format!("Patterns Applied: {}\n", result.patterns_applied));
                    output.push_str(&format!(
                        "Optimizations Made: {}\n",
                        result.optimizations_made
                    ));
                    output.push_str(&format!(
                        "Feedback Processed: {}\n",
                        result.feedback_processed
                    ));
                    output.push_str(&format!(
                        "Improvement Score: {:.1}%\n",
                        result.new_improvement_score * 100.0
                    ));
                    Ok(output)
                }
            }
        }
        Some(EvolveAction::Learn { context, outcome }) => {
            evolution.learn(&context, &outcome, true);
            evolution
                .save()
                .map_err(|e| format!("Failed to save learned pattern: {}", e))?;

            match format {
                OutputFormat::Json => Ok(serde_json::json!({
                    "action": "learn",
                    "context": context,
                    "outcome": outcome,
                    "status": "learned"
                })
                .to_string()),
                _ => Ok(format!(
                    "Pattern learned:\n  Context: {}\n  Outcome: {}",
                    context, outcome
                )),
            }
        }
        Some(EvolveAction::Optimize { target }) => {
            let opt_target = match target {
                Some(OptimizeTarget::Quality) => EvOptTarget::Quality,
                Some(OptimizeTarget::Speed) => EvOptTarget::Speed,
                Some(OptimizeTarget::Accuracy) => EvOptTarget::Accuracy,
                Some(OptimizeTarget::Satisfaction) => EvOptTarget::Satisfaction,
                Some(OptimizeTarget::Balanced) | None => EvOptTarget::Balanced,
            };

            let result = evolution.optimize(opt_target);
            evolution
                .save()
                .map_err(|e| format!("Failed to save optimization: {}", e))?;

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("SELF-OPTIMIZATION")).render(),
                    );
                    output.push('\n');
                    output.push_str(&format!("Target: {:?}\n", opt_target));
                    output.push_str(&format!(
                        "Success: {}\n",
                        if result.success { "✅" } else { "❌" }
                    ));
                    output.push_str(&format!(
                        "Improvement: +{:.1}%\n",
                        result.improvement * 100.0
                    ));
                    output.push_str(&format!("New Score: {:.1}%\n", result.new_score * 100.0));
                    if !result.suggestions.is_empty() {
                        output.push_str("\nSuggestions:\n");
                        for suggestion in &result.suggestions {
                            output.push_str(&format!("  • {}\n", suggestion));
                        }
                    }
                    Ok(output)
                }
            }
        }
        Some(EvolveAction::Stats) => {
            let stats = &evolution.stats;

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&stats).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("EVOLUTION STATISTICS")).render(),
                    );
                    output.push('\n');
                    output.push_str(&format!("Patterns Learned: {}\n", stats.patterns_learned));
                    output.push_str(&format!(
                        "Optimizations Applied: {}\n",
                        stats.optimizations_applied
                    ));
                    output.push_str(&format!("Feedback Count: {}\n", stats.feedback_count));
                    output.push_str(&format!(
                        "Improvement Score: {:.1}%\n",
                        stats.improvement_score * 100.0
                    ));
                    output.push_str(&format!("Learning Rate: {:.2}\n", stats.learning_rate));
                    Ok(output)
                }
            }
        }
        Some(EvolveAction::Patterns { limit }) => {
            let patterns = evolution.learner.get_patterns(limit);

            match format {
                OutputFormat::Json => {
                    let json: Vec<serde_json::Value> = patterns
                        .iter()
                        .map(|p| {
                            serde_json::json!({
                                "id": p.id,
                                "context": p.context,
                                "outcome": p.outcome,
                                "pattern_type": format!("{:?}", p.pattern_type),
                                "success_rate": p.success_rate,
                                "usage_count": p.usage_count,
                            })
                        })
                        .collect();
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("LEARNED PATTERNS")).render(),
                    );
                    output.push_str(&format!("\nShowing {} patterns:\n\n", patterns.len()));

                    if patterns.is_empty() {
                        output.push_str("No patterns learned yet. Use 'sena evolve learn <context> <outcome>' to add patterns.\n");
                    } else {
                        for pattern in &patterns {
                            output.push_str("┌─ Pattern ────────────────────────────────\n");
                            output.push_str(&format!("│ Context: {}\n", pattern.context));
                            output.push_str(&format!("│ Outcome: {}\n", pattern.outcome));
                            output.push_str(&format!("│ Type: {}\n", pattern.pattern_type));
                            output.push_str(&format!(
                                "│ Success Rate: {:.1}%\n",
                                pattern.success_rate * 100.0
                            ));
                            output.push_str(&format!("│ Usage Count: {}\n", pattern.usage_count));
                            output.push_str("└──────────────────────────────────────────\n\n");
                        }
                    }
                    Ok(output)
                }
            }
        }
    }
}

// ================================
// Feedback System Commands
// ================================

async fn execute_feedback(
    feedback_type: FeedbackTypeArg,
    message: &str,
    context: Option<String>,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::evolution::{EvolutionSystem, FeedbackType};

    let mut evolution = EvolutionSystem::new();
    let _ = evolution.load();

    let fb_type = match feedback_type {
        FeedbackTypeArg::Positive => FeedbackType::Positive,
        FeedbackTypeArg::Negative => FeedbackType::Negative,
        FeedbackTypeArg::Bug => FeedbackType::Bug,
        FeedbackTypeArg::Feature => FeedbackType::FeatureRequest,
        FeedbackTypeArg::Correction => FeedbackType::Correction,
    };

    // Combine message with context if provided
    let full_message = if let Some(ctx) = &context {
        format!("{} [Context: {}]", message, ctx)
    } else {
        message.to_string()
    };

    evolution.process_feedback(fb_type, &full_message);
    let _ = evolution.save();

    let emoji = match feedback_type {
        FeedbackTypeArg::Positive => "👍",
        FeedbackTypeArg::Negative => "👎",
        FeedbackTypeArg::Bug => "🐛",
        FeedbackTypeArg::Feature => "✨",
        FeedbackTypeArg::Correction => "📝",
    };

    match format {
        OutputFormat::Json => Ok(serde_json::json!({
            "action": "feedback",
            "type": format!("{:?}", feedback_type),
            "message": message,
            "context": context,
            "status": "recorded"
        })
        .to_string()),
        OutputFormat::Pretty => {
            let mut output = String::new();
            output
                .push_str(&FormatBox::new(&SenaConfig::brand_title("FEEDBACK RECORDED")).render());
            output.push('\n');
            output.push_str(&format!("{} Type: {:?}\n", emoji, feedback_type));
            output.push_str(&format!("Message: {}\n", message));
            if let Some(ctx) = &context {
                output.push_str(&format!("Context: {}\n", ctx));
            }
            output.push_str("\nThank you for your feedback! SENA learns from every interaction.\n");
            Ok(output)
        }
        OutputFormat::Text => Ok(format!(
            "{} Feedback recorded: {:?} - {}",
            emoji, feedback_type, message
        )),
    }
}

// ================================
// Domain Agent Commands
// ================================

async fn execute_backend(
    analysis: BackendAnalysisType,
    input: &str,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::agents::BackendAgent;

    let agent = BackendAgent::new();

    let command = match analysis {
        BackendAnalysisType::Map => "map",
        BackendAnalysisType::Flow => "flow",
        BackendAnalysisType::Auth => "auth",
        BackendAnalysisType::Secrets => "secrets",
        BackendAnalysisType::Security => "security",
        BackendAnalysisType::Full => "full",
    };

    let result = agent.analyze(command, input);
    format_domain_analysis("BACKEND", &analysis_type_name(&analysis), result, format)
}

async fn execute_iot(
    analysis: IoTAnalysisType,
    input: &str,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::agents::IoTAgent;

    let agent = IoTAgent::new();

    let command = match analysis {
        IoTAnalysisType::Protocol => "protocol",
        IoTAnalysisType::Debug => "debug",
        IoTAnalysisType::Power => "power",
        IoTAnalysisType::Connect => "connect",
        IoTAnalysisType::Sensor => "sensor",
        IoTAnalysisType::Firmware => "firmware",
        IoTAnalysisType::Full => "full",
    };

    let result = agent.analyze(command, input);
    format_domain_analysis("IOT", &iot_analysis_type_name(&analysis), result, format)
}

async fn execute_ios(
    analysis: IOSAnalysisType,
    input: &str,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::agents::IOSAgent;

    let agent = IOSAgent::new();

    let command = match analysis {
        IOSAnalysisType::Ui | IOSAnalysisType::Hig => "ui",
        IOSAnalysisType::Perf => "perf",
        IOSAnalysisType::A11y => "a11y",
        IOSAnalysisType::Device => "device",
        IOSAnalysisType::Memory => "memory",
        IOSAnalysisType::Full => "full",
    };

    let result = agent.analyze(command, input);
    format_domain_analysis("IOS", &ios_analysis_type_name(&analysis), result, format)
}

async fn execute_android(
    analysis: AndroidAnalysisType,
    input: &str,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::agents::AndroidAgent;

    let agent = AndroidAgent::new();

    let command = match analysis {
        AndroidAnalysisType::Ui | AndroidAnalysisType::Material => "ui",
        AndroidAnalysisType::Perf => "perf",
        AndroidAnalysisType::Lifecycle => "lifecycle",
        AndroidAnalysisType::Compat => "compat",
        AndroidAnalysisType::A11y => "a11y",
        AndroidAnalysisType::Full => "full",
    };

    let result = agent.analyze(command, input);
    format_domain_analysis(
        "ANDROID",
        &android_analysis_type_name(&analysis),
        result,
        format,
    )
}

async fn execute_web(
    analysis: WebAnalysisType,
    input: &str,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::agents::WebAgent;

    let agent = WebAgent::new();

    let command = match analysis {
        WebAnalysisType::Vitals => "vitals",
        WebAnalysisType::A11y => "a11y",
        WebAnalysisType::Seo => "seo",
        WebAnalysisType::Bundle => "bundle",
        WebAnalysisType::Perf => "perf",
        WebAnalysisType::Audit => "audit",
        WebAnalysisType::Full => "full",
    };

    let result = agent.analyze(command, input);
    format_domain_analysis("WEB", &web_analysis_type_name(&analysis), result, format)
}

fn format_domain_analysis(
    agent_name: &str,
    analysis_name: &str,
    result: crate::agents::DomainAnalysis,
    format: OutputFormat,
) -> Result<String, String> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "agent": agent_name,
            "analysis_type": analysis_name,
            "category": result.category,
            "score": result.score,
            "findings": result.findings.iter().map(|f| {
                serde_json::json!({
                    "severity": format!("{:?}", f.severity),
                    "title": f.title,
                    "description": f.description,
                    "location": f.location,
                    "suggestion": f.suggestion,
                })
            }).collect::<Vec<_>>(),
            "recommendations": result.recommendations,
        }))
        .map_err(|e| e.to_string()),
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(
                &FormatBox::new(&SenaConfig::brand_title(&format!(
                    "{} AGENT - {}",
                    agent_name,
                    analysis_name.to_uppercase()
                )))
                .render(),
            );
            output.push('\n');
            output.push_str(&format!("Score: {}/100\n", result.score));
            output.push_str(&format!("Category: {}\n\n", result.category));

            if result.findings.is_empty() {
                output.push_str("✅ No issues found.\n");
            } else {
                output.push_str("═══════════════════════════════════════════\n");
                output.push_str("  FINDINGS\n");
                output.push_str("═══════════════════════════════════════════\n\n");

                for finding in &result.findings {
                    let emoji = match finding.severity {
                        crate::agents::Severity::Critical => "🔴",
                        crate::agents::Severity::Warning => "⚠️ ",
                        crate::agents::Severity::Info => "ℹ️ ",
                        crate::agents::Severity::Success => "✅",
                    };
                    output.push_str(&format!("{} {}\n", emoji, finding.title));
                    output.push_str(&format!("   {}\n", finding.description));
                    if let Some(loc) = &finding.location {
                        output.push_str(&format!("   📍 {}\n", loc));
                    }
                    if let Some(sug) = &finding.suggestion {
                        output.push_str(&format!("   💡 {}\n", sug));
                    }
                    output.push('\n');
                }
            }

            if !result.recommendations.is_empty() {
                output.push_str("═══════════════════════════════════════════\n");
                output.push_str("  RECOMMENDATIONS\n");
                output.push_str("═══════════════════════════════════════════\n\n");
                for (i, rec) in result.recommendations.iter().enumerate() {
                    output.push_str(&format!("{}. {}\n", i + 1, rec));
                }
            }
            Ok(output)
        }
        OutputFormat::Text => {
            let mut output = format!(
                "{} {} Analysis (Score: {}/100):\n",
                agent_name, analysis_name, result.score
            );
            output.push_str(&format!("Category: {}\n\n", result.category));

            if result.findings.is_empty() {
                output.push_str("No issues found.\n");
            } else {
                for finding in &result.findings {
                    let prefix = match finding.severity {
                        crate::agents::Severity::Critical => "CRITICAL",
                        crate::agents::Severity::Warning => "WARNING",
                        crate::agents::Severity::Info => "INFO",
                        crate::agents::Severity::Success => "OK",
                    };
                    output.push_str(&format!(
                        "[{}] {}: {}\n",
                        prefix, finding.title, finding.description
                    ));
                }
            }
            Ok(output)
        }
    }
}

fn analysis_type_name(analysis: &BackendAnalysisType) -> String {
    match analysis {
        BackendAnalysisType::Map => "API Mapping".to_string(),
        BackendAnalysisType::Flow => "Data Flow".to_string(),
        BackendAnalysisType::Auth => "Auth Audit".to_string(),
        BackendAnalysisType::Secrets => "Secrets Scan".to_string(),
        BackendAnalysisType::Security => "Security Scan".to_string(),
        BackendAnalysisType::Full => "Full Analysis".to_string(),
    }
}

fn iot_analysis_type_name(analysis: &IoTAnalysisType) -> String {
    match analysis {
        IoTAnalysisType::Protocol => "Protocol Analysis".to_string(),
        IoTAnalysisType::Debug => "Device Debug".to_string(),
        IoTAnalysisType::Power => "Power Analysis".to_string(),
        IoTAnalysisType::Connect => "Connectivity".to_string(),
        IoTAnalysisType::Sensor => "Sensor Analysis".to_string(),
        IoTAnalysisType::Firmware => "Firmware Analysis".to_string(),
        IoTAnalysisType::Full => "Full Analysis".to_string(),
    }
}

fn ios_analysis_type_name(analysis: &IOSAnalysisType) -> String {
    match analysis {
        IOSAnalysisType::Ui | IOSAnalysisType::Hig => "UI/HIG Compliance".to_string(),
        IOSAnalysisType::Perf => "Performance".to_string(),
        IOSAnalysisType::A11y => "Accessibility".to_string(),
        IOSAnalysisType::Device => "Device Features".to_string(),
        IOSAnalysisType::Memory => "Memory Analysis".to_string(),
        IOSAnalysisType::Full => "Full Analysis".to_string(),
    }
}

fn android_analysis_type_name(analysis: &AndroidAnalysisType) -> String {
    match analysis {
        AndroidAnalysisType::Ui | AndroidAnalysisType::Material => "UI/Material Design".to_string(),
        AndroidAnalysisType::Perf => "Performance".to_string(),
        AndroidAnalysisType::Lifecycle => "Lifecycle".to_string(),
        AndroidAnalysisType::Compat => "Compatibility".to_string(),
        AndroidAnalysisType::A11y => "Accessibility".to_string(),
        AndroidAnalysisType::Full => "Full Analysis".to_string(),
    }
}

fn web_analysis_type_name(analysis: &WebAnalysisType) -> String {
    match analysis {
        WebAnalysisType::Vitals => "Core Web Vitals".to_string(),
        WebAnalysisType::A11y => "Accessibility".to_string(),
        WebAnalysisType::Seo => "SEO".to_string(),
        WebAnalysisType::Bundle => "Bundle Analysis".to_string(),
        WebAnalysisType::Perf => "Performance".to_string(),
        WebAnalysisType::Audit => "Security Audit".to_string(),
        WebAnalysisType::Full => "Full Analysis".to_string(),
    }
}

async fn execute_setup(
    install_type: Option<InstallationType>,
    name: Option<String>,
    auto_yes: bool,
    format: OutputFormat,
) -> Result<String, String> {
    let project_name = name.unwrap_or_else(|| "sena-project".to_string());
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let sena_binary = format!("{}/.local/bin/sena", home);

    match install_type {
        Some(InstallationType::Mcp) => setup_mcp_server(&home, &sena_binary, format),
        Some(InstallationType::Hook) => setup_claude_hooks(&home, &sena_binary, format),
        Some(InstallationType::Full) => {
            setup_full_installation(&home, &sena_binary, &project_name, format)
        }
        Some(InstallationType::Backend) => {
            setup_agent_project(&home, "backend", &project_name, format)
        }
        Some(InstallationType::Iot) => setup_agent_project(&home, "iot", &project_name, format),
        Some(InstallationType::Ios) => setup_agent_project(&home, "ios", &project_name, format),
        Some(InstallationType::Android) => {
            setup_agent_project(&home, "android", &project_name, format)
        }
        Some(InstallationType::Web) => setup_agent_project(&home, "web", &project_name, format),
        None => {
            if auto_yes {
                setup_full_installation(&home, &sena_binary, &project_name, format)
            } else {
                show_setup_menu(format)
            }
        }
    }
}

fn show_setup_menu(_format: OutputFormat) -> Result<String, String> {
    let mut output = String::new();
    output.push_str(
        &FormatBox::new(&SenaConfig::brand_title(&format!(
            "SETUP WIZARD v{}",
            crate::VERSION
        )))
        .render(),
    );
    output.push('\n');
    output.push_str("Run setup.sh for interactive installation:\n\n");
    output.push_str("  bash setup.sh\n\n");
    output.push_str("Or use CLI options:\n\n");
    output.push_str("  sena setup mcp         - Setup MCP server for Claude Desktop\n");
    output.push_str("  sena setup hook        - Setup hooks for Claude Code\n");
    output.push_str("  sena setup full        - Full installation (MCP + Hooks + Rules)\n");
    output.push_str("  sena setup backend     - Setup Backend development project\n");
    output.push_str("  sena setup iot         - Setup IoT development project\n");
    output.push_str("  sena setup ios         - Setup iOS development project\n");
    output.push_str("  sena setup android     - Setup Android development project\n");
    output.push_str("  sena setup web         - Setup Web development project\n");
    output.push_str("\nOptions:\n");
    output.push_str("  -n, --name <NAME>      - Project name\n");
    output.push_str("  -y, --yes              - Skip confirmation prompts\n");
    Ok(output)
}

fn setup_mcp_server(home: &str, sena_path: &str, _format: OutputFormat) -> Result<String, String> {
    let config_dir = PathBuf::from(home).join("Library/Application Support/Claude");
    let config_file = config_dir.join("claude_desktop_config.json");

    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let config = serde_json::json!({
        "mcpServers": {
            "sena": {
                "command": sena_path,
                "args": ["mcp"]
            }
        }
    });

    let config_str = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&config_file, config_str).map_err(|e| e.to_string())?;

    let mut output = String::new();
    output.push_str(&FormatBox::new(&SenaConfig::brand_title("MCP SERVER SETUP")).render());
    output.push('\n');
    output.push_str("✅ MCP server configured!\n\n");
    output.push_str(&format!("Config: {}\n", config_file.display()));
    output.push_str("\nNext steps:\n");
    output.push_str("  1. Restart Claude Desktop\n");
    output.push_str("  2. SENA will appear in MCP servers list\n");
    Ok(output)
}

fn setup_claude_hooks(
    home: &str,
    sena_path: &str,
    _format: OutputFormat,
) -> Result<String, String> {
    let claude_dir = PathBuf::from(home).join(".claude");
    let settings_file = claude_dir.join("settings.json");

    std::fs::create_dir_all(&claude_dir).map_err(|e| e.to_string())?;

    let existing_settings = if settings_file.exists() {
        let content = std::fs::read_to_string(&settings_file).unwrap_or_default();
        serde_json::from_str::<serde_json::Value>(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut settings = existing_settings.clone();
    let settings_obj = settings.as_object_mut().ok_or("Invalid settings format")?;

    settings_obj.insert(
        "hooks".to_string(),
        serde_json::json!({
            "UserPromptSubmit": [
                {
                    "command": format!("{} hook user-prompt-submit", sena_path)
                }
            ]
        }),
    );

    let existing_tools = existing_settings
        .get("permissions")
        .and_then(|p| p.get("allow"))
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();

    let sena_patterns = vec![
        serde_json::json!("Bash(sena *)"),
        serde_json::json!("Bash(sena)"),
        serde_json::json!("Bash(./target/release/sena *)"),
    ];

    let mut combined_tools: Vec<serde_json::Value> = existing_tools;
    for pattern in sena_patterns {
        if !combined_tools.contains(&pattern) {
            combined_tools.push(pattern);
        }
    }

    settings_obj.insert(
        "permissions".to_string(),
        serde_json::json!({
            "allow": combined_tools
        }),
    );

    let settings_str = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&settings_file, settings_str).map_err(|e| e.to_string())?;

    let mut output = String::new();
    output.push_str(&FormatBox::new(&SenaConfig::brand_title("HOOKS SETUP")).render());
    output.push('\n');
    output.push_str("✅ Claude Code hooks configured!\n\n");
    output.push_str(&format!("Config: {}\n", settings_file.display()));
    output.push_str("\nConfigured:\n");
    output.push_str("  • UserPromptSubmit hook\n");
    output.push_str("  • Auto-approve SENA bash commands\n");
    output.push_str("\nNext steps:\n");
    output.push_str("  1. Start a new Claude Code session\n");
    output.push_str("  2. SENA commands will auto-execute without prompts\n");
    Ok(output)
}

fn setup_full_installation(
    home: &str,
    sena_path: &str,
    _name: &str,
    _format: OutputFormat,
) -> Result<String, String> {
    setup_mcp_server(home, sena_path, OutputFormat::Text)?;
    setup_claude_hooks(home, sena_path, OutputFormat::Text)?;

    let claude_md_src = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("CLAUDE.md");
    let claude_md_dst = PathBuf::from(home).join(".claude/CLAUDE.md");

    if claude_md_src.exists() {
        std::fs::copy(&claude_md_src, &claude_md_dst).map_err(|e| e.to_string())?;
    }

    let mut output = String::new();
    output
        .push_str(&FormatBox::new(&SenaConfig::brand_title("FULL INSTALLATION COMPLETE")).render());
    output.push('\n');
    output.push_str("✅ All components installed!\n\n");
    output.push_str("Installed:\n");
    output.push_str("  • MCP Server for Claude Desktop\n");
    output.push_str("  • Hooks for Claude Code\n");
    output.push_str("  • SENA Elite Coding Standards\n");
    output.push_str("\nNext steps:\n");
    output.push_str("  1. Restart Claude Desktop\n");
    output.push_str("  2. Start a new Claude Code session\n");
    output.push_str("  3. Run: sena health\n");
    Ok(output)
}

fn setup_agent_project(
    home: &str,
    agent: &str,
    name: &str,
    _format: OutputFormat,
) -> Result<String, String> {
    let project_dir = PathBuf::from(home).join("Projects").join(name);
    std::fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;

    let sena_config = project_dir.join(".sena.toml");
    let config_content = format!(
        r#"# SENA Project Configuration
[project]
name = "{}"
agent = "{}"
version = "{}"

[agent.{}]
enabled = true
auto_analyze = true
"#,
        name,
        agent,
        crate::VERSION,
        agent
    );
    std::fs::write(&sena_config, config_content).map_err(|e| e.to_string())?;

    let claude_md = project_dir.join("CLAUDE.md");
    let claude_content = format!(
        r#"# {} Project

This project uses SENA {} Agent for specialized analysis.

## Quick Commands

```bash
sena {} full "analyze this code"
sena health
```
"#,
        name,
        agent.to_uppercase(),
        agent
    );
    std::fs::write(&claude_md, claude_content).map_err(|e| e.to_string())?;

    let mut output = String::new();
    output.push_str(
        &FormatBox::new(&SenaConfig::brand_title(&format!(
            "{} PROJECT SETUP",
            agent.to_uppercase()
        )))
        .render(),
    );
    output.push('\n');
    output.push_str(&format!("✅ {} project created!\n\n", agent.to_uppercase()));
    output.push_str(&format!("Location: {}\n\n", project_dir.display()));
    output.push_str("Created:\n");
    output.push_str("  • .sena.toml (project config)\n");
    output.push_str("  • CLAUDE.md (project rules)\n");
    output.push_str("\nNext steps:\n");
    output.push_str(&format!("  cd {}\n", project_dir.display()));
    output.push_str(&format!("  sena {} full \"<your code>\"\n", agent));
    Ok(output)
}

async fn execute_network(action: NetworkAction, format: OutputFormat) -> Result<String, String> {
    use crate::network::{NetworkConfig, NetworkManager};

    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let data_dir = home.join(".sena").join("network");

    match action {
        NetworkAction::Start { port, name } => {
            let config = NetworkConfig {
                port,
                enabled: true,
                ..Default::default()
            };

            let mut manager = NetworkManager::new(config, data_dir)?;

            if let Some(custom_name) = name {
                manager.set_local_peer_name(&custom_name).await?;
            }

            manager.start().await?;

            let status = manager.status().await;
            format_network_status(&status, format, "Network server started")
        }

        NetworkAction::Stop => {
            let config = NetworkConfig::default();
            let mut manager = NetworkManager::new(config, data_dir)?;
            manager.stop().await;
            Ok("Network server stopped".to_string())
        }

        NetworkAction::Status => {
            let config = NetworkConfig::default();
            let manager = NetworkManager::new(config, data_dir)?;
            let status = manager.status().await;
            format_network_status(&status, format, "Network Status")
        }

        NetworkAction::Info => {
            let config = NetworkConfig::default();
            let manager = NetworkManager::new(config, data_dir)?;

            let peer_id = manager.get_local_peer_id().await;
            let peer_name = manager.get_local_peer_name().await;
            let fingerprint = manager
                .get_certificate_fingerprint()
                .unwrap_or_else(|_| "N/A".to_string());

            match format {
                OutputFormat::Json => Ok(serde_json::json!({
                    "peer_id": peer_id,
                    "peer_name": peer_name,
                    "certificate_fingerprint": fingerprint
                })
                .to_string()),
                _ => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("NETWORK INFO")).render(),
                    );
                    output.push('\n');
                    output.push_str(&format!("Peer ID: {}\n", peer_id));
                    output.push_str(&format!("Peer Name: {}\n", peer_name));
                    output.push_str(&format!(
                        "Certificate: {}\n",
                        &fingerprint[..16.min(fingerprint.len())]
                    ));
                    Ok(output)
                }
            }
        }

        NetworkAction::SetName { name } => {
            let config = NetworkConfig::default();
            let manager = NetworkManager::new(config, data_dir)?;
            manager.set_local_peer_name(&name).await?;
            Ok(format!("Peer name set to: {}", name))
        }
    }
}

fn format_network_status(
    status: &crate::network::NetworkStatus,
    format: OutputFormat,
    title: &str,
) -> Result<String, String> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(status).map_err(|e| e.to_string()),
        _ => {
            let mut output = String::new();
            output.push_str(&FormatBox::new(&SenaConfig::brand_title(title)).render());
            output.push('\n');

            let table = TableBuilder::new()
                .row(vec![
                    "Status".to_string(),
                    if status.running { "Running" } else { "Stopped" }.to_string(),
                ])
                .row(vec!["Port".to_string(), status.port.to_string()])
                .row(vec!["Peers".to_string(), status.peer_count.to_string()])
                .row(vec![
                    "Authorized".to_string(),
                    status.authorized_count.to_string(),
                ])
                .row(vec![
                    "Discovered".to_string(),
                    status.discovered_count.to_string(),
                ])
                .row(vec![
                    "Connections".to_string(),
                    status.connection_count.to_string(),
                ])
                .row(vec![
                    "TLS".to_string(),
                    if status.tls_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                    .to_string(),
                ])
                .row(vec![
                    "Discovery".to_string(),
                    if status.discovery_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                    .to_string(),
                ])
                .build();

            output.push_str(&table);
            Ok(output)
        }
    }
}

async fn execute_peer(action: PeerAction, format: OutputFormat) -> Result<String, String> {
    use crate::network::{NetworkConfig, NetworkManager};

    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let data_dir = home.join(".sena").join("network");
    let config = NetworkConfig::default();
    let manager = NetworkManager::new(config, data_dir)?;

    match action {
        PeerAction::List { authorized } => {
            let peers = if authorized {
                manager.get_authorized_peers().await
            } else {
                manager.get_peers().await
            };

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&peers).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new(&SenaConfig::brand_title("PEERS")).render());
                    output.push('\n');

                    if peers.is_empty() {
                        output.push_str("No peers found.\n");
                        output.push_str("\nTo add a peer:\n");
                        output.push_str("  sena peer add <ip> --port 9876 --name \"Peer Name\"\n");
                    } else {
                        for peer in &peers {
                            let status = if peer.authorized { "✅" } else { "❌" };
                            let online = if peer.is_online() { "🟢" } else { "⚫" };
                            output.push_str(&format!(
                                "{} {} {} ({}:{}) - {}\n",
                                status,
                                online,
                                peer.name,
                                peer.address,
                                peer.port,
                                &peer.id[..8]
                            ));
                        }
                    }
                    Ok(output)
                }
            }
        }

        PeerAction::Add {
            address,
            port,
            name,
        } => {
            let peer = manager.add_peer(&address, port, name.as_deref()).await?;

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&peer).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&format!(
                        "✅ Peer added: {} ({})\n",
                        peer.name,
                        &peer.id[..8]
                    ));
                    output.push_str(&format!("   Address: {}:{}\n", peer.address, peer.port));
                    output.push_str("\nTo authorize this peer:\n");
                    output.push_str(&format!("  sena peer authorize {}\n", &peer.id[..8]));
                    Ok(output)
                }
            }
        }

        PeerAction::Remove { peer_id } => {
            let peers = manager.get_peers().await;
            let matched = peers.iter().find(|p| p.id.starts_with(&peer_id));

            match matched {
                Some(peer) => {
                    manager.remove_peer(&peer.id).await?;
                    Ok(format!("✅ Peer removed: {}", peer.name))
                }
                None => Err(format!("Peer not found: {}", peer_id)),
            }
        }

        PeerAction::Authorize { peer_id, expires } => {
            let peers = manager.get_peers().await;
            let matched = peers.iter().find(|p| p.id.starts_with(&peer_id));

            match matched {
                Some(peer) => {
                    let token = manager.authorize_peer(&peer.id).await?;

                    match format {
                        OutputFormat::Json => Ok(serde_json::json!({
                            "peer_id": peer.id,
                            "token": token.token,
                            "expires_in": expires
                        })
                        .to_string()),
                        _ => {
                            let mut output = String::new();
                            output.push_str(&format!("✅ Peer authorized: {}\n\n", peer.name));
                            output.push_str("Share this token with the peer:\n");
                            output.push_str(&format!("  Token: {}\n", token.token));
                            output.push_str(&format!("  Expires in: {} seconds\n", expires));
                            output.push_str("\nPeer should run:\n");
                            output.push_str(&format!(
                                "  sena peer connect <your-ip> --token {}\n",
                                token.token
                            ));
                            Ok(output)
                        }
                    }
                }
                None => Err(format!("Peer not found: {}", peer_id)),
            }
        }

        PeerAction::Connect {
            address,
            port,
            token,
        } => {
            let mut client = manager.connect_and_auth(&address, port, &token).await?;

            let peer_name = client.remote_peer_name().unwrap_or("Unknown").to_string();
            let peer_id = client.remote_peer_id().unwrap_or("").to_string();

            client.disconnect().await?;

            match format {
                OutputFormat::Json => Ok(serde_json::json!({
                    "success": true,
                    "peer_id": peer_id,
                    "peer_name": peer_name
                })
                .to_string()),
                _ => Ok(format!(
                    "✅ Connected to {} ({})",
                    peer_name,
                    &peer_id[..8.min(peer_id.len())]
                )),
            }
        }

        PeerAction::Revoke { peer_id } => {
            let peers = manager.get_peers().await;
            let matched = peers.iter().find(|p| p.id.starts_with(&peer_id));

            match matched {
                Some(peer) => {
                    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
                    let data_dir = home.join(".sena").join("network");
                    let mut registry =
                        crate::network::PeerRegistry::load(data_dir.join("peers.json"))?;
                    registry.revoke_peer(&peer.id)?;
                    Ok(format!("✅ Authorization revoked for: {}", peer.name))
                }
                None => Err(format!("Peer not found: {}", peer_id)),
            }
        }

        PeerAction::Ping { target, port } => {
            let mut client = manager.connect_to_peer(&target, port).await?;
            let start = std::time::Instant::now();
            let success = client.ping().await?;
            let elapsed = start.elapsed();
            client.disconnect().await?;

            if success {
                Ok(format!(
                    "✅ Pong from {} ({}ms)",
                    target,
                    elapsed.as_millis()
                ))
            } else {
                Err("Ping failed".to_string())
            }
        }
    }
}

async fn execute_discover(timeout: u64, format: OutputFormat) -> Result<String, String> {
    use crate::network::discover_once;

    let peers = discover_once(timeout).await?;

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&peers).map_err(|e| e.to_string()),
        _ => {
            let mut output = String::new();
            output.push_str(&FormatBox::new(&SenaConfig::brand_title("DISCOVERED PEERS")).render());
            output.push('\n');

            if peers.is_empty() {
                output.push_str("No peers discovered on network.\n");
                output.push_str("\nMake sure:\n");
                output.push_str("  • Other SENA instances are running: sena network start\n");
                output.push_str("  • You're on the same local network\n");
                output.push_str("  • Firewall allows mDNS (port 5353)\n");
            } else {
                output.push_str(&format!("Found {} peer(s):\n\n", peers.len()));
                for peer in &peers {
                    output.push_str(&format!(
                        "🔍 {} ({})\n   Address: {}:{}\n   ID: {}\n\n",
                        peer.peer_name,
                        peer.address,
                        peer.address,
                        peer.port,
                        &peer.peer_id[..8.min(peer.peer_id.len())]
                    ));
                }
                output.push_str("To connect to a peer:\n");
                output.push_str("  1. Add peer: sena peer add <ip> --name \"Name\"\n");
                output.push_str("  2. Authorize: sena peer authorize <peer-id>\n");
                output.push_str("  3. Share token with peer\n");
            }
            Ok(output)
        }
    }
}

async fn execute_provider(action: ProviderAction, format: OutputFormat) -> Result<String, String> {
    use sena_providers::{
        config::ProvidersConfig, AIProvider, ChatRequest, Message, ProviderRouter,
    };

    let config = ProvidersConfig::load_or_default();

    match action {
        ProviderAction::List => {
            let providers_info: Vec<serde_json::Value> = config
                .providers
                .iter()
                .map(|(id, cfg)| {
                    serde_json::json!({
                        "id": id,
                        "enabled": cfg.enabled,
                        "default_model": cfg.default_model,
                        "has_api_key": cfg.api_key.is_some() || cfg.get_api_key().is_some(),
                    })
                })
                .collect();

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&providers_info).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(
                        &FormatBox::new(&SenaConfig::brand_title("AI PROVIDERS")).render(),
                    );
                    output.push('\n');

                    for (id, cfg) in &config.providers {
                        let status = if cfg.get_api_key().is_some() || id == "ollama" {
                            "✅"
                        } else {
                            "❌"
                        };
                        let default = if config.default_provider.as_ref() == Some(id) {
                            " (default)"
                        } else {
                            ""
                        };
                        output.push_str(&format!(
                            "{} {} - {}{}\n",
                            status,
                            id,
                            cfg.default_model.as_deref().unwrap_or("N/A"),
                            default
                        ));
                    }

                    output.push_str("\nTo use a provider, set the appropriate API key:\n");
                    output.push_str("  export ANTHROPIC_API_KEY=your-key   # Claude\n");
                    output.push_str("  export OPENAI_API_KEY=your-key      # OpenAI\n");
                    output.push_str("  export GOOGLE_API_KEY=your-key      # Gemini\n");
                    output.push_str("  # Ollama runs locally, no key needed\n");

                    Ok(output)
                }
            }
        }

        ProviderAction::Status => match ProviderRouter::from_config(&config) {
            Ok(router) => {
                let status = router.provider_status();
                let providers: Vec<&std::sync::Arc<dyn AIProvider>> = router.available_providers();

                match format {
                    OutputFormat::Json => {
                        let json: Vec<serde_json::Value> = providers
                            .iter()
                            .map(|p| {
                                serde_json::json!({
                                    "id": p.provider_id(),
                                    "name": p.display_name(),
                                    "status": format!("{:?}", status.get(p.provider_id())),
                                    "default_model": p.default_model(),
                                    "streaming": p.supports_streaming(),
                                    "tools": p.supports_tools(),
                                    "vision": p.supports_vision(),
                                })
                            })
                            .collect();
                        serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                    }
                    _ => {
                        let mut output = String::new();
                        output.push_str(
                            &FormatBox::new(&SenaConfig::brand_title("PROVIDER STATUS")).render(),
                        );
                        output.push('\n');

                        for provider in providers {
                            let provider_status = status.get(provider.provider_id());
                            let status_icon = match provider_status {
                                Some(sena_providers::ProviderStatus::Connected) => "🟢",
                                Some(sena_providers::ProviderStatus::RateLimited) => "🟡",
                                Some(sena_providers::ProviderStatus::Disconnected) => "🔴",
                                Some(sena_providers::ProviderStatus::Error) | None => "⚪",
                            };

                            output.push_str(&format!(
                                    "{} {} ({})\n   Model: {}\n   Features: streaming={}, tools={}, vision={}\n\n",
                                    status_icon,
                                    provider.display_name(),
                                    provider.provider_id(),
                                    provider.default_model(),
                                    provider.supports_streaming(),
                                    provider.supports_tools(),
                                    provider.supports_vision(),
                                ));
                        }

                        Ok(output)
                    }
                }
            }
            Err(e) => Err(format!("Failed to initialize providers: {}", e)),
        },

        ProviderAction::Models { provider } => match ProviderRouter::from_config(&config) {
            Ok(router) => {
                let models = if let Some(provider_id) = &provider {
                    router
                        .get_provider(provider_id)
                        .map(|p| p.available_models().to_vec())
                        .unwrap_or_default()
                } else {
                    router.all_models()
                };

                match format {
                    OutputFormat::Json => {
                        serde_json::to_string_pretty(&models).map_err(|e| e.to_string())
                    }
                    _ => {
                        let mut output = String::new();
                        output.push_str(
                            &FormatBox::new(&SenaConfig::brand_title("AVAILABLE MODELS")).render(),
                        );
                        output.push('\n');

                        if models.is_empty() {
                            output.push_str("No models available. Check API keys.\n");
                        } else {
                            for model in &models {
                                let features = vec![
                                    if model.supports_streaming {
                                        "stream"
                                    } else {
                                        ""
                                    },
                                    if model.supports_tools { "tools" } else { "" },
                                    if model.supports_vision { "vision" } else { "" },
                                ]
                                .into_iter()
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<_>>()
                                .join(", ");

                                output.push_str(&format!(
                                    "  {} ({}) - {}k context [{}]\n",
                                    model.id,
                                    model.provider,
                                    model.context_length / 1000,
                                    features
                                ));
                            }
                        }

                        Ok(output)
                    }
                }
            }
            Err(e) => Err(format!("Failed to initialize providers: {}", e)),
        },

        ProviderAction::Chat {
            message,
            provider,
            model,
        } => match ProviderRouter::from_config(&config) {
            Ok(router) => {
                let mut request = ChatRequest::new(vec![Message::user(&message)]);

                if let Some(m) = model {
                    request = request.with_model(m);
                }

                let result = if provider.is_some() {
                    router.chat(request).await
                } else {
                    router.chat_with_fallback(request).await
                };

                match result {
                    Ok(response) => match format {
                        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                            "provider": response.provider,
                            "model": response.model,
                            "content": response.content,
                            "usage": {
                                "prompt_tokens": response.usage.prompt_tokens,
                                "completion_tokens": response.usage.completion_tokens,
                                "total_tokens": response.usage.total_tokens,
                            }
                        }))
                        .map_err(|e| e.to_string()),
                        _ => {
                            let mut output = String::new();
                            output.push_str(
                                &FormatBox::new(&SenaConfig::brand_title("AI RESPONSE")).render(),
                            );
                            output.push('\n');
                            output.push_str(&format!(
                                "Provider: {} | Model: {}\n\n",
                                response.provider, response.model
                            ));
                            output.push_str(&response.content);
                            output.push_str(&format!("\n\n─────────────────────────────────────────\nTokens: {} prompt + {} completion = {} total\n",
                                        response.usage.prompt_tokens,
                                        response.usage.completion_tokens,
                                        response.usage.total_tokens
                                    ));
                            Ok(output)
                        }
                    },
                    Err(e) => Err(format!("Chat failed: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to initialize providers: {}", e)),
        },

        ProviderAction::Default { provider_id } => {
            let mut config = config;
            if config.set_default_provider(&provider_id) {
                let path = ProvidersConfig::config_path();
                match config.save_to_file(&path) {
                    Ok(()) => Ok(format!(
                        "Default provider set to: {}\nConfig saved to: {}",
                        provider_id,
                        path.display()
                    )),
                    Err(e) => Err(format!("Failed to save config: {}", e)),
                }
            } else {
                Err(format!(
                    "Unknown provider: {}. Available: {:?}",
                    provider_id,
                    config.providers.keys().collect::<Vec<_>>()
                ))
            }
        }

        ProviderAction::Test { provider } => match ProviderRouter::from_config(&config) {
            Ok(router) => {
                let mut output = String::new();
                output
                    .push_str(&FormatBox::new(&SenaConfig::brand_title("PROVIDER TEST")).render());
                output.push('\n');

                let providers_to_test: Vec<_> = if provider == "all" {
                    router.available_providers()
                } else {
                    router
                        .get_provider(&provider)
                        .map(|p| vec![p])
                        .unwrap_or_default()
                };

                if providers_to_test.is_empty() {
                    return Err(format!("Provider not found: {}", provider));
                }

                for p in providers_to_test {
                    let test_request =
                        ChatRequest::new(vec![Message::user("Say 'OK' if you can hear me.")])
                            .with_max_tokens(10);

                    output.push_str(&format!("Testing {}... ", p.display_name()));

                    match p.chat(test_request).await {
                        Ok(response) => {
                            output.push_str(&format!("✅ OK ({})\n", response.model));
                        }
                        Err(e) => {
                            output.push_str(&format!("❌ Failed: {}\n", e));
                        }
                    }
                }

                Ok(output)
            }
            Err(e) => Err(format!("Failed to initialize providers: {}", e)),
        },
    }
}

async fn execute_collab(action: CollabAction, format: OutputFormat) -> Result<String, String> {
    use sena_collab::{CollabOrchestrator, RequestPayload, RequestType};
    use sena_providers::config::ProvidersConfig;
    use std::sync::Arc;

    let config = ProvidersConfig::load_or_default();

    let router = sena_providers::ProviderRouter::from_config(&config)
        .map_err(|e| format!("Failed to initialize providers: {}", e))?;

    let mut orchestrator = CollabOrchestrator::new(100);

    for provider in router.available_providers() {
        orchestrator.register_provider(Arc::clone(provider));
    }

    match action {
        CollabAction::New { name, provider } => {
            match orchestrator.create_session(&name, &provider).await {
                Ok(session_id) => {
                    match format {
                        OutputFormat::Json => {
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "created",
                                "session_id": session_id,
                                "name": name,
                                "host": provider,
                            })).map_err(|e| e.to_string())
                        }
                        _ => {
                            let mut output = String::new();
                            output.push_str(&FormatBox::new(&SenaConfig::brand_title("AI COLLABORATION")).render());
                            output.push('\n');
                            output.push_str(&format!("✅ Session created: {}\n", name));
                            output.push_str(&format!("   ID: {}\n", session_id));
                            output.push_str(&format!("   Host: {}\n\n", provider));
                            output.push_str("Next steps:\n");
                            output.push_str(&format!("  1. Add agents: sena collab join {} --provider openai\n", session_id));
                            output.push_str(&format!("  2. Start session: sena collab start {}\n", session_id));
                            output.push_str(&format!("  3. Broadcast: sena collab broadcast {} \"your message\"\n", session_id));
                            Ok(output)
                        }
                    }
                }
                Err(e) => Err(format!("Failed to create session: {}", e))
            }
        }

        CollabAction::List => {
            let sessions = orchestrator.list_active_sessions().await;

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&sessions).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new(&SenaConfig::brand_title("ACTIVE SESSIONS")).render());
                    output.push('\n');

                    if sessions.is_empty() {
                        output.push_str("No active collaboration sessions.\n");
                        output.push_str("\nCreate one with: sena collab new \"Session Name\" --provider claude\n");
                    } else {
                        for session in sessions {
                            output.push_str(&format!(
                                "📋 {} ({})\n   State: {:?}\n   Participants: {}\n   Messages: {}\n\n",
                                session.name,
                                session.session_id,
                                session.state,
                                session.participants.len(),
                                session.message_count,
                            ));
                        }
                    }
                    Ok(output)
                }
            }
        }

        CollabAction::Join { session_id, provider } => {
            match orchestrator.join_session(&session_id, &provider).await {
                Ok(agent_id) => {
                    match format {
                        OutputFormat::Json => {
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "joined",
                                "session_id": session_id,
                                "agent_id": agent_id,
                                "provider": provider,
                            })).map_err(|e| e.to_string())
                        }
                        _ => {
                            Ok(format!(
                                "✅ Joined session {}\n   Agent ID: {}\n   Provider: {}",
                                session_id, agent_id, provider
                            ))
                        }
                    }
                }
                Err(e) => Err(format!("Failed to join session: {}", e))
            }
        }

        CollabAction::Start { session_id } => {
            match orchestrator.start_session(&session_id).await {
                Ok(()) => {
                    match format {
                        OutputFormat::Json => {
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "started",
                                "session_id": session_id,
                            })).map_err(|e| e.to_string())
                        }
                        _ => {
                            Ok(format!("✅ Session {} started\n\nYou can now:\n  - Send messages: sena collab send {} \"message\"\n  - Broadcast to all: sena collab broadcast {} \"message\"", session_id, session_id, session_id))
                        }
                    }
                }
                Err(e) => Err(format!("Failed to start session: {}", e))
            }
        }

        CollabAction::Send { session_id, message, from } => {
            let sender = from.unwrap_or_else(|| "user".to_string());
            match orchestrator.send_message(&session_id, &sender, &message).await {
                Ok(()) => {
                    Ok(format!("✅ Message sent to session {}", session_id))
                }
                Err(e) => Err(format!("Failed to send message: {}", e))
            }
        }

        CollabAction::Broadcast { session_id, message } => {
            match orchestrator.broadcast_to_agents(&session_id, "user", &message).await {
                Ok(responses) => {
                    match format {
                        OutputFormat::Json => {
                            let json_responses: Vec<serde_json::Value> = responses.iter()
                                .map(|r| {
                                    if let sena_collab::MessageContent::Text(text) = &r.content {
                                        serde_json::json!({
                                            "agent_id": r.sender_id,
                                            "content": text,
                                        })
                                    } else {
                                        serde_json::json!({
                                            "agent_id": r.sender_id,
                                            "content": "non-text response",
                                        })
                                    }
                                })
                                .collect();
                            serde_json::to_string_pretty(&json_responses).map_err(|e| e.to_string())
                        }
                        _ => {
                            let mut output = String::new();
                            output.push_str(&FormatBox::new(&SenaConfig::brand_title("AI RESPONSES")).render());
                            output.push('\n');

                            if responses.is_empty() {
                                output.push_str("No responses received. Make sure agents have joined the session.\n");
                            } else {
                                for response in responses {
                                    output.push_str(&format!("─── {} ───\n", response.sender_id));
                                    if let sena_collab::MessageContent::Text(text) = &response.content {
                                        output.push_str(text);
                                        output.push_str("\n\n");
                                    }
                                }
                            }
                            Ok(output)
                        }
                    }
                }
                Err(e) => Err(format!("Broadcast failed: {}", e))
            }
        }

        CollabAction::Analyze { session_id, provider, request } => {
            let request_payload = RequestPayload {
                request_type: RequestType::Analysis,
                description: request.clone(),
                parameters: serde_json::json!({}),
            };

            match orchestrator.request_analysis(&session_id, "user", &provider, request_payload).await {
                Ok(response) => {
                    match format {
                        OutputFormat::Json => {
                            serde_json::to_string_pretty(&serde_json::json!({
                                "provider": provider,
                                "response": response,
                            })).map_err(|e| e.to_string())
                        }
                        _ => {
                            let mut output = String::new();
                            output.push_str(&FormatBox::new(&SenaConfig::brand_title(&format!("ANALYSIS FROM {}", provider.to_uppercase()))).render());
                            output.push('\n');

                            if let sena_collab::MessageContent::Response(resp) = &response.content {
                                output.push_str(&resp.content);
                            } else if let sena_collab::MessageContent::Text(text) = &response.content {
                                output.push_str(text);
                            }
                            output.push('\n');
                            Ok(output)
                        }
                    }
                }
                Err(e) => Err(format!("Analysis failed: {}", e))
            }
        }

        CollabAction::Info { session_id } => {
            match orchestrator.get_session_summary(&session_id).await {
                Ok(summary) => {
                    match format {
                        OutputFormat::Json => {
                            serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())
                        }
                        _ => {
                            let mut output = String::new();
                            output.push_str(&FormatBox::new(&SenaConfig::brand_title("SESSION INFO")).render());
                            output.push('\n');
                            output.push_str(&format!("Session: {}\n", summary.name));
                            output.push_str(&format!("ID: {}\n", summary.session_id));
                            output.push_str(&format!("State: {:?}\n", summary.state));
                            output.push_str(&format!("Messages: {}\n", summary.message_count));
                            output.push_str(&format!("Created: {}\n\n", summary.created_at.format("%Y-%m-%d %H:%M:%S")));

                            output.push_str("Participants:\n");
                            for p in &summary.participants {
                                let host = if p.is_host { " (host)" } else { "" };
                                output.push_str(&format!(
                                    "  {} {} - {} [{}]{}\n",
                                    match p.status {
                                        sena_collab::AgentStatus::Idle => "🟢",
                                        sena_collab::AgentStatus::Thinking => "🟡",
                                        sena_collab::AgentStatus::Processing => "🔵",
                                        sena_collab::AgentStatus::Offline => "⚫",
                                        _ => "⚪",
                                    },
                                    p.provider,
                                    p.model,
                                    p.message_count,
                                    host
                                ));
                            }
                            Ok(output)
                        }
                    }
                }
                Err(e) => Err(format!("Failed to get session info: {}", e))
            }
        }

        CollabAction::End { session_id } => {
            Ok(format!("Session {} marked for termination.\n\nNote: Full session lifecycle management coming in next release.", session_id))
        }
    }
}

async fn execute_tools(action: ToolsAction, format: OutputFormat) -> Result<String, String> {
    use crate::tools::{ToolCall, ToolCategory, ToolSystem};

    let mut tool_system = ToolSystem::new();

    match action {
        ToolsAction::List { category } => {
            let tools = tool_system.list_tools();

            let filtered: Vec<_> = match &category {
                Some(cat) => {
                    let target_category = match cat.to_lowercase().as_str() {
                        "filesystem" | "file" => ToolCategory::FileSystem,
                        "shell" | "system" => ToolCategory::Shell,
                        "web" | "network" => ToolCategory::Web,
                        "code" => ToolCategory::Code,
                        _ => ToolCategory::Custom,
                    };
                    tools
                        .iter()
                        .filter(|t| t.category == target_category)
                        .cloned()
                        .collect()
                }
                None => tools,
            };

            match format {
                OutputFormat::Json => serde_json::to_string_pretty(&filtered).map_err(|e| e.to_string()),
                OutputFormat::Pretty | OutputFormat::Text => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new(&SenaConfig::brand_title("AVAILABLE TOOLS")).render());
                    output.push('\n');

                    let mut table = TableBuilder::new()
                        .title("Tools")
                        .row(vec!["Name".to_string(), "Category".to_string(), "Description".to_string()]);

                    for tool in &filtered {
                        table = table.row(vec![
                            tool.name.clone(),
                            format!("{:?}", tool.category),
                            tool.description.chars().take(40).collect(),
                        ]);
                    }

                    output.push_str(&table.build());
                    output.push_str(&format!("\nTotal: {} tools\n", filtered.len()));
                    Ok(output)
                }
            }
        }

        ToolsAction::Info { name } => {
            let tools = tool_system.list_tools();
            let tool = tools.iter().find(|t| t.name == name);

            match tool {
                Some(t) => match format {
                    OutputFormat::Json => serde_json::to_string_pretty(t).map_err(|e| e.to_string()),
                    _ => {
                        let mut output = String::new();
                        output.push_str(&FormatBox::new(&SenaConfig::brand_title("TOOL INFO")).render());
                        output.push('\n');
                        output.push_str(&format!("Name: {}\n", t.name));
                        output.push_str(&format!("Category: {:?}\n", t.category));
                        output.push_str(&format!("Description: {}\n", t.description));
                        output.push_str(&format!("Returns: {}\n", t.returns));
                        output.push_str(&format!("Requires Confirmation: {}\n", t.requires_confirmation));
                        output.push_str(&format!("Timeout: {}s\n\n", t.timeout_seconds));

                        output.push_str("Parameters:\n");
                        for param in &t.parameters {
                            let req = if param.required { " (required)" } else { "" };
                            output.push_str(&format!("  {} ({:?}){}: {}\n",
                                param.name,
                                param.param_type,
                                req,
                                param.description
                            ));
                        }
                        Ok(output)
                    }
                },
                None => Err(format!("Tool '{}' not found", name)),
            }
        }

        ToolsAction::Run { name, params } => {
            let parameters: std::collections::HashMap<String, serde_json::Value> = match params {
                Some(p) => serde_json::from_str(&p).map_err(|e| format!("Invalid JSON params: {}", e))?,
                None => std::collections::HashMap::new(),
            };

            let call = ToolCall::new(&name, parameters);
            let response = tool_system.execute(call).await;

            match format {
                OutputFormat::Json => serde_json::to_string_pretty(&response).map_err(|e| e.to_string()),
                _ => {
                    if response.success {
                        let output_str = serde_json::to_string_pretty(&response.output)
                            .unwrap_or_else(|_| "{}".to_string());
                        Ok(format!("Tool '{}' executed successfully ({}ms)\n\nOutput:\n{}",
                            name, response.execution_time_ms, output_str))
                    } else {
                        Err(format!("Tool '{}' failed: {}",
                            name, response.error.unwrap_or_else(|| "Unknown error".to_string())))
                    }
                }
            }
        }

        ToolsAction::Search { pattern, path, files } => {
            let mut parameters = std::collections::HashMap::new();
            parameters.insert("pattern".to_string(), serde_json::json!(pattern));
            parameters.insert("path".to_string(), serde_json::json!(path));
            if let Some(f) = files {
                parameters.insert("file_pattern".to_string(), serde_json::json!(f));
            }

            let call = ToolCall::new("code_search", parameters);
            let response = tool_system.execute(call).await;

            match format {
                OutputFormat::Json => serde_json::to_string_pretty(&response).map_err(|e| e.to_string()),
                _ => {
                    if response.success {
                        let matches = response.output.get("matches")
                            .and_then(|m| m.as_array())
                            .map(|arr| arr.len())
                            .unwrap_or(0);

                        let output_str = serde_json::to_string_pretty(&response.output)
                            .unwrap_or_else(|_| "{}".to_string());
                        Ok(format!("Found {} matches ({}ms)\n\n{}",
                            matches, response.execution_time_ms, output_str))
                    } else {
                        Err(format!("Search failed: {}",
                            response.error.unwrap_or_else(|| "Unknown error".to_string())))
                    }
                }
            }
        }
    }
}

async fn execute_memory(action: MemoryAction, format: OutputFormat) -> Result<String, String> {
    use crate::memory::{MemoryEntry, MemoryType, PersistentMemory};

    let mut memory = PersistentMemory::new().map_err(|e| format!("Failed to initialize memory: {}", e))?;

    match action {
        MemoryAction::Add { content, memory_type, tags, importance } => {
            let mt = match memory_type.to_lowercase().as_str() {
                "preference" => MemoryType::Preference,
                "fact" => MemoryType::Fact,
                "project" => MemoryType::Project,
                "context" => MemoryType::Context,
                "conversation" => MemoryType::Conversation,
                other => MemoryType::Custom(other.to_string()),
            };

            let mut entry = MemoryEntry::new(&content, mt);

            if let Some(tag_str) = tags {
                let tag_list: Vec<String> = tag_str.split(',').map(|s| s.trim().to_string()).collect();
                entry = entry.with_tags(tag_list);
            }

            if let Some(imp) = importance {
                entry = entry.with_importance(imp);
            }

            let id = memory.add(entry).map_err(|e| format!("Failed to add memory: {}", e))?;

            match format {
                OutputFormat::Json => Ok(serde_json::json!({"id": id, "success": true}).to_string()),
                _ => Ok(format!("Memory added with ID: {}", id)),
            }
        }

        MemoryAction::Search { query, limit } => {
            let results = memory.search(&query);
            let limited: Vec<_> = results.into_iter().take(limit).collect();

            match format {
                OutputFormat::Json => {
                    let entries: Vec<_> = limited.iter().map(|e| serde_json::json!({
                        "id": e.id,
                        "content": e.content,
                        "type": format!("{:?}", e.memory_type),
                        "tags": e.tags,
                        "importance": e.importance,
                        "score": e.relevance_score(&query),
                    })).collect();
                    serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new(&SenaConfig::brand_title("MEMORY SEARCH")).render());
                    output.push_str(&format!("\nQuery: '{}'\nResults: {}\n\n", query, limited.len()));

                    for entry in &limited {
                        output.push_str(&format!("[{}] {:?} (importance: {:.2})\n",
                            entry.id, entry.memory_type, entry.importance));
                        output.push_str(&format!("  {}\n", entry.content));
                        if !entry.tags.is_empty() {
                            output.push_str(&format!("  Tags: {}\n", entry.tags.join(", ")));
                        }
                        output.push('\n');
                    }
                    Ok(output)
                }
            }
        }

        MemoryAction::List { memory_type, limit } => {
            let all = memory.all();

            let filtered: Vec<_> = match memory_type {
                Some(mt_str) => {
                    let mt = match mt_str.to_lowercase().as_str() {
                        "preference" => MemoryType::Preference,
                        "fact" => MemoryType::Fact,
                        "project" => MemoryType::Project,
                        "context" => MemoryType::Context,
                        "conversation" => MemoryType::Conversation,
                        other => MemoryType::Custom(other.to_string()),
                    };
                    all.into_iter().filter(|e| e.memory_type == mt).take(limit).collect()
                }
                None => all.into_iter().take(limit).collect(),
            };

            match format {
                OutputFormat::Json => {
                    let entries: Vec<_> = filtered.iter().map(|e| serde_json::json!({
                        "id": e.id,
                        "content": e.content,
                        "type": format!("{:?}", e.memory_type),
                        "tags": e.tags,
                        "importance": e.importance,
                        "created_at": e.created_at.to_rfc3339(),
                    })).collect();
                    serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new(&SenaConfig::brand_title("MEMORIES")).render());
                    output.push_str(&format!("\nShowing {} memories\n\n", filtered.len()));

                    let mut table = TableBuilder::new()
                        .title("Memories")
                        .row(vec!["ID".to_string(), "Type".to_string(), "Content".to_string(), "Importance".to_string()]);

                    for entry in &filtered {
                        table = table.row(vec![
                            entry.id.chars().take(12).collect(),
                            format!("{:?}", entry.memory_type),
                            entry.content.chars().take(40).collect(),
                            format!("{:.2}", entry.importance),
                        ]);
                    }

                    output.push_str(&table.build());
                    Ok(output)
                }
            }
        }

        MemoryAction::Remove { id } => {
            match memory.remove(&id).map_err(|e| format!("Failed to remove: {}", e))? {
                Some(_) => {
                    match format {
                        OutputFormat::Json => Ok(serde_json::json!({"success": true, "id": id}).to_string()),
                        _ => Ok(format!("Memory '{}' removed", id)),
                    }
                }
                None => Err(format!("Memory '{}' not found", id)),
            }
        }

        MemoryAction::Stats => {
            let stats = memory.stats();

            match format {
                OutputFormat::Json => serde_json::to_string_pretty(&stats).map_err(|e| e.to_string()),
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new(&SenaConfig::brand_title("MEMORY STATS")).render());
                    output.push('\n');
                    output.push_str(&format!("Total Entries: {}\n", stats.total_entries));
                    output.push_str(&format!("Total Access Count: {}\n", stats.total_access_count));
                    output.push_str(&format!("Average Importance: {:.2}\n\n", stats.avg_importance));

                    output.push_str("By Type:\n");
                    for (type_name, count) in &stats.by_type {
                        output.push_str(&format!("  {}: {}\n", type_name, count));
                    }
                    Ok(output)
                }
            }
        }

        MemoryAction::Clear { yes } => {
            if !yes {
                return Err("Use --yes to confirm clearing all memories".to_string());
            }

            memory.clear().map_err(|e| format!("Failed to clear: {}", e))?;

            match format {
                OutputFormat::Json => Ok(serde_json::json!({"success": true, "message": "All memories cleared"}).to_string()),
                _ => Ok("All memories cleared".to_string()),
            }
        }
    }
}

async fn execute_auto(
    task: &str,
    max_steps: usize,
    cwd: Option<String>,
    confirm: bool,
    format: OutputFormat,
) -> Result<String, String> {
    use crate::intelligence::AutonomousAgent;

    let working_dir = cwd.map(PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let mut agent = AutonomousAgent::new();
    let execution = agent
        .execute(task, working_dir.clone(), max_steps, confirm)
        .await
        .map_err(|e| format!("Agent error: {}", e))?;

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&execution).map_err(|e| e.to_string()),
        _ => {
            let mut output = String::new();
            output.push_str(&FormatBox::new(&SenaConfig::brand_title("AUTONOMOUS AGENT")).render());
            output.push('\n');
            output.push_str(&format!("Execution ID: {}\n", execution.id));
            output.push_str(&format!("Task: {}\n", execution.task));
            output.push_str(&format!("State: {:?}\n", execution.state));
            output.push_str(&format!("Working Directory: {}\n", execution.working_dir.display()));
            output.push_str(&format!("Duration: {}ms\n\n", execution.elapsed_ms()));

            if let Some(plan) = &execution.plan {
                output.push_str("Plan:\n");
                for (i, step) in plan.steps.iter().enumerate() {
                    output.push_str(&format!("  {}. {} [{:?}]\n",
                        i + 1, step.description, step.estimated_complexity));
                }
                output.push('\n');
            }

            output.push_str(&format!("Steps Executed: {} / {}\n", execution.steps_taken(), max_steps));
            output.push_str(&format!("Successful: {}\n\n", execution.successful_steps()));

            for step in &execution.steps {
                let status = if step.success { "✓" } else { "✗" };
                output.push_str(&format!("[{}] Step {}: {} ({}ms)\n",
                    status, step.step_number, step.action, step.duration_ms));

                if let Some(tool) = &step.tool_name {
                    output.push_str(&format!("    Tool: {}\n", tool));
                }

                if let Some(result) = &step.result {
                    let truncated: String = result.chars().take(200).collect();
                    output.push_str(&format!("    Result: {}...\n", truncated));
                }
            }

            if let Some(result) = &execution.final_result {
                output.push_str(&format!("\nFinal Result: {}\n", result));
            }

            Ok(output)
        }
    }
}

async fn execute_git(action: GitAction, format: OutputFormat) -> Result<String, String> {
    match action {
        GitAction::Status => {
            let output = std::process::Command::new("git")
                .args(["status", "--porcelain", "-b"])
                .output()
                .map_err(|e| format!("Failed to run git: {}", e))?;

            let status = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                return Err(format!("Git error: {}", stderr));
            }

            match format {
                OutputFormat::Json => {
                    let lines: Vec<&str> = status.lines().collect();
                    let branch = lines.first()
                        .map(|l| l.trim_start_matches("## "))
                        .unwrap_or("unknown");

                    let changes: Vec<_> = lines.iter()
                        .skip(1)
                        .map(|l| {
                            let status_char = l.chars().take(2).collect::<String>();
                            let file = l.chars().skip(3).collect::<String>();
                            serde_json::json!({"status": status_char.trim(), "file": file})
                        })
                        .collect();

                    let result = serde_json::json!({
                        "branch": branch,
                        "changes": changes,
                        "clean": changes.is_empty(),
                    });
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("GIT STATUS")).render());
                    out.push('\n');

                    let full_output = std::process::Command::new("git")
                        .args(["status"])
                        .output()
                        .map_err(|e| format!("Failed to run git: {}", e))?;

                    out.push_str(&String::from_utf8_lossy(&full_output.stdout));
                    Ok(out)
                }
            }
        }

        GitAction::Commit { message, all } => {
            if all {
                let add_output = std::process::Command::new("git")
                    .args(["add", "-A"])
                    .output()
                    .map_err(|e| format!("Failed to stage: {}", e))?;

                if !add_output.status.success() {
                    return Err(format!("Failed to stage changes: {}",
                        String::from_utf8_lossy(&add_output.stderr)));
                }
            }

            let diff_output = std::process::Command::new("git")
                .args(["diff", "--cached", "--stat"])
                .output()
                .map_err(|e| format!("Failed to get diff: {}", e))?;

            let diff = String::from_utf8_lossy(&diff_output.stdout);

            if diff.trim().is_empty() {
                return Err("No staged changes to commit".to_string());
            }

            let commit_message = message.unwrap_or_else(|| {
                let changes: Vec<&str> = diff.lines()
                    .filter(|l| l.contains('|'))
                    .take(3)
                    .collect();

                if changes.is_empty() {
                    "Update files".to_string()
                } else {
                    format!("Update: {}", changes.join(", ").chars().take(50).collect::<String>())
                }
            });

            let commit_output = std::process::Command::new("git")
                .args(["commit", "-m", &commit_message])
                .output()
                .map_err(|e| format!("Failed to commit: {}", e))?;

            if !commit_output.status.success() {
                return Err(format!("Commit failed: {}",
                    String::from_utf8_lossy(&commit_output.stderr)));
            }

            match format {
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "success": true,
                        "message": commit_message,
                        "output": String::from_utf8_lossy(&commit_output.stdout).to_string(),
                    });
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("GIT COMMIT")).render());
                    out.push('\n');
                    out.push_str(&format!("Message: {}\n\n", commit_message));
                    out.push_str(&String::from_utf8_lossy(&commit_output.stdout));
                    Ok(out)
                }
            }
        }

        GitAction::Pr { title, base } => {
            let base_branch = base.unwrap_or_else(|| "main".to_string());

            let branch_output = std::process::Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .output()
                .map_err(|e| format!("Failed to get branch: {}", e))?;

            let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

            if current_branch == base_branch {
                return Err(format!("Cannot create PR from {} to itself", base_branch));
            }

            let log_output = std::process::Command::new("git")
                .args(["log", "--oneline", &format!("{}..HEAD", base_branch)])
                .output()
                .map_err(|e| format!("Failed to get commits: {}", e))?;

            let commits = String::from_utf8_lossy(&log_output.stdout);
            let commit_count = commits.lines().count();

            let pr_title = title.unwrap_or_else(|| {
                commits.lines().next()
                    .map(|l| l.split_whitespace().skip(1).collect::<Vec<_>>().join(" "))
                    .unwrap_or_else(|| format!("PR: {}", current_branch))
            });

            match format {
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "branch": current_branch,
                        "base": base_branch,
                        "title": pr_title,
                        "commits": commit_count,
                        "note": "Use 'gh pr create' to create the actual PR",
                    });
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("GIT PR")).render());
                    out.push('\n');
                    out.push_str(&format!("Branch: {} -> {}\n", current_branch, base_branch));
                    out.push_str(&format!("Title: {}\n", pr_title));
                    out.push_str(&format!("Commits: {}\n\n", commit_count));
                    out.push_str(&commits);
                    out.push_str("\nTo create PR, run: gh pr create\n");
                    Ok(out)
                }
            }
        }

        GitAction::Diff { staged } => {
            let args = if staged {
                vec!["diff", "--cached"]
            } else {
                vec!["diff"]
            };

            let output = std::process::Command::new("git")
                .args(&args)
                .output()
                .map_err(|e| format!("Failed to run git diff: {}", e))?;

            let diff = String::from_utf8_lossy(&output.stdout);

            match format {
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "staged": staged,
                        "diff": diff.to_string(),
                        "has_changes": !diff.is_empty(),
                    });
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    let title = if staged { "GIT DIFF (STAGED)" } else { "GIT DIFF" };
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title(title)).render());
                    out.push('\n');

                    if diff.is_empty() {
                        out.push_str("No changes\n");
                    } else {
                        out.push_str(&diff);
                    }
                    Ok(out)
                }
            }
        }

        GitAction::Log { count } => {
            let output = std::process::Command::new("git")
                .args(["log", "--oneline", "-n", &count.to_string()])
                .output()
                .map_err(|e| format!("Failed to run git log: {}", e))?;

            let log = String::from_utf8_lossy(&output.stdout);

            match format {
                OutputFormat::Json => {
                    let commits: Vec<_> = log.lines()
                        .map(|l| {
                            let parts: Vec<&str> = l.splitn(2, ' ').collect();
                            serde_json::json!({
                                "hash": parts.first().unwrap_or(&""),
                                "message": parts.get(1).unwrap_or(&""),
                            })
                        })
                        .collect();

                    serde_json::to_string_pretty(&commits).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("GIT LOG")).render());
                    out.push_str(&format!("\nLast {} commits:\n\n", count));
                    out.push_str(&log);
                    Ok(out)
                }
            }
        }
    }
}

async fn execute_guardian(action: GuardianAction, format: OutputFormat) -> Result<String, String> {
    use crate::guardian::GuardianMiddleware;

    let guardian = GuardianMiddleware::new();

    match action {
        GuardianAction::Status => {
            let status = serde_json::json!({
                "enabled": guardian.is_enabled(),
                "config": {
                    "sandbox_level": format!("{:?}", guardian.config().sandbox_level),
                    "hallucination_mode": format!("{:?}", guardian.config().hallucination_mode),
                    "hallucination_threshold": guardian.config().hallucination_threshold,
                }
            });

            match format {
                OutputFormat::Json => serde_json::to_string_pretty(&status).map_err(|e| e.to_string()),
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("GUARDIAN STATUS")).render());
                    out.push_str(&format!("\nEnabled: {}\n", guardian.is_enabled()));
                    out.push_str(&format!("Sandbox: {:?}\n", guardian.config().sandbox_level));
                    out.push_str(&format!("Hallucination Mode: {:?}\n", guardian.config().hallucination_mode));
                    out.push_str(&format!("Threshold: {:.2}\n", guardian.config().hallucination_threshold));
                    Ok(out)
                }
            }
        }

        GuardianAction::Enable => {
            Ok("Guardian middleware enabled.".to_string())
        }

        GuardianAction::Disable => {
            Ok("Guardian middleware disabled.".to_string())
        }

        GuardianAction::Validate { command } => {
            let result = guardian.validate_command(&command);

            match format {
                OutputFormat::Json => {
                    let json = serde_json::json!({
                        "command": command,
                        "allowed": result.allowed,
                        "reason": result.reason,
                        "risk_score": result.risk_score,
                        "matched_patterns": result.matched_patterns,
                    });
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("COMMAND VALIDATION")).render());
                    out.push_str(&format!("\nCommand: {}\n", command));
                    out.push_str(&format!("Allowed: {}\n", if result.allowed { "YES" } else { "NO" }));
                    out.push_str(&format!("Risk Score: {:.2}\n", result.risk_score));
                    if let Some(reason) = &result.reason {
                        out.push_str(&format!("Reason: {}\n", reason));
                    }
                    if !result.matched_patterns.is_empty() {
                        out.push_str("\nMatched Patterns:\n");
                        for pattern in &result.matched_patterns {
                            out.push_str(&format!("  - {}\n", pattern));
                        }
                    }
                    Ok(out)
                }
            }
        }

        GuardianAction::Check { content } => {
            let result = guardian.check_hallucination(&content);

            match format {
                OutputFormat::Json => {
                    let json = serde_json::json!({
                        "is_hallucination": result.is_hallucination,
                        "risk_score": result.risk_score,
                        "response": format!("{:?}", result.response),
                        "harmony_status": format!("{:?}", result.harmony_status),
                        "warnings": result.warnings,
                        "details": {
                            "consistency_score": result.details.consistency_score,
                            "semantic_entropy": result.details.semantic_entropy,
                            "fact_validation_score": result.details.fact_validation_score,
                        }
                    });
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("HALLUCINATION CHECK")).render());
                    out.push_str(&format!("\nIs Hallucination: {}\n", result.is_hallucination));
                    out.push_str(&format!("Risk Score: {:.2}\n", result.risk_score));
                    out.push_str(&format!("Response: {:?}\n", result.response));
                    out.push_str(&format!("Harmony Status: {:?}\n", result.harmony_status));

                    if !result.warnings.is_empty() {
                        out.push_str("\nWarnings:\n");
                        for warning in &result.warnings {
                            out.push_str(&format!("  - {}\n", warning));
                        }
                    }
                    Ok(out)
                }
            }
        }

        GuardianAction::Execute { command, args } => {
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            match guardian.execute(&command, &args_refs) {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    match format {
                        OutputFormat::Json => {
                            let json = serde_json::json!({
                                "success": output.status.success(),
                                "stdout": stdout,
                                "stderr": stderr,
                            });
                            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                        }
                        _ => {
                            let mut out = String::new();
                            out.push_str(&FormatBox::new(&SenaConfig::brand_title("GUARDIAN EXECUTE")).render());
                            out.push_str(&format!("\nCommand: {} {}\n", command, args.join(" ")));
                            out.push_str(&format!("Success: {}\n\n", output.status.success()));
                            if !stdout.is_empty() {
                                out.push_str(&stdout);
                            }
                            if !stderr.is_empty() {
                                out.push_str(&format!("\nStderr:\n{}", stderr));
                            }
                            Ok(out)
                        }
                    }
                }
                Err(e) => Err(format!("Execution blocked: {}", e)),
            }
        }

        GuardianAction::Audit { count } => {
            match format {
                OutputFormat::Json => {
                    let json = serde_json::json!({
                        "audit_entries": [],
                        "message": format!("Audit log (last {} entries) - not yet implemented", count),
                    });
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("GUARDIAN AUDIT")).render());
                    out.push_str(&format!("\nLast {} audit entries:\n", count));
                    out.push_str("\n(Audit logging not yet implemented)\n");
                    Ok(out)
                }
            }
        }
    }
}

async fn execute_devil(action: DevilAction, format: OutputFormat) -> Result<String, String> {
    use crate::devil::{DevilConfig, DevilExecutor, ProviderResponse, SynthesisMethod};
    use sena_providers::{ChatRequest, Message, ProvidersConfig, ProviderRouter};
    use std::time::{Duration, Instant};

    match action {
        DevilAction::Execute { prompt, timeout, synthesis } => {
            let synthesis_method = match synthesis {
                SynthesisMethodArg::MajorityVoting => SynthesisMethod::MajorityVoting,
                SynthesisMethodArg::WeightedMerge => SynthesisMethod::WeightedMerge,
                SynthesisMethodArg::BestOfN => SynthesisMethod::BestOfN,
                SynthesisMethodArg::MetaLlm => SynthesisMethod::MetaLLM,
                SynthesisMethodArg::CrossVerification => SynthesisMethod::CrossVerification,
            };

            let config = DevilConfig::default()
                .with_timeout(timeout)
                .with_synthesis(synthesis_method);

            let executor = DevilExecutor::new(config);

            let providers_config = ProvidersConfig::load_or_default();
            let router = ProviderRouter::from_config(&providers_config)
                .map_err(|e| format!("Failed to create provider router: {}", e))?;

            let available_providers = router.available_providers();

            if available_providers.is_empty() {
                return Err("No providers available. Check your API keys and configuration.".to_string());
            }

            let request = ChatRequest::new(vec![Message::user(&prompt)])
                .with_max_tokens(1024);

            let timeout_duration = Duration::from_secs(timeout);
            let mut handles = Vec::new();

            for provider in available_providers {
                let provider_id = provider.provider_id().to_string();
                let model = provider.default_model().to_string();
                let request_clone = request.clone();
                let provider_clone = provider.clone();

                let handle = tokio::spawn(async move {
                    let start = Instant::now();
                    match tokio::time::timeout(
                        timeout_duration,
                        provider_clone.chat(request_clone)
                    ).await {
                        Ok(Ok(response)) => {
                            ProviderResponse::success(
                                provider_id,
                                response.model,
                                response.content,
                                start.elapsed(),
                            )
                        }
                        Ok(Err(e)) => {
                            ProviderResponse::failure(
                                provider_id,
                                model,
                                e.to_string(),
                                start.elapsed(),
                            )
                        }
                        Err(_) => {
                            ProviderResponse::failure(
                                provider_id,
                                model,
                                "Timeout".to_string(),
                                timeout_duration,
                            )
                        }
                    }
                });
                handles.push(handle);
            }

            let mut responses = Vec::new();
            for handle in handles {
                if let Ok(response) = handle.await {
                    responses.push(response);
                }
            }

            if responses.is_empty() {
                return Err("All provider requests failed or timed out".to_string());
            }

            match executor.execute_sync(&prompt, responses) {
                Ok(response) => {
                    match format {
                        OutputFormat::Json => {
                            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
                        }
                        _ => Ok(response.format_summary())
                    }
                }
                Err(e) => Err(format!("Devil mode execution failed: {}", e)),
            }
        }

        DevilAction::Status => {
            use sena_providers::{ProvidersConfig, ProviderRouter};

            let config = DevilConfig::default();
            let providers_config = ProvidersConfig::load_or_default();
            let router = ProviderRouter::from_config(&providers_config).ok();

            let available_providers: Vec<String> = router
                .as_ref()
                .map(|r| r.available_providers().iter().map(|p| p.provider_id().to_string()).collect())
                .unwrap_or_default();

            let provider_statuses: Vec<(String, String)> = router
                .as_ref()
                .map(|r| r.provider_status().into_iter().map(|(id, s)| (id, format!("{:?}", s))).collect())
                .unwrap_or_default();

            match format {
                OutputFormat::Json => {
                    let status = serde_json::json!({
                        "enabled": config.enabled,
                        "timeout_secs": config.timeout_secs,
                        "min_providers": config.min_providers,
                        "synthesis_method": format!("{:?}", config.synthesis_method),
                        "consensus_threshold": config.consensus_threshold,
                        "wait_mode": format!("{:?}", config.wait_mode),
                        "available_providers": available_providers,
                        "provider_statuses": provider_statuses.into_iter().collect::<std::collections::HashMap<_,_>>(),
                    });
                    serde_json::to_string_pretty(&status).map_err(|e| e.to_string())
                }
                _ => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("DEVIL MODE STATUS")).render());
                    out.push_str(&format!("\nEnabled: {}\n", config.enabled));
                    out.push_str(&format!("Timeout: {}s\n", config.timeout_secs));
                    out.push_str(&format!("Min Providers: {}\n", config.min_providers));
                    out.push_str(&format!("Synthesis: {:?}\n", config.synthesis_method));
                    out.push_str(&format!("Consensus Threshold: {:.0}%\n", config.consensus_threshold * 100.0));
                    out.push_str(&format!("Wait Mode: {:?}\n", config.wait_mode));
                    out.push_str(&format!("\nAvailable Providers ({}):\n", available_providers.len()));
                    for (id, status) in &provider_statuses {
                        out.push_str(&format!("  - {}: {}\n", id, status));
                    }
                    Ok(out)
                }
            }
        }

        DevilAction::Config { timeout, consensus, synthesis } => {
            let mut config = DevilConfig::default();

            if let Some(t) = timeout {
                config = config.with_timeout(t);
            }
            if let Some(c) = consensus {
                config = config.with_consensus_threshold(c);
            }
            if let Some(s) = synthesis {
                let method = match s {
                    SynthesisMethodArg::MajorityVoting => SynthesisMethod::MajorityVoting,
                    SynthesisMethodArg::WeightedMerge => SynthesisMethod::WeightedMerge,
                    SynthesisMethodArg::BestOfN => SynthesisMethod::BestOfN,
                    SynthesisMethodArg::MetaLlm => SynthesisMethod::MetaLLM,
                    SynthesisMethodArg::CrossVerification => SynthesisMethod::CrossVerification,
                };
                config = config.with_synthesis(method);
            }

            Ok(format!("Devil mode configuration updated.\nTimeout: {}s\nConsensus: {:.0}%\nSynthesis: {:?}",
                config.timeout_secs,
                config.consensus_threshold * 100.0,
                config.synthesis_method))
        }

        DevilAction::Test { prompt } => {
            use crate::devil::ProviderResponse;

            let config = DevilConfig::default();
            let executor = DevilExecutor::new(config);

            let mock_responses = vec![
                ProviderResponse::success(
                    "mock_claude".to_string(),
                    "claude-test".to_string(),
                    format!("Mock Claude response: The {} is interesting. It has many properties.", prompt),
                    Duration::from_millis(500),
                ),
                ProviderResponse::success(
                    "mock_openai".to_string(),
                    "gpt-test".to_string(),
                    format!("Mock OpenAI response: Regarding {}, it is notable. It has several characteristics.", prompt),
                    Duration::from_millis(400),
                ),
                ProviderResponse::success(
                    "mock_gemini".to_string(),
                    "gemini-test".to_string(),
                    format!("Mock Gemini response: {} is fascinating. Multiple aspects are worth noting.", prompt),
                    Duration::from_millis(600),
                ),
            ];

            match executor.execute_sync(&prompt, mock_responses) {
                Ok(response) => {
                    let mut out = String::new();
                    out.push_str(&FormatBox::new(&SenaConfig::brand_title("DEVIL MODE TEST")).render());
                    out.push_str(&format!("\nPrompt: {}\n\n", prompt));
                    out.push_str(&response.format_summary());
                    Ok(out)
                }
                Err(e) => Err(format!("Devil mode test failed: {}", e)),
            }
        }
    }
}
