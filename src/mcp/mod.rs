//! SENA MCP Server Module
//!
//! Model Context Protocol server implementation using JSON-RPC over stdio

pub mod handlers;
pub mod protocol;
pub mod server;

pub use protocol::*;
pub use server::run_server;
