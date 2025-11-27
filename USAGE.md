# Sena1996 AI Tool 🦁 - Usage Guide

## Make Your AI Collaborative and Smarter™

Complete guide to using Sena1996 AI Tool with Claude Code.

---

## Table of Contents

1. [Installation](#installation)
2. [Network Collaboration](#network-collaboration)
3. [Session Management](#session-management)
4. [Cross-Session Messaging](#cross-session-messaging)
5. [Task Management](#task-management)
6. [Domain Agents](#domain-agents)
7. [Intelligence System](#intelligence-system)
8. [Knowledge System](#knowledge-system)
9. [Evolution System](#evolution-system)
10. [Health & Metrics](#health--metrics)
11. [Hooks & MCP](#hooks--mcp)
12. [Configuration](#configuration)
13. [Troubleshooting](#troubleshooting)

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
sena --version
sena health
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

### Connection Flow
```bash
# On Machine A:
sena network start --name "Workstation"
sena peer list
# Note your peer ID

# On Machine B:
sena discover
# See Machine A in list
sena peer add 192.168.1.50 --name "Workstation"

# Back on Machine A:
sena peer authorize <machine-b-peer-id>
# Share the generated token with Machine B

# On Machine B:
sena peer connect 192.168.1.50 --token <auth-token>
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

# Start Android session
sena session start --name 'AndroidDev' --role android

# List sessions
sena session list

# End session
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
- `quick` - Fast response
- `standard` - Balanced (default)
- `deep` - Thorough analysis
- `maximum` - Most comprehensive

### Examples
```bash
# Quick thinking
sena think "How to optimize this query?"

# Deep analysis
sena think --depth deep "Should we use microservices?"

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
- **Reasoning**: First Principles, 5 Whys, Decision Matrix
- **Security**: OWASP Top 10, Auth patterns, Crypto
- **Performance**: Algorithm optimization, Caching, N+1
- **Architecture**: SOLID, Design Patterns, DDD, CQRS

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

### Config Options
```toml
[user]
name = "YourName"       # Your name
emoji = "🦁"            # Your emoji
prefix = "SENA"         # Display prefix
command = "sena"        # CLI command name

[general]
log_level = "info"      # Log level

[intelligence]
default_thinking_depth = "standard"  # quick/standard/deep/maximum
default_model = "balanced"           # fast/balanced/powerful
auto_agent_selection = true          # Auto-select agents
primary_agent = "general"            # Default agent

[evolution]
pattern_learning = true
self_optimization = true
feedback_collection = true

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

---

## Quick Reference

| Task | Command |
|------|---------|
| Check health | `sena health` |
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

## Credits

**Sena1996™** - Creator and Lead Developer
**Claude (Anthropic)** - AI Development Partner

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                   Sena1996 AI Tool 🦁                        ║
║                                                              ║
║         Make Your AI Collaborative and Smarter™             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
