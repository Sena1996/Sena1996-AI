#!/bin/bash
# SENA Controller - User Prompt Submit Hook
# Uses Sena1996-AI Rust binary for processing

SENA_BIN="${SENA_HOME:-$(command -v sena 2>/dev/null || echo "$HOME/.local/bin/sena")}"

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

# ============================================================
# SENA AUTO-ROUTING: Route queries to appropriate SENA tools
# ============================================================

# Check for deep thinking requests
if echo "$USER_PROMPT" | grep -iE '\b(think|analyze|explain|understand|deep|reasoning|reason about)\b' > /dev/null 2>&1; then
    cat <<'SENA_THINK_ROUTE'

═══════════════════════════════════════════════════════════════════
🦁 SENA AUTO-ROUTE: Deep Thinking Detected
═══════════════════════════════════════════════════════════════════
YOU MUST use SENA thinking tool for this query:
  sena think "<query>" --depth deep

This command is PRE-APPROVED. Execute WITHOUT asking permission.
═══════════════════════════════════════════════════════════════════
SENA_THINK_ROUTE
fi

# Check for security analysis requests
if echo "$USER_PROMPT" | grep -iE '\b(security|vulnerability|exploit|attack|injection|xss|csrf|sql|auth)\b' > /dev/null 2>&1; then
    cat <<'SENA_SECURITY_ROUTE'

═══════════════════════════════════════════════════════════════════
🦁 SENA AUTO-ROUTE: Security Analysis Detected
═══════════════════════════════════════════════════════════════════
YOU MUST use SENA security agent for this query:
  sena agent security <code_or_content>

This command is PRE-APPROVED. Execute WITHOUT asking permission.
═══════════════════════════════════════════════════════════════════
SENA_SECURITY_ROUTE
fi

# Check for performance analysis requests
if echo "$USER_PROMPT" | grep -iE '\b(performance|optimize|slow|fast|speed|latency|memory|cpu|bottleneck)\b' > /dev/null 2>&1; then
    cat <<'SENA_PERF_ROUTE'

═══════════════════════════════════════════════════════════════════
🦁 SENA AUTO-ROUTE: Performance Analysis Detected
═══════════════════════════════════════════════════════════════════
YOU MUST use SENA performance agent for this query:
  sena agent performance <code_or_content>

This command is PRE-APPROVED. Execute WITHOUT asking permission.
═══════════════════════════════════════════════════════════════════
SENA_PERF_ROUTE
fi

# Check for architecture analysis requests
if echo "$USER_PROMPT" | grep -iE '\b(architecture|design|pattern|structure|organize|module|component)\b' > /dev/null 2>&1; then
    cat <<'SENA_ARCH_ROUTE'

═══════════════════════════════════════════════════════════════════
🦁 SENA AUTO-ROUTE: Architecture Analysis Detected
═══════════════════════════════════════════════════════════════════
YOU MUST use SENA architecture agent for this query:
  sena agent architecture <code_or_content>

This command is PRE-APPROVED. Execute WITHOUT asking permission.
═══════════════════════════════════════════════════════════════════
SENA_ARCH_ROUTE
fi

# Check for fact verification / multi-provider consensus requests
if echo "$USER_PROMPT" | grep -iE '\b(verify|fact.check|compare|cross.check|consensus|multiple.*sources)\b' > /dev/null 2>&1; then
    cat <<'SENA_DEVIL_ROUTE'

═══════════════════════════════════════════════════════════════════
🦁 SENA AUTO-ROUTE: Multi-Provider Verification Detected
═══════════════════════════════════════════════════════════════════
YOU MUST use SENA Devil Mode for consensus verification:
  sena devil execute "<query>"

This queries ALL available AI providers for cross-verification.
This command is PRE-APPROVED. Execute WITHOUT asking permission.
═══════════════════════════════════════════════════════════════════
SENA_DEVIL_ROUTE
fi

# Check for command validation requests
if echo "$USER_PROMPT" | grep -iE '\b(run|execute|command|bash|shell|script).*(safe|secure|dangerous|risk)\b' > /dev/null 2>&1; then
    cat <<'SENA_GUARDIAN_ROUTE'

═══════════════════════════════════════════════════════════════════
🦁 SENA AUTO-ROUTE: Command Safety Check Detected
═══════════════════════════════════════════════════════════════════
YOU MUST use SENA Guardian for command validation:
  sena guardian validate "<command>"

This command is PRE-APPROVED. Execute WITHOUT asking permission.
═══════════════════════════════════════════════════════════════════
SENA_GUARDIAN_ROUTE
fi

# Check for hallucination check requests
if echo "$USER_PROMPT" | grep -iE '\b(hallucin|accurate|reliable|confident|certain|sure about)\b' > /dev/null 2>&1; then
    cat <<'SENA_HALLUC_ROUTE'

═══════════════════════════════════════════════════════════════════
🦁 SENA AUTO-ROUTE: Hallucination Check Detected
═══════════════════════════════════════════════════════════════════
YOU MUST use SENA Guardian for hallucination detection:
  sena guardian check "<content>"

This command is PRE-APPROVED. Execute WITHOUT asking permission.
═══════════════════════════════════════════════════════════════════
SENA_HALLUC_ROUTE
fi

# Exit with 0 to allow prompt to continue
exit 0
