//! Component Registry
//! Dependency Injection container that manages component lifecycle

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::Instant;

/// Error when circular dependency is detected
#[derive(Debug, Clone)]
pub struct CircularDependencyError {
    pub component: String,
    pub chain: Vec<String>,
}

impl std::fmt::Display for CircularDependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Circular dependency detected: {} -> ... -> {}",
            self.component, self.component
        )
    }
}

impl std::error::Error for CircularDependencyError {}

/// Factory function type
type Factory = Box<dyn Fn() -> Box<dyn Any + Send + Sync> + Send + Sync>;

/// Factory registration info
struct FactoryInfo {
    factory: Factory,
    singleton: bool,
}

/// Registry metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryMetrics {
    pub total_registered: usize,
    pub active_instances: usize,
    pub initialization_times: HashMap<String, f64>,
    pub average_init_time: f64,
}

/// Dependency Injection Container
/// Manages component lifecycle and prevents circular dependencies
pub struct ComponentRegistry {
    factories: RwLock<HashMap<String, FactoryInfo>>,
    instances: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,
    initializing: RwLock<HashSet<String>>,
    metrics: RwLock<RegistryInternalMetrics>,
}

struct RegistryInternalMetrics {
    total_components: usize,
    active_instances: usize,
    initialization_time: HashMap<String, f64>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
            initializing: RwLock::new(HashSet::new()),
            metrics: RwLock::new(RegistryInternalMetrics {
                total_components: 0,
                active_instances: 0,
                initialization_time: HashMap::new(),
            }),
        }
    }

    /// Register a component factory
    ///
    /// # Arguments
    /// * `name` - Component name
    /// * `factory` - Factory function that creates the component
    /// * `singleton` - If true, only one instance is created (default: true)
    pub fn register<F, T>(&self, name: &str, factory: F, singleton: bool)
    where
        F: Fn() -> T + Send + Sync + 'static,
        T: Any + Send + Sync + 'static,
    {
        let boxed_factory: Factory = Box::new(move || Box::new(factory()));

        let mut factories = self.factories.write().unwrap();
        factories.insert(
            name.to_string(),
            FactoryInfo {
                factory: boxed_factory,
                singleton,
            },
        );

        let mut metrics = self.metrics.write().unwrap();
        metrics.total_components += 1;
    }

    /// Get or create component instance
    ///
    /// # Arguments
    /// * `name` - Component name
    ///
    /// # Returns
    /// Arc to the component instance, or error
    pub fn get<T: Any + Send + Sync + Clone + 'static>(
        &self,
        name: &str,
    ) -> Result<T, ComponentRegistryError> {
        // Check if component exists
        let factories = self.factories.read().unwrap();
        if !factories.contains_key(name) {
            return Err(ComponentRegistryError::NotFound(name.to_string()));
        }
        drop(factories);

        // Check for cached singleton instance
        {
            let instances = self.instances.read().unwrap();
            if let Some(instance) = instances.get(name) {
                if let Some(typed) = instance.downcast_ref::<T>() {
                    return Ok(typed.clone());
                }
            }
        }

        // Check for circular dependency
        {
            let initializing = self.initializing.read().unwrap();
            if initializing.contains(name) {
                return Err(ComponentRegistryError::CircularDependency(
                    CircularDependencyError {
                        component: name.to_string(),
                        chain: initializing.iter().cloned().collect(),
                    },
                ));
            }
        }

        // Mark as initializing
        {
            let mut initializing = self.initializing.write().unwrap();
            initializing.insert(name.to_string());
        }

        // Create instance
        let start = Instant::now();
        let result = {
            let factories = self.factories.read().unwrap();
            let factory_info = factories.get(name).unwrap();
            let instance = (factory_info.factory)();
            (instance, factory_info.singleton)
        };

        let duration = start.elapsed();

        // Remove from initializing
        {
            let mut initializing = self.initializing.write().unwrap();
            initializing.remove(name);
        }

        // Cache if singleton
        if result.1 {
            let mut instances = self.instances.write().unwrap();
            instances.insert(name.to_string(), result.0);

            let mut metrics = self.metrics.write().unwrap();
            metrics.active_instances += 1;
            metrics
                .initialization_time
                .insert(name.to_string(), duration.as_secs_f64());

            // Get the cached instance
            let instances = self.instances.read().unwrap();
            if let Some(instance) = instances.get(name) {
                if let Some(typed) = instance.downcast_ref::<T>() {
                    return Ok(typed.clone());
                }
            }
        }

        Err(ComponentRegistryError::TypeMismatch(name.to_string()))
    }

    /// Check if component is registered
    pub fn has(&self, name: &str) -> bool {
        let factories = self.factories.read().unwrap();
        factories.contains_key(name)
    }

    /// Check if component instance exists
    pub fn is_initialized(&self, name: &str) -> bool {
        let instances = self.instances.read().unwrap();
        instances.contains_key(name)
    }

    /// Unregister a component
    pub fn unregister(&self, name: &str) {
        let mut factories = self.factories.write().unwrap();
        if factories.remove(name).is_some() {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_components = metrics.total_components.saturating_sub(1);
        }

        let mut instances = self.instances.write().unwrap();
        if instances.remove(name).is_some() {
            let mut metrics = self.metrics.write().unwrap();
            metrics.active_instances = metrics.active_instances.saturating_sub(1);
        }
    }

    /// Clear all components and cleanup
    pub fn clear(&self) {
        let mut factories = self.factories.write().unwrap();
        factories.clear();

        let mut instances = self.instances.write().unwrap();
        instances.clear();

        let mut initializing = self.initializing.write().unwrap();
        initializing.clear();

        let mut metrics = self.metrics.write().unwrap();
        *metrics = RegistryInternalMetrics {
            total_components: 0,
            active_instances: 0,
            initialization_time: HashMap::new(),
        };
    }

    /// Get registry metrics
    pub fn get_metrics(&self) -> RegistryMetrics {
        let metrics = self.metrics.read().unwrap();

        let avg_time = if metrics.initialization_time.is_empty() {
            0.0
        } else {
            metrics.initialization_time.values().sum::<f64>()
                / metrics.initialization_time.len() as f64
        };

        RegistryMetrics {
            total_registered: metrics.total_components,
            active_instances: metrics.active_instances,
            initialization_times: metrics.initialization_time.clone(),
            average_init_time: avg_time,
        }
    }

    /// Get list of registered components
    pub fn get_registered_components(&self) -> Vec<String> {
        let factories = self.factories.read().unwrap();
        factories.keys().cloned().collect()
    }

    /// Get list of initialized components
    pub fn get_initialized_components(&self) -> Vec<String> {
        let instances = self.instances.read().unwrap();
        instances.keys().cloned().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Component registry errors
#[derive(Debug)]
pub enum ComponentRegistryError {
    NotFound(String),
    CircularDependency(CircularDependencyError),
    TypeMismatch(String),
}

impl std::fmt::Display for ComponentRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Component '{}' not registered", name),
            Self::CircularDependency(err) => write!(f, "{}", err),
            Self::TypeMismatch(name) => write!(f, "Type mismatch for component '{}'", name),
        }
    }
}

impl std::error::Error for ComponentRegistryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_registry_creation() {
        let registry = ComponentRegistry::new();
        assert!(!registry.has("test"));
    }

    #[test]
    fn test_register_and_get() {
        let registry = ComponentRegistry::new();
        registry.register("counter", || Arc::new(RwLock::new(0_i32)), true);
        assert!(registry.has("counter"));
    }

    #[test]
    fn test_metrics() {
        let registry = ComponentRegistry::new();
        registry.register("test", || "value".to_string(), true);

        let metrics = registry.get_metrics();
        assert_eq!(metrics.total_registered, 1);
    }

    #[test]
    fn test_unregister() {
        let registry = ComponentRegistry::new();
        registry.register("test", || "value".to_string(), true);
        assert!(registry.has("test"));

        registry.unregister("test");
        assert!(!registry.has("test"));
    }

    #[test]
    fn test_clear() {
        let registry = ComponentRegistry::new();
        registry.register("test1", || "value1".to_string(), true);
        registry.register("test2", || "value2".to_string(), true);

        registry.clear();

        assert!(!registry.has("test1"));
        assert!(!registry.has("test2"));
    }
}
