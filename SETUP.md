# SENA Controller v9.0 - Setup Guide 🦁

This guide covers all installation scenarios for SENA Controller with Claude Code and Claude Desktop.

---

## Quick Start

```bash
git clone https://github.com/Sena1996/Sena1996-AI.git
cd Sena1996-AI
./setup.sh
```

The interactive setup wizard will guide you through the installation.

---

## Installation Options

### Option 1: Fresh Installation (Recommended for New Users)

Best for users who:
- Are new to Claude Code/Desktop
- Want a clean, optimized setup
- Had issues with previous configurations

**What it does:**
1. Backs up your existing configuration
2. Cleans all Claude Code and Desktop data
3. Builds SENA from source
4. Installs SENA binary to `~/.local/bin/`
5. Sets up optimal Claude Code hooks
6. Configures Claude Desktop MCP server
7. Installs SENA Elite Coding Standards

**Backup location:** `~/.sena_backup_YYYYMMDD_HHMMSS/`

---

### Option 2: Merge Installation (Keep Existing + Add SENA)

Best for users who:
- Have existing Claude configurations they want to keep
- Use other MCP servers or hooks
- Have custom CLAUDE.md rules

**What it does:**
1. Backs up your existing configuration
2. Builds SENA from source
3. Installs SENA binary
4. **Merges** SENA hooks into existing `settings.json`
5. **Adds** SENA MCP server to existing Desktop config
6. Gives you choice for CLAUDE.md:
   - Keep existing rules only
   - Replace with SENA rules
   - Append SENA rules to existing

---

### Option 3: Minimal Installation (Binary Only)

Best for users who:
- Want full control over configuration
- Only need the SENA binary
- Prefer manual setup

**What it does:**
1. Builds SENA from source
2. Installs binary to `~/.local/bin/sena`
3. No configuration changes

**Manual setup required** (see below)

---

## Manual Configuration

### Claude Code Hooks

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "~/.local/bin/sena hook user-prompt-submit"
      }
    ]
  }
}
```

### Claude Desktop MCP Server

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sena": {
      "command": "/Users/YOUR_USERNAME/.local/bin/sena",
      "args": ["mcp"]
    }
  }
}
```

### SENA Elite Coding Standards

Copy `CLAUDE.md` from this repo to `~/.claude/CLAUDE.md`:

```bash
cp CLAUDE.md ~/.claude/CLAUDE.md
```

---

## Prerequisites

### Required
- **Rust** (1.70+): Install from https://rustup.rs
- **macOS** or **Linux**

### Optional
- **Claude Code CLI**: For hook integration
- **Claude Desktop**: For MCP server integration
- **Python 3**: For merge installation (JSON manipulation)

---

## Directory Structure After Installation

```
~/.local/bin/
└── sena                    # SENA binary

~/.claude/
├── settings.json           # Claude Code settings with SENA hooks
└── CLAUDE.md               # SENA Elite Coding Standards

~/.sena/
└── config.toml             # SENA configuration (created on first run)

~/Library/Application Support/Claude/
└── claude_desktop_config.json   # Claude Desktop with SENA MCP
```

---

## Verifying Installation

### Check SENA Binary
```bash
sena --version
# Expected: sena 9.0.4
```

### Check Health
```bash
sena health
```

### Test MCP Server
```bash
sena mcp
# Should start MCP server (Ctrl+C to stop)
```

### Test Hook
```bash
echo '{"prompt":"test"}' | sena hook user-prompt-submit
```

---

## Troubleshooting

### "command not found: sena"

Add to your shell profile (`~/.zshrc` or `~/.bashrc`):
```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then reload:
```bash
source ~/.zshrc
```

### Claude Desktop MCP Errors

If you see "spawn ENOENT" errors:
1. Use absolute path in config (not `~`)
2. Verify binary exists: `ls -la ~/.local/bin/sena`
3. Restart Claude Desktop completely

### Build Failures

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Restore from Backup

If something goes wrong:
```bash
# Find your backup
ls -la ~/.sena_backup_*

# Restore Claude Code config
cp -r ~/.sena_backup_XXXXXX/claude_code_config/* ~/.claude/

# Restore Claude Desktop config
cp -r ~/.sena_backup_XXXXXX/claude_desktop_config/* ~/Library/Application\ Support/Claude/
```

---

## Uninstalling

Run the setup wizard and choose option 4 (Uninstall):

```bash
./setup.sh
# Choose option 4
```

Or manually:
```bash
rm -f ~/.local/bin/sena
rm -rf ~/.sena
# Then manually edit settings.json and claude_desktop_config.json
```

---

## Configuration File

SENA stores its configuration at `~/.sena/config.toml`:

```toml
[general]
version = "9.0.4"
log_level = "info"

[knowledge]
memory_level = "project"
enable_security_patterns = true
enable_performance_patterns = true
enable_architecture_patterns = true

[intelligence]
default_thinking_depth = "standard"
enable_agents = true

[evolution]
enable_learning = true
enable_feedback = true
```

---

## What's Included

### SENA Controller Features
- **Knowledge System** - Multi-level memory, reasoning frameworks
- **Intelligence System** - Extended thinking, specialized agents
- **Evolution System** - Pattern learning, self-optimization
- **7 Ancient Wisdom Layers** - Truth-embedded architecture
- **MCP Server** - Claude Desktop integration
- **Hooks** - Claude Code integration

### Elite Coding Standards (CLAUDE.md)
- 50 coding rules (15 Critical, 20 Important, 15 Best Practice)
- Rust-specific guidelines
- SOLID principles
- Clean code practices
- Security patterns

---

## Support

- **Issues**: https://github.com/Sena1996/Sena1996-AI/issues
- **Documentation**: See README.md for full feature list

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║         SENA 🦁 v9.0.4: Truth-Embedded Architecture          ║
║                                                              ║
║         Robust • Clean Code • Battle-Tested • Ancient        ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
