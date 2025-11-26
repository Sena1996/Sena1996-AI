use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<SenaConfig> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SenaConfig {
    #[serde(default)]
    pub user: UserConfig,
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub intelligence: IntelligenceConfig,
    #[serde(default)]
    pub evolution: EvolutionConfig,
    #[serde(default)]
    pub hub: HubConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_user_name")]
    pub name: String,
    #[serde(default = "default_emoji")]
    pub emoji: String,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default = "default_command")]
    pub command: String,
}

impl UserConfig {
    pub fn brand(&self) -> String {
        format!("{} {}", self.prefix, self.emoji)
    }

    pub fn brand_title(&self, title: &str) -> String {
        format!("{} {} {}", self.prefix, self.emoji, title)
    }

    pub fn prompt(&self) -> String {
        format!("{} {}> ", self.prefix, self.emoji)
    }

    pub fn greeting(&self) -> String {
        format!("Welcome, {}! {} is ready.", self.name, self.prefix)
    }

    pub fn farewell(&self) -> String {
        format!("Thank you for using {} v{}! {}", self.prefix, crate::VERSION, self.emoji)
    }

    pub fn command_name(&self) -> &str {
        &self.command
    }
}

fn default_user_name() -> String {
    whoami::username()
}

fn default_emoji() -> String {
    "🦁".to_string()
}

fn default_prefix() -> String {
    "SENA".to_string()
}

fn default_command() -> String {
    "sena".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub data_dir: Option<String>,
    #[serde(default = "default_true")]
    pub telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    #[serde(default = "default_thinking_depth")]
    pub default_thinking_depth: String,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default = "default_true")]
    pub auto_agent_selection: bool,
    #[serde(default = "default_primary_agent")]
    pub primary_agent: String,
}

fn default_primary_agent() -> String {
    "general".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    #[serde(default = "default_true")]
    pub pattern_learning: bool,
    #[serde(default = "default_true")]
    pub self_optimization: bool,
    #[serde(default = "default_true")]
    pub feedback_collection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubConfig {
    #[serde(default = "default_socket_path")]
    pub socket_path: String,
    #[serde(default = "default_true")]
    pub auto_start: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default = "default_true")]
    pub unicode: bool,
    #[serde(default = "default_true")]
    pub progress_bars: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_thinking_depth() -> String {
    "standard".to_string()
}

fn default_model() -> String {
    "balanced".to_string()
}

fn default_socket_path() -> String {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".sena").join("hub.sock").to_string_lossy().to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            name: default_user_name(),
            emoji: default_emoji(),
            prefix: default_prefix(),
            command: default_command(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            data_dir: None,
            telemetry: true,
        }
    }
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            default_thinking_depth: default_thinking_depth(),
            default_model: default_model(),
            auto_agent_selection: true,
            primary_agent: default_primary_agent(),
        }
    }
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            pattern_learning: true,
            self_optimization: true,
            feedback_collection: true,
        }
    }
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            socket_path: default_socket_path(),
            auto_start: true,
            timeout_seconds: default_timeout(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            color: true,
            unicode: true,
            progress_bars: true,
        }
    }
}


impl SenaConfig {
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sena")
            .join("config.toml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;

        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::WriteError(e.to_string()))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        fs::write(&path, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))
    }

    pub fn data_dir(&self) -> PathBuf {
        self.general.data_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".sena")
                    .join("data")
            })
    }

    pub fn generate_default_config() -> String {
        let config = Self::default();
        toml::to_string_pretty(&config).unwrap_or_default()
    }

    pub fn global() -> &'static SenaConfig {
        GLOBAL_CONFIG.get_or_init(|| Self::load().unwrap_or_default())
    }

    pub fn user() -> &'static UserConfig {
        &Self::global().user
    }

    pub fn brand() -> String {
        Self::user().brand()
    }

    pub fn brand_title(title: &str) -> String {
        Self::user().brand_title(title)
    }
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    ReadError(String),
    WriteError(String),
    ParseError(String),
    SerializeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ReadError(e) => write!(f, "Failed to read config: {}", e),
            ConfigError::WriteError(e) => write!(f, "Failed to write config: {}", e),
            ConfigError::ParseError(e) => write!(f, "Failed to parse config: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Failed to serialize config: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SenaConfig::default();
        assert_eq!(config.general.log_level, "info");
        assert!(config.output.color);
    }

    #[test]
    fn test_config_serialization() {
        let config = SenaConfig::default();
        let toml_str = toml::to_string(&config);
        assert!(toml_str.is_ok());
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[general]
log_level = "debug"

[output]
color = false
"#;
        let config: Result<SenaConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.general.log_level, "debug");
        assert!(!config.output.color);
    }

    #[test]
    fn test_generate_default_config() {
        let content = SenaConfig::generate_default_config();
        assert!(!content.is_empty());
        assert!(content.contains("[general]"));
    }
}
