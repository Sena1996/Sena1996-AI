#!/bin/bash
# SENA Controller - Response Enforcer Hook
# Validates Claude's response for SENA compliance

SENA_BIN="${SENA_HOME:-$HOME/AI/Sena1996-AI}/target/release/sena"

# Read response from stdin
INPUT=$(cat)

# Extract response text
RESPONSE=$(echo "$INPUT" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('response', ''))" 2>/dev/null || echo "$INPUT")

# ============================================================
# SENA ALWAYS-ON MODE: Verify prefix if enabled
# ============================================================

if [ -f "$HOME/.claude/.sena_always_on" ]; then
    # Check if response starts with SENA prefix
    if ! echo "$RESPONSE" | head -5 | grep -q "SENA"; then
        echo ""
        echo "═══════════════════════════════════════════════════════════════════"
        echo "⚠️  SENA COMPLIANCE WARNING"
        echo "═══════════════════════════════════════════════════════════════════"
        echo "Response should start with: **SENA 🦁**"
        echo "Always-on mode is ACTIVE."
        echo "═══════════════════════════════════════════════════════════════════"
    fi
fi

# ============================================================
# Use Rust binary for response validation
# ============================================================

if [ -x "$SENA_BIN" ]; then
    echo "$INPUT" | "$SENA_BIN" hook assistant-response 2>/dev/null
fi

# Always allow response to continue
exit 0
