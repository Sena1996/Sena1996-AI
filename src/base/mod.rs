//! SENA Base Module - Core Traits and Components
//!
//! This module provides:
//! - Interface traits (IVerifier, IStorage, IExecutor, etc.)
//! - Component registry for dependency injection
//! - Base component trait for lifecycle management

pub mod interfaces;
pub mod registry;
pub mod component;

pub use interfaces::*;
pub use registry::{ComponentRegistry, CircularDependencyError};
pub use component::{BaseComponent, ComponentMetrics, ComponentStatus, ComponentState};
