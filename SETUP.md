# Sena1996 AI Tool 🦁 - Setup Guide

## Make Your AI Collaborative and Smarter™

This guide covers all installation scenarios for Sena1996 AI Tool with Claude Code and Claude Desktop.

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
8. **Generates custom slash commands** based on your chosen name

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

## Custom Command Names

During setup, you can choose a custom command name:

```
Enter custom command name (or press Enter for 'sena'): jarvis
```

This creates:
- A symlink: `~/.local/bin/jarvis -> ~/.local/bin/sena`
- Custom slash commands: `/jarvis-health`, `/jarvis-network`, etc.
- Branded output: `JARVIS 🤖 HEALTH STATUS`

### Available Custom Names Examples
- `jarvis` - Iron Man's AI
- `lucy` - Her movie AI
- `friday` - Tony Stark's second AI
- `hal` - 2001 Space Odyssey
- Or any name you prefer!

---

## Network Collaboration Setup

### Enable Network Features

After installation, start the network server:

```bash
# Start network server
sena network start --name "My Workstation"

# Check status
sena network status

# View your peer info
sena network info
```

### Connect to Other Machines

```bash
# Discover peers on local network
sena discover

# Add a peer manually
sena peer add 192.168.1.100 --name "Other Mac"

# Authorize the peer (generates token)
sena peer authorize <peer-id>
# Share this token with the other machine

# On the other machine, connect with token
sena peer connect 192.168.1.50 --token <auth-token>
```

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
  },
  "permissions": {
    "allow": [
      "Bash(sena *)",
      "Bash(./target/release/sena *)"
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
├── sena                    # SENA binary
└── jarvis                  # Symlink to sena (if custom name chosen)

~/.claude/
├── settings.json           # Claude Code settings with SENA hooks
├── CLAUDE.md               # SENA Elite Coding Standards
└── commands/
    ├── sena-health.md      # Slash commands (or jarvis-health.md)
    ├── sena-network.md
    └── ...

~/.sena/
├── config.toml             # SENA configuration
└── data/
    ├── peers.json          # Network peer registry
    ├── tokens.json         # Auth tokens
    └── tls/                # TLS certificates

~/Library/Application Support/Claude/
└── claude_desktop_config.json   # Claude Desktop with SENA MCP
```

---

## Configuration File

SENA stores its configuration at `~/.sena/config.toml`:

```toml
[user]
name = "YourName"           # Your name
emoji = "🦁"                # Your emoji
prefix = "SENA"             # Display prefix
command = "sena"            # CLI command name

[general]
log_level = "info"

[intelligence]
default_thinking_depth = "standard"
default_model = "balanced"
auto_agent_selection = true
primary_agent = "general"

[evolution]
pattern_learning = true
self_optimization = true
feedback_collection = true

[output]
color = true
unicode = true
progress_bars = true
```

---

## Verifying Installation

### Check SENA Binary
```bash
sena --version
sena health
```

### Check Network
```bash
sena network status
sena network info
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

### Network Discovery Not Working

- Ensure both machines are on the same network
- Check firewall allows port 9876
- Try manual peer add instead of discovery

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
rm -f ~/.local/bin/jarvis  # If custom command was created
rm -rf ~/.sena
# Then manually edit settings.json and claude_desktop_config.json
```

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
