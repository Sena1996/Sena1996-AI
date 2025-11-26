# SENA Controller v11.0.0 🦁 - User Customization Edition

**Truth-Embedded Architecture with Personalized Branding & Session Management**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://www.rust-lang.org/)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-blue)](https://modelcontextprotocol.io/)
[![Claude Code](https://img.shields.io/badge/Claude_Code-Hooks-green)](https://github.com/Sena1996/Sena1996-AI)
[![Version](https://img.shields.io/badge/version-11.0.0-brightgreen)](https://github.com/Sena1996/Sena1996-AI)
[![Tests](https://img.shields.io/badge/tests-213%20passing-success)](https://github.com/Sena1996/Sena1996-AI)
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-success)](https://github.com/Sena1996/Sena1996-AI)

---

## What is SENA v11?

SENA v11 is the **User Customization Edition** - personalize your AI assistant:

- **User Branding** - Configure name, emoji, prefix in `~/.sena/config.toml`
- **Session Management** - Fixed 24-hour session persistence (was 60s)
- **Domain Agents** - Backend, IoT, iOS, Android, Web specialized analysis
- **Zero Clippy Warnings** - Clean code that passes `cargo clippy -- -D warnings`
- **Proper Error Handling** - No silent `.ok()` calls, proper `?` propagation
- **Knowledge System** - 47 patterns across reasoning, security, performance, architecture
- **Intelligence System** - Extended thinking with Quick/Standard/Deep/Maximum
- **Evolution System** - Pattern learning and self-optimization
- **7 Ancient Wisdom Layers** - Truth-embedded architecture
- **Collaboration Hub** - Multi-session collaboration
- **MCP Server & Hooks** - Claude Desktop/Code integration

**Personalized. Clean. Production Ready.**

---

## NEW in v11.0.0: User Customization

### Personalized Branding
Configure your assistant's identity in `~/.sena/config.toml`:

```toml
[user]
name = "YourName"    # Your name or brand
emoji = "🦁"          # Your chosen emoji
prefix = "SENA"       # Prefix for all outputs
```

All outputs now use your configured branding:
```
╔══════════════════════════════════════════════════════════════╗
║                     SENA 🦁 HEALTH STATUS                     ║
╚══════════════════════════════════════════════════════════════╝
```

### Session Management Fixes
- **24-hour stale timeout** - Sessions no longer expire after 60 seconds
- **Consistent `who` and `session list`** - Both now show same sessions
- **Proper error handling** - No more silent failures with `.ok()`

### Domain-Specific Agents
```bash
sena backend map "GET /api/users"           # API endpoint analysis
sena backend security "SELECT * FROM users" # SQL injection detection
sena iot protocol "mqtt.connect(...)"       # Protocol analysis
sena ios ui "struct ContentView: View"      # SwiftUI HIG compliance
sena android lifecycle "AppCompatActivity"  # Lifecycle analysis
sena web audit "<script>alert()</script>"   # XSS detection
```

### Code Quality
- Zero clippy warnings (`cargo clippy -- -D warnings`)
- 213 tests passing
- All `from_str` methods renamed to `parse` for clarity
- Collapsible if statements fixed
- Useless format! calls removed
- Proper error propagation throughout

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

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/Sena1996/Sena1996-AI.git
cd Sena1996-AI

# Build release
cargo build --release

# Copy to PATH
cp target/release/sena ~/.local/bin/

# Verify
sena --version
# sena 11.0.0
```

### Configuration

Create `~/.sena/config.toml`:
```toml
[user]
name = "YourName"
emoji = "🦁"
prefix = "SENA"

[general]
log_level = "info"
```

---

## CLI Commands

### Health & Metrics
```bash
sena health                    # Quick health check
sena health --detailed         # Detailed health report
sena metrics                   # Full system metrics
```

### Session Management
```bash
sena session --name 'MySession' start   # Start named session
sena session list                       # List active sessions
sena session info                       # Current session info
sena who                                # Who's online
sena session --id <id> end              # End session
```

### Collaboration Hub
```bash
sena hub status                         # Hub status
sena join --role backend --name 'API'   # Join as role
sena task new 'Fix bug' --to <id>       # Create task
sena task list                          # List tasks
sena task done <id>                     # Complete task
sena tell <id> 'message'                # Send message
sena inbox                              # Check inbox
```

### Domain Agents
```bash
sena backend map <code>         # API mapping
sena backend flow <code>        # Data flow analysis
sena backend security <code>    # Security scan
sena iot protocol <code>        # Protocol analysis
sena iot power <code>           # Power optimization
sena ios ui <code>              # SwiftUI HIG check
sena ios perf <code>            # Performance analysis
sena android lifecycle <code>   # Lifecycle check
sena android compat <code>      # Compatibility check
sena web audit <code>           # Security audit
sena web a11y <code>            # Accessibility check
```

### Knowledge System
```bash
sena knowledge search "pattern"     # Search knowledge base
sena knowledge list reasoning       # List reasoning frameworks
sena knowledge list security        # List security patterns
sena knowledge stats               # Knowledge statistics
```

### Intelligence System
```bash
sena think "How to optimize?"       # Quick analysis
sena think --depth deep "query"     # Deep analysis
sena agent security <code>          # Security agent
sena agent performance <code>       # Performance agent
```

### Evolution System
```bash
sena evolve stats                   # Evolution statistics
sena feedback positive "Great!"     # Submit feedback
sena feedback bug "Issue found"     # Report bug
```

### Daemon & MCP
```bash
sena daemon status             # Daemon status
sena mcp                       # Start MCP server
```

---

## Claude Integration

### Claude Desktop (MCP Server)

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

### Claude Code (Hooks)

Add to `~/.claude/settings.json`:
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "/path/to/sena hook user-prompt-submit"
      }
    ]
  },
  "permissions": {
    "allow": [
      "Bash(sena:*)",
      "Bash(./target/release/sena:*)"
    ]
  }
}
```

---

## Performance

| Metric | Value |
|--------|-------|
| Binary Size | ~3.5MB |
| Startup Time | <10ms |
| Memory Usage | ~5MB |
| Tests | 213 passing |
| Clippy Warnings | 0 |

---

## Version History

### v11.0.0 (2025-11-26) - **User Customization Edition**
- User branding configuration (`~/.sena/config.toml`)
- Configurable name, emoji, prefix for all outputs
- Session stale timeout increased from 60s to 24 hours
- Fixed `sena who` and `sena session list` consistency
- Proper error handling (removed all `.ok()` silent failures)
- Zero clippy warnings
- Renamed `from_str` methods to `parse` for clarity
- Fixed collapsible if statements
- Removed useless format! calls
- 213 tests passing

### v10.0.6 (2025-11-26)
- System username as default when user skips name input
- Session naming support
- Auto-approve SENA commands (no bash prompts)
- Full hooks configuration

### v9.0.4 (2025-11-26)
- Zero unwrap() in production code
- Lazy static regex initialization
- 191 tests passing

### v9.0.0-v9.0.3
- Production ready with robust error handling
- Configuration file support
- Elite coding standards (CLAUDE.md)

### v8.x
- Knowledge, Intelligence, Evolution systems
- 47 knowledge patterns

### v7.x
- Collaboration Hub with multi-session support

### v5.x-v6.x
- Rust rewrite from Python
- Live progress bars

---

## Credits

- **Creator**: SENA
- **MCP Protocol**: [Anthropic PBC](https://www.anthropic.com/)

---

## License

MIT License

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║       SENA 🦁 v11.0.0: User Customization Edition           ║
║                                                              ║
║     Personalized • Clean Code • 213 Tests • Zero Warnings   ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
