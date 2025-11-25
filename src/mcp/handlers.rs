//! MCP Request Handlers
//!
//! Handles MCP protocol requests

use super::protocol::*;
use crate::metrics::SenaHealth;
use crate::integration::AutoIntegration;
use crate::ancient::HarmonyValidationEngine;
use std::collections::HashMap;

/// Handle MCP requests
pub fn handle_request(request: &JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(request),
        "initialized" => handle_initialized(request),
        "tools/list" => handle_tools_list(request),
        "tools/call" => handle_tools_call(request),
        "resources/list" => handle_resources_list(request),
        "resources/read" => handle_resources_read(request),
        "ping" => handle_ping(request),
        _ => JsonRpcResponse::error(
            request.id.clone(),
            error_codes::METHOD_NOT_FOUND,
            &format!("Method not found: {}", request.method),
        ),
    }
}

fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: false }),
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
            prompts: None,
        },
        server_info: ServerInfo {
            name: "sena-controller".to_string(),
            version: crate::VERSION.to_string(),
        },
    };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::to_value(result).unwrap_or_default(),
    )
}

fn handle_initialized(_request: &JsonRpcRequest) -> JsonRpcResponse {
    // Notification, no response needed
    JsonRpcResponse::success(None, serde_json::Value::Null)
}

fn handle_ping(request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(request.id.clone(), serde_json::json!({}))
}

fn handle_tools_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let tools = vec![
        Tool {
            name: "sena_health".to_string(),
            description: "Get SENA system health status".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "detailed": {
                        "type": "boolean",
                        "description": "Show detailed health information"
                    }
                }
            }),
        },
        Tool {
            name: "sena_metrics".to_string(),
            description: "Get SENA metrics and statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "enum": ["health", "innovation", "tests", "config", "phase", "all"],
                        "description": "Metric category to retrieve"
                    }
                }
            }),
        },
        Tool {
            name: "sena_detect_format".to_string(),
            description: "Detect required SENA format for text".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to analyze for format detection"
                    }
                },
                "required": ["text"]
            }),
        },
        Tool {
            name: "sena_validate".to_string(),
            description: "Validate content against SENA rules".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content to validate"
                    },
                    "strict": {
                        "type": "boolean",
                        "description": "Use strict validation mode"
                    }
                },
                "required": ["content"]
            }),
        },
        Tool {
            name: "sena_process".to_string(),
            description: "Process a request through SENA ancient wisdom layers".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Content to process"
                    },
                    "request_type": {
                        "type": "string",
                        "description": "Type of request"
                    }
                },
                "required": ["content"]
            }),
        },
        Tool {
            name: "sena_format_table".to_string(),
            description: "Generate a formatted Unicode table".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "headers": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Table headers"
                    },
                    "rows": {
                        "type": "array",
                        "items": {
                            "type": "array",
                            "items": {"type": "string"}
                        },
                        "description": "Table rows"
                    },
                    "title": {
                        "type": "string",
                        "description": "Optional table title"
                    }
                },
                "required": ["headers", "rows"]
            }),
        },
        Tool {
            name: "sena_progress".to_string(),
            description: "Generate a progress bar display".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "tasks": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "percent": {"type": "number"}
                            }
                        },
                        "description": "List of tasks with progress"
                    }
                },
                "required": ["tasks"]
            }),
        },
    ];

    let result = ToolsListResult { tools };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::to_value(result).unwrap_or_default(),
    )
}

fn handle_tools_call(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: ToolCallParams = match &request.params {
        Some(p) => match serde_json::from_value(p.clone()) {
            Ok(params) => params,
            Err(e) => {
                return JsonRpcResponse::error(
                    request.id.clone(),
                    error_codes::INVALID_PARAMS,
                    &format!("Invalid params: {}", e),
                );
            }
        },
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                error_codes::INVALID_PARAMS,
                "Missing params",
            );
        }
    };

    let args = params.arguments.unwrap_or_default();

    let result = match params.name.as_str() {
        "sena_health" => call_health(&args),
        "sena_metrics" => call_metrics(&args),
        "sena_detect_format" => call_detect_format(&args),
        "sena_validate" => call_validate(&args),
        "sena_process" => call_process(&args),
        "sena_format_table" => call_format_table(&args),
        "sena_progress" => call_progress(&args),
        _ => ToolCallResult {
            content: vec![ToolContent::text(&format!("Unknown tool: {}", params.name))],
            is_error: true,
        },
    };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::to_value(result).unwrap_or_default(),
    )
}

fn call_health(args: &HashMap<String, serde_json::Value>) -> ToolCallResult {
    let detailed = args.get("detailed").and_then(|v| v.as_bool()).unwrap_or(false);
    let health = SenaHealth::new();
    let report = health.get_health();

    let text = if detailed {
        serde_json::to_string_pretty(&report).unwrap_or_default()
    } else {
        format!(
            "SENA v{} - Status: {} ({}%)",
            report.version,
            report.overall_status,
            report.metrics.overall_health_percentage
        )
    };

    ToolCallResult {
        content: vec![ToolContent::text(&text)],
        is_error: false,
    }
}

fn call_metrics(args: &HashMap<String, serde_json::Value>) -> ToolCallResult {
    let category = args.get("category").and_then(|v| v.as_str()).unwrap_or("all");
    let health = SenaHealth::new();

    let metrics = match category {
        "health" => serde_json::to_value(health.get_health()).ok(),
        "innovation" => serde_json::to_value(health.get_innovation_metrics()).ok(),
        "phase" => serde_json::to_value(health.get_phase_status()).ok(),
        _ => Some(crate::metrics::SenaMetrics::collect()),
    };

    let text = serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| "{}".to_string());

    ToolCallResult {
        content: vec![ToolContent::text(&text)],
        is_error: false,
    }
}

fn call_detect_format(args: &HashMap<String, serde_json::Value>) -> ToolCallResult {
    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let ai = AutoIntegration::new();
    let detected = ai.detect_format(text);

    let result = match detected {
        Some(fmt) => format!("Detected format: {}", fmt.name()),
        None => "No special format detected".to_string(),
    };

    ToolCallResult {
        content: vec![ToolContent::text(&result)],
        is_error: false,
    }
}

fn call_validate(args: &HashMap<String, serde_json::Value>) -> ToolCallResult {
    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let mut engine = HarmonyValidationEngine::new();
    let result = engine.validate(content);

    let violations = result.rule_violations.len();
    let checks_count = result.checks.len();

    let text = format!(
        "Valid: {} | Confidence: {:.1}% | Checks: {} | Violations: {}",
        result.is_valid(),
        result.overall_confidence * 100.0,
        checks_count,
        violations
    );

    ToolCallResult {
        content: vec![ToolContent::text(&text)],
        is_error: false,
    }
}

fn call_process(args: &HashMap<String, serde_json::Value>) -> ToolCallResult {
    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let request_type = args.get("request_type").and_then(|v| v.as_str()).unwrap_or("general");

    // Use first principles analysis
    use crate::ancient::FirstPrinciplesEngine;
    let mut engine = FirstPrinciplesEngine::new();

    let observation = engine.observe(
        content.to_string(),
        HashMap::new(),
        "mcp_request".to_string(),
    );

    let text = format!(
        "Processed: {} | Questions: {} | Anomaly: {}",
        request_type,
        observation.questions_raised.len(),
        observation.anomaly
    );

    ToolCallResult {
        content: vec![ToolContent::text(&text)],
        is_error: false,
    }
}

fn call_format_table(args: &HashMap<String, serde_json::Value>) -> ToolCallResult {
    let headers: Vec<String> = args.get("headers")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let rows: Vec<Vec<String>> = args.get("rows")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let title = args.get("title").and_then(|v| v.as_str());

    use crate::output::TableBuilder;
    let mut builder = TableBuilder::new();

    if let Some(t) = title {
        builder = builder.title(t);
    }

    builder = builder.row(headers);
    for row in rows {
        builder = builder.row(row);
    }

    ToolCallResult {
        content: vec![ToolContent::text(&builder.build())],
        is_error: false,
    }
}

fn call_progress(args: &HashMap<String, serde_json::Value>) -> ToolCallResult {
    let tasks: Vec<serde_json::Value> = args.get("tasks")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    use crate::output::ProgressBar;
    let mut output = String::new();

    for task in tasks {
        let name = task.get("name").and_then(|v| v.as_str()).unwrap_or("Task");
        let percent = task.get("percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
        output.push_str(&ProgressBar::new(name, percent as f32).render());
        output.push('\n');
    }

    ToolCallResult {
        content: vec![ToolContent::text(&output)],
        is_error: false,
    }
}

fn handle_resources_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let resources = vec![
        Resource {
            uri: "sena://health".to_string(),
            name: "SENA Health Status".to_string(),
            description: Some("Current health status of SENA system".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "sena://metrics".to_string(),
            name: "SENA Metrics".to_string(),
            description: Some("System metrics and statistics".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "sena://config".to_string(),
            name: "SENA Configuration".to_string(),
            description: Some("Current system configuration".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ];

    let result = ResourcesListResult { resources };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::to_value(result).unwrap_or_default(),
    )
}

fn handle_resources_read(request: &JsonRpcRequest) -> JsonRpcResponse {
    let uri = request.params
        .as_ref()
        .and_then(|p| p.get("uri"))
        .and_then(|u| u.as_str())
        .unwrap_or("");

    let content = match uri {
        "sena://health" => {
            let health = SenaHealth::new();
            serde_json::to_string_pretty(&health.get_health()).unwrap_or_default()
        }
        "sena://metrics" => {
            serde_json::to_string_pretty(&crate::metrics::SenaMetrics::collect()).unwrap_or_default()
        }
        "sena://config" => {
            serde_json::json!({
                "version": crate::VERSION,
                "codename": crate::CODENAME,
            }).to_string()
        }
        _ => {
            return JsonRpcResponse::error(
                request.id.clone(),
                error_codes::INVALID_PARAMS,
                &format!("Unknown resource: {}", uri),
            );
        }
    };

    let result = serde_json::json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": content,
        }]
    });

    JsonRpcResponse::success(request.id.clone(), result)
}
