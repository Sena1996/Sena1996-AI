# Changelog

All notable changes to Sena1996 AI Tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [12.0.0] - 2025-11-27

### Added
- **Multi-AI Provider Integration** - Support for Claude, OpenAI, Gemini, Ollama, and Mistral
- **AI-to-AI Collaboration** - Multiple AI agents working together via `sena-collab` crate
- **Consensus Voting System** - Democratic decision-making between AI agents
  - Unanimous, Majority, SuperMajority, Weighted voting strategies
  - Vote weight based on expertise scores
  - Proposal lifecycle management
- **Specialist Routing** - Automatic task delegation based on domain expertise
  - Domains: CodeGeneration, CodeReview, Security, Performance, Architecture, Testing, Documentation, NaturalLanguage, Creative, DataAnalysis, Mathematics, Research
  - Strategies: BestMatch, RoundRobin, LeastLoaded, Random
- **Tauri 2.0 Desktop Application** - Cross-platform GUI
  - React + TypeScript frontend
  - Provider management dashboard
  - Collaboration session interface
  - System health monitoring
- **sena-providers crate** - Provider abstraction layer
  - Unified API across all providers
  - Fallback chain support
  - Cost optimization options
- **sena-collab crate** - Collaboration framework
  - Session management
  - Message routing
  - Agent permissions

### Changed
- Restructured into Cargo workspace with multiple crates
- Updated README.md with v12 features
- Complete rewrite of USAGE.md
- Enhanced error handling with proper Result types

### Technical
- 280 tests passing
- Zero clippy warnings
- SENA1996-AI Elite Standards enforced

---

## [11.0.2] - 2025-11-26

### Added
- CLAUDE.md configuration for elite coding standards
- SENA context hook for Claude Code integration
- USAGE.md documentation

### Fixed
- Setup script no longer deletes Claude credentials
- Bash permission format updated per Claude Code docs

---

## [11.0.0] - 2025-11-25

### Added - Network Collaboration
- **Network Module** - Full TCP networking for cross-machine collaboration
- **mDNS Discovery** - Automatic peer discovery using `_sena._tcp.local.` service
- **TLS Encryption** - Self-signed certificates for secure communication
- **Token Authentication** - Time-limited authorization tokens with expiry
- **Peer Management** - Add, authorize, connect, revoke peers
- **Network Commands**:
  - `sena network start/stop/status/info/set-name`
  - `sena peer list/add/authorize/connect/revoke/ping`
  - `sena discover` - Find peers on local network

### Added - Custom Command System
- Custom CLI command names via symlinks (jarvis, lucy, etc.)
- Setup wizard creates symlink: `~/.local/bin/{command} -> ~/.local/bin/sena`
- All outputs use user-configured branding

---

## [10.0.0] - 2025-11-20

### Added - Session Collaboration
- Multi-session collaboration (Android/Web/Backend/IoT roles)
- Unix socket server for real-time IPC
- Task management across sessions
- Inter-session messaging
- `sena tell` and `sena inbox` commands

---

## [9.0.0] - 2025-11-15

### Added - Intelligence & Knowledge
- **Knowledge System** - 47 patterns across domains
  - Reasoning frameworks (First Principles, 5 Whys, Decision Matrix)
  - Security patterns (OWASP Top 10, Auth, Crypto)
  - Performance patterns
  - Architecture patterns (SOLID, Design Patterns, DDD)
- **Intelligence System** - Extended thinking engine
  - Thinking depths: Quick, Standard, Deep, Maximum
- **Evolution System** - Pattern learning and self-optimization
- **7 Ancient Wisdom Layers** - Truth-embedded architecture

---

## [8.0.0] - 2025-11-10

### Added - Claude Integration
- MCP server for Claude Desktop
- Claude Code hooks (UserPromptSubmit)
- Domain agents (Backend, iOS, Android, Web, IoT)

---

## [7.0.0] - 2025-11-05

### Added - Initial Release
- Core CLI functionality
- Health monitoring system
- Configuration management
- Rust implementation (~3.5MB binary)

---

## Release Types

| Type | Version | Description |
|------|---------|-------------|
| Major | X.0.0 | Breaking changes or major features |
| Minor | X.Y.0 | New features, backward compatible |
| Patch | X.Y.Z | Bug fixes, backward compatible |

---

## Links

- [GitHub Releases](https://github.com/Sena1996/Sena1996-AI/releases)
- [Documentation](https://github.com/Sena1996/Sena1996-AI/blob/main/USAGE.md)
- [Issues](https://github.com/Sena1996/Sena1996-AI/issues)

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                   Sena1996 AI Tool 🦁                        ║
║                       v12.0.0                                ║
║                                                              ║
║         Make Your AI Collaborative and Smarter™             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
