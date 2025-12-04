use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SandboxLevel {
    None,
    Basic,
    #[default]
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HallucinationMode {
    BlockOnly,
    WarnOnly,
    LogOnly,
    #[default]
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianConfig {
    pub enabled: bool,
    pub sandbox_level: SandboxLevel,
    pub hallucination_mode: HallucinationMode,
    pub hallucination_threshold: f64,
    pub block_threshold: f64,
    pub warn_threshold: f64,
    pub log_threshold: f64,
    pub intercept_all: bool,
    pub audit_enabled: bool,
    pub max_command_length: usize,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sandbox_level: SandboxLevel::Full,
            hallucination_mode: HallucinationMode::All,
            hallucination_threshold: 0.7,
            block_threshold: 0.85,
            warn_threshold: 0.70,
            log_threshold: 0.50,
            intercept_all: true,
            audit_enabled: true,
            max_command_length: 4096,
        }
    }
}

impl GuardianConfig {
    pub fn minimal() -> Self {
        Self {
            enabled: true,
            sandbox_level: SandboxLevel::None,
            hallucination_mode: HallucinationMode::LogOnly,
            hallucination_threshold: 0.9,
            block_threshold: 0.95,
            warn_threshold: 0.85,
            log_threshold: 0.70,
            intercept_all: false,
            audit_enabled: false,
            max_command_length: 8192,
        }
    }

    pub fn strict() -> Self {
        Self {
            enabled: true,
            sandbox_level: SandboxLevel::Full,
            hallucination_mode: HallucinationMode::All,
            hallucination_threshold: 0.5,
            block_threshold: 0.70,
            warn_threshold: 0.50,
            log_threshold: 0.30,
            intercept_all: true,
            audit_enabled: true,
            max_command_length: 2048,
        }
    }
}
