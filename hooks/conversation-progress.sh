#!/bin/bash
# SENA Conversation Progress Hook - Rust Edition
# Shows progress bar for each conversation prompt

SENA_BIN="${SENA_HOME:-$(command -v sena 2>/dev/null || echo "$HOME/.local/bin/sena")}"

# Function to show progress for conversation
show_conversation_progress() {
    local prompt="$1"

    if [ -x "$SENA_BIN" ]; then
        # Start progress via Rust binary
        (
            "$SENA_BIN" progress --interactive --timeout 5 2>/dev/null &
            PROGRESS_PID=$!

            # Let it run for conversation processing
            sleep 5

            # Clean termination
            kill -TERM $PROGRESS_PID 2>/dev/null
        ) &
    fi
}

# Check if this is a conversation prompt
if [ -n "$CLAUDE_PROMPT" ]; then
    show_conversation_progress "$CLAUDE_PROMPT"
fi

# Continue with normal processing
exec "$@"
