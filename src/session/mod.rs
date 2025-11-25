//! SENA Session Module
//!
//! ⚠️ DEPRECATED: This module has been merged into `hub::session`
//!
//! Use `hub::SessionRegistry` and `hub::Session` instead:
//! - Command history: `Session::record_command()`
//! - Preferences: `Session::set_preference()` / `SessionRegistry::set_global_preference()`
//! - Session stats: `Session::stats()`
//!
//! This module remains for backwards compatibility but will be removed in v8.0.0

#[deprecated(since = "7.0.0", note = "Use hub::SessionRegistry and hub::Session instead")]
pub mod manager;

#[deprecated(since = "7.0.0", note = "Use hub::SessionRegistry and hub::Session instead")]
pub use manager::{SessionManager, SessionState};
