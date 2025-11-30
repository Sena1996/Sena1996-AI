---
description: Analyze code using SENA specialized agents
allowed-tools: Bash(sena agent:*), Bash(sena:*)
argument-hint: [type] [code-or-file]
---

Analyze code using SENA's specialized agent system.

Available agent types:
- `security` - Vulnerability detection, OWASP, authentication
- `performance` - Optimization, efficiency, benchmarking
- `architecture` - Design patterns, SOLID principles, structure
- `general` - Multi-domain comprehensive analysis

Run the appropriate command:
```bash
sena agent $ARGUMENTS
```

Example usages:
- `sena agent security src/auth.rs`
- `sena agent performance "fn process()..."`
- `sena agent architecture src/`

Display the analysis results with recommendations to the user.
