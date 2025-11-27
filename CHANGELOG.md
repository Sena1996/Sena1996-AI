# Changelog

All notable changes to Sena1996 AI Tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

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
- **Dynamic Peer Names** - Peer name derived from user config prefix
- **Slash Commands Generation** - Setup generates custom slash commands

### Added - Custom Command System
- Custom CLI command names via symlinks
- `command` field in `UserConfig` for custom command name
- Setup wizard creates symlink: `~/.local/bin/{command} -> ~/.local/bin/sena`
- Claude Code auto-approves custom commands
- All outputs use user-configured branding (prefix, emoji)

### Added - Session Messaging
- Session name support in `tell` and `task` commands
- Auto-check inbox on prompt submit via hook
- `resolve_session()` method to find session by name or ID

### Changed
- Session stale timeout increased from 60 seconds to 24 hours
- `sena session list` now uses `get_active()` for consistency
- All `from_str` methods renamed to `parse` for clarity
- Setup script preserves existing configs with backups

### Fixed
- Setup script no longer deletes Claude Desktop credentials
- Silent error ignoring replaced with proper `?` operator
- Zero clippy warnings

---

## Previous Releases

### User Customization Edition
- User branding configuration via `~/.sena/config.toml`
- Configurable name, emoji, and prefix for all outputs
- Global config accessor pattern using `OnceLock`

### Session Collaboration Edition
- Multi-session collaboration (Android/Web/Backend/IoT)
- Unix socket server for real-time IPC
- Task management across sessions
- Inter-session messaging
- File conflict detection

### Knowledge & Intelligence Edition
- Knowledge System with multi-level memory
- Reasoning frameworks (First Principles, 5 Whys, Decision Matrix)
- Security patterns library (OWASP, Auth, Crypto)
- Performance patterns library
- Architecture patterns library (SOLID, Design Patterns, DDD)
- Intelligence System with extended thinking engine
- Specialized sub-agents (Security, Performance, Architecture)
- Evolution System with pattern learner

### Rust Rewrite Edition
- Complete rewrite from Python to Rust
- 7 Ancient Wisdom Layers
- MCP Server & Claude Code Hooks
- Native binary (~3.5MB)

---

## Credits

**Sena1996™** - Creator and Lead Developer
**Claude (Anthropic)** - AI Development Partner

---

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                   Sena1996 AI Tool 🦁                        ║
║                                                              ║
║         Make Your AI Collaborative and Smarter™             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```
