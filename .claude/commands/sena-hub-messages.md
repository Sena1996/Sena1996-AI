---
description: Hub messaging and broadcasting
allowed-tools: Bash(sena hub:*), Bash(sena:*)
argument-hint: [messages|tell <name> <msg>|broadcast <msg>]
---

Manage SENA Hub messaging for cross-session communication.

## Available Commands

### View all hub messages
```bash
sena hub messages
```

### Send message to specific session
```bash
sena hub tell <session-name> "<message>"
```

### Broadcast to all sessions
```bash
sena hub broadcast "<message>"
```

## Examples

```bash
# Check inbox
sena hub messages

# Send to specific session
sena hub tell "Backend-Dev" "API refactor complete"

# Broadcast announcement
sena hub broadcast "Starting deployment in 5 minutes"
```

## Cross-Hub Messaging

For federated hubs, use the `@HubName:SessionName` syntax:

```bash
sena hub tell "@RemoteHub:Frontend" "Integration ready"
```

## Features

- Real-time message delivery
- Cross-session communication
- Hub federation support
- Message persistence

Run the appropriate command based on user's request.
