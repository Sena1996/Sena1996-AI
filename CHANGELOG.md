# Changelog

All notable changes to SENA Controller will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [11.0.2] - 2025-11-27

### Added
- Session name support in `tell` and `task` commands (use name instead of ID)
- Auto-check inbox on prompt submit via hook
- `resolve_session()` method to find session by name or ID

### Changed
- `sena tell Android "message"` now works (no need for session ID)
- `sena task new "task" --to Android` now works (no need for session ID)
- Hook shows unread messages at start of each prompt

## [11.0.1] - 2025-11-26

### Added
- Custom CLI command names via symlinks
- `command` field in `UserConfig` for custom command name
- Setup wizard creates symlink: `~/.local/bin/{command} -> ~/.local/bin/sena`
- Claude Code auto-approves custom commands (no bash prompts)

### Changed
- Setup script now asks for custom command name
- Permissions configured for both `sena` and custom command

## [11.0.0] - 2025-11-26

### Added
- User branding configuration via `~/.sena/config.toml`
- Configurable name, emoji, and prefix for all outputs
- Global config accessor pattern using `OnceLock` for thread-safe lazy initialization
- `SenaConfig::brand()`, `SenaConfig::brand_title()` helper methods
- `whoami` crate for system username detection

### Changed
- Session stale timeout increased from 60 seconds to 24 hours
- `sena session list` now uses `get_active()` for consistency with `sena who`
- All `from_str` methods renamed to `parse` for clarity (SessionRole, TaskPriority, TaskStatus, DomainAgentType)
- Replaced `map_or(true, ...)` with `is_none_or(...)` for cleaner code
- Replaced `vec![]` with array literals where appropriate
- Collapsible if statements combined for cleaner code

### Fixed
- Session management: `sena who` and `sena session list` now show same sessions
- Silent error ignoring: Replaced `.ok()` with proper `?` operator and `if let Err(e)` patterns
- Redundant field names in struct initialization
- Unused imports and variables
- Dead code warnings with `#[allow(dead_code)]` where intentional
- Needless borrows in `serde_json::from_str` calls
- Useless `format!` calls replaced with string literals
- `format!` inside `println!` combined into single call

### Removed
- Manual `Default` implementation for `SenaConfig` (now derived)

## [10.0.6] - 2025-11-26

### Added
- System username as default when user skips name input
- Session naming support with `--name` flag

### Changed
- Auto-approve SENA commands (no bash prompts)
- Full hooks configuration in settings.json

## [10.0.5] - 2025-11-26

### Fixed
- MCP Server configuration
- Full hooks setup

## [10.0.4] - 2025-11-26

### Added
- Session naming support

## [10.0.3] - 2025-11-26

### Changed
- Auto-approve SENA commands without bash prompts

## [9.0.4] - 2025-11-26

### Changed
- Zero `unwrap()` in production code
- Lazy static regex initialization with `once_cell`
- 191 tests passing

## [9.0.3] - 2025-11-26

### Added
- CLAUDE.md with 50 elite coding rules
- Self-reminder rule (Rule 0) for instruction persistence

## [9.0.2] - 2025-11-26

### Added
- Full CLI implementation for Knowledge, Intelligence & Evolution systems
- Knowledge commands: search, list, stats
- Intelligence commands: think, agent
- Evolution commands: evolve, learn, optimize, patterns
- Feedback command

## [9.0.1] - 2025-11-25

### Added
- Configuration file support (`~/.sena/config.toml`)
- Expanded `SenaError` type with IO and Serialization variants

### Changed
- Robust error handling (eliminated 127 `unwrap()` calls)
- Clean `RwLock` handling with descriptive `expect()` messages
- 195 tests passing

## [9.0.0] - 2025-11-25

### Added
- Production ready status

## [8.1.0] - 2025-11-25

### Changed
- Single global VERSION constant from Cargo.toml
- Removed all comments from source files (clean code philosophy)
- Unlimited Extended Thinking (no token limits)
- Unlimited Pattern Learning (no pattern count limits)
- 191 tests passing

## [8.0.0] - 2025-11-25

### Added
- Knowledge System with multi-level memory
- Reasoning frameworks (First Principles, 5 Whys, Decision Matrix)
- Security patterns library (OWASP, Auth, Crypto, Secure Coding)
- Performance patterns library (Algorithm optimization, Caching, N+1)
- Architecture patterns library (SOLID, Design Patterns, DDD, CQRS)
- Intelligence System with extended thinking engine
- Specialized sub-agents (Security, Performance, Architecture)
- Multi-model routing (Fast/Balanced/Powerful)
- Autonomous self-activating skills
- Evolution System with pattern learner
- Self-optimizer for quality/speed/accuracy/satisfaction
- Feedback loop with sentiment analysis

## [7.0.0] - 2025-11-25

### Added
- Multi-session collaboration (Android/Web/Backend/IoT)
- Unix socket server for real-time IPC
- Task management across sessions
- Inter-session messaging
- File conflict detection
- CRDT state synchronization

## [6.0.0] - 2025-11-25

### Added
- Live ANSI progress bars
- Real-time terminal updates
- Enhanced output formatting

## [5.0.0] - 2025-11-25

### Added
- Complete rewrite from Python to Rust
- 7 Ancient Wisdom Layers
- MCP Server & Claude Code Hooks
- 3MB native binary
