#!/bin/bash
# SENA Tool Permission Request Hook - Rust Edition
# Dynamically allows/denies tools based on SENA runtime preferences
# AUTO-ALLOWS all SENA commands without prompts

SENA_BIN="${SENA_HOME:-$(command -v sena 2>/dev/null || echo "$HOME/.local/bin/sena")}"
SENA_TOOL_PERMISSIONS="$HOME/.claude/.sena_tool_permissions.json"

# Read JSON input from stdin if provided
if [ ! -t 0 ]; then
    INPUT=$(cat)
else
    INPUT=""
fi

# ============================================================
# SENA AUTO-ALLOW: Always allow SENA commands without prompts
# ============================================================

if [ -n "$INPUT" ]; then
    # Extract command from input JSON
    COMMAND=$(echo "$INPUT" | grep -oE '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"command"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')

    # Check if command is a SENA command - AUTO ALLOW
    if echo "$COMMAND" | grep -qiE '^(sena|\.\/target\/release\/sena|/Users/sena/AI/Sena1996-AI/target/release/sena)'; then
        echo '{"decision": "allow", "reason": "SENA command auto-approved"}'
        exit 0
    fi

    # Check for sena slash commands - AUTO ALLOW
    TOOL_INPUT=$(echo "$INPUT" | grep -oE '"input"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1)
    if echo "$TOOL_INPUT" | grep -qiE '/sena-'; then
        echo '{"decision": "allow", "reason": "SENA slash command auto-approved"}'
        exit 0
    fi
fi

# Use Rust binary for permission checking if available
if [ -x "$SENA_BIN" ] && [ -n "$INPUT" ]; then
    RESULT=$(echo "$INPUT" | "$SENA_BIN" hook permission-request 2>/dev/null)
    if [ -n "$RESULT" ]; then
        echo "$RESULT"
        exit 0
    fi
fi

# Fallback: Extract tool name from input
if [ -n "$INPUT" ]; then
    # Simple extraction without Python
    TOOL_NAME=$(echo "$INPUT" | grep -o '"tool"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"tool"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
    if [ -z "$TOOL_NAME" ]; then
        TOOL_NAME=$(echo "$INPUT" | grep -o '"toolName"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"toolName"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
    fi
    if [ -z "$TOOL_NAME" ]; then
        TOOL_NAME=$(echo "$INPUT" | grep -o '"name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
    fi
else
    TOOL_NAME="$1"
fi

# If no tool name found, allow by default
if [ -z "$TOOL_NAME" ]; then
    echo "allow"
    exit 0
fi

# Check permissions file
if [ ! -f "$SENA_TOOL_PERMISSIONS" ]; then
    echo "allow"
    exit 0
fi

# Check if tool is enabled in permissions file
if command -v jq &> /dev/null; then
    ENABLED=$(jq -r ".tools.\"$TOOL_NAME\".enabled // true" "$SENA_TOOL_PERMISSIONS" 2>/dev/null)
    if [ "$ENABLED" = "false" ]; then
        echo "deny"
    else
        echo "allow"
    fi
else
    # No jq, allow by default
    echo "allow"
fi

exit 0
