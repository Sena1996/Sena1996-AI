# Sena1996 AI Tool 🦁

## Make Your AI Collaborative and Smarter™

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://www.rust-lang.org/)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-blue)](https://modelcontextprotocol.io/)
[![Claude Code](https://img.shields.io/badge/Claude_Code-Hooks-green)](https://github.com/Sena1996/Sena1996-AI)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

---

## Our Ideology

**Make Your AI Collaborative and Smarter** - This is the core philosophy behind Sena1996 AI Tool. We believe AI assistants should:

- **Collaborate** across multiple sessions, machines, and networks
- **Learn** from patterns and continuously improve
- **Adapt** to your personal workflow with custom branding
- **Connect** with other AI instances over local networks
- **Think** deeply with multi-layered analysis inspired by ancient wisdom

---

## What is Sena1996 AI Tool?

Sena1996 AI Tool (SENA) is a powerful controller that enhances AI assistants like Claude. It provides:

### Core Features

| Feature | Description |
|---------|-------------|
| **Network Collaboration** | Connect multiple SENA instances across WiFi/LAN |
| **Custom Command Names** | Use `jarvis`, `lucy`, or any custom name |
| **Multi-Session Hub** | Collaborate across multiple Claude windows |
| **Cross-Machine Messaging** | Send messages between AI instances |
| **Peer Discovery** | Automatic mDNS/Bonjour peer finding |
| **TLS Encryption** | Secure communication with certificates |
| **Token Authentication** | Time-limited authorization tokens |

### Intelligence Systems

| System | Description |
|--------|-------------|
| **Knowledge System** | 47 patterns across reasoning, security, performance, architecture |
| **Intelligence System** | Extended thinking with Quick/Standard/Deep/Maximum depths |
| **Evolution System** | Pattern learning and self-optimization |
| **7 Ancient Wisdom Layers** | Truth-embedded architecture |
| **Domain Agents** | Backend, iOS, Android, Web, IoT specialized analysis |

---

## Network Collaboration

### Cross-Machine AI Collaboration

Connect multiple SENA instances across your local network:

```bash
# Start network server
sena network start --name "My Workstation"

# Discover peers on network
sena discover

# Add and authorize a peer
sena peer add 192.168.1.100 --name "MacBook"
sena peer authorize <peer-id>

# Connect from another machine
sena peer connect 192.168.1.50 --token <auth-token>
```

### Network Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  SENA Network Collaboration                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐         mDNS Discovery        ┌──────────┐│
│  │  Peer A     │◄─────────────────────────────►│  Peer B  ││
│  │  (SENA)     │    _sena._tcp.local.         │  (JARVIS)││
│  └──────┬──────┘                              └────┬──────┘│
│         │            TCP + TLS (Port 9876)        │       │
│         └─────────────[AUTH_TOKEN]────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Custom Command Names

Personalize SENA with your own command name:

```bash
./setup.sh
# Choose custom command: jarvis, lucy, or any name

# Now use your custom command
jarvis health
jarvis network status
jarvis peer list
```

All outputs adapt to your branding:
```
╔══════════════════════════════════════════════════════════════╗
║                    JARVIS 🤖 HEALTH STATUS                    ║
╚══════════════════════════════════════════════════════════════╝
```

---

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/Sena1996/Sena1996-AI.git
cd Sena1996-AI

# Run setup wizard
./setup.sh

# Or manual install
cargo build --release
cp target/release/sena ~/.local/bin/
```

### Configuration

Create `~/.sena/config.toml`:
```toml
[user]
name = "YourName"
emoji = "🦁"
prefix = "SENA"
command = "sena"

[general]
log_level = "info"

[output]
color = true
unicode = true
```

---

## CLI Commands

### Network Collaboration
```bash
sena network start              # Start network server
sena network stop               # Stop server
sena network status             # Show status
sena network info               # Show peer ID and name
sena network set-name "Name"    # Set display name
sena discover                   # Discover peers
sena peer list                  # List known peers
sena peer add <ip> --name "X"   # Add peer manually
sena peer authorize <id>        # Generate auth token
sena peer connect <ip> --token  # Connect with token
```

### Session Management
```bash
sena session start --name 'MySession'   # Start session
sena session list                       # List sessions
sena who                                # Who's online
sena tell <name> "message"              # Send message
sena inbox                              # Check inbox
```

### Domain Agents
```bash
sena backend map <code>         # API mapping
sena ios ui <code>              # SwiftUI analysis
sena android lifecycle <code>   # Lifecycle check
sena web audit <code>           # Security audit
sena iot protocol <code>        # Protocol analysis
```

### Intelligence System
```bash
sena think "question"           # Quick analysis
sena think --depth deep "?"     # Deep analysis
sena agent security <code>      # Security agent
sena agent performance <code>   # Performance agent
```

### Health & Metrics
```bash
sena health                     # Quick health check
sena health --detailed          # Detailed report
sena metrics                    # Full metrics
sena evolve stats               # Evolution stats
```

---

## 7 Ancient Wisdom Layers

| Layer | Name | Inspired By | Principle |
|-------|------|-------------|-----------|
| 0 | First Principles | Eratosthenes (240 BCE) | Understand WHY before building |
| 1 | Constraint-as-Feature | Persian Qanats (3000+ yrs) | Treat limitations as features |
| 2 | Negative Space | Sushruta (600 BCE) | Define failure before success |
| 3 | Relationship Model | Mayan Mathematics | Store connections, not just values |
| 4 | Self-Healing | Roman Concrete (2000+ yrs) | Embed repair in damage pathways |
| 5 | Harmony Validation | Antikythera (150 BCE) | Ensure model mirrors reality |
| 6 | Millennium Test | All Ancient Wisdom | Build for 1,000+ years |

---

## Claude Integration

### Claude Desktop (MCP Server)

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "sena": {
      "command": "/path/to/sena",
      "args": ["mcp"]
    }
  }
}
```

### Claude Code (Hooks)

Add to `~/.claude/settings.json`:
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "/path/to/sena hook user-prompt-submit"
      }
    ]
  },
  "permissions": {
    "allow": [
      "Bash(sena *)",
      "Bash(./target/release/sena *)"
    ]
  }
}
```

---

## Credits

### Created By

**Sena1996** - Creator, Lead Developer, and Maintainer
- GitHub: [@Sena1996](https://github.com/Sena1996)
- Vision: Make AI Collaborative and Smarter

### AI Development Partner

**Claude (Anthropic)** - AI Pair Programming Partner
- Assisted with architecture, implementation, and code review
- Powered by Claude Opus 4.5

### Technologies

- **MCP Protocol**: [Anthropic PBC](https://www.anthropic.com/)
- **Rust Language**: [rust-lang.org](https://www.rust-lang.org/)
- **mDNS Discovery**: mdns-sd crate
- **TLS**: rustls + rcgen

---

## Trademark

**Sena1996™** and the **SENA 🦁** logo are trademarks of Sena1996.

"Make Your AI Collaborative and Smarter" is the registered tagline of Sena1996 AI Tool.

---

## License

MIT License

Copyright (c) 2025 Sena1996

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                   Sena1996 AI Tool 🦁                        ║
║                                                              ║
║         Make Your AI Collaborative and Smarter™             ║
║                                                              ║
║         Created by Sena1996 • AI Partner: Claude            ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
