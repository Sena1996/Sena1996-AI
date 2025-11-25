//! SENA CLI Module
//!
//! Command-line interface for SENA Controller

pub mod args;
pub mod commands;

pub use args::{Cli, Commands, HookType};
pub use commands::execute_command;
