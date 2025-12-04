#!/bin/bash
# SENA Controller - Post Tool Use Hook
# Processes tool results for clean output

SENA_BIN="${SENA_HOME:-$(command -v sena 2>/dev/null || echo "$HOME/.local/bin/sena")}"

# Read tool result from stdin
INPUT=$(cat)

# Use Rust binary if available
if [ -x "$SENA_BIN" ]; then
    echo "$INPUT" | "$SENA_BIN" hook tool-execution 2>/dev/null
fi

exit 0
