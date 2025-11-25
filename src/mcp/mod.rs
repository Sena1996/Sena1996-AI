//! SENA MCP Server Module
//!
//! Model Context Protocol server implementation using JSON-RPC over stdio

pub mod server;
pub mod protocol;
pub mod handlers;

pub use server::run_server;
pub use protocol::*;
