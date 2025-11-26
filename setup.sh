#!/bin/bash

set -e

SENA_VERSION="10.0.0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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
    echo ""
    echo -e "  ${YELLOW}2)${NC} ${BOLD}Merge Installation${NC} (Keep existing + add SENA)"
    echo -e "     Keep your existing setup and add SENA on top"
    echo -e "     • Preserves your current hooks and rules"
    echo -e "     • Adds SENA MCP server to existing config"
    echo -e "     • Merges SENA rules with your CLAUDE.md"
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

    cp "$SCRIPT_DIR/target/release/sena" "$INSTALL_DIR/sena"
    chmod +x "$INSTALL_DIR/sena"

    print_success "Installed to: $INSTALL_DIR/sena"

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "Add to your PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
}

clean_claude_code() {
    print_info "Cleaning Claude Code config..."

    if [ -d "$HOME/.claude" ]; then
        rm -rf "$HOME/.claude"
        print_success "Removed ~/.claude"
    fi

    mkdir -p "$HOME/.claude"
}

clean_claude_desktop() {
    print_info "Cleaning Claude Desktop config..."

    CLAUDE_APP_SUPPORT="$HOME/Library/Application Support/Claude"

    if [ -d "$CLAUDE_APP_SUPPORT" ]; then
        rm -rf "$CLAUDE_APP_SUPPORT"
        print_success "Removed Claude Desktop data"
    fi

    rm -rf "$HOME/Library/Caches/claude-cli-nodejs" 2>/dev/null
    rm -rf "$HOME/Library/Caches/com.anthropic.claudefordesktop" 2>/dev/null
    rm -rf "$HOME/Library/Caches/com.anthropic.claudefordesktop.ShipIt" 2>/dev/null
    rm -f "$HOME/Library/Preferences/com.anthropic.claudefordesktop.plist" 2>/dev/null

    mkdir -p "$CLAUDE_APP_SUPPORT"
}

setup_claude_code_config() {
    print_step "Setting Up Claude Code Configuration"

    mkdir -p "$HOME/.claude"

    SENA_PATH="$HOME/.local/bin/sena"

    cat > "$HOME/.claude/settings.json" << EOF
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "$SENA_PATH hook user-prompt-submit"
      }
    ]
  }
}
EOF

    print_success "Created ~/.claude/settings.json with SENA hooks"
}

setup_claude_desktop_config() {
    print_step "Setting Up Claude Desktop Configuration"

    CLAUDE_CONFIG_DIR="$HOME/Library/Application Support/Claude"
    mkdir -p "$CLAUDE_CONFIG_DIR"

    SENA_PATH="$HOME/.local/bin/sena"

    cat > "$CLAUDE_CONFIG_DIR/claude_desktop_config.json" << EOF
{
  "mcpServers": {
    "sena": {
      "command": "$SENA_PATH",
      "args": ["mcp"]
    }
  }
}
EOF

    print_success "Created Claude Desktop config with SENA MCP server"
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

    backup_existing
    build_sena
    install_binary
    clean_claude_code
    clean_claude_desktop
    setup_claude_code_config
    setup_claude_desktop_config
    setup_claude_md

    print_step "Installation Complete! 🦁"

    echo ""
    echo -e "${GREEN}SENA v${SENA_VERSION} has been installed successfully!${NC}"
    echo ""
    echo "What was installed:"
    echo "  • SENA binary: ~/.local/bin/sena"
    echo "  • Claude Code hooks: ~/.claude/settings.json"
    echo "  • Claude Desktop MCP: ~/Library/Application Support/Claude/"
    echo "  • SENA rules: ~/.claude/CLAUDE.md"
    echo ""
    echo "Next steps:"
    echo "  1. Restart Claude Desktop"
    echo "  2. Start a new Claude Code session"
    echo "  3. Run: sena --version"
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

    backup_existing
    build_sena
    install_binary

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

with open(settings_path, 'r') as f:
    settings = json.load(f)

if 'hooks' not in settings:
    settings['hooks'] = {}

if 'UserPromptSubmit' not in settings['hooks']:
    settings['hooks']['UserPromptSubmit'] = []

sena_hook = {"command": f"{sena_path} hook user-prompt-submit"}
if sena_hook not in settings['hooks']['UserPromptSubmit']:
    settings['hooks']['UserPromptSubmit'].append(sena_hook)

with open(settings_path, 'w') as f:
    json.dump(settings, f, indent=2)

print("Merged SENA hook into settings.json")
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

with open(config_path, 'r') as f:
    config = json.load(f)

if 'mcpServers' not in config:
    config['mcpServers'] = {}

config['mcpServers']['sena'] = {
    "command": sena_path,
    "args": ["mcp"]
}

with open(config_path, 'w') as f:
    json.dump(config, f, indent=2)

print("Added SENA MCP server to config")
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

    print_step "Merge Installation Complete! 🦁"

    echo ""
    echo -e "${GREEN}SENA v${SENA_VERSION} has been merged with your existing setup!${NC}"
    echo ""
    echo "Your existing configuration was preserved and SENA was added."
    echo ""
    echo "Next steps:"
    echo "  1. Restart Claude Desktop"
    echo "  2. Start a new Claude Code session"
    echo "  3. Run: sena --version"
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

    build_sena
    install_binary

    print_step "Minimal Installation Complete! 🦁"

    echo ""
    echo -e "${GREEN}SENA binary installed to ~/.local/bin/sena${NC}"
    echo ""
    echo "Manual setup required:"
    echo ""
    echo "For Claude Code hooks, add to ~/.claude/settings.json:"
    echo '  {'
    echo '    "hooks": {'
    echo '      "UserPromptSubmit": ['
    echo '        {"command": "~/.local/bin/sena hook user-prompt-submit"}'
    echo '      ]'
    echo '    }'
    echo '  }'
    echo ""
    echo "For Claude Desktop MCP, add to claude_desktop_config.json:"
    echo '  {'
    echo '    "mcpServers": {'
    echo '      "sena": {"command": "~/.local/bin/sena", "args": ["mcp"]}'
    echo '    }'
    echo '  }'
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

    if [ -f "$HOME/.local/bin/sena" ]; then
        rm -f "$HOME/.local/bin/sena"
        print_success "Removed SENA binary"
    fi

    if [ -d "$HOME/.sena" ]; then
        rm -rf "$HOME/.sena"
        print_success "Removed SENA config directory"
    fi

    print_info "Note: Claude configurations were not modified."
    print_info "You may want to manually remove SENA from:"
    echo "  • ~/.claude/settings.json (hooks)"
    echo "  • Claude Desktop config (MCP servers)"
    echo "  • ~/.claude/CLAUDE.md (rules)"

    print_step "Uninstall Complete"
}

print_banner
detect_existing_setup
show_menu
