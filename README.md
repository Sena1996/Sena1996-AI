# SENA Controller v5.0 - Rust Edition

**Truth-Embedded Architecture - Complete Rewrite in Rust**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://www.rust-lang.org/)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-blue)](https://modelcontextprotocol.io/)
[![Claude Code](https://img.shields.io/badge/Claude_Code-Hooks-green)](https://github.com/Sena1996/sena-controller-v4)
[![Version](https://img.shields.io/badge/version-5.0.0-brightgreen)](https://github.com/Sena1996/sena-controller-v4)
[![Tests](https://img.shields.io/badge/tests-108%20passing-success)](https://github.com/Sena1996/sena-controller-v4)

---

## What is SENA v4?

SENA v4 is a **complete rewrite** of the SENA Controller in Rust, featuring:

- **Native Performance** - 3MB binary vs ~50MB Python
- **7 Ancient Wisdom Layers** - Truth-embedded architecture
- **MCP Server** - Model Context Protocol for Claude Desktop
- **Claude Code Hooks** - Terminal behavior enhancement
- **Zero Dependencies at Runtime** - Single static binary

**One binary. Complete intelligence. Native speed.**

---

## NEW in v5.0: Rust Architecture

### Before (Python v3.x):
```
- Multiple Python files (~162KB)
- Python runtime required
- Startup time: ~500ms
- Memory: ~50MB
```

### After (Rust v5.0):
```
- Single binary (3MB)
- No runtime dependencies
- Startup time: <10ms
- Memory: ~5MB
```

---

## 7 Ancient Wisdom Layers

SENA v4 implements the complete Truth-Embedded Architecture:

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

### MCP Server (Works: Desktop + CLI)
Enterprise-grade AI tools through official MCP protocol:
- **sena_health** - System health status
- **sena_metrics** - Performance metrics
- **sena_detect_format** - Auto format detection
- **sena_validate** - Content validation
- **sena_process** - Ancient wisdom processing
- **sena_format_table** - Unicode table generation
- **sena_progress** - Progress bar display

### Claude Code Hooks (Works: CLI)
- **UserPromptSubmit** - Pre-prompt analysis & trigger detection
- **AssistantResponse** - Response validation & SENA compliance
- **ToolExecution** - Tool call validation
- **PreValidation** - Pre-processing validation
- **PostValidation** - Post-processing validation

### CLI Commands
```bash
sena                    # Interactive mode
sena mcp               # Start MCP server
sena hook <type>       # Handle Claude Code hooks
sena health            # System health
sena health --detailed # Detailed health report
sena metrics           # System metrics
sena validate <text>   # Validate content
sena process <text>    # Process through wisdom layers
sena detect <text>     # Detect format requirements
sena format <type>     # Generate formatted output
sena daemon start      # Start background daemon
sena session start     # Start new session
```

### Output Formatting
- **Unicode Tables** - Beautiful box-drawing tables
- **Progress Bars** - SENA-branded progress indicators
- **Format Boxes** - Brilliant Thinking, Truth Verification, Code Analysis

---

## Quick Installation

### From Source (Recommended)

```bash
# Clone repository
git clone https://github.com/Sena1996/sena-controller-v4.git
cd sena-controller-v4

# Build release binary
cargo build --release

# Binary location
./target/release/sena
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
    ],
    "AssistantResponse": [
      {
        "command": "/path/to/sena hook assistant-response"
      }
    ],
    "ToolExecution": [
      {
        "command": "/path/to/sena hook tool-execution"
      }
    ]
  }
}
```

---

## Usage Examples

### Interactive Mode

```bash
$ sena

╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║     SENA Controller v5.0.0 - Ancient Lion                    ║
║                                                              ║
║     Truth-Embedded Architecture in Rust                      ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝

SENA 🦁> /help
SENA 🦁> /status
SENA 🦁> /test
SENA 🦁> /layers
```

### CLI Commands

```bash
# Health check
$ sena health --detailed
╔══════════════════════════════════════════════════════════════╗
║                   SENA 🦁 HEALTH STATUS                       ║
╚══════════════════════════════════════════════════════════════╝
Version: 5.0.0
Status: Excellent
Health: 100%

# Validate content
$ sena -f pretty validate "test content"
╔══════════════════════════════════════════════════════════════╗
║                   SENA 🦁 VALIDATION RESULT                   ║
╚══════════════════════════════════════════════════════════════╝
Valid: true
Confidence: 100.0%
```

### MCP Server

```bash
# Start MCP server (stdio mode)
$ sena mcp

# With debug output
$ sena mcp --debug
```

---

## Project Structure

```
sena-controller-v4/
├── Cargo.toml              # Rust package manifest
├── Cargo.lock              # Dependency lock file
├── README.md               # This file
└── src/
    ├── lib.rs              # Library root
    ├── main.rs             # CLI binary entry point
    │
    ├── ancient/            # 7 Ancient Wisdom Layers
    │   ├── mod.rs
    │   ├── first_principles.rs
    │   ├── constraint_feature.rs
    │   ├── negative_space.rs
    │   ├── relationship_model.rs
    │   ├── self_healing.rs
    │   ├── harmony_validation.rs
    │   └── millennium_test.rs
    │
    ├── base/               # Component Registry & Interfaces
    │   ├── mod.rs
    │   ├── component.rs
    │   ├── interfaces.rs
    │   └── registry.rs
    │
    ├── cli/                # Command Line Interface
    │   ├── mod.rs
    │   ├── args.rs         # Clap argument definitions
    │   └── commands.rs     # Command execution
    │
    ├── mcp/                # Model Context Protocol Server
    │   ├── mod.rs
    │   ├── protocol.rs     # JSON-RPC types
    │   ├── handlers.rs     # Request handlers
    │   └── server.rs       # Stdio server
    │
    ├── hooks/              # Claude Code Hooks
    │   ├── mod.rs
    │   └── handler.rs      # Hook handlers
    │
    ├── output/             # Unicode Formatting
    │   ├── mod.rs
    │   ├── tables.rs       # Table builder
    │   ├── progress.rs     # Progress bars
    │   └── format_box.rs   # Format boxes
    │
    ├── integration/        # Auto Format Detection
    │   ├── mod.rs
    │   └── auto_format.rs
    │
    ├── metrics/            # Health & Metrics
    │   ├── mod.rs
    │   └── health.rs
    │
    ├── session/            # Session Management
    │   ├── mod.rs
    │   └── manager.rs
    │
    ├── sync/               # CRDT & Offline Sync
    │   ├── mod.rs
    │   ├── crdt.rs
    │   └── offline.rs
    │
    └── daemon/             # Background Daemon
        └── mod.rs
```

---

## MCP Tools Reference

### `sena_health`
Get SENA system health status.

**Parameters:**
- `detailed` (boolean): Show detailed health information

---

### `sena_metrics`
Get SENA metrics and statistics.

**Parameters:**
- `category` (string): `health`, `innovation`, `tests`, `config`, `phase`, `all`

---

### `sena_detect_format`
Detect required SENA format for text.

**Parameters:**
- `text` (string, required): Text to analyze

---

### `sena_validate`
Validate content against SENA rules.

**Parameters:**
- `content` (string, required): Content to validate
- `strict` (boolean): Use strict validation mode

---

### `sena_process`
Process request through SENA ancient wisdom layers.

**Parameters:**
- `content` (string, required): Content to process
- `request_type` (string): Type of request

---

### `sena_format_table`
Generate a formatted Unicode table.

**Parameters:**
- `headers` (array, required): Table headers
- `rows` (array, required): Table rows
- `title` (string): Optional table title

---

### `sena_progress`
Generate a progress bar display.

**Parameters:**
- `tasks` (array, required): List of tasks with name and percent

---

## Hook Triggers (Auto Detection)

| User Input | Auto-Applied Format |
|------------|---------------------|
| "why", "how", "explain" | BRILLIANT_THINKING |
| "table", "tabular" | TABLE_FORMAT |
| "is X true", "fact check", "verify" | TRUTH_VERIFICATION |
| "analyze code", "code review" | CODE_ANALYSIS |
| "progress", "status", "show tasks" | PROGRESS_BAR |

---

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_millennium_test
```

**Test Results:** 108 tests passing

---

## Performance

| Metric | Python v3.x | Rust v5.0 |
|--------|-------------|-----------|
| Binary Size | ~50MB (with runtime) | 3MB |
| Startup Time | ~500ms | <10ms |
| Memory Usage | ~50MB | ~5MB |
| Tests | 81 | 108 |
| Dependencies | 20+ Python packages | 15 Rust crates |

---

## Feature Compatibility Matrix

| Feature | Python v3.x | Rust v5.0 |
|---------|-------------|-----------|
| 7 Ancient Wisdom Layers | ✅ | ✅ |
| MCP Server | ✅ | ✅ |
| Claude Code Hooks | ✅ | ✅ |
| Auto Format Detection | ✅ | ✅ |
| Unicode Tables | ✅ | ✅ |
| Progress Bars | ✅ | ✅ |
| SENA Brilliant Thinking | ✅ | ✅ |
| Truth Verification | ✅ | ✅ |
| Code Analysis | ✅ | ✅ |
| Session Management | ✅ | ✅ |
| CRDT Sync | ✅ | ✅ |
| Background Daemon | ✅ | ✅ |
| Millennium Test | ✅ | ✅ |

---

## Dependencies

```toml
[dependencies]
serde = "1.0"           # Serialization
serde_json = "1.0"      # JSON handling
tokio = "1.0"           # Async runtime
chrono = "0.4"          # Time handling
sha2 = "0.10"           # Hashing
regex = "1.10"          # Pattern matching
uuid = "1.0"            # UUID generation
clap = "4.4"            # CLI parsing
log = "0.4"             # Logging
env_logger = "0.10"     # Logger implementation
thiserror = "1.0"       # Error handling
anyhow = "1.0"          # Error handling
indexmap = "2.0"        # Ordered maps
dirs = "5.0"            # System directories
hostname = "0.4"        # Hostname detection
```

---

## Credits

- **Creator**: SENA
- **Previous Version**: [sena-mcp-server](https://github.com/Sena1996/sena-mcp-server) (Python)
- **MCP Protocol**: [Anthropic PBC](https://www.anthropic.com/)

---

## License

MIT License - see LICENSE file for details.

---

## Version History

### v5.0.0 (2025-11-25) - **Complete Rust Rewrite**
- Complete rewrite from Python to Rust
- 7 Ancient Wisdom Layers implemented
- MCP Server with 7 tools
- Claude Code Hooks (5 hook types)
- CLI with clap derive macros
- Unicode output formatting
- Session management with CRDT sync
- Background daemon support
- 108 tests passing
- 3MB native binary

---

╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║       SENA v5.0: Truth-Embedded Architecture in Rust        ║
║                                                              ║
║       Native Performance • Ancient Wisdom • Modern Code     ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
