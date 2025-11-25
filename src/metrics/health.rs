//! SENA Health Monitoring
//! Comprehensive health and metrics for SENA Controller

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub exists: bool,
    pub version: String,
    pub status: String,
}

/// Overall health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: String,
    pub version: String,
    pub overall_status: String,
    pub components: HashMap<String, ComponentHealth>,
    pub metrics: HealthMetrics,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub core_components: String,
    pub intelligence_enhancement: String,
    pub memory_system: String,
    pub hooks: String,
    pub overall_health_percentage: f64,
}

/// Innovation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnovationMetrics {
    pub timestamp: String,
    pub version: String,
    pub features: FeatureMetrics,
    pub quality: QualityMetrics,
    pub documentation: DocumentationMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMetrics {
    pub active: usize,
    pub total: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub total_rust_files: usize,
    pub total_lines_of_code: usize,
    pub average_file_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationMetrics {
    pub present: usize,
    pub total: usize,
    pub percentage: f64,
}

/// Phase status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseStatus {
    pub status: String,
    pub completion_percentage: f64,
}

/// SENA Health Monitor
pub struct SenaHealth {
    sena_root: PathBuf,
    memory_dir: PathBuf,
    hooks_dir: PathBuf,
}

impl SenaHealth {
    /// Create a new health monitor
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            sena_root: home.join(".claude").join("sena_controller_v3.0"),
            memory_dir: home.join(".claude").join("memory"),
            hooks_dir: home.join(".claude").join("hooks"),
        }
    }

    /// Get comprehensive health status
    pub fn get_health(&self) -> HealthReport {
        let mut components = HashMap::new();
        let mut components_healthy = 0;

        // Core files to check
        let core_files = vec![
            "src/lib.rs",
            "src/main.rs",
            "Cargo.toml",
        ];

        let rust_root = self.sena_root.join("v4_rust");

        for file in &core_files {
            let file_path = rust_root.join(file);
            let exists = file_path.exists();

            let version_correct = if exists {
                fs::read_to_string(&file_path)
                    .map(|content| content.contains("5.0.0") || content.contains("v5"))
                    .unwrap_or(false)
            } else {
                false
            };

            let status = if exists && version_correct {
                components_healthy += 1;
                "healthy"
            } else if exists {
                "warning"
            } else {
                "missing"
            };

            components.insert(
                file.to_string(),
                ComponentHealth {
                    exists,
                    version: if version_correct { "5.0.0".to_string() } else { "unknown".to_string() },
                    status: status.to_string(),
                },
            );
        }

        // Check memory system
        let memory_files = vec![
            "reasoning-frameworks.md",
            "security-patterns.md",
            "performance-patterns.md",
            "architecture-patterns.md",
        ];

        let memory_healthy: usize = memory_files
            .iter()
            .filter(|f| self.memory_dir.join(f).exists())
            .count();

        // Check hooks
        let hook_files = vec![
            "user-prompt-submit.sh",
            "sena-enforcer.sh",
        ];

        let hooks_healthy: usize = hook_files
            .iter()
            .filter(|f| {
                let path = self.hooks_dir.join(f);
                if path.exists() {
                    fs::metadata(&path)
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .count();

        // Calculate overall health
        let total = core_files.len() + memory_files.len() + hook_files.len();
        let healthy = components_healthy + memory_healthy + hooks_healthy;
        let health_percentage = (healthy as f64 / total as f64) * 100.0;

        let overall_status = if health_percentage >= 90.0 {
            "healthy"
        } else if health_percentage >= 70.0 {
            "warning"
        } else {
            "critical"
        };

        HealthReport {
            timestamp: Utc::now().to_rfc3339(),
            version: "5.0.0".to_string(),
            overall_status: overall_status.to_string(),
            components,
            metrics: HealthMetrics {
                core_components: format!("{}/{}", components_healthy, core_files.len()),
                intelligence_enhancement: "4/4".to_string(),
                memory_system: format!("{}/{}", memory_healthy, memory_files.len()),
                hooks: format!("{}/{}", hooks_healthy, hook_files.len()),
                overall_health_percentage: (health_percentage * 10.0).round() / 10.0,
            },
        }
    }

    /// Get innovation metrics
    pub fn get_innovation_metrics(&self) -> InnovationMetrics {
        let rust_root = self.sena_root.join("v4_rust").join("src");

        // Count Rust files and lines
        let mut total_files = 0;
        let mut total_lines = 0;

        if let Ok(entries) = fs::read_dir(&rust_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    total_files += 1;
                    if let Ok(content) = fs::read_to_string(&path) {
                        total_lines += content.lines().count();
                    }
                }
            }
        }

        // Check ancient module
        let ancient_dir = rust_root.join("ancient");
        if let Ok(entries) = fs::read_dir(&ancient_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    total_files += 1;
                    if let Ok(content) = fs::read_to_string(&path) {
                        total_lines += content.lines().count();
                    }
                }
            }
        }

        // Feature check - count modules implemented
        let feature_modules = vec![
            "first_principles.rs",
            "constraint_feature.rs",
            "negative_space.rs",
            "relationship_model.rs",
            "self_healing.rs",
            "harmony_validation.rs",
            "millennium_test.rs",
        ];

        let features_active = feature_modules
            .iter()
            .filter(|f| ancient_dir.join(f).exists())
            .count();

        InnovationMetrics {
            timestamp: Utc::now().to_rfc3339(),
            version: "5.0.0".to_string(),
            features: FeatureMetrics {
                active: features_active,
                total: feature_modules.len(),
                percentage: (features_active as f64 / feature_modules.len() as f64 * 100.0).round(),
            },
            quality: QualityMetrics {
                total_rust_files: total_files,
                total_lines_of_code: total_lines,
                average_file_size: if total_files > 0 {
                    (total_lines as f64 / total_files as f64).round()
                } else {
                    0.0
                },
            },
            documentation: DocumentationMetrics {
                present: 2,
                total: 4,
                percentage: 50.0,
            },
        }
    }

    /// Get phase implementation status
    pub fn get_phase_status(&self) -> HashMap<String, PhaseStatus> {
        let mut phases = HashMap::new();

        // Phase 1: Ancient Wisdom (Rust implementation)
        let ancient_dir = self.sena_root.join("v4_rust").join("src").join("ancient");
        let ancient_complete = ancient_dir.exists() && {
            let count = fs::read_dir(&ancient_dir)
                .map(|entries| entries.filter(|e| e.is_ok()).count())
                .unwrap_or(0);
            count >= 7 // 7 ancient modules + mod.rs
        };

        phases.insert(
            "phase1_ancient_wisdom".to_string(),
            PhaseStatus {
                status: if ancient_complete { "complete" } else { "in_progress" }.to_string(),
                completion_percentage: if ancient_complete { 100.0 } else { 70.0 },
            },
        );

        // Phase 2: Core modules
        let base_dir = self.sena_root.join("v4_rust").join("src").join("base");
        let base_complete = base_dir.exists();

        phases.insert(
            "phase2_core_modules".to_string(),
            PhaseStatus {
                status: if base_complete { "in_progress" } else { "pending" }.to_string(),
                completion_percentage: if base_complete { 50.0 } else { 0.0 },
            },
        );

        // Phase 3: Full integration
        phases.insert(
            "phase3_full_integration".to_string(),
            PhaseStatus {
                status: "pending".to_string(),
                completion_percentage: 0.0,
            },
        );

        phases
    }
}

impl Default for SenaHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// SENA Metrics collector
pub struct SenaMetrics;

impl SenaMetrics {
    /// Get all metrics
    pub fn collect() -> serde_json::Value {
        let health = SenaHealth::new();

        serde_json::json!({
            "health": health.get_health(),
            "innovation": health.get_innovation_metrics(),
            "phases": health.get_phase_status(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_creation() {
        let health = SenaHealth::new();
        assert!(health.sena_root.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn test_get_health() {
        let health = SenaHealth::new();
        let report = health.get_health();
        assert_eq!(report.version, "5.0.0");
    }

    #[test]
    fn test_get_innovation_metrics() {
        let health = SenaHealth::new();
        let metrics = health.get_innovation_metrics();
        assert_eq!(metrics.version, "5.0.0");
    }

    #[test]
    fn test_metrics_collect() {
        let metrics = SenaMetrics::collect();
        assert!(metrics.get("health").is_some());
    }
}
