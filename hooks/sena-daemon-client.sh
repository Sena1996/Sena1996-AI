#!/bin/bash
# SENA Daemon Client - Rust Edition
# Provides simple interface to SENA Rust binary operations

SENA_BIN="${SENA_HOME:-$(command -v sena 2>/dev/null || echo "$HOME/.local/bin/sena")}"

# Check if SENA binary exists
is_sena_available() {
    [ -x "$SENA_BIN" ]
}

# Make RPC-style call to SENA binary
sena_call() {
    local method="$1"
    local params="$2"

    if ! is_sena_available; then
        echo '{"error": "sena_binary_not_found"}' >&2
        return 1
    fi

    case "$method" in
        "detect_format")
            echo "$params" | "$SENA_BIN" detect-format 2>/dev/null
            ;;
        "apply_format")
            echo "$params" | "$SENA_BIN" apply-format 2>/dev/null
            ;;
        "check_always_on")
            "$SENA_BIN" status --always-on 2>/dev/null
            ;;
        "health_check")
            "$SENA_BIN" health 2>/dev/null
            ;;
        *)
            echo '{"error": "unknown_method"}' >&2
            return 1
            ;;
    esac
}

# Detect format for user input
detect_format() {
    local user_input="$1"
    sena_call "detect_format" "$user_input"
}

# Apply format to user input
apply_format() {
    local user_input="$1"
    local format_type="$2"

    if [ -n "$format_type" ]; then
        echo "$user_input" | "$SENA_BIN" apply-format --type "$format_type" 2>/dev/null
    else
        echo "$user_input" | "$SENA_BIN" apply-format 2>/dev/null
    fi
}

# Check if SENA always-on mode is active
check_always_on() {
    if [ -f "$HOME/.claude/.sena_always_on" ]; then
        echo "true"
    else
        echo "false"
    fi
}

# Health check
health_check() {
    if is_sena_available; then
        "$SENA_BIN" health 2>/dev/null || echo '{"status": "binary_error"}'
    else
        echo '{"status": "binary_not_found"}'
    fi
}

# Export functions for sourcing
export -f is_sena_available
export -f sena_call
export -f detect_format
export -f apply_format
export -f check_always_on
export -f health_check

# If called directly (not sourced), execute command
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    case "$1" in
        is_running|is_available)
            if is_sena_available; then
                echo "SENA binary is available"
                exit 0
            else
                echo "SENA binary is NOT available"
                exit 1
            fi
            ;;
        detect_format)
            detect_format "$2"
            ;;
        apply_format)
            apply_format "$2" "$3"
            ;;
        check_always_on)
            check_always_on
            ;;
        health)
            health_check
            ;;
        *)
            echo "Usage: $0 {is_available|detect_format|apply_format|check_always_on|health}"
            echo ""
            echo "SENA Daemon Client - Rust Edition"
            echo "  Interfaces with SENA binary at: $SENA_BIN"
            exit 1
            ;;
    esac
fi
