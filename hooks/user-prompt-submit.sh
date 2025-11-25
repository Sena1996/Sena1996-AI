#!/bin/bash
# SENA Controller - User Prompt Submit Hook
# Uses Sena1996-AI Rust binary for processing

SENA_BIN="${SENA_HOME:-$HOME/AI/Sena1996-AI}/target/release/sena"

# Read JSON input from stdin
INPUT=$(cat)

# Extract the prompt field from JSON
USER_PROMPT=$(echo "$INPUT" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('prompt', ''))" 2>/dev/null || echo "")

# ============================================================
# SENA ALWAYS-ON MODE: Check if persistent SENA mode is enabled
# ============================================================

if [ -f "$HOME/.claude/.sena_always_on" ]; then
    cat <<'SENA_ALWAYS_ON'

═══════════════════════════════════════════════════════════════════
🦁🦁🦁 SENA ALWAYS-ON MODE ACTIVE 🦁🦁🦁
═══════════════════════════════════════════════════════════════════

🔴🔴🔴 CRITICAL SYSTEM RULE - READ THIS FIRST 🔴🔴🔴

BEFORE you respond to the user's message, you MUST:

1. Start your ENTIRE response with exactly: **SENA 🦁**

   DO NOT skip this. DO NOT forget this. DO NOT ignore this.

   The VERY FIRST LINE of your response must be: **SENA 🦁**

2. After the prefix, provide the actual response content

═══════════════════════════════════════════════════════════════════
EXAMPLE OF CORRECT RESPONSE:
═══════════════════════════════════════════════════════════════════
**SENA 🦁**

[your actual response content here...]

═══════════════════════════════════════════════════════════════════
THIS IS NON-NEGOTIABLE. SEE CLAUDE.md RULE 0.
═══════════════════════════════════════════════════════════════════

SENA_ALWAYS_ON
fi

# ============================================================
# RULE 6: AUTO PROGRESS INJECTION
# ============================================================

if echo "$USER_PROMPT" | grep -iE '\b(search|find|scan|check|analyze|all|every|multiple|files|read.*files|write.*files)\b' > /dev/null 2>&1; then
    echo ""
    echo "┌──────────────────────────────────────────────────────────────┐"
    echo "│ Processing request  [🦁░░░░░░░░░░░░░░░░░░░] 0% - Starting   │"
    echo "└──────────────────────────────────────────────────────────────┘"
    echo ""
fi

# ============================================================
# RULE ENFORCEMENT: Check for SENA trigger words using Rust binary
# ============================================================

if [ -x "$SENA_BIN" ]; then
    # Use Rust binary for hook processing
    echo "$INPUT" | "$SENA_BIN" hook user-prompt-submit 2>/dev/null
else
    # Fallback: Manual trigger detection

    # Check for why/how/explain triggers (RULE 2)
    if echo "$USER_PROMPT" | grep -iE '\b(why|how|explain|what causes|what makes|how come)\b' > /dev/null 2>&1; then
        echo ""
        echo "═══════════════════════════════════════════════════════════════════"
        echo "🔴 RULE 2 AUTO-TRIGGER: Brilliant Thinking Format Required"
        echo "═══════════════════════════════════════════════════════════════════"
        echo "Use SENA Brilliant Thinking format with Unicode box-drawing."
        echo "═══════════════════════════════════════════════════════════════════"
    fi

    # Check for table triggers (RULE 1)
    if echo "$USER_PROMPT" | grep -iE '\b(table|tabular|tabular format|in table form)\b' > /dev/null 2>&1; then
        echo ""
        echo "═══════════════════════════════════════════════════════════════════"
        echo "🔴 RULE 1 AUTO-TRIGGER: Table Format Required"
        echo "═══════════════════════════════════════════════════════════════════"
        echo "Generate Unicode table directly. NO markdown tables."
        echo "═══════════════════════════════════════════════════════════════════"
    fi

    # Check for fact verification triggers (RULE 3)
    if echo "$USER_PROMPT" | grep -iE '\b(is .+ true|fact check|verify that|confirm that|true or false)\b' > /dev/null 2>&1; then
        echo ""
        echo "═══════════════════════════════════════════════════════════════════"
        echo "🔴 RULE 3 AUTO-TRIGGER: Truth Verification Format Required"
        echo "═══════════════════════════════════════════════════════════════════"
        echo "Use SENA Truth Verification format."
        echo "═══════════════════════════════════════════════════════════════════"
    fi

    # Check for code analysis triggers (RULE 4)
    if echo "$USER_PROMPT" | grep -iE '\b(analyze|review|check|examine).*(code|script|function|program)|code.*(review|analysis|quality)\b' > /dev/null 2>&1; then
        echo ""
        echo "═══════════════════════════════════════════════════════════════════"
        echo "🔴 RULE 4 AUTO-TRIGGER: Code Analysis Format Required"
        echo "═══════════════════════════════════════════════════════════════════"
        echo "Use SENA Code Analysis format."
        echo "═══════════════════════════════════════════════════════════════════"
    fi
fi

# Exit with 0 to allow prompt to continue
exit 0
