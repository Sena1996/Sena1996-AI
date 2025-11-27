# Sena1996 AI Tool 🦁 - Complete Usage Guide

## Make Your AI Collaborative and Smarter™

**Version 12.0.0** - Complete guide to using Sena1996 AI Tool with Claude Code.

---

## Table of Contents

1. [Installation](#installation)
2. [Multi-AI Provider Integration](#multi-ai-provider-integration)
3. [AI-to-AI Collaboration](#ai-to-ai-collaboration)
4. [Consensus Voting System](#consensus-voting-system)
5. [Specialist Routing](#specialist-routing)
6. [Network Collaboration](#network-collaboration)
7. [Session Management](#session-management)
8. [Cross-Session Messaging](#cross-session-messaging)
9. [Task Management](#task-management)
10. [Domain Agents](#domain-agents)
11. [Intelligence System](#intelligence-system)
12. [Knowledge System](#knowledge-system)
13. [Evolution System](#evolution-system)
14. [Health & Metrics](#health--metrics)
15. [Desktop Application](#desktop-application)
16. [Hooks & MCP](#hooks--mcp)
17. [Configuration](#configuration)
18. [Troubleshooting](#troubleshooting)
19. [Quick Reference](#quick-reference)

---

## Installation

### Quick Install
```bash
cd ~/AI/Sena1996-AI
./setup.sh
# Choose option 2 (Merge) to upgrade existing setup
# Choose option 1 (Fresh) for new installation
```

### Manual Install
```bash
cargo build --release
cp target/release/sena ~/.local/bin/sena
```

### Verify Installation
```bash
sena --version    # Should show: sena 12.0.0
sena health       # System health check
```

### Environment Setup
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"

# API Keys for multi-provider support
export ANTHROPIC_API_KEY="your-anthropic-key"
export OPENAI_API_KEY="your-openai-key"
export GOOGLE_API_KEY="your-google-key"
export MISTRAL_API_KEY="your-mistral-key"
```

---

## Multi-AI Provider Integration

SENA supports multiple AI providers for enhanced capabilities and fallback options.

### Supported Providers

| Provider | Environment Variable | Default Model | Features |
|----------|---------------------|---------------|----------|
| **Claude (Anthropic)** | `ANTHROPIC_API_KEY` | claude-sonnet-4-5 | Streaming, Tools, Vision |
| **OpenAI** | `OPENAI_API_KEY` | gpt-4.1 | Streaming, Tools, Vision |
| **Google Gemini** | `GOOGLE_API_KEY` | gemini-2.5-flash | Streaming, Tools, Vision |
| **Ollama (Local)** | None (localhost:11434) | llama3.2 | Local inference |
| **Mistral AI** | `MISTRAL_API_KEY` | mistral-large-latest | Streaming, Tools |

### Provider Commands

| Command | Description |
|---------|-------------|
| `sena provider list` | List all configured providers |
| `sena provider models` | List available models across all providers |
| `sena provider models --provider claude` | List models for specific provider |
| `sena provider status` | Check connectivity for all providers |
| `sena provider set-default <id>` | Set default provider |
| `sena provider test <id>` | Test a specific provider |

### Provider Configuration File

Create `~/.sena/providers.toml`:
```toml
[providers.claude]
provider_id = "claude"
enabled = true
api_key_env = "ANTHROPIC_API_KEY"
default_model = "claude-sonnet-4-5-20250929"

[providers.openai]
provider_id = "openai"
enabled = true
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4.1"

[providers.gemini]
provider_id = "gemini"
enabled = true
api_key_env = "GOOGLE_API_KEY"
default_model = "gemini-2.5-flash"

[providers.ollama]
provider_id = "ollama"
enabled = true
base_url = "http://localhost:11434"
default_model = "llama3.2"

[providers.mistral]
provider_id = "mistral"
enabled = true
api_key_env = "MISTRAL_API_KEY"
default_model = "mistral-large-latest"

default_provider = "claude"
fallback_chain = ["openai", "gemini", "ollama"]
cost_optimization = false
```

### Examples
```bash
# List all providers and their status
sena provider list

# Check which models are available
sena provider models

# Test OpenAI connectivity
sena provider test openai

# Set Claude as default
sena provider set-default claude
```

---

## AI-to-AI Collaboration

Create collaboration sessions where multiple AI agents work together on tasks.

### Collaboration Commands

| Command | Description |
|---------|-------------|
| `sena collab create "Name" --host <provider>` | Create new collaboration session |
| `sena collab join <session-id> --provider <id>` | Join session with specified provider |
| `sena collab start <session-id>` | Start the collaboration |
| `sena collab broadcast <session-id> "message"` | Send message to all participants |
| `sena collab send <session-id> <agent-id> "msg"` | Send to specific agent |
| `sena collab list` | List all active sessions |
| `sena collab info <session-id>` | Get session details |
| `sena collab end <session-id>` | End collaboration session |

### Creating a Collaboration Session
```bash
# Create session with Claude as host
sena collab create "Code Review Session" --host claude

# Output: Session created: collab_abc12345
# Session ID: collab_abc12345
# Host: claude (claude-sonnet-4-5)
```

### Adding Participants
```bash
# Add OpenAI GPT-4 to the session
sena collab join collab_abc12345 --provider openai

# Add Gemini
sena collab join collab_abc12345 --provider gemini

# List participants
sena collab info collab_abc12345
```

### Starting Collaboration
```bash
# Start the session (all participants ready)
sena collab start collab_abc12345

# Broadcast a task to all AI agents
sena collab broadcast collab_abc12345 "Review this code for security issues: [code]"

# Each AI will respond with their analysis
```

### Example Workflow
```bash
# Step 1: Create session
sena collab create "Architecture Review" --host claude

# Step 2: Add participants
sena collab join collab_abc12345 --provider openai
sena collab join collab_abc12345 --provider gemini

# Step 3: Start collaboration
sena collab start collab_abc12345

# Step 4: Send task
sena collab broadcast collab_abc12345 "Evaluate microservices vs monolith for our use case"

# Step 5: Each AI provides their perspective
# Claude: Focuses on code quality and maintainability
# GPT-4: Provides operational considerations
# Gemini: Analyzes data flow implications

# Step 6: End session when done
sena collab end collab_abc12345
```

---

## Consensus Voting System

Enable democratic decision-making between AI agents.

### Voting Strategies

| Strategy | Threshold | Description |
|----------|-----------|-------------|
| `unanimous` | 100% | All participants must approve |
| `majority` | >50% | More than half must approve |
| `supermajority` | >67% | Two-thirds must approve |
| `weighted` | >50% weighted | Votes weighted by expertise |

### Consensus Commands

| Command | Description |
|---------|-------------|
| `sena collab propose "Title" --strategy <type>` | Create proposal |
| `sena collab vote <proposal-id> <choice>` | Cast vote |
| `sena collab vote <id> approve --reasoning "why"` | Vote with reasoning |
| `sena collab proposals <session-id>` | List proposals |
| `sena collab result <proposal-id>` | Get voting result |

### Vote Choices
- `approve` - Vote in favor
- `reject` - Vote against
- `abstain` - No vote (doesn't count toward threshold)

### Example: Architecture Decision
```bash
# Create proposal for architecture decision
sena collab propose "Should we use microservices?" \
  --session collab_abc12345 \
  --strategy supermajority

# Output: Proposal created: prop_xyz789

# Each AI agent votes
# Claude votes:
sena collab vote prop_xyz789 approve \
  --reasoning "Better scalability and team autonomy"

# GPT-4 votes:
sena collab vote prop_xyz789 approve \
  --reasoning "Easier deployment and fault isolation"

# Gemini votes:
sena collab vote prop_xyz789 reject \
  --reasoning "Increased complexity for current team size"

# Check result
sena collab result prop_xyz789
# Result: APPROVED (66.7% approval, threshold: 67%)
```

### Weighted Voting Example
```bash
# Create weighted proposal (experts have more influence)
sena collab propose "Use Rust for backend?" \
  --session collab_abc12345 \
  --strategy weighted

# Claude (weight: 2.0 - Rust expert)
sena collab vote prop_xyz789 approve --weight 2.0

# GPT-4 (weight: 1.0 - General)
sena collab vote prop_xyz789 reject --weight 1.0

# Result: APPROVED (weighted score: 2.0 vs 1.0)
```

---

## Specialist Routing

SENA automatically routes tasks to the best AI based on domain expertise.

### Task Domains

| Domain | Detection Keywords | Best Specialist |
|--------|-------------------|-----------------|
| `CodeGeneration` | implement, create function, write code | Claude |
| `CodeReview` | review, refactor, code quality | Claude |
| `Security` | security, vulnerability, exploit, CVE | Claude |
| `Performance` | performance, optimize, benchmark, latency | Claude |
| `Architecture` | architecture, design pattern, system design | Claude |
| `Testing` | test, spec, coverage, mock | Claude |
| `Documentation` | document, readme, api doc | Claude/GPT-4 |
| `NaturalLanguage` | translate, summarize, explain, write | GPT-4 |
| `Creative` | creative, story, design, brainstorm | GPT-4 |
| `DataAnalysis` | data, analyze, statistic, chart | Gemini |
| `Mathematics` | math, equation, calculate, proof | Gemini |
| `Research` | research, investigate, find information | GPT-4/Gemini |

### Routing Strategies

| Strategy | Description |
|----------|-------------|
| `BestMatch` | Route to highest expertise score (default) |
| `RoundRobin` | Distribute evenly across agents |
| `LeastLoaded` | Route to agent with lowest current load |
| `Random` | Random selection from available agents |

### How Routing Works
```bash
# Task: "Review this code for security vulnerabilities"
# Domain detected: Security
# Best match: Claude (score: 0.95)
# Task routed to Claude

# Task: "Write a creative story about AI"
# Domain detected: Creative
# Best match: GPT-4 (score: 0.92)
# Task routed to GPT-4

# Task: "Analyze this dataset for trends"
# Domain detected: DataAnalysis
# Best match: Gemini (score: 0.93)
# Task routed to Gemini
```

---

## Network Collaboration

Connect multiple SENA instances across your local network (WiFi or LAN).

### Start Network Server
```bash
sena network start                      # Start on default port 9876
sena network start --name "My Mac"      # Start with custom name
sena network start --port 9877          # Start on custom port
```

### Network Commands

| Command | Description |
|---------|-------------|
| `sena network start` | Start network server |
| `sena network stop` | Stop network server |
| `sena network status` | Show server status |
| `sena network info` | Show peer ID and name |
| `sena network set-name "Name"` | Change display name |

### Peer Discovery
```bash
sena discover                           # Find peers on network
sena discover --timeout 10              # Extended discovery (10 seconds)
```

### Peer Management

| Command | Description |
|---------|-------------|
| `sena peer list` | List known peers |
| `sena peer add <ip> --name "Name"` | Add peer manually |
| `sena peer authorize <id>` | Generate auth token |
| `sena peer connect <ip> --token <token>` | Connect with token |
| `sena peer revoke <id>` | Revoke authorization |
| `sena peer ping <id>` | Ping peer |

### Complete Connection Flow
```bash
# On Machine A (192.168.1.50):
sena network start --name "Workstation"
sena peer list
# Note your peer ID: peer_abc123

# On Machine B (192.168.1.100):
sena discover
# See "Workstation" at 192.168.1.50
sena peer add 192.168.1.50 --name "Workstation"

# Back on Machine A:
sena peer list
# See Machine B's request
sena peer authorize peer_xyz789
# Output: Token: eyJhbGciOiJIUzI1NiJ9...

# Share the token with Machine B securely

# On Machine B:
sena peer connect 192.168.1.50 --token eyJhbGciOiJIUzI1NiJ9...
# Output: Connected to Workstation!

# Now both machines can collaborate
sena tell Workstation "Hello from Machine B!"
```

---

## Session Management

Sessions allow multiple Claude Code windows to collaborate.

### Rules
- Each **role** can have only ONE active session
- Available roles: `general`, `backend`, `web`, `android`, `ios`, `iot`
- Sessions expire after 24 hours of inactivity

### Commands

| Command | Description |
|---------|-------------|
| `sena session start --name 'MySession'` | Start a new session |
| `sena session start --name 'API' --role backend` | Start with specific role |
| `sena session list` | List all active sessions |
| `sena session info` | Current session info |
| `sena session end --id <session-id>` | End a session |
| `sena who` | Quick view of online sessions |

### Examples
```bash
# Start backend session
sena session start --name 'BackendDev' --role backend

# Start Android session in another terminal
sena session start --name 'AndroidDev' --role android

# List all active sessions
sena session list

# See who's online
sena who

# End a session
sena session end --id backend-8de02244
```

---

## Cross-Session Messaging

Send messages between Claude Code sessions.

### Rules
- Use **session name** OR **session ID** to send messages
- Session names are case-insensitive
- Messages are stored until read

### Commands

| Command | Description |
|---------|-------------|
| `sena tell <name/id> "message"` | Send message to session |
| `sena inbox` | Check received messages |

### Examples
```bash
# Send by session name (easy)
sena tell Android "Check the UI component"
sena tell BackendDev "API is ready"

# Send by session ID
sena tell backend-8de02244 "Deploy complete"

# Check inbox
sena inbox
```

---

## Task Management

Create and assign tasks across sessions.

### Rules
- Tasks can be assigned by **name** or **ID**
- Priority levels: `low`, `medium`, `high`, `critical`
- Status: `pending`, `in_progress`, `completed`

### Commands

| Command | Description |
|---------|-------------|
| `sena task new "title" --to <name/id>` | Create task |
| `sena task new "title" --to <name> --priority high` | Create with priority |
| `sena task list` | List all tasks |
| `sena task done <task-id>` | Mark complete |

### Examples
```bash
# Create task for Android team
sena task new "Fix login screen" --to Android

# High priority task
sena task new "Critical bug fix" --to Backend --priority critical

# List tasks
sena task list

# Complete task
sena task done 1
```

---

## Domain Agents

Specialized analysis for different platforms.

### Backend Agent
```bash
sena backend map "GET /api/users"        # API endpoint analysis
sena backend flow "db.query(...)"         # Data flow analysis
sena backend security "SELECT * FROM"     # SQL injection detection
```

### Android Agent
```bash
sena android lifecycle "AppCompatActivity"  # Lifecycle analysis
sena android compat "minSdk 21"             # Compatibility check
```

### iOS Agent
```bash
sena ios ui "struct ContentView: View"    # SwiftUI HIG compliance
sena ios perf "DispatchQueue.main"        # Performance analysis
```

### IoT Agent
```bash
sena iot protocol "mqtt.connect(...)"     # Protocol analysis
sena iot power "sleep_mode()"             # Power optimization
```

### Web Agent
```bash
sena web audit "<script>alert()</script>"  # XSS detection
sena web a11y "<img src='...'>"            # Accessibility check
```

---

## Intelligence System

Deep thinking and analysis capabilities.

### Commands

| Command | Description |
|---------|-------------|
| `sena think "question"` | Quick analysis |
| `sena think --depth deep "question"` | Deep analysis |
| `sena agent security "code"` | Security agent |
| `sena agent performance "code"` | Performance agent |
| `sena agent architecture "code"` | Architecture agent |

### Thinking Depths
- `quick` - Fast response (~1 second)
- `standard` - Balanced (default, ~3 seconds)
- `deep` - Thorough analysis (~10 seconds)
- `maximum` - Most comprehensive (~30 seconds)

### Examples
```bash
# Quick thinking
sena think "How to optimize this query?"

# Deep analysis
sena think --depth deep "Should we use microservices?"

# Maximum depth for critical decisions
sena think --depth maximum "Design a scalable authentication system"

# Security analysis
sena agent security "user_input = request.get('data')"
```

---

## Knowledge System

Access built-in knowledge patterns.

### Commands

| Command | Description |
|---------|-------------|
| `sena knowledge search "pattern"` | Search knowledge base |
| `sena knowledge list reasoning` | List reasoning frameworks |
| `sena knowledge list security` | List security patterns |
| `sena knowledge list performance` | List performance patterns |
| `sena knowledge list architecture` | List architecture patterns |
| `sena knowledge stats` | Knowledge statistics |

### Available Categories
- **Reasoning**: First Principles, 5 Whys, Decision Matrix, Root Cause
- **Security**: OWASP Top 10, Auth patterns, Crypto, Input validation
- **Performance**: Algorithm optimization, Caching, N+1, Memory
- **Architecture**: SOLID, Design Patterns, DDD, CQRS, Event Sourcing

---

## Evolution System

Pattern learning and self-optimization.

### Commands

| Command | Description |
|---------|-------------|
| `sena evolve stats` | Evolution statistics |
| `sena feedback positive "Great work!"` | Positive feedback |
| `sena feedback negative "Needs improvement"` | Negative feedback |
| `sena feedback bug "Found issue"` | Report bug |

---

## Health & Metrics

System health monitoring.

### Commands

| Command | Description |
|---------|-------------|
| `sena health` | Quick health check |
| `sena health --detailed` | Detailed health report |
| `sena metrics` | Full system metrics |

### Health Output
```bash
sena health
# Output:
# ╔══════════════════════════════════════════════════════════════╗
# ║                    SENA 🦁 HEALTH STATUS                      ║
# ╠══════════════════════════════════════════════════════════════╣
# ║  Status: HEALTHY                          Score: 95/100      ║
# ║  Providers: 4/5 connected                                    ║
# ║  Sessions: 2 active                                          ║
# ║  Uptime: 3h 42m                                              ║
# ╚══════════════════════════════════════════════════════════════╝
```

---

## Desktop Application

SENA includes a cross-platform desktop application built with Tauri 2.0.

### Features
- Provider management with visual status
- Collaboration session dashboard
- Real-time chat interface
- System health monitoring
- Dark/Light theme support

### Building the Desktop App
```bash
cd sena-ui

# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Platforms
- **macOS**: `.dmg` and `.app` bundles
- **Windows**: `.msi` installer and `.exe`
- **Linux**: `.deb`, `.rpm`, and `.AppImage`

---

## Hooks & MCP

### Claude Code Hooks
SENA automatically integrates via hooks:
- **UserPromptSubmit**: Checks inbox, detects formats
- Auto-approved bash commands (no prompts)

### MCP Server
For Claude Desktop integration:
```bash
sena mcp
```

### Daemon
```bash
sena daemon status   # Check daemon status
sena daemon start    # Start daemon
sena daemon stop     # Stop daemon
```

---

## Configuration

### Config File Location
```
~/.sena/config.toml
```

### Complete Config Options
```toml
[user]
name = "YourName"       # Your name
emoji = "🦁"            # Your emoji
prefix = "SENA"         # Display prefix
command = "sena"        # CLI command name

[general]
log_level = "info"      # Log level: trace, debug, info, warn, error

[intelligence]
default_thinking_depth = "standard"  # quick/standard/deep/maximum
default_model = "balanced"           # fast/balanced/powerful
auto_agent_selection = true          # Auto-select agents
primary_agent = "general"            # Default agent

[evolution]
pattern_learning = true
self_optimization = true
feedback_collection = true

[network]
default_port = 9876
auto_discovery = true
tls_enabled = true

[output]
color = true
unicode = true
progress_bars = true
```

### Custom Command Name
Set a custom command name (e.g., `jarvis` instead of `sena`):
```bash
./setup.sh
# Enter custom command name when prompted
```

Then use:
```bash
jarvis health
jarvis session start --name 'MySession'
jarvis network status
```

---

## Troubleshooting

### Bash command asking for permission?
Restart Claude Code after setup. The permissions are loaded at startup.

### Session already exists error?
Each role can only have one session. Either:
1. Use a different role: `--role backend`
2. End existing session: `sena session end --id <id>`

### Messages not received?
Messages are stored but Claude doesn't auto-check. The hook checks inbox on prompt submit.

### Command not found?
Ensure `~/.local/bin` is in your PATH:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Hook not running?
Check `~/.claude/settings.json` has the hook configured:
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {"command": "/Users/YOUR_USER/.local/bin/sena hook user-prompt-submit"}
    ]
  }
}
```

### Network peers not discovering?
- Ensure both machines are on same network
- Check firewall allows port 9876
- Use manual `peer add` as fallback

### Provider not connecting?
```bash
# Check API key is set
echo $ANTHROPIC_API_KEY

# Test specific provider
sena provider test claude

# Check provider status
sena provider status
```

### Collaboration session fails?
```bash
# List active sessions
sena collab list

# Check session info
sena collab info <session-id>

# Ensure all participants have joined before starting
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Check health | `sena health` |
| List providers | `sena provider list` |
| Create collab session | `sena collab create "Name" --host claude` |
| Join collab session | `sena collab join <id> --provider openai` |
| Start collaboration | `sena collab start <id>` |
| Broadcast message | `sena collab broadcast <id> "message"` |
| Create proposal | `sena collab propose "Question?" --strategy majority` |
| Vote on proposal | `sena collab vote <id> approve` |
| Start session | `sena session start --name 'Name' --role backend` |
| List sessions | `sena who` |
| Send message | `sena tell SessionName "message"` |
| Check inbox | `sena inbox` |
| Create task | `sena task new "title" --to SessionName` |
| Deep think | `sena think --depth deep "question"` |
| Security scan | `sena agent security "code"` |
| Start network | `sena network start --name "My PC"` |
| Discover peers | `sena discover` |
| List peers | `sena peer list` |

---

## Version History

| Version | Highlights |
|---------|------------|
| **12.0.0** | Multi-AI providers, AI-to-AI collaboration, consensus voting, specialist routing, Tauri desktop app |
| 11.0.x | Network collaboration, peer discovery, TLS encryption |
| 10.0.x | Session management, cross-session messaging |
| 9.0.x | Domain agents, intelligence system |

---

## Credits

**Sena1996™** - Creator and Lead Developer
**Claude (Anthropic)** - AI Development Partner

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                   Sena1996 AI Tool 🦁                        ║
║                       v12.0.0                                ║
║                                                              ║
║         Make Your AI Collaborative and Smarter™             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
