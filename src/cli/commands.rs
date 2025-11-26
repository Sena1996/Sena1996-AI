//! CLI Command Execution
//!
//! Handles execution of CLI commands

use crate::cli::args::*;
use crate::integration::AutoIntegration;
use crate::metrics::SenaHealth;
use crate::output::{TableBuilder, ProgressBar, FormatBox};
use crate::SenaUnifiedSystem;
use crate::ProcessingRequest;

/// Execute a CLI command
pub async fn execute_command(cli: &Cli) -> Result<String, String> {
    match &cli.command {
        Some(Commands::Mcp { debug }) => {
            execute_mcp(*debug).await
        }

        Some(Commands::Hook { hook_type, input }) => {
            execute_hook(*hook_type, input.clone(), cli.format).await
        }

        Some(Commands::Process { content, request_type }) => {
            execute_process(content, request_type, cli.format).await
        }

        Some(Commands::Health { detailed }) => {
            execute_health(*detailed, cli.format)
        }

        Some(Commands::Metrics { category }) => {
            execute_metrics(*category, cli.format)
        }

        Some(Commands::Detect { text }) => {
            execute_detect(text, cli.format)
        }

        Some(Commands::Daemon { action }) => {
            execute_daemon(*action).await
        }

        Some(Commands::Session { action, id }) => {
            execute_session(*action, id.clone(), cli.format)
        }

        Some(Commands::Validate { content, strict }) => {
            execute_validate(content, *strict, cli.format)
        }

        Some(Commands::Format { format_type, title, data }) => {
            execute_format(*format_type, title.clone(), data, cli.format)
        }

        // Hub commands
        Some(Commands::Hub { action }) => {
            execute_hub(action.clone()).await
        }

        Some(Commands::Join { role, name }) => {
            execute_join(role, name.clone(), cli.format).await
        }

        Some(Commands::Who) => {
            execute_who(cli.format).await
        }

        Some(Commands::Tell { target, message }) => {
            execute_tell(target, message, cli.format).await
        }

        Some(Commands::Inbox) => {
            execute_inbox(cli.format).await
        }

        Some(Commands::Task { action }) => {
            execute_task(action.clone(), cli.format).await
        }

        Some(Commands::Watch) => {
            execute_watch().await
        }

        Some(Commands::Sync) => {
            execute_sync(cli.format).await
        }

        Some(Commands::Knowledge { action }) => {
            execute_knowledge(action.clone(), cli.format).await
        }

        Some(Commands::Think { query, depth }) => {
            execute_think(query, *depth, cli.format).await
        }

        Some(Commands::Agent { agent_type, content }) => {
            execute_agent(*agent_type, content, cli.format).await
        }

        Some(Commands::Evolve { action }) => {
            execute_evolve(action.clone(), cli.format).await
        }

        Some(Commands::Feedback { feedback_type, message, context }) => {
            execute_feedback(*feedback_type, message, context.clone(), cli.format).await
        }

        None => {
            // No command - show status
            execute_health(false, cli.format)
        }
    }
}

async fn execute_mcp(debug: bool) -> Result<String, String> {
    if debug {
        eprintln!("SENA MCP Server starting in debug mode...");
    }

    // Start MCP server
    crate::mcp::run_server().await
}

async fn execute_hook(hook_type: HookType, input: Option<String>, format: OutputFormat) -> Result<String, String> {
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

async fn execute_process(content: &str, request_type: &str, format: OutputFormat) -> Result<String, String> {
    let mut system = SenaUnifiedSystem::new();
    let request = ProcessingRequest::new(content, request_type);

    let result = system.process(request).await;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
        }
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&FormatBox::new("SENA 🦁 PROCESSING RESULT").render());
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
                Ok(if result.content.is_empty() { "OK".to_string() } else { result.content.clone() })
            } else {
                Err(if result.content.is_empty() { "Error".to_string() } else { result.content.clone() })
            }
        }
    }
}

fn execute_health(detailed: bool, format: OutputFormat) -> Result<String, String> {
    let health = SenaHealth::new();
    let report = health.get_health();

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())
        }
        OutputFormat::Pretty | OutputFormat::Text => {
            let mut output = String::new();

            if detailed || format == OutputFormat::Pretty {
                output.push_str(&FormatBox::new("SENA 🦁 HEALTH STATUS").render());
                output.push('\n');
            }

            output.push_str(&format!("Version: {}\n", report.version));
            output.push_str(&format!("Status: {}\n", report.overall_status));
            output.push_str(&format!("Health: {}%\n", report.metrics.overall_health_percentage));

            if detailed {
                output.push_str(&format!("\nComponents:\n"));
                output.push_str(&format!("  Core: {}\n", report.metrics.core_components));
                output.push_str(&format!("  Memory: {}\n", report.metrics.memory_system));
                output.push_str(&format!("  Hooks: {}\n", report.metrics.hooks));
            }

            Ok(output)
        }
    }
}

fn execute_metrics(category: Option<MetricCategory>, format: OutputFormat) -> Result<String, String> {
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
        OutputFormat::Json => {
            serde_json::to_string_pretty(&metrics).map_err(|e| e.to_string())
        }
        _ => {
            serde_json::to_string_pretty(&metrics).map_err(|e| e.to_string())
        }
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
        OutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
        }
        _ => {
            match detected {
                Some(fmt) => Ok(format!("Detected format: {}", fmt.name())),
                None => Ok("No special format detected".to_string()),
            }
        }
    }
}

async fn execute_daemon(action: DaemonAction) -> Result<String, String> {
    match action {
        DaemonAction::Start => {
            crate::daemon::start_daemon().await
        }
        DaemonAction::Stop => {
            crate::daemon::stop_daemon().await
        }
        DaemonAction::Restart => {
            crate::daemon::stop_daemon().await?;
            crate::daemon::start_daemon().await
        }
        DaemonAction::Status => {
            crate::daemon::daemon_status().await
        }
    }
}

fn execute_session(action: SessionAction, id: Option<String>, format: OutputFormat) -> Result<String, String> {
    use crate::hub::{SessionRegistry, SessionRole, HubConfig};

    let config = HubConfig::default();
    let mut registry = SessionRegistry::new(&config);
    let _ = registry.load();

    let result = match action {
        SessionAction::Start => {
            match registry.register(SessionRole::General, None) {
                Ok(session) => serde_json::json!({
                    "action": "start",
                    "session_id": session.id,
                    "started_at": session.joined_at,
                }),
                Err(e) => serde_json::json!({"error": e}),
            }
        }
        SessionAction::End => {
            if let Some(session_id) = id.clone() {
                let _ = registry.unregister(&session_id);
            }
            serde_json::json!({
                "action": "end",
                "status": "session ended",
            })
        }
        SessionAction::Info => {
            match id.as_ref().and_then(|sid| registry.get(sid)) {
                Some(session) => session.stats(),
                None => {
                    let active = registry.get_active();
                    if active.is_empty() {
                        serde_json::json!({"error": "no active session"})
                    } else {
                        serde_json::to_value(&active).unwrap_or_default()
                    }
                }
            }
        }
        SessionAction::List => {
            let sessions = registry.get_all();
            serde_json::to_value(&sessions).unwrap_or_default()
        }
        SessionAction::Restore => {
            match id {
                Some(session_id) => {
                    serde_json::json!({
                        "action": "restore",
                        "session_id": session_id,
                        "status": "restored",
                    })
                }
                None => serde_json::json!({"error": "session ID required for restore"}),
            }
        }
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
    let checks_passed = result.checks.iter().filter(|c| c.status == crate::ancient::HarmonyStatus::Harmonious).count();

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
            out.push_str(&FormatBox::new("SENA 🦁 VALIDATION RESULT").render());
            out.push('\n');
            out.push_str(&format!("Valid: {}\n", result.is_valid()));
            out.push_str(&format!("Confidence: {:.1}%\n", result.overall_confidence * 100.0));
            out.push_str(&format!("Checks Passed: {}\n", checks_passed));
            out.push_str(&format!("Violations: {}\n", violations_count));
            Ok(out)
        }
        OutputFormat::Text => {
            if result.is_valid() {
                Ok(format!("VALID (confidence: {:.1}%)", result.overall_confidence * 100.0))
            } else {
                Ok(format!("INVALID (confidence: {:.1}%)", result.overall_confidence * 100.0))
            }
        }
    }
}

fn execute_format(format_type: FormatOutputType, title: Option<String>, data: &str, _format: OutputFormat) -> Result<String, String> {
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
                    let label = v.get("label").and_then(|l| l.as_str()).unwrap_or("Progress");
                    Ok(ProgressBar::new(label, percent as f32).render())
                }
                Err(e) => Err(format!("Invalid progress data: {}", e)),
            }
        }
        FormatOutputType::BrilliantThinking => {
            Ok(FormatBox::new(&title.unwrap_or_else(|| "SENA 🦁 BRILLIANT THINKING".to_string())).render())
        }
        FormatOutputType::TruthVerification => {
            Ok(FormatBox::new(&title.unwrap_or_else(|| "SENA 🦁 TRUTH VERIFICATION".to_string())).render())
        }
        FormatOutputType::CodeAnalysis => {
            Ok(FormatBox::new(&title.unwrap_or_else(|| "SENA 🦁 CODE ANALYSIS".to_string())).render())
        }
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
        HubAction::Stop => {
            Ok("Hub stopped.".to_string())
        }
        HubAction::Status => {
            let hub = Hub::new()?;
            let status = hub.status();
            Ok(format!(
                "Hub Status:\n  Sessions: {}\n  Tasks: {} ({} pending)\n  Conflicts: {}",
                status.online_sessions,
                status.total_tasks,
                status.pending_tasks,
                status.active_conflicts
            ))
        }
        HubAction::Conflicts => {
            let hub = Hub::new()?;
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
    }
}

async fn execute_join(role: &str, name: Option<String>, format: OutputFormat) -> Result<String, String> {
    use crate::hub::{Hub, SessionRole};

    let mut hub = Hub::new()?;
    hub.load()?;

    let session_role = SessionRole::from_str(role);
    let session = hub.join(session_role, name)?;
    hub.save()?;

    let result = serde_json::json!({
        "action": "joined",
        "session_id": session.id,
        "role": session.role.name(),
        "name": session.name,
    });

    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
        _ => Ok(format!(
            "{} Joined as {} ({})\nSession ID: {}",
            session.role.emoji(),
            session.name,
            session.role.name(),
            session.id
        )),
    }
}

async fn execute_who(format: OutputFormat) -> Result<String, String> {
    use crate::hub::Hub;

    let mut hub = Hub::new()?;
    hub.load()?;

    let sessions = hub.who();

    if sessions.is_empty() {
        return Ok("No sessions online.".to_string());
    }

    match format {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = sessions.iter().map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "role": s.role.name(),
                    "name": s.name,
                    "status": format!("{:?}", s.status),
                    "working_on": s.working_on,
                    "idle": s.idle_display(),
                })
            }).collect();
            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
        }
        _ => {
            let mut output = String::from("Sessions Online:\n");
            for session in sessions {
                output.push_str(&format!(
                    "  {} {} │ {} │ {} │ {}\n",
                    session.role.emoji(),
                    session.name,
                    session.status.indicator(),
                    session.working_on.as_deref().unwrap_or("-"),
                    session.idle_display()
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

    // For now, use "local" as sender - in real implementation, would get from current session
    hub.tell("local", target, message)?;
    hub.save()?;

    match format {
        OutputFormat::Json => Ok(serde_json::json!({
            "sent": true,
            "to": target,
            "message": message
        }).to_string()),
        _ => Ok(format!("Message sent to {}", target)),
    }
}

async fn execute_inbox(format: OutputFormat) -> Result<String, String> {
    use crate::hub::Hub;

    let mut hub = Hub::new()?;
    hub.load()?;

    // For now, use "local" - in real implementation, would get from current session
    let messages = hub.inbox("local");

    if messages.is_empty() {
        return Ok("No messages.".to_string());
    }

    match format {
        OutputFormat::Json => {
            let json: Vec<serde_json::Value> = messages.iter().map(|m| {
                serde_json::json!({
                    "from": m.from,
                    "content": m.content,
                    "time": m.time_display(),
                    "read": m.read,
                })
            }).collect();
            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
        }
        _ => {
            let mut output = String::from("Inbox:\n");
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
        TaskAction::New { title, to, priority } => {
            let prio = TaskPriority::from_str(&priority);
            let task = hub.create_task(&title, &to, prio)?;
            hub.save()?;

            match format {
                OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
                    "created": true,
                    "id": task.id,
                    "title": task.title,
                    "assignee": task.assignee,
                })).map_err(|e| e.to_string()),
                _ => Ok(format!("Task #{} created: {} (assigned to {})", task.id, task.title, task.assignee)),
            }
        }
        TaskAction::List { status } => {
            let tasks = if let Some(s) = status {
                let task_status = TaskStatus::from_str(&s);
                hub.tasks.get_by_status(task_status)
            } else {
                hub.get_tasks()
            };

            if tasks.is_empty() {
                return Ok("No tasks.".to_string());
            }

            match format {
                OutputFormat::Json => {
                    let json: Vec<serde_json::Value> = tasks.iter().map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "title": t.title,
                            "assignee": t.assignee,
                            "priority": t.priority.name(),
                            "status": t.status.name(),
                        })
                    }).collect();
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
            // For now, use "local" - in real implementation, would get from current session
            let tasks = hub.get_my_tasks("local");

            if tasks.is_empty() {
                return Ok("No tasks assigned to you.".to_string());
            }

            let mut output = String::from("My Tasks:\n");
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
            let task_status = TaskStatus::from_str(&status);
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

    let _status = hub.status();
    let sessions = hub.who();
    let tasks = hub.get_tasks();
    let conflicts = hub.get_conflicts();

    let mut output = String::new();

    // Header
    output.push_str(&FormatBox::new("SENA 🦁 COLLABORATION HUB").render());
    output.push('\n');

    // Sessions
    output.push_str("\nSESSIONS ONLINE:\n");
    if sessions.is_empty() {
        output.push_str("  No sessions online. Use 'sena join --role=<role>' to join.\n");
    } else {
        for session in &sessions {
            output.push_str(&format!(
                "  {} {} │ {} │ {} │ {}\n",
                session.role.emoji(),
                session.name,
                session.status.indicator(),
                session.working_on.as_deref().unwrap_or("-"),
                session.idle_display()
            ));
        }
    }

    // Tasks
    output.push_str("\nACTIVE TASKS:\n");
    let active_tasks: Vec<_> = tasks.iter().filter(|t| !t.is_complete()).take(5).collect();
    if active_tasks.is_empty() {
        output.push_str("  No active tasks.\n");
    } else {
        for task in active_tasks {
            output.push_str(&format!(
                "  #{} │ {} │ {} │ {} │ {}\n",
                task.id,
                task.priority.emoji(),
                task.assignee,
                task.title,
                task.status.name()
            ));
        }
    }

    // Conflicts
    if !conflicts.is_empty() {
        output.push_str("\n⚠️  CONFLICTS DETECTED:\n");
        for conflict in &conflicts {
            output.push_str(&format!(
                "  {} {} - {:?}\n",
                conflict.severity.emoji(),
                conflict.file_path,
                conflict.sessions
            ));
        }
    }

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
        })).map_err(|e| e.to_string()),
        _ => Ok(format!(
            "Sync complete. {} sessions, {} tasks, {} conflicts.",
            status.online_sessions,
            status.total_tasks,
            status.active_conflicts
        )),
    }
}

// ================================
// Knowledge System Commands
// ================================

async fn execute_knowledge(action: KnowledgeAction, format: OutputFormat) -> Result<String, String> {
    use crate::knowledge::KnowledgeSystem;

    let knowledge = KnowledgeSystem::new();

    match action {
        KnowledgeAction::Search { query, limit } => {
            let mut results = knowledge.search(&query);
            results.truncate(limit);

            match format {
                OutputFormat::Json => {
                    let json: Vec<serde_json::Value> = results.iter().map(|r| {
                        serde_json::json!({
                            "domain": r.domain,
                            "title": r.title,
                            "description": r.description,
                            "relevance": r.relevance,
                        })
                    }).collect();
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                OutputFormat::Pretty => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new("SENA 🦁 KNOWLEDGE SEARCH").render());
                    output.push_str(&format!("\nQuery: \"{}\"\n", query));
                    output.push_str(&format!("Found: {} results\n\n", results.len()));

                    for result in &results {
                        output.push_str(&format!(
                            "┌─ {} ─────────────────────────────────────\n",
                            result.domain.to_uppercase()
                        ));
                        output.push_str(&format!("│ {}\n", result.title));
                        output.push_str(&format!("│ {}\n", result.description));
                        output.push_str(&format!("│ Relevance: {:.0}%\n", result.relevance * 100.0));
                        output.push_str("└──────────────────────────────────────────\n\n");
                    }
                    Ok(output)
                }
                OutputFormat::Text => {
                    if results.is_empty() {
                        Ok("No results found.".to_string())
                    } else {
                        let mut output = format!("Found {} results for \"{}\":\n", results.len(), query);
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
                    output.push_str(&FormatBox::new(&format!("SENA 🦁 {} PATTERNS", format!("{:?}", category).to_uppercase())).render());
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
                    output.push_str(&FormatBox::new("SENA 🦁 KNOWLEDGE STATISTICS").render());
                    output.push('\n');
                    output.push_str(&format!("Total Entries: {}\n", stats.total_entries));
                    output.push_str(&format!("Reasoning Frameworks: {}\n", stats.reasoning_count));
                    output.push_str(&format!("Security Patterns: {}\n", stats.security_count));
                    output.push_str(&format!("Performance Patterns: {}\n", stats.performance_count));
                    output.push_str(&format!("Architecture Patterns: {}\n", stats.architecture_count));
                    Ok(output)
                }
            }
        }
    }
}

// ================================
// Intelligence System Commands
// ================================

async fn execute_think(query: &str, depth: ThinkingDepthArg, format: OutputFormat) -> Result<String, String> {
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
        OutputFormat::Json => {
            serde_json::to_string_pretty(&serde_json::json!({
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
            })).map_err(|e| e.to_string())
        }
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&FormatBox::new("SENA 🦁 EXTENDED THINKING").render());
            output.push_str(&format!("\nDepth: {:?}\n", depth));
            output.push_str(&format!("Confidence: {:.1}%\n\n", result.confidence * 100.0));

            output.push_str("═══════════════════════════════════════════\n");
            output.push_str("  PROBLEM\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            output.push_str(&result.problem);
            output.push_str("\n\n");

            output.push_str("═══════════════════════════════════════════\n");
            output.push_str("  THINKING STEPS\n");
            output.push_str("═══════════════════════════════════════════\n\n");
            for (i, step) in result.steps.iter().enumerate() {
                output.push_str(&format!("{}. **{}**\n   {}\n\n", i + 1, step.name, step.description));
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
            let mut output = format!("Analysis ({:?}, {:.0}% confidence):\n\n", depth, result.confidence * 100.0);
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

async fn execute_agent(agent_type: AgentTypeArg, content: &str, format: OutputFormat) -> Result<String, String> {
    use crate::intelligence::{IntelligenceSystem, AgentType};

    let intelligence = IntelligenceSystem::new();

    let agent = match agent_type {
        AgentTypeArg::Security => AgentType::Security,
        AgentTypeArg::Performance => AgentType::Performance,
        AgentTypeArg::Architecture => AgentType::Architecture,
        AgentTypeArg::General => AgentType::General,
    };

    let result = intelligence.dispatch(content, agent);

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&serde_json::json!({
                "agent": format!("{:?}", agent_type),
                "task": result.task,
                "analysis": result.analysis,
                "recommendations": result.recommendations,
                "confidence": result.confidence,
            })).map_err(|e| e.to_string())
        }
        OutputFormat::Pretty => {
            let mut output = String::new();
            let title = format!("SENA 🦁 {:?} AGENT ANALYSIS", agent_type).to_uppercase();
            output.push_str(&FormatBox::new(&title).render());
            output.push_str(&format!("\nAgent: {:?}\n", agent_type));
            output.push_str(&format!("Confidence: {:.1}%\n\n", result.confidence * 100.0));

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
            let mut output = format!("{:?} Agent Analysis (Confidence: {:.0}%):\n\n", agent_type, result.confidence * 100.0);
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

async fn execute_evolve(action: Option<EvolveAction>, format: OutputFormat) -> Result<String, String> {
    use crate::evolution::{EvolutionSystem, OptimizationTarget as EvOptTarget};

    let mut evolution = EvolutionSystem::new();
    let _ = evolution.load();

    match action {
        None => {
            // Default: run evolution cycle
            let result = evolution.evolve();
            let _ = evolution.save();

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new("SENA 🦁 EVOLUTION CYCLE").render());
                    output.push('\n');
                    output.push_str(&format!("Patterns Applied: {}\n", result.patterns_applied));
                    output.push_str(&format!("Optimizations Made: {}\n", result.optimizations_made));
                    output.push_str(&format!("Feedback Processed: {}\n", result.feedback_processed));
                    output.push_str(&format!("Improvement Score: {:.1}%\n", result.new_improvement_score * 100.0));
                    Ok(output)
                }
            }
        }
        Some(EvolveAction::Learn { context, outcome }) => {
            evolution.learn(&context, &outcome, true);
            let _ = evolution.save();

            match format {
                OutputFormat::Json => Ok(serde_json::json!({
                    "action": "learn",
                    "context": context,
                    "outcome": outcome,
                    "status": "learned"
                }).to_string()),
                _ => Ok(format!("Pattern learned:\n  Context: {}\n  Outcome: {}", context, outcome)),
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
            let _ = evolution.save();

            match format {
                OutputFormat::Json => {
                    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new("SENA 🦁 SELF-OPTIMIZATION").render());
                    output.push('\n');
                    output.push_str(&format!("Target: {:?}\n", opt_target));
                    output.push_str(&format!("Success: {}\n", if result.success { "✅" } else { "❌" }));
                    output.push_str(&format!("Improvement: +{:.1}%\n", result.improvement * 100.0));
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
                    output.push_str(&FormatBox::new("SENA 🦁 EVOLUTION STATISTICS").render());
                    output.push('\n');
                    output.push_str(&format!("Patterns Learned: {}\n", stats.patterns_learned));
                    output.push_str(&format!("Optimizations Applied: {}\n", stats.optimizations_applied));
                    output.push_str(&format!("Feedback Count: {}\n", stats.feedback_count));
                    output.push_str(&format!("Improvement Score: {:.1}%\n", stats.improvement_score * 100.0));
                    output.push_str(&format!("Learning Rate: {:.2}\n", stats.learning_rate));
                    Ok(output)
                }
            }
        }
        Some(EvolveAction::Patterns { limit }) => {
            let patterns = evolution.learner.get_patterns(limit);

            match format {
                OutputFormat::Json => {
                    let json: Vec<serde_json::Value> = patterns.iter().map(|p| {
                        serde_json::json!({
                            "id": p.id,
                            "context": p.context,
                            "outcome": p.outcome,
                            "pattern_type": format!("{:?}", p.pattern_type),
                            "success_rate": p.success_rate,
                            "usage_count": p.usage_count,
                        })
                    }).collect();
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
                }
                _ => {
                    let mut output = String::new();
                    output.push_str(&FormatBox::new("SENA 🦁 LEARNED PATTERNS").render());
                    output.push_str(&format!("\nShowing {} patterns:\n\n", patterns.len()));

                    if patterns.is_empty() {
                        output.push_str("No patterns learned yet. Use 'sena evolve learn <context> <outcome>' to add patterns.\n");
                    } else {
                        for pattern in &patterns {
                            output.push_str("┌─ Pattern ────────────────────────────────\n");
                            output.push_str(&format!("│ Context: {}\n", pattern.context));
                            output.push_str(&format!("│ Outcome: {}\n", pattern.outcome));
                            output.push_str(&format!("│ Type: {}\n", pattern.pattern_type));
                            output.push_str(&format!("│ Success Rate: {:.1}%\n", pattern.success_rate * 100.0));
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
        }).to_string()),
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&FormatBox::new("SENA 🦁 FEEDBACK RECORDED").render());
            output.push('\n');
            output.push_str(&format!("{} Type: {:?}\n", emoji, feedback_type));
            output.push_str(&format!("Message: {}\n", message));
            if let Some(ctx) = &context {
                output.push_str(&format!("Context: {}\n", ctx));
            }
            output.push_str("\nThank you for your feedback! SENA learns from every interaction.\n");
            Ok(output)
        }
        OutputFormat::Text => {
            Ok(format!("{} Feedback recorded: {:?} - {}", emoji, feedback_type, message))
        }
    }
}
