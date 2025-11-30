---
description: Manage SENA persistent memory system
allowed-tools: Bash(sena memory:*), Bash(sena:*)
argument-hint: [add|search|list|stats] [content/query]
---

Manage SENA's persistent memory for long-term knowledge storage.

## Available Commands

### Add a new memory
```bash
sena memory add "<content>"
```

### Search memories
```bash
sena memory search <query>
```

### List all memories
```bash
sena memory list
```

### Show memory statistics
```bash
sena memory stats
```

## Memory Types

| Type | Description |
|------|-------------|
| Fact | Verified information |
| Context | Contextual knowledge |
| Pattern | Recognized patterns |
| Preference | User preferences |

Run the appropriate command based on user's request.
