//! MCP Server Implementation
//!
//! JSON-RPC server over stdio for Claude Code integration

use std::io::{self, BufRead, Write};
use super::protocol::*;
use super::handlers::handle_request;
use crate::config::SenaConfig;

pub async fn run_server() -> Result<String, String> {
    use std::io::BufReader;

    let brand = SenaConfig::brand();
    eprintln!("{} MCP Server v{} starting...", brand, crate::VERSION);

    let stdin = io::stdin();
    let stdout = io::stdout();

    let stdin_handle = stdin.lock();
    let mut stdout_handle = stdout.lock();
    let mut reader = BufReader::new(stdin_handle);

    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line) {
            Ok(0) => {
                eprintln!("EOF received, shutting down");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        eprintln!("Received: {}", &trimmed[..trimmed.len().min(100)]);

        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                let error_response = JsonRpcResponse::error(
                    None,
                    error_codes::PARSE_ERROR,
                    &format!("Parse error: {}", e),
                );
                let response_str = serde_json::to_string(&error_response).unwrap_or_default();
                let _ = writeln!(stdout_handle, "{}", response_str);
                let _ = stdout_handle.flush();
                continue;
            }
        };

        let response = handle_request(&request);

        if request.id.is_none() {
            eprintln!("Notification received: {}", request.method);
            continue;
        }

        let response_str = serde_json::to_string(&response).unwrap_or_default();
        eprintln!("Sending: {}", &response_str[..response_str.len().min(100)]);

        if let Err(e) = writeln!(stdout_handle, "{}", response_str) {
            eprintln!("Error writing response: {}", e);
            break;
        }
        if let Err(e) = stdout_handle.flush() {
            eprintln!("Error flushing stdout: {}", e);
            break;
        }
    }

    eprintln!("MCP Server loop ended");
    Ok("MCP Server stopped".to_string())
}

/// Run MCP server with async support
pub async fn run_server_async() -> Result<String, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let mut reader = BufReader::new(stdin);
    let mut stdout = stdout;

    let brand = SenaConfig::brand();
    eprintln!("{} MCP Server v{} starting (async)...", brand, crate::VERSION);

    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                continue;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                let error_response = JsonRpcResponse::error(
                    None,
                    error_codes::PARSE_ERROR,
                    &format!("Parse error: {}", e),
                );
                let response_str = serde_json::to_string(&error_response).unwrap_or_default();
                let _ = stdout.write_all(format!("{}\n", response_str).as_bytes()).await;
                let _ = stdout.flush().await;
                continue;
            }
        };

        // Handle request
        let response = handle_request(&request);

        // Skip response for notifications
        if request.id.is_none() && response.result == Some(serde_json::Value::Null) {
            continue;
        }

        // Send response
        let response_str = serde_json::to_string(&response).unwrap_or_default();
        if let Err(e) = stdout.write_all(format!("{}\n", response_str).as_bytes()).await {
            eprintln!("Error writing response: {}", e);
        }
        if let Err(e) = stdout.flush().await {
            eprintln!("Error flushing stdout: {}", e);
        }
    }

    Ok("MCP Server stopped".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_initialize() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test",
                    "version": "1.0"
                }
            })),
        };

        let response = handle_request(&request);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_handle_tools_list() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_request(&request);
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        let tools = result.get("tools").and_then(|t| t.as_array());
        assert!(tools.is_some());
        assert!(!tools.unwrap().is_empty());
    }

    #[test]
    fn test_handle_unknown_method() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(3)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = handle_request(&request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, error_codes::METHOD_NOT_FOUND);
    }
}
