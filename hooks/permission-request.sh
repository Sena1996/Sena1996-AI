#!/bin/bash
# SENA Tool Permission Request Hook - Rust Edition
# Dynamically allows/denies tools based on SENA runtime preferences

SENA_BIN="${SENA_HOME:-$HOME/AI/Sena1996-AI}/target/release/sena"
SENA_TOOL_PERMISSIONS="$HOME/.claude/.sena_tool_permissions.json"

# Read JSON input from stdin if provided
if [ ! -t 0 ]; then
    INPUT=$(cat)
else
    INPUT=""
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
