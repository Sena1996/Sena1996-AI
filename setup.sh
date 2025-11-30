#!/bin/bash

set -e

SENA_VERSION="13.0.2"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

INSTALL_DIR="$HOME/.local/bin"
SENA_HOME="$HOME/.sena"
CLAUDE_HOME="$HOME/.claude"
CLAUDE_DESKTOP_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"

USER_NAME=""
USER_EMOJI="🦁"
USER_PREFIX="SENA"
USER_COMMAND="sena"
THINKING_DEPTH="standard"
PRIMARY_AGENT="general"

INSTALLED_VERSION=""
INSTALLATION_MODE=""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'

print_banner() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                                  ║${NC}"
    echo -e "${CYAN}║${NC}    ${BOLD}Sena1996 AI Tool${NC} 🦁 - Make Your AI Collaborative & Smarter    ${CYAN}║${NC}"
    echo -e "${CYAN}║                                                                  ║${NC}"
    echo -e "${CYAN}║${NC}    ${DIM}Professional Installation Manager${NC}                              ${CYAN}║${NC}"
    echo -e "${CYAN}║                                                                  ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_step() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }
print_info() { echo -e "${CYAN}ℹ${NC} $1"; }
print_detail() { echo -e "  ${DIM}└─${NC} $1"; }

detect_installed_version() {
    if [ -f "$INSTALL_DIR/sena" ]; then
        INSTALLED_VERSION=$("$INSTALL_DIR/sena" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "")
    fi
}

compare_versions() {
    local current="$1"
    local new="$2"

    if [ -z "$current" ]; then
        echo "new"
        return
    fi

    local current_major=$(echo "$current" | cut -d. -f1)
    local current_minor=$(echo "$current" | cut -d. -f2)
    local current_patch=$(echo "$current" | cut -d. -f3)

    local new_major=$(echo "$new" | cut -d. -f1)
    local new_minor=$(echo "$new" | cut -d. -f2)
    local new_patch=$(echo "$new" | cut -d. -f3)

    if [ "$new_major" -gt "$current_major" ]; then
        echo "upgrade"
    elif [ "$new_major" -lt "$current_major" ]; then
        echo "downgrade"
    elif [ "$new_minor" -gt "$current_minor" ]; then
        echo "upgrade"
    elif [ "$new_minor" -lt "$current_minor" ]; then
        echo "downgrade"
    elif [ "$new_patch" -gt "$current_patch" ]; then
        echo "upgrade"
    elif [ "$new_patch" -lt "$current_patch" ]; then
        echo "downgrade"
    else
        echo "same"
    fi
}

check_system_requirements() {
    print_step "System Requirements Check"

    local requirements_met=true

    if command -v cargo &> /dev/null; then
        local rust_version=$(rustc --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
        print_success "Rust/Cargo installed (v$rust_version)"
    else
        print_error "Rust/Cargo not found"
        print_detail "Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        requirements_met=false
    fi

    if command -v git &> /dev/null; then
        print_success "Git installed"
    else
        print_warning "Git not found (optional, for updates)"
    fi

    if command -v python3 &> /dev/null; then
        print_success "Python3 installed (for config merging)"
    else
        print_warning "Python3 not found (config merging will be limited)"
    fi

    if [ -w "$HOME" ]; then
        print_success "Home directory writable"
    else
        print_error "Cannot write to home directory"
        requirements_met=false
    fi

    local free_space=$(df -m "$HOME" 2>/dev/null | tail -1 | awk '{print $4}')
    if [ -n "$free_space" ] && [ "$free_space" -gt 100 ]; then
        print_success "Disk space available (${free_space}MB free)"
    else
        print_warning "Low disk space"
    fi

    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        print_success "~/.local/bin in PATH"
    else
        print_warning "~/.local/bin NOT in PATH"
        print_detail "Will need to add: export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi

    if [ "$requirements_met" = false ]; then
        echo ""
        print_error "System requirements not met. Please fix the issues above."
        exit 1
    fi

    echo ""
}

detect_existing_installation() {
    print_step "Detecting Existing Installation"

    local found_installation=false

    detect_installed_version
    if [ -n "$INSTALLED_VERSION" ]; then
        found_installation=true
        local version_status=$(compare_versions "$INSTALLED_VERSION" "$SENA_VERSION")

        case $version_status in
            "same")
                print_info "SENA v$INSTALLED_VERSION installed (same as repo)"
                ;;
            "upgrade")
                print_info "SENA v$INSTALLED_VERSION installed (upgrade available: v$SENA_VERSION)"
                ;;
            "downgrade")
                print_warning "SENA v$INSTALLED_VERSION installed (newer than repo v$SENA_VERSION)"
                ;;
        esac
        print_detail "Binary: $INSTALL_DIR/sena"
    else
        print_info "No SENA binary found"
    fi

    if [ -d "$SENA_HOME" ]; then
        found_installation=true
        print_info "SENA config directory found"
        print_detail "Location: $SENA_HOME"

        if [ -f "$SENA_HOME/config.toml" ]; then
            local config_user=$(grep -E "^name\s*=" "$SENA_HOME/config.toml" 2>/dev/null | cut -d'"' -f2 || echo "")
            local config_cmd=$(grep -E "^command\s*=" "$SENA_HOME/config.toml" 2>/dev/null | cut -d'"' -f2 || echo "")
            if [ -n "$config_user" ]; then
                print_detail "Configured user: $config_user"
            fi
            if [ -n "$config_cmd" ]; then
                print_detail "Custom command: $config_cmd"
                USER_COMMAND="$config_cmd"
            fi
        fi
    fi

    for file in "$INSTALL_DIR"/*; do
        if [ -L "$file" ]; then
            local target=$(readlink "$file" 2>/dev/null || echo "")
            if [[ "$target" == *"sena"* ]]; then
                print_info "Found custom command symlink: $(basename "$file")"
                print_detail "Links to: $target"
            fi
        fi
    done

    if [ -d "$CLAUDE_HOME" ]; then
        print_info "Claude Code config found"

        if [ -f "$CLAUDE_HOME/settings.json" ]; then
            if grep -q "sena" "$CLAUDE_HOME/settings.json" 2>/dev/null; then
                print_detail "SENA hooks configured"
            fi
            if grep -q '"allow"' "$CLAUDE_HOME/settings.json" 2>/dev/null; then
                print_detail "Permissions configured"
            fi
        fi

        if [ -f "$CLAUDE_HOME/CLAUDE.md" ]; then
            if grep -q "SENA1996" "$CLAUDE_HOME/CLAUDE.md" 2>/dev/null; then
                print_detail "SENA Elite Standards installed"
            fi
        fi

        if [ -d "$CLAUDE_HOME/commands" ]; then
            local sena_commands=$(ls "$CLAUDE_HOME/commands/" 2>/dev/null | grep -E "^sena-|^session-|^deep-think" | wc -l | tr -d ' ')
            if [ "$sena_commands" -gt 0 ]; then
                print_detail "SENA slash commands: $sena_commands found"
            fi
        fi
    fi

    if [ -f "$CLAUDE_DESKTOP_CONFIG" ]; then
        print_info "Claude Desktop config found"
        if grep -q "sena" "$CLAUDE_DESKTOP_CONFIG" 2>/dev/null; then
            print_detail "SENA MCP server configured"
        fi
    fi

    if [ "$found_installation" = true ]; then
        echo ""
        print_info "Existing installation detected"
    else
        echo ""
        print_info "No existing installation found"
    fi
}

show_main_menu() {
    print_step "Installation Options"

    echo ""
    detect_installed_version

    if [ -n "$INSTALLED_VERSION" ]; then
        local version_status=$(compare_versions "$INSTALLED_VERSION" "$SENA_VERSION")
        echo -e "  ${DIM}Current: v$INSTALLED_VERSION | Available: v$SENA_VERSION${NC}"
        echo ""
    fi

    echo -e "  ${GREEN}1)${NC} ${BOLD}Install${NC}"
    echo -e "     Fresh installation or reinstall SENA"
    echo ""
    echo -e "  ${BLUE}2)${NC} ${BOLD}Upgrade${NC}"
    echo -e "     Update binary while preserving configuration"
    echo ""
    echo -e "  ${YELLOW}3)${NC} ${BOLD}Repair${NC}"
    echo -e "     Fix broken installation, restore missing files"
    echo ""
    echo -e "  ${MAGENTA}4)${NC} ${BOLD}Configure${NC}"
    echo -e "     Change settings without reinstalling"
    echo ""
    echo -e "  ${RED}5)${NC} ${BOLD}Uninstall${NC}"
    echo -e "     Completely remove SENA from system"
    echo ""
    echo -e "  ${DIM}6)${NC} ${BOLD}Diagnostics${NC}"
    echo -e "     Check installation health and troubleshoot"
    echo ""
    echo -e "  ${NC}0)${NC} Exit"
    echo ""

    read -p "Select option [0-6]: " choice
    echo ""

    case $choice in
        1) install_menu ;;
        2) upgrade_sena ;;
        3) repair_sena ;;
        4) configure_menu ;;
        5) uninstall_menu ;;
        6) run_diagnostics ;;
        0) echo "Goodbye! 🦁"; exit 0 ;;
        *) print_error "Invalid choice"; show_main_menu ;;
    esac
}

install_menu() {
    print_step "Installation Type"

    echo ""
    echo -e "  ${GREEN}1)${NC} ${BOLD}Standard Installation${NC} (Recommended)"
    echo -e "     Full setup with personalization"
    echo ""
    echo -e "  ${YELLOW}2)${NC} ${BOLD}Merge Installation${NC}"
    echo -e "     Add SENA to existing Claude setup"
    echo ""
    echo -e "  ${BLUE}3)${NC} ${BOLD}Minimal Installation${NC}"
    echo -e "     Binary only, manual configuration"
    echo ""
    echo -e "  ${NC}0)${NC} Back to main menu"
    echo ""

    read -p "Select option [0-3]: " choice
    echo ""

    case $choice in
        1) standard_installation ;;
        2) merge_installation ;;
        3) minimal_installation ;;
        0) show_main_menu ;;
        *) print_error "Invalid choice"; install_menu ;;
    esac
}

collect_user_preferences() {
    print_step "Personalization"

    echo ""
    echo -e "${BOLD}Let's personalize your SENA installation${NC}"
    echo ""

    DEFAULT_NAME="$(whoami)"
    read -p "Your name [$DEFAULT_NAME]: " input_name
    USER_NAME="${input_name:-$DEFAULT_NAME}"
    print_success "Name: $USER_NAME"

    echo ""
    DEFAULT_COMMAND=$(echo "$USER_NAME" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]')
    echo "Choose your command name (what you'll type in terminal)"
    echo -e "  ${DIM}Examples: sena, jarvis, hal, friday${NC}"
    read -p "Command [$DEFAULT_COMMAND]: " input_command
    if [ -n "$input_command" ]; then
        USER_COMMAND=$(echo "$input_command" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]')
    else
        USER_COMMAND="$DEFAULT_COMMAND"
    fi
    print_success "Command: $USER_COMMAND"

    echo ""
    DEFAULT_PREFIX=$(echo "$USER_COMMAND" | tr '[:lower:]' '[:upper:]')
    read -p "Display prefix [$DEFAULT_PREFIX]: " input_prefix
    USER_PREFIX="${input_prefix:-$DEFAULT_PREFIX}"
    print_success "Prefix: $USER_PREFIX"

    echo ""
    echo "Choose your emoji:"
    echo "  1) 🦁 Lion    2) 🤖 Robot    3) 🧠 Brain"
    echo "  4) ⚡ Lightning    5) 🔮 Crystal    6) Custom"
    read -p "Choice [1]: " emoji_choice

    case $emoji_choice in
        1|"") USER_EMOJI="🦁" ;;
        2) USER_EMOJI="🤖" ;;
        3) USER_EMOJI="🧠" ;;
        4) USER_EMOJI="⚡" ;;
        5) USER_EMOJI="🔮" ;;
        6) read -p "Enter emoji: " USER_EMOJI ;;
    esac
    print_success "Emoji: $USER_EMOJI"

    echo ""
    echo "Default thinking depth:"
    echo "  1) Quick    2) Standard    3) Deep    4) Maximum"
    read -p "Choice [2]: " depth_choice

    case $depth_choice in
        1) THINKING_DEPTH="quick" ;;
        2|"") THINKING_DEPTH="standard" ;;
        3) THINKING_DEPTH="deep" ;;
        4) THINKING_DEPTH="maximum" ;;
    esac
    print_success "Thinking depth: $THINKING_DEPTH"

    echo ""
    echo "Primary development focus:"
    echo "  1) General    2) Backend    3) IoT"
    echo "  4) iOS    5) Android    6) Web"
    read -p "Choice [1]: " agent_choice

    case $agent_choice in
        1|"") PRIMARY_AGENT="general" ;;
        2) PRIMARY_AGENT="backend" ;;
        3) PRIMARY_AGENT="iot" ;;
        4) PRIMARY_AGENT="ios" ;;
        5) PRIMARY_AGENT="android" ;;
        6) PRIMARY_AGENT="web" ;;
    esac
    print_success "Primary agent: $PRIMARY_AGENT"

    echo ""
    echo -e "${BOLD}Configuration Summary:${NC}"
    echo "  Name:    $USER_NAME"
    echo "  Command: $USER_COMMAND"
    echo "  Prefix:  $USER_PREFIX"
    echo "  Emoji:   $USER_EMOJI"
    echo "  Depth:   $THINKING_DEPTH"
    echo "  Focus:   $PRIMARY_AGENT"
    echo ""
}

create_backup() {
    local backup_type="$1"
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_dir="$SENA_HOME/backups/${backup_type}_$timestamp"

    mkdir -p "$backup_dir"

    if [ -f "$INSTALL_DIR/sena" ]; then
        cp "$INSTALL_DIR/sena" "$backup_dir/" 2>/dev/null || true
    fi

    if [ -f "$SENA_HOME/config.toml" ]; then
        cp "$SENA_HOME/config.toml" "$backup_dir/" 2>/dev/null || true
    fi

    if [ -f "$CLAUDE_HOME/settings.json" ]; then
        cp "$CLAUDE_HOME/settings.json" "$backup_dir/" 2>/dev/null || true
    fi

    if [ -f "$CLAUDE_HOME/CLAUDE.md" ]; then
        cp "$CLAUDE_HOME/CLAUDE.md" "$backup_dir/" 2>/dev/null || true
    fi

    echo "$backup_dir"
}

rotate_backups() {
    local backup_base="$SENA_HOME/backups"
    local max_backups=5

    if [ -d "$backup_base" ]; then
        local backup_count=$(ls -d "$backup_base"/*/ 2>/dev/null | wc -l | tr -d ' ')

        if [ "$backup_count" -gt "$max_backups" ]; then
            local to_remove=$((backup_count - max_backups))
            ls -dt "$backup_base"/*/ 2>/dev/null | tail -n "$to_remove" | xargs rm -rf 2>/dev/null || true
            print_detail "Cleaned up $to_remove old backup(s)"
        fi
    fi
}

build_binary() {
    print_step "Building SENA Binary"

    cd "$SCRIPT_DIR"

    print_info "Compiling release build..."
    if cargo build --release 2>&1 | tail -3; then
        if [ -f "$SCRIPT_DIR/target/release/sena" ]; then
            print_success "Build successful"
            return 0
        fi
    fi

    print_error "Build failed"
    return 1
}

install_binary() {
    print_step "Installing Binary"

    mkdir -p "$INSTALL_DIR"

    if [ -f "$INSTALL_DIR/sena" ]; then
        print_info "Removing existing binary..."
        rm -f "$INSTALL_DIR/sena"
    fi

    for file in "$INSTALL_DIR"/*; do
        if [ -L "$file" ]; then
            local target=$(readlink "$file" 2>/dev/null || echo "")
            if [[ "$target" == *"sena"* ]]; then
                print_info "Removing old symlink: $(basename "$file")"
                rm -f "$file"
            fi
        fi
    done

    cp "$SCRIPT_DIR/target/release/sena" "$INSTALL_DIR/sena"
    chmod +x "$INSTALL_DIR/sena"
    print_success "Installed: $INSTALL_DIR/sena"

    if [ -x "$INSTALL_DIR/sena" ]; then
        local version=$("$INSTALL_DIR/sena" --version 2>/dev/null || echo "unknown")
        print_success "Verified: $version"
    else
        print_error "Binary installation failed!"
        return 1
    fi

    if [ "$USER_COMMAND" != "sena" ] && [ -n "$USER_COMMAND" ]; then
        ln -sf "$INSTALL_DIR/sena" "$INSTALL_DIR/$USER_COMMAND"
        print_success "Created alias: $USER_COMMAND -> sena"
    fi

    install_sena_latest_wrapper

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        setup_shell_path
    fi
}

install_sena_latest_wrapper() {
    print_info "Creating sena-latest wrapper (auto-selects newest version)..."

    cat > "$INSTALL_DIR/sena-latest" << 'WRAPPER_EOF'
#!/bin/bash
SENA_DEV_PATHS=(
    "$HOME/AI/Sena1996-AI/target/release/sena"
    "$HOME/Projects/Sena1996-AI/target/release/sena"
    "$HOME/Code/Sena1996-AI/target/release/sena"
)
SENA_INSTALLED="$HOME/.local/bin/sena"

for dev_path in "${SENA_DEV_PATHS[@]}"; do
    expanded_path="${dev_path/#\$HOME/$HOME}"
    if [[ -x "$expanded_path" ]]; then
        exec "$expanded_path" "$@"
    fi
done

if [[ -x "$SENA_INSTALLED" ]]; then
    exec "$SENA_INSTALLED" "$@"
fi

echo "Error: sena binary not found" >&2
exit 1
WRAPPER_EOF

    chmod +x "$INSTALL_DIR/sena-latest"
    print_success "Created: $INSTALL_DIR/sena-latest"
    print_detail "This wrapper always uses the latest built version"
}

setup_shell_path() {
    print_step "PATH Configuration"

    local shell_config=""
    local current_shell=$(basename "$SHELL")

    case $current_shell in
        zsh)  shell_config="$HOME/.zshrc" ;;
        bash) shell_config="$HOME/.bashrc" ;;
        *)    shell_config="$HOME/.profile" ;;
    esac

    if [ -f "$shell_config" ]; then
        if grep -q 'HOME/.local/bin' "$shell_config" 2>/dev/null; then
            print_success "PATH already configured in $shell_config"
            return 0
        fi
    fi

    echo ""
    echo "~/.local/bin is not in your PATH."
    echo ""
    echo "Options:"
    echo "  1) Add to $shell_config automatically"
    echo "  2) Show me the command to add manually"
    echo "  3) Skip (I'll handle it myself)"
    echo ""
    read -p "Choice [1]: " path_choice

    case $path_choice in
        1|"")
            echo '' >> "$shell_config"
            echo '# SENA - Added by installer' >> "$shell_config"
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_config"
            print_success "Added PATH to $shell_config"
            print_warning "Run: source $shell_config  OR  restart your terminal"
            ;;
        2)
            echo ""
            echo "Add this line to your shell config ($shell_config):"
            echo ""
            echo '  export PATH="$HOME/.local/bin:$PATH"'
            echo ""
            ;;
        3)
            print_info "Skipped PATH configuration"
            ;;
    esac
}

setup_sena_config() {
    print_step "Creating SENA Configuration"

    mkdir -p "$SENA_HOME"
    mkdir -p "$SENA_HOME/data"
    mkdir -p "$SENA_HOME/patterns"
    mkdir -p "$SENA_HOME/sessions"
    mkdir -p "$SENA_HOME/backups"

    cat > "$SENA_HOME/config.toml" << EOF
# Sena1996 AI Tool Configuration
# Generated: $(date)
# Version: $SENA_VERSION

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
socket_path = "$SENA_HOME/hub.sock"
auto_start = true
timeout_seconds = 30

[output]
color = true
unicode = true
progress_bars = true

[installation]
installed_version = "$SENA_VERSION"
install_date = "$(date -Iseconds)"
install_path = "$INSTALL_DIR/sena"
EOF

    print_success "Created $SENA_HOME/config.toml"

    if [ -d "$SCRIPT_DIR/hooks" ]; then
        mkdir -p "$SENA_HOME/hooks"
        cp "$SCRIPT_DIR/hooks/"*.sh "$SENA_HOME/hooks/" 2>/dev/null || true
        chmod +x "$SENA_HOME/hooks/"*.sh 2>/dev/null || true
        print_success "Installed hook scripts"
    fi

    if [ -d "$SCRIPT_DIR/memory" ]; then
        mkdir -p "$CLAUDE_HOME/memory"
        cp "$SCRIPT_DIR/memory/"*.md "$CLAUDE_HOME/memory/" 2>/dev/null || true
        print_success "Installed memory patterns"
    fi
}

generate_slash_commands() {
    print_step "Generating Slash Commands"

    mkdir -p "$CLAUDE_HOME/commands"

    local CMD="$USER_COMMAND"
    local PREFIX="$USER_PREFIX"
    local EMOJI="$USER_EMOJI"

    local commands_generated=0

    cat > "$CLAUDE_HOME/commands/${CMD}-health.md" << EOF
Run ${PREFIX} health check and display system status.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Execute: \`${CMD} health --detailed\`

Display the results in ${PREFIX} format with Unicode boxes.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-metrics.md" << EOF
Get ${PREFIX} system metrics.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Execute: \`${CMD} metrics\`
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-status.md" << EOF
Show ${PREFIX} session status.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Execute: \`${CMD} session info\`
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-test.md" << EOF
Run ${PREFIX} system test.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Execute these commands in sequence:
1. \`${CMD} health\`
2. \`${CMD} metrics\`
3. \`${CMD} session info\`

Report all results.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-analyze.md" << EOF
Perform deep analysis using ${PREFIX} ${EMOJI} intelligence.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Use the ${CMD} think command with extended analysis:
\`${CMD} think "\$ARGUMENTS" --depth deep\`

Provide comprehensive insights using ${PREFIX}'s multi-layered analysis.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-code.md" << EOF
${PREFIX} ${EMOJI} Code Analysis Mode

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Analyze the provided code using ${PREFIX}'s specialized agents:

1. For backend code: \`${CMD} backend full "<code>"\`
2. For iOS code: \`${CMD} ios full "<code>"\`
3. For Android code: \`${CMD} android full "<code>"\`
4. For web code: \`${CMD} web full "<code>"\`

Select the appropriate agent based on the code type and provide detailed analysis.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-verify.md" << EOF
${PREFIX} ${EMOJI} Truth Verification Mode

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Use ${PREFIX}'s truth-embedded verification system:

\`${CMD} validate "\$ARGUMENTS"\`

Apply rigorous verification using:
- Factual accuracy check
- Logic consistency analysis
- Source credibility assessment
- Bias detection

Report confidence level and any concerns.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-format-table.md" << EOF
Format data as a ${PREFIX} ${EMOJI} styled table.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Use: \`${CMD} format table --title "Title" '<json-data>'\`

The table will use Unicode box-drawing characters in ${PREFIX} style.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-network.md" << EOF
${PREFIX} ${EMOJI} Network Collaboration

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Manage network collaboration:
- Status: \`${CMD} network status\`
- Start: \`${CMD} network start --name "${PREFIX} Instance"\`
- Info: \`${CMD} network info\`
- Discover: \`${CMD} discover\`
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-peers.md" << EOF
${PREFIX} ${EMOJI} Peer Management

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Manage network peers:
- List: \`${CMD} peer list\`
- Add: \`${CMD} peer add <ip> --name "Peer Name"\`
- Authorize: \`${CMD} peer authorize <id>\`
- Connect: \`${CMD} peer connect <ip> --token <token>\`
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/session-start.md" << EOF
Start a new ${PREFIX} ${EMOJI} collaboration session.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Execute: \`${CMD} join --role "\$ARGUMENTS" --name "${PREFIX}-Claude"\`

This joins the ${PREFIX} collaboration hub for multi-session teamwork.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/session-name.md" << EOF
Set session name for ${PREFIX} ${EMOJI} collaboration.

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Execute: \`${CMD} session start --name "\$ARGUMENTS"\`

This names your session in the ${PREFIX} hub.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/deep-think.md" << EOF
${PREFIX} ${EMOJI} Extended Thinking Mode

IMPORTANT: Use \`${CMD}\` from PATH, NOT ./target/release/${CMD}

Engage deep analysis with maximum thinking depth:

\`${CMD} think "\$ARGUMENTS" --depth maximum\`

This activates ${PREFIX}'s most thorough reasoning process.
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-always-on.md" << EOF
Enable ${PREFIX} hook for all prompts.
Note: Configure in ~/.claude/settings.json
EOF
    ((commands_generated++))

    cat > "$CLAUDE_HOME/commands/${CMD}-always-off.md" << EOF
Disable ${PREFIX} hook temporarily.
Note: Configure in ~/.claude/settings.json
EOF
    ((commands_generated++))

    print_success "Generated $commands_generated slash commands"
}

setup_claude_code_config() {
    print_step "Configuring Claude Code"

    mkdir -p "$CLAUDE_HOME"

    local sena_latest_path="$INSTALL_DIR/sena-latest"
    local sena_path="$INSTALL_DIR/sena"

    cat > "$CLAUDE_HOME/settings.json" << EOF
{
  "permissions": {
    "allow": [
      "Bash(${USER_COMMAND} *)",
      "Bash(${USER_COMMAND} who:*)",
      "Bash(${USER_COMMAND} peer list:*)",
      "Bash(sena *)",
      "Bash(sena who:*)",
      "Bash(sena peer list:*)",
      "Bash(${sena_latest_path} *)",
      "Bash(${sena_path} *)",
      "Bash(./target/release/sena *)"
    ]
  },
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "${sena_latest_path} hook user-prompt-submit"
      }
    ]
  }
}
EOF

    print_success "Created $CLAUDE_HOME/settings.json"
    print_detail "Auto-approved commands: $USER_COMMAND, sena, sena-latest"
    print_detail "Hook: UserPromptSubmit (using sena-latest for auto-version)"
}

setup_claude_desktop_config() {
    print_step "Configuring Claude Desktop"

    local config_dir="$HOME/Library/Application Support/Claude"
    mkdir -p "$config_dir"

    local sena_path="$INSTALL_DIR/sena"

    cat > "$CLAUDE_DESKTOP_CONFIG" << EOF
{
  "mcpServers": {
    "$USER_COMMAND": {
      "command": "$sena_path",
      "args": ["mcp"]
    }
  }
}
EOF

    print_success "Created Claude Desktop config"
    print_detail "MCP server: $USER_COMMAND"
}

setup_claude_md() {
    print_step "Installing CLAUDE.md Rules"

    if [ -f "$SCRIPT_DIR/CLAUDE.md" ]; then
        cp "$SCRIPT_DIR/CLAUDE.md" "$CLAUDE_HOME/CLAUDE.md"
        print_success "Installed SENA Elite Coding Standards"
    else
        print_warning "CLAUDE.md not found in repo"
    fi
}

clear_claude_code_cache() {
    print_step "Clearing Claude Code Cache"

    local project_cache="$CLAUDE_HOME/projects"
    local current_project_hash=$(echo "$SCRIPT_DIR" | sed 's/\//-/g')
    local project_cache_dir="$project_cache/$current_project_hash"

    if [ -d "$project_cache_dir" ]; then
        echo ""
        echo "Claude Code stores session history that can cause it to use"
        echo "old command patterns (like ./target/release/sena)."
        echo ""
        echo "Clear this project's cache to reset learned behavior?"
        echo ""
        read -p "Clear cache? (y/N): " clear_cache

        if [[ "$clear_cache" =~ ^[Yy]$ ]]; then
            rm -rf "$project_cache_dir"
            print_success "Cleared project cache"
            print_detail "New Claude sessions will use fresh patterns"
        else
            print_info "Cache preserved"
        fi
    fi
}

standard_installation() {
    print_step "Standard Installation"

    detect_installed_version
    if [ -n "$INSTALLED_VERSION" ]; then
        echo ""
        print_warning "Existing installation detected (v$INSTALLED_VERSION)"
        echo ""
        read -p "Continue with fresh installation? (y/N): " confirm
        if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
            show_main_menu
            return
        fi

        print_info "Creating backup..."
        local backup_dir=$(create_backup "pre_install")
        print_success "Backup: $backup_dir"
    fi

    collect_user_preferences
    build_binary
    install_binary
    setup_sena_config
    generate_slash_commands
    setup_claude_code_config
    setup_claude_desktop_config
    setup_claude_md
    clear_claude_code_cache
    rotate_backups

    show_installation_complete
}

merge_installation() {
    print_step "Merge Installation"

    print_info "This will add SENA to your existing Claude setup"
    echo ""
    read -p "Continue? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        show_main_menu
        return
    fi

    local backup_dir=$(create_backup "pre_merge")
    print_success "Backup: $backup_dir"

    collect_user_preferences
    build_binary
    install_binary
    setup_sena_config
    generate_slash_commands

    print_step "Merging Claude Code Configuration"

    if [ -f "$CLAUDE_HOME/settings.json" ] && command -v python3 &> /dev/null; then
        python3 << PYEOF
import json
import os

settings_path = os.path.expanduser("$CLAUDE_HOME/settings.json")
sena_path = "$INSTALL_DIR/sena"
sena_latest_path = "$INSTALL_DIR/sena-latest"
user_command = "$USER_COMMAND"

with open(settings_path, 'r') as f:
    settings = json.load(f)

if 'permissions' not in settings:
    settings['permissions'] = {'allow': []}
if 'allow' not in settings['permissions']:
    settings['permissions']['allow'] = []

new_perms = [
    f"Bash({user_command} *)",
    f"Bash({user_command} who:*)",
    f"Bash({user_command} peer list:*)",
    "Bash(sena *)",
    "Bash(sena who:*)",
    "Bash(sena peer list:*)",
    f"Bash({sena_latest_path} *)",
    f"Bash({sena_path} *)",
    "Bash(./target/release/sena *)"
]

existing = set(settings['permissions']['allow'])
for perm in new_perms:
    if perm not in existing:
        settings['permissions']['allow'].append(perm)

if 'hooks' not in settings:
    settings['hooks'] = {}
if 'UserPromptSubmit' not in settings['hooks']:
    settings['hooks']['UserPromptSubmit'] = []

settings['hooks']['UserPromptSubmit'] = [
    h for h in settings['hooks']['UserPromptSubmit']
    if 'sena' not in h.get('command', '').lower()
]
sena_hook = {"command": f"{sena_latest_path} hook user-prompt-submit"}
settings['hooks']['UserPromptSubmit'].append(sena_hook)

with open(settings_path, 'w') as f:
    json.dump(settings, f, indent=2)

print("Merged successfully")
PYEOF
        print_success "Merged Claude Code settings"
    else
        setup_claude_code_config
    fi

    if [ -f "$CLAUDE_DESKTOP_CONFIG" ] && command -v python3 &> /dev/null; then
        python3 << PYEOF
import json
import os

config_path = os.path.expanduser("$CLAUDE_DESKTOP_CONFIG")
sena_path = "$INSTALL_DIR/sena"
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

print("Merged successfully")
PYEOF
        print_success "Merged Claude Desktop config"
    else
        setup_claude_desktop_config
    fi

    if [ -f "$CLAUDE_HOME/CLAUDE.md" ]; then
        echo ""
        echo "Existing CLAUDE.md found. Options:"
        echo "  1) Keep existing"
        echo "  2) Replace with SENA rules"
        echo "  3) Append SENA rules"
        echo ""
        read -p "Choice [1]: " md_choice

        case $md_choice in
            2) setup_claude_md ;;
            3)
                if [ -f "$SCRIPT_DIR/CLAUDE.md" ]; then
                    echo -e "\n\n---\n" >> "$CLAUDE_HOME/CLAUDE.md"
                    cat "$SCRIPT_DIR/CLAUDE.md" >> "$CLAUDE_HOME/CLAUDE.md"
                    print_success "Appended SENA rules"
                fi
                ;;
            *) print_info "Kept existing CLAUDE.md" ;;
        esac
    else
        setup_claude_md
    fi

    rotate_backups
    show_installation_complete
}

minimal_installation() {
    print_step "Minimal Installation"

    print_info "This will only install the SENA binary"
    echo ""
    read -p "Continue? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        show_main_menu
        return
    fi

    echo ""
    read -p "Command name [sena]: " input_command
    USER_COMMAND="${input_command:-sena}"

    build_binary
    install_binary

    print_step "Minimal Installation Complete"

    echo ""
    echo "Binary installed: $INSTALL_DIR/sena"
    if [ "$USER_COMMAND" != "sena" ]; then
        echo "Alias created: $INSTALL_DIR/$USER_COMMAND"
    fi
    echo ""
    echo "Manual setup required for full functionality."
    echo "Run: $USER_COMMAND --help"
    echo ""
}

upgrade_sena() {
    print_step "Upgrade SENA"

    detect_installed_version

    if [ -z "$INSTALLED_VERSION" ]; then
        print_error "No existing installation found"
        echo ""
        read -p "Would you like to install instead? (y/N): " install_instead
        if [[ "$install_instead" =~ ^[Yy]$ ]]; then
            standard_installation
        else
            show_main_menu
        fi
        return
    fi

    local version_status=$(compare_versions "$INSTALLED_VERSION" "$SENA_VERSION")

    echo ""
    echo "Current version: $INSTALLED_VERSION"
    echo "Available version: $SENA_VERSION"
    echo ""

    case $version_status in
        "same")
            print_info "You already have the latest version"
            read -p "Reinstall anyway? (y/N): " reinstall
            if [[ ! "$reinstall" =~ ^[Yy]$ ]]; then
                show_main_menu
                return
            fi
            ;;
        "downgrade")
            print_warning "This would be a DOWNGRADE"
            read -p "Continue anyway? (y/N): " downgrade
            if [[ ! "$downgrade" =~ ^[Yy]$ ]]; then
                show_main_menu
                return
            fi
            ;;
        "upgrade")
            print_info "Upgrade available"
            ;;
    esac

    local backup_dir=$(create_backup "pre_upgrade")
    print_success "Backup: $backup_dir"

    if [ -f "$SENA_HOME/config.toml" ]; then
        local config_cmd=$(grep -E "^command\s*=" "$SENA_HOME/config.toml" 2>/dev/null | cut -d'"' -f2 || echo "sena")
        USER_COMMAND="${config_cmd:-sena}"
    fi

    build_binary
    install_binary

    if [ -f "$SENA_HOME/config.toml" ]; then
        if grep -q "installed_version" "$SENA_HOME/config.toml"; then
            sed -i.bak "s/installed_version = \".*\"/installed_version = \"$SENA_VERSION\"/" "$SENA_HOME/config.toml"
            rm -f "$SENA_HOME/config.toml.bak"
        else
            echo "" >> "$SENA_HOME/config.toml"
            echo "[installation]" >> "$SENA_HOME/config.toml"
            echo "installed_version = \"$SENA_VERSION\"" >> "$SENA_HOME/config.toml"
        fi
    fi

    rotate_backups

    print_step "Upgrade Complete"

    echo ""
    print_success "Upgraded from v$INSTALLED_VERSION to v$SENA_VERSION"
    echo ""
    echo "Your configuration was preserved."
    echo "Backup available at: $backup_dir"
    echo ""
}

repair_sena() {
    print_step "Repair SENA Installation"

    local issues_found=0
    local issues_fixed=0

    echo ""
    print_info "Scanning for issues..."
    echo ""

    if [ ! -f "$INSTALL_DIR/sena" ]; then
        print_error "Binary missing: $INSTALL_DIR/sena"
        ((issues_found++))
    elif [ ! -x "$INSTALL_DIR/sena" ]; then
        print_error "Binary not executable"
        ((issues_found++))
    else
        print_success "Binary OK"
    fi

    if [ ! -f "$SENA_HOME/config.toml" ]; then
        print_error "Config missing: $SENA_HOME/config.toml"
        ((issues_found++))
    else
        print_success "Config OK"
    fi

    if [ ! -f "$CLAUDE_HOME/settings.json" ]; then
        print_error "Claude settings missing"
        ((issues_found++))
    else
        if grep -q "sena" "$CLAUDE_HOME/settings.json" 2>/dev/null; then
            print_success "Claude hooks OK"
        else
            print_error "SENA hooks not configured"
            ((issues_found++))
        fi
    fi

    local slash_cmd_count=$(ls "$CLAUDE_HOME/commands/" 2>/dev/null | grep -E "^sena-|^session-|^deep-think" | wc -l | tr -d ' ')
    if [ "$slash_cmd_count" -lt 10 ]; then
        print_warning "Some slash commands missing ($slash_cmd_count found)"
        ((issues_found++))
    else
        print_success "Slash commands OK ($slash_cmd_count found)"
    fi

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "PATH not configured"
        ((issues_found++))
    else
        print_success "PATH OK"
    fi

    echo ""

    if [ "$issues_found" -eq 0 ]; then
        print_success "No issues found!"
        echo ""
        show_main_menu
        return
    fi

    echo "Found $issues_found issue(s)"
    echo ""
    read -p "Attempt to fix? (y/N): " fix_issues

    if [[ ! "$fix_issues" =~ ^[Yy]$ ]]; then
        show_main_menu
        return
    fi

    local backup_dir=$(create_backup "pre_repair")
    print_success "Backup: $backup_dir"

    if [ -f "$SENA_HOME/config.toml" ]; then
        local config_cmd=$(grep -E "^command\s*=" "$SENA_HOME/config.toml" 2>/dev/null | cut -d'"' -f2 || echo "sena")
        local config_name=$(grep -E "^name\s*=" "$SENA_HOME/config.toml" 2>/dev/null | cut -d'"' -f2 || echo "$(whoami)")
        local config_emoji=$(grep -E "^emoji\s*=" "$SENA_HOME/config.toml" 2>/dev/null | cut -d'"' -f2 || echo "🦁")
        local config_prefix=$(grep -E "^prefix\s*=" "$SENA_HOME/config.toml" 2>/dev/null | cut -d'"' -f2 || echo "SENA")

        USER_COMMAND="${config_cmd:-sena}"
        USER_NAME="${config_name:-$(whoami)}"
        USER_EMOJI="${config_emoji:-🦁}"
        USER_PREFIX="${config_prefix:-SENA}"
    else
        USER_COMMAND="sena"
        USER_NAME="$(whoami)"
        USER_EMOJI="🦁"
        USER_PREFIX="SENA"
    fi

    if [ ! -f "$INSTALL_DIR/sena" ] || [ ! -x "$INSTALL_DIR/sena" ]; then
        print_info "Rebuilding binary..."
        if build_binary && install_binary; then
            ((issues_fixed++))
        fi
    fi

    if [ ! -f "$SENA_HOME/config.toml" ]; then
        print_info "Recreating config..."
        setup_sena_config
        ((issues_fixed++))
    fi

    if [ ! -f "$CLAUDE_HOME/settings.json" ] || ! grep -q "sena" "$CLAUDE_HOME/settings.json" 2>/dev/null; then
        print_info "Fixing Claude hooks..."
        setup_claude_code_config
        ((issues_fixed++))
    fi

    if [ "$slash_cmd_count" -lt 10 ]; then
        print_info "Regenerating slash commands..."
        generate_slash_commands
        ((issues_fixed++))
    fi

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        setup_shell_path
        ((issues_fixed++))
    fi

    rotate_backups

    print_step "Repair Complete"

    echo ""
    print_success "Fixed $issues_fixed of $issues_found issues"
    echo ""
}

configure_menu() {
    print_step "Configuration"

    echo ""
    echo -e "  ${GREEN}1)${NC} Change user preferences"
    echo -e "  ${BLUE}2)${NC} Regenerate slash commands"
    echo -e "  ${YELLOW}3)${NC} Reset Claude Code settings"
    echo -e "  ${MAGENTA}4)${NC} Clear Claude Code cache"
    echo -e "  ${NC}0)${NC} Back"
    echo ""

    read -p "Choice [0]: " config_choice

    case $config_choice in
        1)
            collect_user_preferences
            setup_sena_config
            generate_slash_commands
            setup_claude_code_config
            print_success "Configuration updated"
            ;;
        2)
            if [ -f "$SENA_HOME/config.toml" ]; then
                USER_COMMAND=$(grep -E "^command\s*=" "$SENA_HOME/config.toml" | cut -d'"' -f2)
                USER_PREFIX=$(grep -E "^prefix\s*=" "$SENA_HOME/config.toml" | cut -d'"' -f2)
                USER_EMOJI=$(grep -E "^emoji\s*=" "$SENA_HOME/config.toml" | cut -d'"' -f2)
            fi
            generate_slash_commands
            print_success "Slash commands regenerated"
            ;;
        3)
            if [ -f "$SENA_HOME/config.toml" ]; then
                USER_COMMAND=$(grep -E "^command\s*=" "$SENA_HOME/config.toml" | cut -d'"' -f2)
            fi
            setup_claude_code_config
            print_success "Claude Code settings reset"
            ;;
        4)
            clear_claude_code_cache
            ;;
        0|"")
            show_main_menu
            return
            ;;
    esac

    echo ""
    read -p "Press Enter to continue..."
    configure_menu
}

uninstall_menu() {
    print_step "Uninstall SENA"

    echo ""
    echo -e "  ${YELLOW}1)${NC} ${BOLD}Uninstall SENA only${NC}"
    echo -e "     Remove binary, config, and slash commands"
    echo -e "     Keep Claude configurations"
    echo ""
    echo -e "  ${RED}2)${NC} ${BOLD}Complete uninstall${NC}"
    echo -e "     Remove everything including Claude integrations"
    echo ""
    echo -e "  ${NC}0)${NC} Cancel"
    echo ""

    read -p "Choice [0]: " uninstall_choice

    case $uninstall_choice in
        1) uninstall_sena_only ;;
        2) uninstall_complete ;;
        0|"") show_main_menu ;;
        *) uninstall_menu ;;
    esac
}

uninstall_sena_only() {
    print_step "Uninstalling SENA"

    read -p "Are you sure? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        show_main_menu
        return
    fi

    local backup_dir=$(create_backup "pre_uninstall")
    print_success "Backup: $backup_dir"

    if [ -f "$INSTALL_DIR/sena" ]; then
        rm -f "$INSTALL_DIR/sena"
        print_success "Removed sena binary"
    fi

    for file in "$INSTALL_DIR"/*; do
        if [ -L "$file" ]; then
            local target=$(readlink "$file" 2>/dev/null || echo "")
            if [[ "$target" == *"sena"* ]]; then
                rm -f "$file"
                print_success "Removed symlink: $(basename "$file")"
            fi
        fi
    done

    if [ -d "$SENA_HOME" ]; then
        mv "$SENA_HOME" "${SENA_HOME}_uninstalled_$(date +%Y%m%d%H%M%S)"
        print_success "Archived SENA config"
    fi

    for cmd_file in "$CLAUDE_HOME/commands"/sena-* "$CLAUDE_HOME/commands"/session-* "$CLAUDE_HOME/commands"/deep-think.md; do
        if [ -f "$cmd_file" ]; then
            rm -f "$cmd_file"
        fi
    done
    print_success "Removed slash commands"

    print_step "Uninstall Complete"

    echo ""
    print_info "SENA has been removed"
    print_info "Claude configurations were preserved"
    print_info "Backup available at: $backup_dir"
    echo ""
}

uninstall_complete() {
    print_step "Complete Uninstall"

    echo ""
    print_warning "This will remove ALL SENA integrations including Claude hooks!"
    echo ""
    read -p "Type 'UNINSTALL' to confirm: " confirm

    if [ "$confirm" != "UNINSTALL" ]; then
        print_info "Cancelled"
        show_main_menu
        return
    fi

    local backup_dir=$(create_backup "pre_complete_uninstall")
    print_success "Backup: $backup_dir"

    if [ -f "$INSTALL_DIR/sena" ]; then
        rm -f "$INSTALL_DIR/sena"
        print_success "Removed sena binary"
    fi

    for file in "$INSTALL_DIR"/*; do
        if [ -L "$file" ]; then
            local target=$(readlink "$file" 2>/dev/null || echo "")
            if [[ "$target" == *"sena"* ]]; then
                rm -f "$file"
                print_success "Removed symlink: $(basename "$file")"
            fi
        fi
    done

    if [ -d "$SENA_HOME" ]; then
        rm -rf "$SENA_HOME"
        print_success "Removed SENA config directory"
    fi

    for cmd_file in "$CLAUDE_HOME/commands"/sena-* "$CLAUDE_HOME/commands"/session-* "$CLAUDE_HOME/commands"/deep-think.md; do
        if [ -f "$cmd_file" ]; then
            rm -f "$cmd_file"
        fi
    done
    print_success "Removed slash commands"

    if [ -f "$CLAUDE_HOME/settings.json" ] && command -v python3 &> /dev/null; then
        python3 << PYEOF
import json
import os

settings_path = os.path.expanduser("$CLAUDE_HOME/settings.json")

try:
    with open(settings_path, 'r') as f:
        settings = json.load(f)

    if 'permissions' in settings and 'allow' in settings['permissions']:
        settings['permissions']['allow'] = [
            p for p in settings['permissions']['allow']
            if 'sena' not in p.lower()
        ]

    if 'hooks' in settings and 'UserPromptSubmit' in settings['hooks']:
        settings['hooks']['UserPromptSubmit'] = [
            h for h in settings['hooks']['UserPromptSubmit']
            if 'sena' not in h.get('command', '').lower()
        ]

    with open(settings_path, 'w') as f:
        json.dump(settings, f, indent=2)

    print("Cleaned Claude settings")
except Exception as e:
    print(f"Warning: {e}")
PYEOF
        print_success "Cleaned Claude Code settings"
    fi

    if [ -f "$CLAUDE_DESKTOP_CONFIG" ] && command -v python3 &> /dev/null; then
        python3 << PYEOF
import json
import os

config_path = os.path.expanduser("$CLAUDE_DESKTOP_CONFIG")

try:
    with open(config_path, 'r') as f:
        config = json.load(f)

    if 'mcpServers' in config:
        config['mcpServers'] = {
            k: v for k, v in config['mcpServers'].items()
            if 'sena' not in k.lower() and 'sena' not in str(v.get('command', '')).lower()
        }

    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2)

    print("Cleaned Claude Desktop config")
except Exception as e:
    print(f"Warning: {e}")
PYEOF
        print_success "Cleaned Claude Desktop config"
    fi

    if [ -f "$CLAUDE_HOME/CLAUDE.md" ]; then
        if grep -q "SENA1996" "$CLAUDE_HOME/CLAUDE.md" 2>/dev/null; then
            rm -f "$CLAUDE_HOME/CLAUDE.md"
            print_success "Removed CLAUDE.md"
        fi
    fi

    print_step "Complete Uninstall Done"

    echo ""
    print_success "SENA has been completely removed"
    print_info "Backup available at: $backup_dir"
    echo ""
}

run_diagnostics() {
    print_step "SENA Diagnostics"

    echo ""
    echo -e "${BOLD}System Information${NC}"
    echo "  OS: $(uname -s) $(uname -r)"
    echo "  Shell: $SHELL"
    echo "  User: $(whoami)"
    echo "  Home: $HOME"
    echo ""

    echo -e "${BOLD}Installation Status${NC}"

    detect_installed_version
    if [ -n "$INSTALLED_VERSION" ]; then
        echo -e "  Binary: ${GREEN}v$INSTALLED_VERSION${NC}"
    else
        echo -e "  Binary: ${RED}Not installed${NC}"
    fi

    if [ -f "$SENA_HOME/config.toml" ]; then
        echo -e "  Config: ${GREEN}Present${NC}"
    else
        echo -e "  Config: ${RED}Missing${NC}"
    fi

    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        echo -e "  PATH: ${GREEN}Configured${NC}"
    else
        echo -e "  PATH: ${YELLOW}Not in PATH${NC}"
    fi

    echo ""
    echo -e "${BOLD}Claude Integration${NC}"

    if [ -f "$CLAUDE_HOME/settings.json" ]; then
        if grep -q "sena" "$CLAUDE_HOME/settings.json" 2>/dev/null; then
            echo -e "  Hooks: ${GREEN}Configured${NC}"
        else
            echo -e "  Hooks: ${YELLOW}Not configured${NC}"
        fi
    else
        echo -e "  Hooks: ${RED}settings.json missing${NC}"
    fi

    local cmd_count=$(ls "$CLAUDE_HOME/commands/" 2>/dev/null | grep -E "^sena-|^session-|^deep-think" | wc -l | tr -d ' ')
    echo "  Slash commands: $cmd_count"

    if [ -f "$CLAUDE_DESKTOP_CONFIG" ]; then
        if grep -q "sena" "$CLAUDE_DESKTOP_CONFIG" 2>/dev/null; then
            echo -e "  MCP Server: ${GREEN}Configured${NC}"
        else
            echo -e "  MCP Server: ${YELLOW}Not configured${NC}"
        fi
    else
        echo -e "  MCP Server: ${DIM}No Claude Desktop${NC}"
    fi

    echo ""
    echo -e "${BOLD}Quick Test${NC}"

    if command -v sena &> /dev/null; then
        echo -e "  'sena' command: ${GREEN}Available${NC}"
        local sena_path=$(which sena)
        echo "  Location: $sena_path"
    else
        echo -e "  'sena' command: ${RED}Not found in PATH${NC}"
    fi

    if [ -x "$INSTALL_DIR/sena" ]; then
        echo ""
        echo "Testing sena binary..."
        if "$INSTALL_DIR/sena" --version 2>/dev/null; then
            echo -e "  ${GREEN}Binary works correctly${NC}"
        else
            echo -e "  ${RED}Binary execution failed${NC}"
        fi
    fi

    echo ""
    echo -e "${BOLD}Backups${NC}"
    if [ -d "$SENA_HOME/backups" ]; then
        local backup_count=$(ls -d "$SENA_HOME/backups"/*/ 2>/dev/null | wc -l | tr -d ' ')
        echo "  Available backups: $backup_count"
        ls -dt "$SENA_HOME/backups"/*/ 2>/dev/null | head -3 | while read dir; do
            echo "    - $(basename "$dir")"
        done
    else
        echo "  No backups found"
    fi

    echo ""
    read -p "Press Enter to continue..."
    show_main_menu
}

show_installation_complete() {
    print_step "Installation Complete! $USER_EMOJI"

    echo ""
    echo -e "${GREEN}Sena1996 AI Tool has been installed successfully!${NC}"
    echo ""
    echo -e "Welcome, ${BOLD}$USER_NAME${NC}!"
    echo ""
    echo "What was installed:"
    echo "  • Binary: $INSTALL_DIR/sena"
    if [ "$USER_COMMAND" != "sena" ]; then
        echo "  • Alias: $INSTALL_DIR/$USER_COMMAND"
    fi
    echo "  • Config: $SENA_HOME/config.toml"
    echo "  • Hooks: $CLAUDE_HOME/settings.json"
    echo "  • Commands: $CLAUDE_HOME/commands/"
    echo ""
    echo -e "${BOLD}Quick Start:${NC}"
    echo "  $USER_COMMAND health         # Check system health"
    echo "  $USER_COMMAND session start  # Start a session"
    echo "  $USER_COMMAND who            # See who's online"
    echo "  $USER_COMMAND --help         # Full command list"
    echo ""
    echo -e "${BOLD}Next Steps:${NC}"
    echo "  1. Restart Claude Desktop (if using)"
    echo "  2. Start a NEW Claude Code session"
    echo "  3. Try: $USER_COMMAND health"
    echo ""

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}Note:${NC} Remember to add ~/.local/bin to PATH or restart terminal"
        echo ""
    fi
}

print_banner
check_system_requirements
detect_existing_installation
show_main_menu
