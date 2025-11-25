# SENA Controller v7.0 - Collaboration Hub

**Truth-Embedded Architecture with Multi-Session Collaboration**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://www.rust-lang.org/)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-blue)](https://modelcontextprotocol.io/)
[![Claude Code](https://img.shields.io/badge/Claude_Code-Hooks-green)](https://github.com/Sena1996/Sena1996-AI)
[![Version](https://img.shields.io/badge/version-7.0.0-brightgreen)](https://github.com/Sena1996/Sena1996-AI)
[![Tests](https://img.shields.io/badge/tests-135%20passing-success)](https://github.com/Sena1996/Sena1996-AI)

---

## What is SENA v7?

SENA v7 is the **Collaboration Hub** edition featuring:

- **Multi-Session Collaboration** - Android/Web/Backend/IoT Claude working together
- **Lightning-Fast IPC** - Unix socket communication (<1ms latency)
- **Task Management** - Create and assign tasks across sessions
- **7 Ancient Wisdom Layers** - Truth-embedded architecture
- **MCP Server** - Model Context Protocol for Claude Desktop
- **Claude Code Hooks** - Terminal behavior enhancement

**Multiple Claude sessions. One collaboration hub. Ancient wisdom.**

---

## NEW in v7.0: Collaboration Hub

### Multi-Session Collaboration
```bash
# Terminal 1 - Android Development
sena join android
sena task new "Fix login bug" --assign backend --priority high

# Terminal 2 - Backend Development
sena join backend
sena inbox                    # See assigned task
sena task done 1              # Complete task

# Terminal 3 - Web Development
sena join web
sena who                      # See who's online
sena tell android "API ready" # Send message
```

### Session Roles
| Role | Emoji | Description |
|------|-------|-------------|
| Android | 🤖 | Mobile/Android development |
| Web | 🌐 | Frontend/Web development |
| Backend | ⚙️ | Server/API development |
| IoT | 📡 | Embedded/Hardware development |
| General | 💻 | General purpose |

---

## 7 Ancient Wisdom Layers

| Layer | Name | Inspired By | Principle |
|-------|------|-------------|-----------|
| 0 | First Principles | Eratosthenes (240 BCE) | Understand WHY before building |
| 1 | Constraint-as-Feature | Persian Qanats (3000+ yrs) | Treat limitations as features |
| 2 | Negative Space | Sushruta (600 BCE) | Define failure before success |
| 3 | Relationship Model | Mayan Mathematics | Store connections, not just values |
| 4 | Self-Healing | Roman Concrete (2000+ yrs) | Embed repair in damage pathways |
| 5 | Harmony Validation | Antikythera (150 BCE) | Ensure model mirrors reality |
| 6 | Millennium Test | All Ancient Wisdom | Build for 1,000+ years |

---

## Features

### Collaboration Hub Commands
```bash
# Hub Management
sena hub start              # Start collaboration hub
sena hub stop               # Stop hub
sena hub status             # Hub status

# Session Management
sena join <role>            # Join as android/web/backend/iot
sena who                    # List online sessions

# Messaging
sena tell <target> <msg>    # Send direct message
sena inbox                  # Check messages

# Task Management
sena task new               # Create new task
sena task list              # List all tasks
sena task mine              # Show my tasks
sena task done <id>         # Complete task

# Monitoring
sena watch                  # Live dashboard
sena sync                   # Sync status
```

### MCP Server Tools
- **sena_health** - System health status
- **sena_metrics** - Performance metrics
- **sena_detect_format** - Auto format detection
- **sena_validate** - Content validation
- **sena_process** - Ancient wisdom processing
- **sena_format_table** - Unicode table generation
- **sena_progress** - Progress bar display

### Claude Code Hooks
- **UserPromptSubmit** - Pre-prompt analysis & trigger detection
- **AssistantResponse** - Response validation & SENA compliance
- **ToolExecution** - Tool call validation

### CLI Commands
```bash
sena                    # Interactive mode
sena mcp               # Start MCP server
sena hook <type>       # Handle Claude Code hooks
sena health            # System health
sena metrics           # System metrics
sena validate <text>   # Validate content
sena process <text>    # Process through wisdom layers
sena daemon start      # Start background daemon
```

---

## Quick Installation

### From Source

```bash
# Clone repository
git clone https://github.com/Sena1996/Sena1996-AI.git
cd Sena1996-AI

# Build release binary
cargo build --release

# Binary location
./target/release/sena --version
# sena 7.0.0
```

### Install Binary

```bash
# Copy to local bin
cp target/release/sena ~/.local/bin/

# Or system-wide
sudo cp target/release/sena /usr/local/bin/
```

---

## Configuration

### For Claude Desktop (MCP Server)

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sena": {
      "command": "/path/to/sena",
      "args": ["mcp"]
    }
  }
}
```

### For Claude Code CLI (Hooks)

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "/path/to/sena hook user-prompt-submit"
      }
    ]
  }
}
```

---

## Project Structure

```
Sena1996-AI/
├── Cargo.toml              # Rust package manifest
├── README.md               # This file
└── src/
    ├── lib.rs              # Library root
    ├── main.rs             # CLI binary entry point
    │
    ├── ancient/            # 7 Ancient Wisdom Layers
    │   ├── first_principles.rs
    │   ├── constraint_feature.rs
    │   ├── negative_space.rs
    │   ├── relationship_model.rs
    │   ├── self_healing.rs
    │   ├── harmony_validation.rs
    │   └── millennium_test.rs
    │
    ├── hub/                # Collaboration Hub (NEW in v7)
    │   ├── mod.rs          # Hub controller
    │   ├── session.rs      # Session registry & roles
    │   ├── state.rs        # CRDT state management
    │   ├── tasks.rs        # Task board
    │   ├── messages.rs     # Messaging system
    │   ├── conflicts.rs    # Conflict detection
    │   └── socket.rs       # Unix socket server
    │
    ├── base/               # Component Registry
    ├── cli/                # Command Line Interface
    ├── mcp/                # MCP Server
    ├── hooks/              # Claude Code Hooks
    ├── output/             # Unicode Formatting
    ├── integration/        # Auto Format Detection
    ├── metrics/            # Health & Metrics
    ├── session/            # Session (DEPRECATED)
    ├── sync/               # CRDT & Offline Sync
    └── daemon/             # Background Daemon
```

---

## Performance

| Metric | Value |
|--------|-------|
| Binary Size | ~3MB |
| Startup Time | <10ms |
| Memory Usage | ~5MB |
| Hub IPC Latency | <1ms |
| Tests | 135 passing |

---

## Version History

### v7.0.0 (2025-11-25) - **Collaboration Hub**
- Multi-session collaboration (Android/Web/Backend/IoT)
- Unix socket server for real-time IPC
- Task management across sessions
- Inter-session messaging
- File conflict detection
- CRDT state synchronization
- Deprecated old session module (merged into hub)
- 135 tests passing

### v6.0.0 (2025-11-25) - **Live Progress**
- Live ANSI progress bars
- Real-time terminal updates
- Enhanced output formatting

### v5.0.0 (2025-11-25) - **Rust Rewrite**
- Complete rewrite from Python to Rust
- 7 Ancient Wisdom Layers
- MCP Server & Claude Code Hooks
- 3MB native binary

---

## Credits

- **Creator**: SENA
- **MCP Protocol**: [Anthropic PBC](https://www.anthropic.com/)

---

## License

MIT License

---

╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║       SENA v7.0: Collaboration Hub                          ║
║                                                              ║
║       Multiple Sessions • One Hub • Ancient Wisdom          ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
