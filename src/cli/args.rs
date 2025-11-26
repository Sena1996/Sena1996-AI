use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "sena")]
#[command(author = "SENA Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "SENA Controller - Unified Intelligence")]
pub struct Cli {
    #[arg(short, long, default_value_t = false, help = "Run in verbose mode")]
    pub verbose: bool,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text, help = "Output format")]
    pub format: OutputFormat,

    #[arg(short, long, help = "Configuration file path")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Pretty,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Start MCP server mode")]
    Mcp {
        #[arg(short, long, help = "Enable debug logging")]
        debug: bool,
    },

    #[command(about = "Run as hook handler")]
    Hook {
        #[arg(value_enum, help = "Hook type")]
        hook_type: HookType,

        #[arg(short, long, help = "Input data")]
        input: Option<String>,
    },

    #[command(about = "Process request through SENA")]
    Process {
        #[arg(help = "Request content")]
        content: String,

        #[arg(short = 't', long, default_value = "general", help = "Request type")]
        request_type: String,
    },

    #[command(about = "Check system health")]
    Health {
        #[arg(short, long, help = "Show detailed info")]
        detailed: bool,
    },

    #[command(about = "Get system metrics")]
    Metrics {
        #[arg(value_enum, help = "Metric category")]
        category: Option<MetricCategory>,
    },

    #[command(about = "Detect format for input")]
    Detect {
        #[arg(help = "Text to analyze")]
        text: String,
    },

    #[command(about = "Run in daemon mode")]
    Daemon {
        #[arg(value_enum, help = "Action")]
        action: DaemonAction,
    },

    #[command(about = "Session management")]
    Session {
        #[arg(value_enum, help = "Session action")]
        action: SessionAction,

        #[arg(short, long, help = "Session ID")]
        id: Option<String>,

        #[arg(short, long, help = "Session name")]
        name: Option<String>,
    },

    #[command(about = "Validate content")]
    Validate {
        #[arg(help = "Content to validate")]
        content: String,

        #[arg(short, long, default_value_t = false, help = "Strict mode")]
        strict: bool,
    },

    #[command(about = "Generate formatted output")]
    Format {
        #[arg(value_enum, help = "Format type")]
        format_type: FormatOutputType,

        #[arg(short, long, help = "Title")]
        title: Option<String>,

        #[arg(help = "Data (JSON)")]
        data: String,
    },

    #[command(about = "Collaboration Hub")]
    Hub {
        #[command(subcommand)]
        action: HubAction,
    },

    #[command(about = "Join collaboration hub")]
    Join {
        #[arg(short, long, help = "Role")]
        role: String,

        #[arg(short, long, help = "Display name")]
        name: Option<String>,
    },

    #[command(about = "List online sessions")]
    Who,

    #[command(about = "Send message")]
    Tell {
        #[arg(help = "Target session")]
        target: String,

        #[arg(help = "Message")]
        message: String,
    },

    #[command(about = "Check inbox")]
    Inbox,

    #[command(about = "Task management")]
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    #[command(about = "Watch live dashboard")]
    Watch,

    #[command(about = "Show sync status")]
    Sync,

    #[command(about = "Knowledge system commands")]
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeAction,
    },

    #[command(about = "Extended thinking analysis")]
    Think {
        #[arg(help = "Query to analyze")]
        query: String,

        #[arg(short, long, value_enum, default_value_t = ThinkingDepthArg::Standard, help = "Thinking depth")]
        depth: ThinkingDepthArg,
    },

    #[command(about = "Specialized agent analysis")]
    Agent {
        #[arg(value_enum, help = "Agent type")]
        agent_type: AgentTypeArg,

        #[arg(help = "Content to analyze")]
        content: String,
    },

    #[command(about = "Evolution system commands")]
    Evolve {
        #[command(subcommand)]
        action: Option<EvolveAction>,
    },

    #[command(about = "Submit feedback")]
    Feedback {
        #[arg(value_enum, help = "Feedback type")]
        feedback_type: FeedbackTypeArg,

        #[arg(help = "Feedback message")]
        message: String,

        #[arg(short, long, help = "Context")]
        context: Option<String>,
    },

    #[command(about = "Backend development agent")]
    Backend {
        #[arg(value_enum, help = "Analysis type")]
        analysis: BackendAnalysisType,

        #[arg(help = "Code or file content to analyze")]
        input: String,
    },

    #[command(about = "IoT development agent")]
    Iot {
        #[arg(value_enum, help = "Analysis type")]
        analysis: IoTAnalysisType,

        #[arg(help = "Code or file content to analyze")]
        input: String,
    },

    #[command(about = "iOS development agent")]
    Ios {
        #[arg(value_enum, help = "Analysis type")]
        analysis: IOSAnalysisType,

        #[arg(help = "Code or file content to analyze")]
        input: String,
    },

    #[command(about = "Android development agent")]
    Android {
        #[arg(value_enum, help = "Analysis type")]
        analysis: AndroidAnalysisType,

        #[arg(help = "Code or file content to analyze")]
        input: String,
    },

    #[command(about = "Web development agent")]
    Web {
        #[arg(value_enum, help = "Analysis type")]
        analysis: WebAnalysisType,

        #[arg(help = "Code or file content to analyze")]
        input: String,
    },

    #[command(about = "Interactive setup wizard")]
    Setup {
        #[arg(value_enum, help = "Installation type")]
        install_type: Option<InstallationType>,

        #[arg(short, long, help = "Project name")]
        name: Option<String>,

        #[arg(short, long, help = "Skip confirmation prompts")]
        yes: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum InstallationType {
    Mcp,
    Hook,
    Full,
    Backend,
    Iot,
    Ios,
    Android,
    Web,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum HookType {
    UserPromptSubmit,
    AssistantResponse,
    ToolExecution,
    PreValidation,
    PostValidation,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum MetricCategory {
    Health,
    Innovation,
    Tests,
    Config,
    Phase,
    All,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum DaemonAction {
    Start,
    Stop,
    Restart,
    Status,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum SessionAction {
    Start,
    End,
    Info,
    List,
    Restore,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum FormatOutputType {
    Table,
    Progress,
    BrilliantThinking,
    TruthVerification,
    CodeAnalysis,
}

#[derive(Subcommand, Debug, Clone)]
pub enum HubAction {
    #[command(about = "Start hub daemon")]
    Start,
    #[command(about = "Stop hub daemon")]
    Stop,
    #[command(about = "Check hub status")]
    Status,
    #[command(about = "Show conflicts")]
    Conflicts,
    #[command(about = "Clear hub data")]
    Clear,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TaskAction {
    #[command(about = "Create task")]
    New {
        #[arg(help = "Task title")]
        title: String,

        #[arg(short, long, help = "Assignee")]
        to: String,

        #[arg(short, long, default_value = "medium", help = "Priority")]
        priority: String,
    },

    #[command(about = "List tasks")]
    List {
        #[arg(short, long, help = "Filter by status")]
        status: Option<String>,
    },

    #[command(about = "My tasks")]
    Mine,

    #[command(about = "Mark done")]
    Done {
        #[arg(help = "Task ID")]
        id: u64,
    },

    #[command(about = "Update status")]
    Update {
        #[arg(help = "Task ID")]
        id: u64,

        #[arg(help = "New status")]
        status: String,
    },

    #[command(about = "Reassign task")]
    Assign {
        #[arg(help = "Task ID")]
        id: u64,

        #[arg(help = "New assignee")]
        to: String,
    },

    #[command(about = "Delete task")]
    Delete {
        #[arg(help = "Task ID")]
        id: u64,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum KnowledgeAction {
    #[command(about = "Search knowledge base")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(short, long, default_value_t = 10, help = "Max results")]
        limit: usize,
    },

    #[command(about = "List patterns by category")]
    List {
        #[arg(value_enum, help = "Category")]
        category: KnowledgeCategory,
    },

    #[command(about = "Show knowledge statistics")]
    Stats,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum KnowledgeCategory {
    Reasoning,
    Security,
    Performance,
    Architecture,
    All,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Default)]
pub enum ThinkingDepthArg {
    Quick,
    #[default]
    Standard,
    Deep,
    Maximum,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum AgentTypeArg {
    Security,
    Performance,
    Architecture,
    General,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EvolveAction {
    #[command(about = "Learn new pattern")]
    Learn {
        #[arg(help = "Context")]
        context: String,

        #[arg(help = "Outcome")]
        outcome: String,
    },

    #[command(about = "Run self-optimization")]
    Optimize {
        #[arg(value_enum, help = "Target")]
        target: Option<OptimizeTarget>,
    },

    #[command(about = "Show evolution stats")]
    Stats,

    #[command(about = "Show learned patterns")]
    Patterns {
        #[arg(short, long, default_value_t = 10, help = "Limit")]
        limit: usize,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OptimizeTarget {
    Quality,
    Speed,
    Accuracy,
    Satisfaction,
    Balanced,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum FeedbackTypeArg {
    Positive,
    Negative,
    Bug,
    Feature,
    Correction,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum BackendAnalysisType {
    Map,
    Flow,
    Auth,
    Secrets,
    Security,
    Full,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum IoTAnalysisType {
    Protocol,
    Debug,
    Power,
    Connect,
    Sensor,
    Firmware,
    Full,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum IOSAnalysisType {
    Ui,
    Hig,
    Perf,
    A11y,
    Device,
    Memory,
    Full,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum AndroidAnalysisType {
    Ui,
    Material,
    Perf,
    Lifecycle,
    Compat,
    A11y,
    Full,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum WebAnalysisType {
    Vitals,
    A11y,
    Seo,
    Bundle,
    Perf,
    Audit,
    Full,
}

impl Cli {
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
