#!/bin/bash

set -e

SENA_VERSION="11.0.2"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

USER_NAME=""
USER_EMOJI="🦁"
USER_PREFIX="SENA"
USER_COMMAND="sena"
THINKING_DEPTH="standard"
PRIMARY_AGENT="general"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

print_banner() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                              ║${NC}"
    echo -e "${CYAN}║${NC}       ${BOLD}SENA 🦁 Controller v${SENA_VERSION} - Setup Wizard${NC}            ${CYAN}║${NC}"
    echo -e "${CYAN}║                                                              ║${NC}"
    echo -e "${CYAN}║${NC}       Truth-Embedded Architecture • Ancient Wisdom          ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}       ${BOLD}Now with Custom Command Names!${NC}                         ${CYAN}║${NC}"
    echo -e "${CYAN}║                                                              ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

generate_slash_commands() {
    print_info "Generating customized slash commands..."

    mkdir -p "$HOME/.claude/commands"

    local CMD="$USER_COMMAND"
    local PREFIX="$USER_PREFIX"
    local EMOJI="$USER_EMOJI"

    cat > "$HOME/.claude/commands/${CMD}-health.md" << EOF
Run ${PREFIX} health check and display system status.

Execute: \`${CMD} health --detailed\`

Display the results in ${PREFIX} format with Unicode boxes.
EOF

    cat > "$HOME/.claude/commands/${CMD}-metrics.md" << EOF
Get ${PREFIX} system metrics.

Execute: \`${CMD} metrics all\`
EOF

    cat > "$HOME/.claude/commands/${CMD}-status.md" << EOF
Show ${PREFIX} session status.

Execute: \`${CMD} session info\`
EOF

    cat > "$HOME/.claude/commands/${CMD}-test.md" << EOF
Run ${PREFIX} system test.

Execute these commands in sequence:
1. \`${CMD} health\`
2. \`${CMD} metrics all\`
3. \`${CMD} session info\`

Report all results.
EOF

    cat > "$HOME/.claude/commands/${CMD}-analyze.md" << EOF
Perform deep analysis using ${PREFIX} ${EMOJI} intelligence.

Use the ${CMD} think command with extended analysis:
\`${CMD} think "\$ARGUMENTS" --depth deep\`

Provide comprehensive insights using ${PREFIX}'s multi-layered analysis.
EOF

    cat > "$HOME/.claude/commands/${CMD}-code.md" << EOF
${PREFIX} ${EMOJI} Code Analysis Mode

Analyze the provided code using ${PREFIX}'s specialized agents:

1. For backend code: \`${CMD} backend full "<code>"\`
2. For iOS code: \`${CMD} ios full "<code>"\`
3. For Android code: \`${CMD} android full "<code>"\`
4. For web code: \`${CMD} web full "<code>"\`

Select the appropriate agent based on the code type and provide detailed analysis.
EOF

    cat > "$HOME/.claude/commands/${CMD}-verify.md" << EOF
${PREFIX} ${EMOJI} Truth Verification Mode

Use ${PREFIX}'s truth-embedded verification system:

\`${CMD} validate "\$ARGUMENTS"\`

Apply rigorous verification using:
- Factual accuracy check
- Logic consistency analysis
- Source credibility assessment
- Bias detection

Report confidence level and any concerns.
EOF

    cat > "$HOME/.claude/commands/${CMD}-format-table.md" << EOF
Format data as a ${PREFIX} ${EMOJI} styled table.

Use: \`${CMD} format table --title "Title" '<json-data>'\`

The table will use Unicode box-drawing characters in ${PREFIX} style.
EOF

    cat > "$HOME/.claude/commands/${CMD}-network.md" << EOF
${PREFIX} ${EMOJI} Network Collaboration

Manage network collaboration:
- Status: \`${CMD} network status\`
- Start: \`${CMD} network start --name "${PREFIX} Instance"\`
- Info: \`${CMD} network info\`
- Discover: \`${CMD} discover\`
EOF

    cat > "$HOME/.claude/commands/${CMD}-peers.md" << EOF
${PREFIX} ${EMOJI} Peer Management

Manage network peers:
- List: \`${CMD} peer list\`
- Add: \`${CMD} peer add <ip> --name "Peer Name"\`
- Authorize: \`${CMD} peer authorize <id>\`
- Connect: \`${CMD} peer connect <ip> --token <token>\`
EOF

    cat > "$HOME/.claude/commands/session-start.md" << EOF
Start a new ${PREFIX} ${EMOJI} collaboration session.

Execute: \`${CMD} join --role "\$ARGUMENTS" --name "${PREFIX}-Claude"\`

This joins the ${PREFIX} collaboration hub for multi-session teamwork.
EOF

    cat > "$HOME/.claude/commands/session-name.md" << EOF
Set session name for ${PREFIX} ${EMOJI} collaboration.

Execute: \`${CMD} session start --name "\$ARGUMENTS"\`

This names your session in the ${PREFIX} hub.
EOF

    cat > "$HOME/.claude/commands/deep-think.md" << EOF
${PREFIX} ${EMOJI} Extended Thinking Mode

Engage deep analysis with maximum thinking depth:

\`${CMD} think "\$ARGUMENTS" --depth maximum\`

This activates ${PREFIX}'s most thorough reasoning process.
EOF

    cat > "$HOME/.claude/commands/${CMD}-always-on.md" << EOF
Enable ${PREFIX} hook for all prompts.
Note: Configure in ~/.claude/settings.json
EOF

    cat > "$HOME/.claude/commands/${CMD}-always-off.md" << EOF
Disable ${PREFIX} hook temporarily.
Note: Configure in ~/.claude/settings.json
EOF

    print_success "Generated slash commands: /${CMD}-health, /${CMD}-network, /session-start, etc."
}

detect_existing_setup() {
    print_step "Detecting Existing Claude Setup"

    CLAUDE_CODE_EXISTS=false
    CLAUDE_DESKTOP_EXISTS=false
    EXISTING_MCP=false
    EXISTING_HOOKS=false
    EXISTING_RULES=false

    if [ -d "$HOME/.claude" ]; then
        CLAUDE_CODE_EXISTS=true
        print_info "Found Claude Code config: ~/.claude"

        if [ -f "$HOME/.claude/settings.json" ]; then
            if grep -q "hooks" "$HOME/.claude/settings.json" 2>/dev/null; then
                EXISTING_HOOKS=true
                print_info "  └─ Has existing hooks configuration"
            fi
        fi

        if [ -f "$HOME/.claude/CLAUDE.md" ]; then
            EXISTING_RULES=true
            print_info "  └─ Has existing CLAUDE.md rules"
        fi
    else
        print_info "No Claude Code config found (~/.claude)"
    fi

    CLAUDE_DESKTOP_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
    if [ -f "$CLAUDE_DESKTOP_CONFIG" ]; then
        CLAUDE_DESKTOP_EXISTS=true
        print_info "Found Claude Desktop config"

        if grep -q "mcpServers" "$CLAUDE_DESKTOP_CONFIG" 2>/dev/null; then
            if ! grep -q '"mcpServers": {}' "$CLAUDE_DESKTOP_CONFIG" 2>/dev/null; then
                EXISTING_MCP=true
                print_info "  └─ Has existing MCP servers"
            fi
        fi
    else
        print_info "No Claude Desktop config found"
    fi

    echo ""
}

collect_user_preferences() {
    print_step "User Preferences"

    echo ""
    echo -e "${BOLD}Let's personalize your SENA installation${NC}"
    echo ""

    # Get user name - use system username as default
    DEFAULT_NAME="$(whoami)"
    read -p "What's your name? (default: $DEFAULT_NAME): " input_name
    if [ -n "$input_name" ]; then
        USER_NAME="$input_name"
    else
        USER_NAME="$DEFAULT_NAME"
    fi
    print_success "Hello, $USER_NAME!"

    echo ""

    # Get custom command name
    echo -e "${BOLD}Choose your command name:${NC}"
    echo "  This is what you'll type in terminal to run commands"
    echo "  Examples: sena, sagar, jarvis, hal, friday, etc."
    echo ""
    DEFAULT_COMMAND=$(echo "$USER_NAME" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]')
    read -p "Command name? (default: $DEFAULT_COMMAND): " input_command
    if [ -n "$input_command" ]; then
        USER_COMMAND=$(echo "$input_command" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]')
    else
        USER_COMMAND="$DEFAULT_COMMAND"
    fi
    print_success "Your command will be: $USER_COMMAND"
    echo "  Example: $USER_COMMAND health, $USER_COMMAND session start, $USER_COMMAND who"

    echo ""

    # Get prefix for display (defaults to uppercase of command)
    DEFAULT_PREFIX=$(echo "$USER_COMMAND" | tr '[:lower:]' '[:upper:]')
    read -p "Display prefix? (default: $DEFAULT_PREFIX): " input_prefix
    if [ -n "$input_prefix" ]; then
        USER_PREFIX="$input_prefix"
    else
        USER_PREFIX="$DEFAULT_PREFIX"
    fi
    print_success "Display prefix: $USER_PREFIX"

    echo ""

    # Get preferred emoji
    echo "Choose your assistant emoji:"
    echo "  1) 🦁 Lion (default)"
    echo "  2) 🤖 Robot"
    echo "  3) 🧠 Brain"
    echo "  4) ⚡ Lightning"
    echo "  5) 🔮 Crystal Ball"
    echo "  6) Custom"
    echo ""
    read -p "Enter choice [1-6]: " emoji_choice

    case $emoji_choice in
        1) USER_EMOJI="🦁" ;;
        2) USER_EMOJI="🤖" ;;
        3) USER_EMOJI="🧠" ;;
        4) USER_EMOJI="⚡" ;;
        5) USER_EMOJI="🔮" ;;
        6)
            read -p "Enter your custom emoji: " custom_emoji
            if [ -n "$custom_emoji" ]; then
                USER_EMOJI="$custom_emoji"
            fi
            ;;
        *) USER_EMOJI="🦁" ;;
    esac
    print_success "Emoji set to: $USER_EMOJI"

    echo ""

    # Get thinking depth preference
    echo "Default thinking depth:"
    echo "  1) Quick - Fast responses"
    echo "  2) Standard - Balanced (default)"
    echo "  3) Deep - Thorough analysis"
    echo "  4) Maximum - Comprehensive reasoning"
    echo ""
    read -p "Enter choice [1-4]: " depth_choice

    case $depth_choice in
        1) THINKING_DEPTH="quick" ;;
        2) THINKING_DEPTH="standard" ;;
        3) THINKING_DEPTH="deep" ;;
        4) THINKING_DEPTH="maximum" ;;
        *) THINKING_DEPTH="standard" ;;
    esac
    print_success "Thinking depth: $THINKING_DEPTH"

    echo ""

    # Get primary agent
    echo "Primary development focus:"
    echo "  1) General - All-purpose (default)"
    echo "  2) Backend - Server/API development"
    echo "  3) IoT - Embedded systems"
    echo "  4) iOS - Apple development"
    echo "  5) Android - Android development"
    echo "  6) Web - Frontend/Full-stack"
    echo ""
    read -p "Enter choice [1-6]: " agent_choice

    case $agent_choice in
        1) PRIMARY_AGENT="general" ;;
        2) PRIMARY_AGENT="backend" ;;
        3) PRIMARY_AGENT="iot" ;;
        4) PRIMARY_AGENT="ios" ;;
        5) PRIMARY_AGENT="android" ;;
        6) PRIMARY_AGENT="web" ;;
        *) PRIMARY_AGENT="general" ;;
    esac
    print_success "Primary agent: $PRIMARY_AGENT"

    echo ""

    # Summary
    echo -e "${BOLD}Your Configuration Summary:${NC}"
    echo "  Name:    $USER_NAME"
    echo "  Command: $USER_COMMAND"
    echo "  Prefix:  $USER_PREFIX"
    echo "  Emoji:   $USER_EMOJI"
    echo ""
    echo "  You'll use: ${BOLD}$USER_COMMAND health${NC}, ${BOLD}$USER_COMMAND session start${NC}, etc."
    echo ""
}

setup_sena_config() {
    print_step "Creating SENA Configuration"

    mkdir -p "$HOME/.sena"

    cat > "$HOME/.sena/config.toml" << EOF
# SENA Controller v${SENA_VERSION} Configuration
# Generated on $(date)

[user]
name = "$USER_NAME"
emoji = "$USER_EMOJI"
prefix = "$USER_PREFIX"
command = "$USER_COMMAND"

[general]
log_level = "info"
telemetry = true

[intelligence]
default_thinking_depth = "$THINKING_DEPTH"
default_model = "balanced"
auto_agent_selection = true
primary_agent = "$PRIMARY_AGENT"

[evolution]
pattern_learning = true
self_optimization = true
feedback_collection = true

[hub]
socket_path = "$HOME/.sena/hub.sock"
auto_start = true
timeout_seconds = 30

[output]
color = true
unicode = true
progress_bars = true
EOF

    print_success "Created ~/.sena/config.toml"

    # Create data directories
    mkdir -p "$HOME/.sena/data"
    mkdir -p "$HOME/.sena/patterns"
    mkdir -p "$HOME/.sena/sessions"

    print_success "Created SENA data directories"

    # Copy memory files
    if [ -d "$SCRIPT_DIR/memory" ]; then
        mkdir -p "$HOME/.claude/memory"
        cp "$SCRIPT_DIR/memory/"*.md "$HOME/.claude/memory/" 2>/dev/null || true
        print_success "Installed memory patterns"
    fi

    # Generate customized slash commands
    generate_slash_commands

    # Copy hook scripts (for reference)
    if [ -d "$SCRIPT_DIR/hooks" ]; then
        mkdir -p "$HOME/.sena/hooks"
        cp "$SCRIPT_DIR/hooks/"*.sh "$HOME/.sena/hooks/" 2>/dev/null || true
        chmod +x "$HOME/.sena/hooks/"*.sh 2>/dev/null || true
        print_success "Installed hook scripts"
    fi
}

show_menu() {
    print_step "Choose Installation Type"

    echo ""
    echo -e "${BOLD}Please select how you want to install SENA:${NC}"
    echo ""
    echo -e "  ${GREEN}1)${NC} ${BOLD}Fresh Installation${NC} (Recommended for new users)"
    echo -e "     Clean everything and set up SENA from scratch"
    echo -e "     • Backs up existing config first"
    echo -e "     • Removes all Claude configurations"
    echo -e "     • Installs SENA with optimal settings"
    echo -e "     • ${BOLD}Creates custom command name${NC}"
    echo ""
    echo -e "  ${YELLOW}2)${NC} ${BOLD}Merge Installation${NC} (Keep existing + add SENA)"
    echo -e "     Keep your existing setup and add SENA on top"
    echo -e "     • Preserves your current hooks and rules"
    echo -e "     • Adds SENA MCP server to existing config"
    echo -e "     • Merges SENA rules with your CLAUDE.md"
    echo -e "     • ${BOLD}Creates custom command name${NC}"
    echo ""
    echo -e "  ${BLUE}3)${NC} ${BOLD}Minimal Installation${NC} (Binary only)"
    echo -e "     Just build and install the SENA binary"
    echo -e "     • No configuration changes"
    echo -e "     • Manual setup required"
    echo -e "     • For advanced users"
    echo ""
    echo -e "  ${RED}4)${NC} ${BOLD}Uninstall SENA${NC}"
    echo -e "     Remove SENA completely from your system"
    echo ""
    echo -e "  ${NC}5)${NC} ${BOLD}Exit${NC}"
    echo ""

    read -p "Enter your choice [1-5]: " choice
    echo ""

    case $choice in
        1) fresh_installation ;;
        2) merge_installation ;;
        3) minimal_installation ;;
        4) uninstall_sena ;;
        5) echo "Goodbye! 🦁"; exit 0 ;;
        *) print_error "Invalid choice. Please try again."; show_menu ;;
    esac
}

backup_existing() {
    print_step "Creating Backup"

    BACKUP_DIR="$HOME/.sena_backup_$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$BACKUP_DIR"

    if [ -d "$HOME/.claude" ]; then
        cp -r "$HOME/.claude" "$BACKUP_DIR/claude_code_config"
        print_success "Backed up ~/.claude"
    fi

    if [ -d "$HOME/Library/Application Support/Claude" ]; then
        cp -r "$HOME/Library/Application Support/Claude" "$BACKUP_DIR/claude_desktop_config"
        print_success "Backed up Claude Desktop config"
    fi

    print_success "Backup saved to: $BACKUP_DIR"
    echo ""
}

build_sena() {
    print_step "Building SENA Binary"

    cd "$SCRIPT_DIR"

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust first:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    print_info "Building release binary..."
    cargo build --release 2>&1 | tail -5

    if [ -f "$SCRIPT_DIR/target/release/sena" ]; then
        print_success "SENA binary built successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

install_binary() {
    print_step "Installing SENA Binary"

    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"

    # Install the main binary as 'sena'
    cp "$SCRIPT_DIR/target/release/sena" "$INSTALL_DIR/sena"
    chmod +x "$INSTALL_DIR/sena"
    print_success "Installed main binary: $INSTALL_DIR/sena"

    # Create custom command symlink/copy if different from 'sena'
    if [ "$USER_COMMAND" != "sena" ] && [ -n "$USER_COMMAND" ]; then
        # Remove old symlink if exists
        rm -f "$INSTALL_DIR/$USER_COMMAND" 2>/dev/null || true

        # Create symlink to the main binary
        ln -sf "$INSTALL_DIR/sena" "$INSTALL_DIR/$USER_COMMAND"
        print_success "Created custom command: $INSTALL_DIR/$USER_COMMAND -> sena"
        print_info "You can now use: $USER_COMMAND health, $USER_COMMAND session start, etc."
    fi

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "Add to your PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "Add this to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
    fi
}

clean_claude_code() {
    print_info "Resetting Claude Code config (preserving projects data)..."

    CLAUDE_DIR="$HOME/.claude"

    mkdir -p "$CLAUDE_DIR"

    if [ -f "$CLAUDE_DIR/settings.json" ]; then
        cp "$CLAUDE_DIR/settings.json" "$CLAUDE_DIR/settings.json.backup.$(date +%Y%m%d%H%M%S)"
        print_info "Backed up existing settings.json"
    fi

    if [ -f "$CLAUDE_DIR/CLAUDE.md" ]; then
        cp "$CLAUDE_DIR/CLAUDE.md" "$CLAUDE_DIR/CLAUDE.md.backup.$(date +%Y%m%d%H%M%S)"
        print_info "Backed up existing CLAUDE.md"
    fi

    print_success "Claude Code config ready (data preserved)"
}

clean_claude_desktop() {
    print_info "Resetting Claude Desktop MCP config (preserving credentials)..."

    CLAUDE_APP_SUPPORT="$HOME/Library/Application Support/Claude"
    CLAUDE_DESKTOP_CONFIG="$CLAUDE_APP_SUPPORT/claude_desktop_config.json"

    mkdir -p "$CLAUDE_APP_SUPPORT"

    if [ -f "$CLAUDE_DESKTOP_CONFIG" ]; then
        cp "$CLAUDE_DESKTOP_CONFIG" "$CLAUDE_DESKTOP_CONFIG.backup.$(date +%Y%m%d%H%M%S)"
        print_info "Backed up existing config"
    fi

    print_success "Claude Desktop config ready (credentials preserved)"
}

setup_claude_code_config() {
    print_step "Setting Up Claude Code Configuration"

    mkdir -p "$HOME/.claude"

    SENA_PATH="$HOME/.local/bin/sena"
    CMD_PATH="$HOME/.local/bin/$USER_COMMAND"

    cat > "$HOME/.claude/settings.json" << EOF
{
  "permissions": {
    "allow": [
      "Bash(sena *)",
      "Bash($USER_COMMAND *)",
      "Bash($SENA_PATH *)",
      "Bash(./target/release/sena *)"
    ]
  },
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "$SENA_PATH hook user-prompt-submit"
      }
    ]
  }
}
EOF

    print_success "Created ~/.claude/settings.json"
    print_info "Auto-approved commands: sena, $USER_COMMAND"
}

setup_claude_desktop_config() {
    print_step "Setting Up Claude Desktop Configuration"

    CLAUDE_CONFIG_DIR="$HOME/Library/Application Support/Claude"
    mkdir -p "$CLAUDE_CONFIG_DIR"

    SENA_PATH="$HOME/.local/bin/sena"

    cat > "$CLAUDE_CONFIG_DIR/claude_desktop_config.json" << EOF
{
  "mcpServers": {
    "$USER_COMMAND": {
      "command": "$SENA_PATH",
      "args": ["mcp"]
    }
  }
}
EOF

    print_success "Created Claude Desktop config with $USER_COMMAND MCP server"
}

setup_claude_md() {
    print_step "Setting Up CLAUDE.md Rules"

    if [ -f "$SCRIPT_DIR/CLAUDE.md" ]; then
        cp "$SCRIPT_DIR/CLAUDE.md" "$HOME/.claude/CLAUDE.md"
        print_success "Installed SENA Elite Coding Standards to ~/.claude/CLAUDE.md"
    else
        print_warning "CLAUDE.md not found in repo, skipping"
    fi
}

fresh_installation() {
    print_step "Fresh Installation"
    print_info "This will clean all existing Claude configurations"

    read -p "Continue? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        show_menu
        return
    fi

    collect_user_preferences
    backup_existing
    build_sena
    install_binary
    clean_claude_code
    clean_claude_desktop
    setup_sena_config
    setup_claude_code_config
    setup_claude_desktop_config
    setup_claude_md

    print_step "Installation Complete! $USER_EMOJI"

    echo ""
    echo -e "${GREEN}SENA v${SENA_VERSION} has been installed successfully!${NC}"
    echo ""
    echo -e "Welcome, ${BOLD}$USER_NAME${NC}! Your personalized AI assistant is ready."
    echo ""
    echo "What was installed:"
    echo "  • Main binary: ~/.local/bin/sena"
    if [ "$USER_COMMAND" != "sena" ]; then
        echo "  • Your command: ~/.local/bin/$USER_COMMAND -> sena"
    fi
    echo "  • SENA config: ~/.sena/config.toml"
    echo "  • SENA data: ~/.sena/data/, patterns/, sessions/"
    echo "  • Claude Code hooks: ~/.claude/settings.json"
    echo "  • Claude Desktop MCP: ~/Library/Application Support/Claude/"
    echo "  • SENA rules: ~/.claude/CLAUDE.md"
    echo ""
    echo "Your configuration:"
    echo "  • Name:    $USER_NAME"
    echo "  • Command: $USER_COMMAND"
    echo "  • Prefix:  $USER_PREFIX"
    echo "  • Emoji:   $USER_EMOJI"
    echo "  • Depth:   $THINKING_DEPTH"
    echo "  • Agent:   $PRIMARY_AGENT"
    echo ""
    echo -e "${BOLD}Try these commands:${NC}"
    echo "  $USER_COMMAND health          # Check system health"
    echo "  $USER_COMMAND session start   # Start a session"
    echo "  $USER_COMMAND who             # Who's online"
    echo "  $USER_COMMAND think \"query\"   # Extended thinking"
    echo "  $USER_COMMAND backend map     # Backend analysis"
    echo ""
    echo "Next steps:"
    echo "  1. Restart Claude Desktop"
    echo "  2. Start a new Claude Code session"
    echo "  3. Run: $USER_COMMAND health"
    echo ""
}

merge_installation() {
    print_step "Merge Installation"
    print_info "This will add SENA to your existing setup"

    read -p "Continue? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        show_menu
        return
    fi

    collect_user_preferences
    backup_existing
    build_sena
    install_binary
    setup_sena_config

    print_step "Merging Claude Code Configuration"

    mkdir -p "$HOME/.claude"
    SENA_PATH="$HOME/.local/bin/sena"

    if [ -f "$HOME/.claude/settings.json" ]; then
        if command -v python3 &> /dev/null; then
            python3 << EOF
import json
import os

settings_path = os.path.expanduser("~/.claude/settings.json")
sena_path = "$SENA_PATH"
user_command = "$USER_COMMAND"

with open(settings_path, 'r') as f:
    settings = json.load(f)

# Add permissions
if 'permissions' not in settings:
    settings['permissions'] = {'allow': [], 'deny': []}
if 'allow' not in settings['permissions']:
    settings['permissions']['allow'] = []

new_perms = [
    "Bash(sena *)",
    f"Bash({user_command} *)",
    f"Bash({sena_path} *)",
    "Bash(./target/release/sena *)"
]
for perm in new_perms:
    if perm not in settings['permissions']['allow']:
        settings['permissions']['allow'].append(perm)

# Add hooks
if 'hooks' not in settings:
    settings['hooks'] = {}

if 'UserPromptSubmit' not in settings['hooks']:
    settings['hooks']['UserPromptSubmit'] = []

sena_hook = {"command": f"{sena_path} hook user-prompt-submit"}
if sena_hook not in settings['hooks']['UserPromptSubmit']:
    settings['hooks']['UserPromptSubmit'].append(sena_hook)

with open(settings_path, 'w') as f:
    json.dump(settings, f, indent=2)

print("Merged SENA into settings.json")
EOF
        else
            print_warning "Python3 not found, creating new settings.json"
            setup_claude_code_config
        fi
    else
        setup_claude_code_config
    fi

    print_step "Merging Claude Desktop Configuration"

    CLAUDE_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"

    if [ -f "$CLAUDE_CONFIG" ]; then
        if command -v python3 &> /dev/null; then
            python3 << EOF
import json
import os

config_path = os.path.expanduser("~/Library/Application Support/Claude/claude_desktop_config.json")
sena_path = "$SENA_PATH"
user_command = "$USER_COMMAND"

with open(config_path, 'r') as f:
    config = json.load(f)

if 'mcpServers' not in config:
    config['mcpServers'] = {}

config['mcpServers'][user_command] = {
    "command": sena_path,
    "args": ["mcp"]
}

with open(config_path, 'w') as f:
    json.dump(config, f, indent=2)

print(f"Added {user_command} MCP server to config")
EOF
        else
            print_warning "Python3 not found, creating new config"
            setup_claude_desktop_config
        fi
    else
        setup_claude_desktop_config
    fi

    print_step "Merging CLAUDE.md Rules"

    if [ -f "$HOME/.claude/CLAUDE.md" ]; then
        print_info "You have existing CLAUDE.md rules"
        echo ""
        echo "Options:"
        echo "  1) Keep existing rules only"
        echo "  2) Replace with SENA rules"
        echo "  3) Append SENA rules to existing"
        echo ""
        read -p "Choose [1-3]: " md_choice

        case $md_choice in
            1) print_info "Keeping existing CLAUDE.md" ;;
            2) setup_claude_md ;;
            3)
                if [ -f "$SCRIPT_DIR/CLAUDE.md" ]; then
                    echo "" >> "$HOME/.claude/CLAUDE.md"
                    echo "---" >> "$HOME/.claude/CLAUDE.md"
                    echo "" >> "$HOME/.claude/CLAUDE.md"
                    cat "$SCRIPT_DIR/CLAUDE.md" >> "$HOME/.claude/CLAUDE.md"
                    print_success "Appended SENA rules to existing CLAUDE.md"
                fi
                ;;
        esac
    else
        setup_claude_md
    fi

    print_step "Merge Installation Complete! $USER_EMOJI"

    echo ""
    echo -e "${GREEN}SENA v${SENA_VERSION} has been merged with your existing setup!${NC}"
    echo ""
    echo -e "Welcome, ${BOLD}$USER_NAME${NC}! Your personalized AI assistant is ready."
    echo ""
    echo "Your configuration:"
    echo "  • Name:    $USER_NAME"
    echo "  • Command: $USER_COMMAND"
    echo "  • Prefix:  $USER_PREFIX"
    echo "  • Emoji:   $USER_EMOJI"
    echo ""
    echo -e "${BOLD}Try these commands:${NC}"
    echo "  $USER_COMMAND health"
    echo "  $USER_COMMAND session start"
    echo "  $USER_COMMAND who"
    echo ""
    echo "Next steps:"
    echo "  1. Restart Claude Desktop"
    echo "  2. Start a new Claude Code session"
    echo "  3. Run: $USER_COMMAND health"
    echo ""
}

minimal_installation() {
    print_step "Minimal Installation"
    print_info "This will only build and install the SENA binary"

    read -p "Continue? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        show_menu
        return
    fi

    # Still ask for command name
    echo ""
    DEFAULT_COMMAND="sena"
    read -p "Command name? (default: $DEFAULT_COMMAND): " input_command
    if [ -n "$input_command" ]; then
        USER_COMMAND=$(echo "$input_command" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]')
    else
        USER_COMMAND="$DEFAULT_COMMAND"
    fi

    build_sena
    install_binary

    print_step "Minimal Installation Complete! 🦁"

    echo ""
    echo -e "${GREEN}SENA binary installed${NC}"
    echo ""
    echo "Installed:"
    echo "  • ~/.local/bin/sena"
    if [ "$USER_COMMAND" != "sena" ]; then
        echo "  • ~/.local/bin/$USER_COMMAND -> sena"
    fi
    echo ""
    echo "Manual setup required:"
    echo ""
    echo "1. Create ~/.sena/config.toml:"
    echo "   [user]"
    echo "   name = \"YourName\""
    echo "   emoji = \"🦁\""
    echo "   prefix = \"SENA\""
    echo "   command = \"$USER_COMMAND\""
    echo ""
    echo "2. For Claude Code hooks, add to ~/.claude/settings.json:"
    echo '   {'
    echo '     "hooks": {'
    echo '       "UserPromptSubmit": ['
    echo "         {\"command\": \"~/.local/bin/sena hook user-prompt-submit\"}"
    echo '       ]'
    echo '     }'
    echo '   }'
    echo ""
    echo "3. For Claude Desktop MCP, add to claude_desktop_config.json:"
    echo '   {'
    echo '     "mcpServers": {'
    echo "       \"$USER_COMMAND\": {\"command\": \"~/.local/bin/sena\", \"args\": [\"mcp\"]}"
    echo '     }'
    echo '   }'
    echo ""
}

uninstall_sena() {
    print_step "Uninstall SENA"
    print_warning "This will remove SENA from your system"

    read -p "Continue? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        show_menu
        return
    fi

    # Remove main binary
    if [ -f "$HOME/.local/bin/sena" ]; then
        rm -f "$HOME/.local/bin/sena"
        print_success "Removed sena binary"
    fi

    # Remove any symlinks pointing to sena
    for file in "$HOME/.local/bin/"*; do
        if [ -L "$file" ]; then
            target=$(readlink "$file")
            if [[ "$target" == *"sena"* ]]; then
                rm -f "$file"
                print_success "Removed symlink: $(basename "$file")"
            fi
        fi
    done

    # Remove config
    if [ -d "$HOME/.sena" ]; then
        rm -rf "$HOME/.sena"
        print_success "Removed SENA config directory"
    fi

    print_info "Note: Claude configurations were not modified."
    print_info "You may want to manually remove SENA from:"
    echo "  • ~/.claude/settings.json (hooks & permissions)"
    echo "  • Claude Desktop config (MCP servers)"
    echo "  • ~/.claude/CLAUDE.md (rules)"

    print_step "Uninstall Complete"
}

print_banner
detect_existing_setup
show_menu
