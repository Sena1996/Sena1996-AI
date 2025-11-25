//! CLI Argument Definitions
//!
//! Uses clap for argument parsing

use clap::{Parser, Subcommand, ValueEnum};

/// SENA Controller v5.0 - Truth-Embedded Architecture (Personalized AI)
#[derive(Parser, Debug)]
#[command(name = "sena")]
#[command(author = "SENA Team")]
#[command(version = "5.0.0")]
#[command(about = "SENA Controller - Ancient Wisdom meets Modern AI", long_about = None)]
pub struct Cli {
    /// Run in verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Output format options
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
    /// Pretty formatted output with Unicode
    Pretty,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start MCP server mode (JSON-RPC over stdio)
    Mcp {
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,
    },

    /// Run as a hook handler
    Hook {
        /// Hook type to handle
        #[arg(value_enum)]
        hook_type: HookType,

        /// Input data (or read from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
    },

    /// Process a request through SENA
    Process {
        /// Request content
        content: String,

        /// Request type
        #[arg(short = 't', long, default_value = "general")]
        request_type: String,
    },

    /// Check system health
    Health {
        /// Show detailed health information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Get system metrics
    Metrics {
        /// Metric category
        #[arg(value_enum)]
        category: Option<MetricCategory>,
    },

    /// Detect format for input text
    Detect {
        /// Text to analyze
        text: String,
    },

    /// Run in daemon mode (background)
    Daemon {
        /// Action to perform
        #[arg(value_enum)]
        action: DaemonAction,
    },

    /// Session management
    Session {
        /// Session action
        #[arg(value_enum)]
        action: SessionAction,

        /// Session ID (for restore)
        #[arg(short, long)]
        id: Option<String>,
    },

    /// Validate content against SENA rules
    Validate {
        /// Content to validate
        content: String,

        /// Validation strictness
        #[arg(short, long, default_value_t = false)]
        strict: bool,
    },

    /// Generate formatted output
    Format {
        /// Format type
        #[arg(value_enum)]
        format_type: FormatOutputType,

        /// Title for the output
        #[arg(short, long)]
        title: Option<String>,

        /// Data (JSON string)
        data: String,
    },
}

/// Hook types
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum HookType {
    /// User prompt submit hook
    UserPromptSubmit,
    /// Assistant response hook
    AssistantResponse,
    /// Tool execution hook
    ToolExecution,
    /// Pre-validation hook
    PreValidation,
    /// Post-validation hook
    PostValidation,
}

/// Metric categories
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum MetricCategory {
    /// Overall health metrics
    Health,
    /// Innovation metrics
    Innovation,
    /// Test results
    Tests,
    /// Configuration status
    Config,
    /// Phase status
    Phase,
    /// All metrics
    All,
}

/// Daemon actions
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum DaemonAction {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Restart the daemon
    Restart,
    /// Check daemon status
    Status,
}

/// Session actions
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum SessionAction {
    /// Start new session
    Start,
    /// End current session
    End,
    /// Get current session info
    Info,
    /// List session history
    List,
    /// Restore a previous session
    Restore,
}

/// Format output types
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum FormatOutputType {
    /// Table format
    Table,
    /// Progress bar
    Progress,
    /// Brilliant thinking format
    BrilliantThinking,
    /// Truth verification format
    TruthVerification,
    /// Code analysis format
    CodeAnalysis,
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_creation() {
        let cli = Cli {
            verbose: false,
            format: OutputFormat::Text,
            config: None,
            command: None,
        };
        assert!(!cli.verbose);
    }

    #[test]
    fn test_output_format() {
        assert_eq!(OutputFormat::Text, OutputFormat::Text);
        assert_ne!(OutputFormat::Text, OutputFormat::Json);
    }
}
