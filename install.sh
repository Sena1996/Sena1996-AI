#!/bin/bash
# Sena1996-AI Installation Script
# Installs SENA Controller with Rust binary
# Technology is for everyone - customize your experience!

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SENA_HOME="$SCRIPT_DIR"
CLAUDE_DIR="$HOME/.claude"

# Default branding
DEFAULT_PREFIX="SENA"
DEFAULT_EMOJI="🦁"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║           SENA 🦁 CONTROLLER INSTALLER                       ║"
echo "║                                                              ║"
echo "║       Technology is for everyone - Make it yours!           ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ═══════════════════════════════════════════════════════════════════
#  PERSONALIZATION - Your AI, Your Identity
# ═══════════════════════════════════════════════════════════════════
echo "┌──────────────────────────────────────────────────────────────┐"
echo "│  PERSONALIZATION - Your AI, Your Identity                   │"
echo "└──────────────────────────────────────────────────────────────┘"
echo ""
echo "Default branding: $DEFAULT_PREFIX $DEFAULT_EMOJI"
echo ""
read -p "Would you like to customize your prefix and emoji? [y/n]: " customize

if [ "$customize" = "y" ] || [ "$customize" = "Y" ]; then
    echo ""
    echo "Examples:"
    echo "  Name: JARVIS, FRIDAY, ALEX, MAX, or your own name"
    echo "  Emoji: 🤖 🧠 ⚡ 🔮 🌟 💫 🎯 🚀 or any emoji you like"
    echo ""
    read -p "Enter your custom prefix [$DEFAULT_PREFIX]: " CUSTOM_PREFIX
    read -p "Enter your custom emoji [$DEFAULT_EMOJI]: " CUSTOM_EMOJI

    # Use defaults if empty
    USER_PREFIX="${CUSTOM_PREFIX:-$DEFAULT_PREFIX}"
    USER_EMOJI="${CUSTOM_EMOJI:-$DEFAULT_EMOJI}"
else
    USER_PREFIX="$DEFAULT_PREFIX"
    USER_EMOJI="$DEFAULT_EMOJI"
fi

echo ""
echo "✅ Your branding: $USER_PREFIX $USER_EMOJI"
echo ""

# Save user preferences to config file
save_user_config() {
    CONFIG_FILE="$CLAUDE_DIR/.sena_user_config.json"
    cat > "$CONFIG_FILE" << EOF
{
  "prefix": "$USER_PREFIX",
  "emoji": "$USER_EMOJI",
  "installed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "version": "5.0.0"
}
EOF
    echo "✅ User preferences saved"
}

# Check if Rust binary exists, if not build it
if [ ! -f "$SENA_HOME/target/release/sena" ]; then
    echo "Building SENA binary..."
    cd "$SENA_HOME"
    cargo build --release
    echo "✅ Binary built successfully"
else
    echo "✅ Binary already exists"
fi

# Create Claude directories
mkdir -p "$CLAUDE_DIR/hooks"
mkdir -p "$CLAUDE_DIR/commands"
mkdir -p "$CLAUDE_DIR/memory"

# Make hooks executable
chmod +x "$SENA_HOME/hooks/"*.sh

echo ""
echo "Select installation type:"
echo "  1) Full    - Hooks + Commands + Memory + CLAUDE.md"
echo "  2) Minimal - Hooks only"
echo "  3) Custom  - Choose components"
echo ""
read -p "Enter choice [1-3]: " choice

install_hooks() {
    echo "Installing hooks..."

    # Update hooks with correct path
    for hook in "$SENA_HOME/hooks/"*.sh; do
        filename=$(basename "$hook")
        sed "s|{SENA_HOME}|$SENA_HOME|g" "$hook" > "$CLAUDE_DIR/hooks/$filename"
        chmod +x "$CLAUDE_DIR/hooks/$filename"
    done

    echo "✅ Hooks installed"
}

install_commands() {
    echo "Installing slash commands..."

    for cmd in "$SENA_HOME/commands/"*.md; do
        filename=$(basename "$cmd")
        sed "s|{SENA_HOME}|$SENA_HOME|g" "$cmd" > "$CLAUDE_DIR/commands/$filename"
    done

    echo "✅ Commands installed"
}

install_memory() {
    echo "Installing memory files..."
    cp "$SENA_HOME/memory/"*.md "$CLAUDE_DIR/memory/"
    echo "✅ Memory files installed"
}

install_claude_md() {
    echo "Installing CLAUDE.md..."

    if [ -f "$CLAUDE_DIR/CLAUDE.md" ]; then
        cp "$CLAUDE_DIR/CLAUDE.md" "$CLAUDE_DIR/CLAUDE.md.backup"
        echo "  (backed up existing CLAUDE.md)"
    fi

    # Replace placeholders with user's custom branding
    sed -e "s|{SENA_HOME}|$SENA_HOME|g" \
        -e "s|{USER_PREFIX}|$USER_PREFIX|g" \
        -e "s|{USER_EMOJI}|$USER_EMOJI|g" \
        "$SENA_HOME/config/CLAUDE.md.template" > "$CLAUDE_DIR/CLAUDE.md"

    echo "✅ CLAUDE.md installed with branding: $USER_PREFIX $USER_EMOJI"
}

update_settings() {
    echo "Updating settings.json..."

    SETTINGS_FILE="$CLAUDE_DIR/settings.json"

    if [ -f "$SETTINGS_FILE" ]; then
        cp "$SETTINGS_FILE" "$SETTINGS_FILE.backup"
        echo "  (backed up existing settings.json)"
    fi

    # Create settings with hooks
    cat > "$SETTINGS_FILE" << EOF
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "$CLAUDE_DIR/hooks/user-prompt-submit.sh"
      }
    ],
    "AssistantResponse": [
      {
        "command": "$CLAUDE_DIR/hooks/sena-enforcer.sh"
      }
    ]
  }
}
EOF

    echo "✅ Settings updated"
}

case $choice in
    1)
        save_user_config
        install_hooks
        install_commands
        install_memory
        install_claude_md
        update_settings
        ;;
    2)
        save_user_config
        install_hooks
        update_settings
        ;;
    3)
        read -p "Install hooks? [y/n]: " h
        read -p "Install commands? [y/n]: " c
        read -p "Install memory? [y/n]: " m
        read -p "Install CLAUDE.md? [y/n]: " d
        read -p "Update settings.json? [y/n]: " s

        save_user_config
        [ "$h" = "y" ] && install_hooks
        [ "$c" = "y" ] && install_commands
        [ "$m" = "y" ] && install_memory
        [ "$d" = "y" ] && install_claude_md
        [ "$s" = "y" ] && update_settings
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

# Enable Always-On mode with custom branding
echo ""
read -p "Enable Always-On mode? (responses start with '$USER_PREFIX $USER_EMOJI') [y/n]: " always_on
if [ "$always_on" = "y" ]; then
    touch "$CLAUDE_DIR/.sena_always_on"
    echo "✅ Always-On mode enabled - Every response will start with: $USER_PREFIX $USER_EMOJI"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║           INSTALLATION COMPLETE! $USER_EMOJI                          ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "┌──────────────────────────────────────────────────────────────┐"
echo "│  Your Personal AI Assistant: $USER_PREFIX $USER_EMOJI"
echo "└──────────────────────────────────────────────────────────────┘"
echo ""
echo "Binary: $SENA_HOME/target/release/sena"
echo "Config: $CLAUDE_DIR/.sena_user_config.json"
echo ""
echo "To use CLI directly:"
echo "  export PATH=\"\$PATH:$SENA_HOME/target/release\""
echo "  sena --help"
echo ""
echo "Restart Claude Code to apply changes."
echo ""
echo "Technology is for everyone. Enjoy your personalized AI! $USER_EMOJI"
