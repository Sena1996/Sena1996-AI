# Changelog

All notable changes to Sena1996 AI Tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [13.1.3] - 2025-11-30

### Added - Tool System & Automation
- **Tool/Function Calling System** - Provider-agnostic tool execution
  - `sena tools list` - List all available tools
  - `sena tools execute <name>` - Execute tools with parameters
  - Built-in tools: read_file, write_file, search_files, execute_command, web_search
  - Tools UI page in Hub desktop app
- **Persistent Memory System** - Cross-session AI memory
  - `sena memory add "<content>" --type <type>` - Store memories
  - `sena memory search <query>` - Search by content/tags
  - `sena memory list` - List all memories
  - `sena memory stats` - Memory statistics
  - Memory types: Preference, Fact, Project, Context, Conversation
  - Memory UI page with add/search/filter
- **Autonomous Agent Loop** - Multi-step task automation
  - `sena auto "<task>"` - Run autonomous task
  - `sena auto "<task>" --max-steps N` - Limit execution steps
  - `sena auto "<task>" --confirm` - Require step confirmation
  - Planning and execution with tool integration
- **Git Integration** - AI-powered git operations
  - `sena git status` - Enhanced status with insights
  - `sena git commit` - AI-generated commit messages
  - `sena git log` - Formatted commit history
  - `sena git diff` - Show changes
- **Streaming Response System** - Real-time output
  - StreamWriter/StreamReader for buffered streaming
  - WebSocket broadcaster for UI updates
  - Console and JSON renderers
- **Semantic Memory Search** - Vector embeddings
  - Cosine similarity for text matching
  - Hybrid search (keyword + semantic)
  - SimpleHashEmbedder for local embeddings
- **Performance Benchmarks** - Criterion benchmarks
  - Memory operations benchmarks
  - Tool system benchmarks
  - Scaling tests

### Added - Hub UI Pages
- **Tools Page** - Execute and manage AI tools visually
- **Memory Page** - Add, search, filter memories with stats
- **Features Page** - Complete documentation with 10 features, commands, examples

### Changed
- Updated CLAUDE.md with new v13.1.3 commands
- Navigation updated with Tools, Memory, Features pages
- Tauri backend commands for tools and memory

---

## [13.0.0] - 2025-11-28

### Added - Cross-Hub Federation (Hub v2.0)
- **Hub Identity System** - Persistent UUID-based hub identification
  - Each SENA installation gets a unique hub ID
  - Customizable hub display name
  - Hostname and port configuration
- **Auth Passkey** - Secure authentication for hub-to-hub connections
  - Generate/regenerate passkeys in Settings
  - Share passkey with trusted hubs
  - Passkey-based connection approval
- **Cross-Hub Peer Management** - Connect SENA hubs across machines
  - `sena hub peers` - List connected remote hubs
  - `sena hub requests` - View pending connection requests
  - `sena hub approve/reject` - Manage connection requests
  - `sena hub connect` - Request connection to remote hub
  - `sena hub disconnect` - Remove trusted hub
- **Federated Sessions** - View all sessions across connected hubs
  - `sena hub federation` - List local + remote sessions
  - Combined session view in Desktop UI
- **Cross-Hub Messaging** - Message sessions on remote hubs
  - Syntax: `@HubName:SessionName message`
  - Syntax: `tell HubName:SessionName message`
- **Peers UI Page** - New desktop page for hub management
  - View/edit hub identity
  - Manage connected peers
  - Approve/reject connection requests
- **Settings Hub Credentials** - Hub management in Settings page
  - View hub identity (name, ID, port)
  - Edit hub display name
  - Generate/copy auth passkeys

### Added - Network Protocol v2.0
- `ConnectionRequest` - Request connection to another hub
- `ConnectionApproved/Denied` - Approval flow responses
- `SessionListRequest/Response` - Query remote hub sessions
- `CrossHubMessage` - Direct message to remote session
- `CrossHubBroadcast` - Broadcast to all remote sessions

### Changed
- Hub module header updated to v2.0
- Protocol version updated from 1.0 to 2.0
- Chat page supports cross-hub addressing
- Smart input parser handles `HubName:SessionName` syntax

### Fixed
- Session persistence across app restarts (from v12.0.x)
- CLI session integration with Desktop UI (from v12.0.x)

### Technical
- 254 tests passing
- Zero clippy warnings
- New modules: `hub/identity.rs`, `hub/peers.rs`
- Tauri commands: `get_hub_identity`, `set_hub_name`, `get_hub_passkey`, `generate_hub_passkey`, `get_connected_peers`, `get_pending_requests`, `approve_peer_request`, `reject_peer_request`, `disconnect_peer`, `get_federated_sessions`

---

## [12.0.0] - 2025-11-27

### Known Issues (Fixed in v13.0.0)
- Session state not persisting properly across restarts
- CLI sessions not syncing with Desktop UI in real-time

### Added
- **Multi-AI Provider Integration** - Support for Claude, OpenAI, Gemini, Ollama, and Mistral
- **AI-to-AI Collaboration** - Multiple AI agents working together via `sena-collab` crate
- **Consensus Voting System** - Democratic decision-making between AI agents
  - Unanimous, Majority, SuperMajority, Weighted voting strategies
  - Vote weight based on expertise scores
  - Proposal lifecycle management
- **Specialist Routing** - Automatic task delegation based on domain expertise
  - Domains: CodeGeneration, CodeReview, Security, Performance, Architecture, Testing, Documentation, NaturalLanguage, Creative, DataAnalysis, Mathematics, Research
  - Strategies: BestMatch, RoundRobin, LeastLoaded, Random
- **Tauri 2.0 Desktop Application** - Cross-platform GUI
  - React + TypeScript frontend
  - Provider management dashboard
  - Collaboration session interface
  - System health monitoring
- **sena-providers crate** - Provider abstraction layer
  - Unified API across all providers
  - Fallback chain support
  - Cost optimization options
- **sena-collab crate** - Collaboration framework
  - Session management
  - Message routing
  - Agent permissions

### Changed
- Restructured into Cargo workspace with multiple crates
- Updated README.md with v12 features
- Complete rewrite of USAGE.md
- Enhanced error handling with proper Result types

### Technical
- 280 tests passing
- Zero clippy warnings
- SENA1996-AI Elite Standards enforced

---

## [11.0.2] - 2025-11-26

### Added
- CLAUDE.md configuration for elite coding standards
- SENA context hook for Claude Code integration
- USAGE.md documentation

### Fixed
- Setup script no longer deletes Claude credentials
- Bash permission format updated per Claude Code docs

---

## [11.0.0] - 2025-11-25

### Added - Network Collaboration
- **Network Module** - Full TCP networking for cross-machine collaboration
- **mDNS Discovery** - Automatic peer discovery using `_sena._tcp.local.` service
- **TLS Encryption** - Self-signed certificates for secure communication
- **Token Authentication** - Time-limited authorization tokens with expiry
- **Peer Management** - Add, authorize, connect, revoke peers
- **Network Commands**:
  - `sena network start/stop/status/info/set-name`
  - `sena peer list/add/authorize/connect/revoke/ping`
  - `sena discover` - Find peers on local network

### Added - Custom Command System
- Custom CLI command names via symlinks (jarvis, lucy, etc.)
- Setup wizard creates symlink: `~/.local/bin/{command} -> ~/.local/bin/sena`
- All outputs use user-configured branding

---

## [10.0.0] - 2025-11-20

### Added - Session Collaboration
- Multi-session collaboration (Android/Web/Backend/IoT roles)
- Unix socket server for real-time IPC
- Task management across sessions
- Inter-session messaging
- `sena tell` and `sena inbox` commands

---

## [9.0.0] - 2025-11-15

### Added - Intelligence & Knowledge
- **Knowledge System** - 47 patterns across domains
  - Reasoning frameworks (First Principles, 5 Whys, Decision Matrix)
  - Security patterns (OWASP Top 10, Auth, Crypto)
  - Performance patterns
  - Architecture patterns (SOLID, Design Patterns, DDD)
- **Intelligence System** - Extended thinking engine
  - Thinking depths: Quick, Standard, Deep, Maximum
- **Evolution System** - Pattern learning and self-optimization
- **7 Ancient Wisdom Layers** - Truth-embedded architecture

---

## [8.0.0] - 2025-11-10

### Added - Claude Integration
- MCP server for Claude Desktop
- Claude Code hooks (UserPromptSubmit)
- Domain agents (Backend, iOS, Android, Web, IoT)

---

## [7.0.0] - 2025-11-05

### Added - Initial Release
- Core CLI functionality
- Health monitoring system
- Configuration management
- Rust implementation (~3.5MB binary)

---

## Release Types

| Type | Version | Description |
|------|---------|-------------|
| Major | X.0.0 | Breaking changes or major features |
| Minor | X.Y.0 | New features, backward compatible |
| Patch | X.Y.Z | Bug fixes, backward compatible |

---

## Links

- [GitHub Releases](https://github.com/Sena1996/Sena1996-AI/releases)
- [Documentation](https://github.com/Sena1996/Sena1996-AI/blob/main/USAGE.md)
- [Issues](https://github.com/Sena1996/Sena1996-AI/issues)

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                   Sena1996 AI Tool 🦁                        ║
║                       v13.0.0                                ║
║                                                              ║
║         Make Your AI Collaborative and Smarter™             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
