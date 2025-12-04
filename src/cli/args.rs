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

    #[command(about = "Network collaboration")]
    Network {
        #[command(subcommand)]
        action: NetworkAction,
    },

    #[command(about = "Peer management")]
    Peer {
        #[command(subcommand)]
        action: PeerAction,
    },

    #[command(about = "Discover peers on network")]
    Discover {
        #[arg(
            short,
            long,
            default_value_t = 5,
            help = "Discovery timeout in seconds"
        )]
        timeout: u64,
    },

    #[command(about = "AI Provider management")]
    Provider {
        #[command(subcommand)]
        action: ProviderAction,
    },

    #[command(about = "AI-to-AI collaboration")]
    Collab {
        #[command(subcommand)]
        action: CollabAction,
    },

    #[command(about = "Tool management and execution")]
    Tools {
        #[command(subcommand)]
        action: ToolsAction,
    },

    #[command(about = "Persistent memory management")]
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },

    #[command(about = "Autonomous agent execution")]
    Auto {
        #[arg(help = "Task description")]
        task: String,

        #[arg(short, long, default_value_t = 10, help = "Maximum steps")]
        max_steps: usize,

        #[arg(short, long, help = "Working directory")]
        cwd: Option<String>,

        #[arg(short, long, default_value_t = false, help = "Require confirmation for actions")]
        confirm: bool,
    },

    #[command(about = "Git operations with AI assistance")]
    Git {
        #[command(subcommand)]
        action: GitAction,
    },

    #[command(about = "Guardian middleware for AI safety")]
    Guardian {
        #[command(subcommand)]
        action: GuardianAction,
    },

    #[command(about = "Devil mode - parallel AI execution")]
    Devil {
        #[command(subcommand)]
        action: DevilAction,
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
    #[command(about = "List all sessions with details")]
    Sessions,
    #[command(about = "Send message from Hub to any session")]
    Tell {
        #[arg(help = "Target session (name or ID, or HubName:SessionName for remote)")]
        target: String,
        #[arg(help = "Message content")]
        message: String,
    },
    #[command(about = "View all messages across sessions")]
    Messages {
        #[arg(short, long, default_value = "20", help = "Number of messages")]
        count: usize,
    },
    #[command(about = "Broadcast message to all sessions")]
    Broadcast {
        #[arg(help = "Message to broadcast")]
        message: String,
    },
    #[command(about = "Show conflicts")]
    Conflicts,
    #[command(about = "Clear hub data")]
    Clear,
    #[command(about = "Show this hub's identity")]
    Identity,
    #[command(about = "Set hub display name")]
    SetName {
        #[arg(help = "New hub name")]
        name: String,
    },
    #[command(about = "List connected remote hubs")]
    Peers,
    #[command(about = "Show pending connection requests")]
    Requests,
    #[command(about = "Request connection to remote hub")]
    Connect {
        #[arg(help = "Hub address (IP:port or hostname:port)")]
        address: String,
        #[arg(short, long, help = "Optional connection message")]
        message: Option<String>,
    },
    #[command(about = "Approve pending connection request")]
    Approve {
        #[arg(help = "Request ID (first 8 characters)")]
        request_id: String,
    },
    #[command(about = "Reject pending connection request")]
    Reject {
        #[arg(help = "Request ID (first 8 characters)")]
        request_id: String,
    },
    #[command(about = "Disconnect from remote hub")]
    Disconnect {
        #[arg(help = "Hub ID or name")]
        hub: String,
    },
    #[command(about = "List all sessions (local + remote)")]
    Federation,
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

#[derive(Subcommand, Debug, Clone)]
pub enum NetworkAction {
    #[command(about = "Start network server")]
    Start {
        #[arg(short, long, default_value_t = 9876, help = "Port to listen on")]
        port: u16,

        #[arg(short, long, help = "Display name for this peer")]
        name: Option<String>,
    },

    #[command(about = "Stop network server")]
    Stop,

    #[command(about = "Show network status")]
    Status,

    #[command(about = "Show network info")]
    Info,

    #[command(about = "Set peer display name")]
    SetName {
        #[arg(help = "New display name")]
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ProviderAction {
    #[command(about = "List all available AI providers")]
    List,

    #[command(about = "Show provider status")]
    Status,

    #[command(about = "List all available models")]
    Models {
        #[arg(short, long, help = "Filter by provider")]
        provider: Option<String>,
    },

    #[command(about = "Chat with an AI provider")]
    Chat {
        #[arg(help = "Message to send")]
        message: String,

        #[arg(short, long, help = "Provider to use")]
        provider: Option<String>,

        #[arg(short, long, help = "Model to use")]
        model: Option<String>,
    },

    #[command(about = "Set default provider")]
    Default {
        #[arg(help = "Provider ID")]
        provider_id: String,
    },

    #[command(about = "Test provider connectivity")]
    Test {
        #[arg(help = "Provider to test (or 'all')")]
        provider: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum PeerAction {
    #[command(about = "List known peers")]
    List {
        #[arg(short, long, help = "Show only authorized peers")]
        authorized: bool,
    },

    #[command(about = "Add peer manually")]
    Add {
        #[arg(help = "Peer IP address")]
        address: String,

        #[arg(short, long, default_value_t = 9876, help = "Peer port")]
        port: u16,

        #[arg(short, long, help = "Peer name")]
        name: Option<String>,
    },

    #[command(about = "Remove peer")]
    Remove {
        #[arg(help = "Peer ID")]
        peer_id: String,
    },

    #[command(about = "Authorize peer (generate token)")]
    Authorize {
        #[arg(help = "Peer ID")]
        peer_id: String,

        #[arg(short, long, default_value_t = 300, help = "Token expiry in seconds")]
        expires: i64,
    },

    #[command(about = "Connect to peer with token")]
    Connect {
        #[arg(help = "Peer IP address")]
        address: String,

        #[arg(short, long, default_value_t = 9876, help = "Peer port")]
        port: u16,

        #[arg(short, long, help = "Authorization token")]
        token: String,
    },

    #[command(about = "Revoke peer authorization")]
    Revoke {
        #[arg(help = "Peer ID")]
        peer_id: String,
    },

    #[command(about = "Ping peer")]
    Ping {
        #[arg(help = "Peer ID or address")]
        target: String,

        #[arg(short, long, default_value_t = 9876, help = "Port")]
        port: u16,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CollabAction {
    #[command(about = "Create a new collaboration session")]
    New {
        #[arg(help = "Session name")]
        name: String,

        #[arg(short, long, default_value = "claude", help = "Host provider")]
        provider: String,
    },

    #[command(about = "List active collaboration sessions")]
    List,

    #[command(about = "Join an existing session")]
    Join {
        #[arg(help = "Session ID")]
        session_id: String,

        #[arg(short, long, default_value = "openai", help = "Provider to join as")]
        provider: String,
    },

    #[command(about = "Start a session")]
    Start {
        #[arg(help = "Session ID")]
        session_id: String,
    },

    #[command(about = "Send message to session")]
    Send {
        #[arg(help = "Session ID")]
        session_id: String,

        #[arg(help = "Message content")]
        message: String,

        #[arg(short, long, help = "Sender agent ID")]
        from: Option<String>,
    },

    #[command(about = "Broadcast to all agents and get responses")]
    Broadcast {
        #[arg(help = "Session ID")]
        session_id: String,

        #[arg(help = "Message to broadcast")]
        message: String,
    },

    #[command(about = "Request analysis from specific provider")]
    Analyze {
        #[arg(help = "Session ID")]
        session_id: String,

        #[arg(help = "Target provider")]
        provider: String,

        #[arg(help = "Analysis request")]
        request: String,
    },

    #[command(about = "Show session details")]
    Info {
        #[arg(help = "Session ID")]
        session_id: String,
    },

    #[command(about = "End a session")]
    End {
        #[arg(help = "Session ID")]
        session_id: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ToolsAction {
    #[command(about = "List available tools")]
    List {
        #[arg(short, long, help = "Filter by category")]
        category: Option<String>,
    },

    #[command(about = "Show tool details")]
    Info {
        #[arg(help = "Tool name")]
        name: String,
    },

    #[command(about = "Execute a tool")]
    Run {
        #[arg(help = "Tool name")]
        name: String,

        #[arg(short, long, help = "Parameters as JSON")]
        params: Option<String>,
    },

    #[command(about = "Search in code")]
    Search {
        #[arg(help = "Search pattern")]
        pattern: String,

        #[arg(short, long, default_value = ".", help = "Path to search")]
        path: String,

        #[arg(short, long, help = "File pattern (e.g., *.rs)")]
        files: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum MemoryAction {
    #[command(about = "Add a memory")]
    Add {
        #[arg(help = "Memory content")]
        content: String,

        #[arg(short = 't', long, default_value = "fact", help = "Memory type (preference, fact, project, context)")]
        memory_type: String,

        #[arg(short, long, help = "Tags (comma-separated)")]
        tags: Option<String>,

        #[arg(short, long, help = "Importance (0.0-1.0)")]
        importance: Option<f64>,
    },

    #[command(about = "Search memories")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(short, long, default_value_t = 10, help = "Max results")]
        limit: usize,
    },

    #[command(about = "List all memories")]
    List {
        #[arg(short = 't', long, help = "Filter by type")]
        memory_type: Option<String>,

        #[arg(short, long, default_value_t = 20, help = "Max results")]
        limit: usize,
    },

    #[command(about = "Remove a memory")]
    Remove {
        #[arg(help = "Memory ID")]
        id: String,
    },

    #[command(about = "Show memory statistics")]
    Stats,

    #[command(about = "Clear all memories")]
    Clear {
        #[arg(short, long, default_value_t = false, help = "Skip confirmation")]
        yes: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum GitAction {
    #[command(about = "Show git status")]
    Status,

    #[command(about = "Create a commit with AI-generated message")]
    Commit {
        #[arg(short, long, help = "Custom message (optional)")]
        message: Option<String>,

        #[arg(short, long, default_value_t = false, help = "Stage all changes")]
        all: bool,
    },

    #[command(about = "Create a pull request")]
    Pr {
        #[arg(short, long, help = "PR title")]
        title: Option<String>,

        #[arg(short, long, help = "Base branch")]
        base: Option<String>,
    },

    #[command(about = "Show git diff")]
    Diff {
        #[arg(short, long, default_value_t = false, help = "Show staged changes")]
        staged: bool,
    },

    #[command(about = "Show recent commits")]
    Log {
        #[arg(short, long, default_value_t = 10, help = "Number of commits")]
        count: usize,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum GuardianAction {
    #[command(about = "Show guardian status")]
    Status,

    #[command(about = "Enable guardian middleware")]
    Enable,

    #[command(about = "Disable guardian middleware")]
    Disable,

    #[command(about = "Validate a command")]
    Validate {
        #[arg(help = "Command to validate")]
        command: String,
    },

    #[command(about = "Check content for hallucination")]
    Check {
        #[arg(help = "Content to check")]
        content: String,
    },

    #[command(about = "Execute command through guardian")]
    Execute {
        #[arg(help = "Command to execute")]
        command: String,

        #[arg(help = "Arguments")]
        args: Vec<String>,
    },

    #[command(about = "Show audit log")]
    Audit {
        #[arg(short, long, default_value_t = 20, help = "Number of entries")]
        count: usize,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DevilAction {
    #[command(about = "Execute prompt across all AI providers in parallel")]
    Execute {
        #[arg(help = "Prompt to execute")]
        prompt: String,

        #[arg(short, long, default_value_t = 30, help = "Timeout in seconds")]
        timeout: u64,

        #[arg(short, long, value_enum, default_value_t = SynthesisMethodArg::CrossVerification, help = "Synthesis method")]
        synthesis: SynthesisMethodArg,
    },

    #[command(about = "Show devil mode status")]
    Status,

    #[command(about = "Configure devil mode")]
    Config {
        #[arg(short, long, help = "Set timeout")]
        timeout: Option<u64>,

        #[arg(short, long, help = "Set consensus threshold (0.0-1.0)")]
        consensus: Option<f64>,

        #[arg(short, long, value_enum, help = "Set synthesis method")]
        synthesis: Option<SynthesisMethodArg>,
    },

    #[command(about = "Test parallel execution with mock providers")]
    Test {
        #[arg(help = "Test prompt")]
        prompt: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Default)]
pub enum SynthesisMethodArg {
    MajorityVoting,
    WeightedMerge,
    BestOfN,
    MetaLlm,
    #[default]
    CrossVerification,
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
