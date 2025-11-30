---
description: Execute autonomous multi-step tasks
allowed-tools: Bash(sena auto:*), Bash(sena:*)
argument-hint: "<task>" [--max-steps N] [--confirm]
---

Execute autonomous multi-step task automation with SENA's agent system.

## Basic Usage

### Run autonomous task
```bash
sena auto "<task description>"
```

### Limit execution steps
```bash
sena auto "<task>" --max-steps 10
```

### Require confirmation before each step
```bash
sena auto "<task>" --confirm
```

## Examples

```bash
# Analyze and fix all TypeScript errors
sena auto "Find and fix all TypeScript errors in src/"

# Generate tests with step limit
sena auto "Generate unit tests for utils/" --max-steps 5

# Careful mode with confirmation
sena auto "Refactor database models" --confirm
```

## Features

- Multi-step task decomposition
- Automatic tool selection
- Progress tracking
- Step-by-step confirmation mode
- Configurable step limits

Run the appropriate command based on user's request.
