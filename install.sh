#!/bin/bash
# Sena1996-AI Installation Script
# Installs SENA Controller with Rust binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SENA_HOME="$SCRIPT_DIR"
CLAUDE_DIR="$HOME/.claude"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║           SENA 🦁 CONTROLLER INSTALLER                       ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

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

    sed "s|{SENA_HOME}|$SENA_HOME|g" "$SENA_HOME/config/CLAUDE.md.template" > "$CLAUDE_DIR/CLAUDE.md"
    echo "✅ CLAUDE.md installed"
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
        install_hooks
        install_commands
        install_memory
        install_claude_md
        update_settings
        ;;
    2)
        install_hooks
        update_settings
        ;;
    3)
        read -p "Install hooks? [y/n]: " h
        read -p "Install commands? [y/n]: " c
        read -p "Install memory? [y/n]: " m
        read -p "Install CLAUDE.md? [y/n]: " d
        read -p "Update settings.json? [y/n]: " s

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

# Enable SENA always-on mode
read -p "Enable SENA Always-On mode? [y/n]: " always_on
if [ "$always_on" = "y" ]; then
    touch "$CLAUDE_DIR/.sena_always_on"
    echo "✅ Always-On mode enabled"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║           INSTALLATION COMPLETE! 🦁                          ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "SENA Binary: $SENA_HOME/target/release/sena"
echo ""
echo "To use SENA CLI directly:"
echo "  export PATH=\"\$PATH:$SENA_HOME/target/release\""
echo "  sena --help"
echo ""
echo "Restart Claude Code to apply changes."
