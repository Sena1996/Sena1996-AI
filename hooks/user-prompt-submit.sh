#!/bin/bash
# SENA Controller - Minimal Token Hook
# Optimized for minimal context injection

INPUT=$(cat)
USER_PROMPT=$(echo "$INPUT" | python3 -c "import sys,json;print(json.load(sys.stdin).get('prompt',''))" 2>/dev/null || echo "")

# Only inject context for EXPLICIT sena commands
if echo "$USER_PROMPT" | grep -qiE '^sena\s|/sena-'; then
    echo "SENA: Run via Bash(sena <cmd>) - pre-approved"
fi

exit 0
