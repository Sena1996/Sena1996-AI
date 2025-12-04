#!/bin/bash
# SENA Auto Progress Hook - Rust Edition
# Automatically shows progress for conversation processing

SENA_BIN="${SENA_HOME:-$(command -v sena 2>/dev/null || echo "$HOME/.local/bin/sena")}"

# Start progress display via Rust binary (non-blocking)
if [ -x "$SENA_BIN" ]; then
    # Use Rust binary for progress display
    (
        "$SENA_BIN" progress --auto --timeout 10 2>/dev/null &
        PROGRESS_PID=$!

        # Let it run for conversation processing
        sleep 10

        # Gracefully terminate
        kill -TERM $PROGRESS_PID 2>/dev/null
    ) &
fi

# Continue with original hook processing
exec "$@"
