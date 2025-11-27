//! SENA Base Module - Core Traits and Components
//!
//! This module provides:
//! - Interface traits (IVerifier, IStorage, IExecutor, etc.)
//! - Component registry for dependency injection
//! - Base component trait for lifecycle management

pub mod component;
pub mod interfaces;
pub mod registry;

pub use component::{BaseComponent, ComponentMetrics, ComponentState, ComponentStatus};
pub use interfaces::*;
pub use registry::{CircularDependencyError, ComponentRegistry};
