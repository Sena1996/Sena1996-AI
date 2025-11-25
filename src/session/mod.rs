//! SENA Session Module
//!
//! Cross-session continuity, state persistence, and preference management

pub mod manager;

pub use manager::{SessionManager, SessionState};
