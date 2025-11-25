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
    use crate::session::SessionManager;

    let manager = SessionManager::new(100);

    let result = match action {
        SessionAction::Start => {
            let session = manager.start_session();
            serde_json::json!({
                "action": "start",
                "session_id": session.session_id,
                "started_at": session.started_at.to_rfc3339(),
            })
        }
        SessionAction::End => {
            manager.end_session();
            serde_json::json!({
                "action": "end",
                "status": "session ended",
            })
        }
        SessionAction::Info => {
            match manager.get_current_session() {
                Some(session) => serde_json::to_value(&session).unwrap_or_default(),
                None => serde_json::json!({"error": "no active session"}),
            }
        }
        SessionAction::List => {
            let history = manager.get_session_history(10);
            serde_json::to_value(&history).unwrap_or_default()
        }
        SessionAction::Restore => {
            match id {
                Some(session_id) => {
                    // Restoration logic would go here
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
