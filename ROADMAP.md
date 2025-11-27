# Sena1996 AI Tool - Product Roadmap

**Vision:** Universal AI Collaboration Platform - Make Your AI Collaborative and Smarter

---

## Current State Analysis

### What We Have
- CLI tool with 73 Rust source files (~29,000 LOC)
- Network collaboration (mDNS discovery, TLS, peer-to-peer)
- Claude Code integration (hooks, MCP server, slash commands)
- Specialized agents (Backend, iOS, Android, Web, IoT)
- Intelligence system (thinking depths, routing)
- Professional installer (setup.sh)

### What We're Missing
- GUI/Desktop application
- Multi-AI provider support
- CI/CD automation
- Cross-platform installers (DMG, EXE, AppImage)
- Production logging
- Integration tests
- Security scanning

---

## Phase 1: Foundation (Current Sprint)

### 1.1 Code Quality & CI/CD
| Task | Priority | Status |
|------|----------|--------|
| Fix broken doc tests | Critical | Pending |
| Add LICENSE (MIT) | Critical | Pending |
| Create GitHub Actions CI | Critical | Pending |
| Add integration tests | High | Pending |
| Enable logging (log/tracing) | High | Pending |
| Security scanning (cargo-audit) | High | Pending |
| Code coverage (tarpaulin) | Medium | Pending |
| Benchmarks | Low | Pending |

### 1.2 Project Structure
```
sena1996-ai/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml
│   │   ├── release.yml
│   │   └── security.yml
│   ├── ISSUE_TEMPLATE/
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── dependabot.yml
├── src/                    # Core CLI (existing)
├── sena-ui/               # NEW: Tauri desktop app
│   ├── src/
│   ├── src-tauri/
│   └── package.json
├── sena-providers/        # NEW: Multi-AI providers
│   ├── src/
│   │   ├── lib.rs
│   │   ├── claude.rs
│   │   ├── openai.rs
│   │   ├── gemini.rs
│   │   └── router.rs
│   └── Cargo.toml
├── tests/                 # Integration tests
├── benches/               # Performance benchmarks
├── examples/              # Usage examples
└── docs/                  # Additional documentation
```

---

## Phase 2: Multi-AI Provider Integration

### 2.1 Architecture Decision

**Recommended: Custom Implementation inspired by LiteLLM**

| Option | Pros | Cons |
|--------|------|------|
| LiteLLM-rs | Production ready, 100+ providers | External dependency |
| FlyLLM | Rust native, load balancing | Less mature |
| Custom | Full control, tailored to needs | Development time |

**Decision: Build custom `sena-providers` crate using traits**

### 2.2 Provider Interface Design

```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> Vec<ModelInfo>;

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn stream_chat(&self, request: ChatRequest) -> Result<ChatStream>;

    fn supports_tools(&self) -> bool;
    fn supports_vision(&self) -> bool;
    fn supports_streaming(&self) -> bool;
}

pub struct ProviderRouter {
    providers: HashMap<String, Box<dyn AIProvider>>,
    default_provider: String,
    fallback_chain: Vec<String>,
}
```

### 2.3 Supported Providers (Priority Order)

| Provider | API | Priority | Features |
|----------|-----|----------|----------|
| Anthropic Claude | claude.ai/api | P0 | Tools, Vision, Streaming |
| OpenAI GPT | api.openai.com | P0 | Tools, Vision, Streaming |
| Google Gemini | generativelanguage.googleapis.com | P1 | Tools, Vision, Streaming |
| Ollama (Local) | localhost:11434 | P1 | Local models, Privacy |
| Mistral | api.mistral.ai | P2 | Fast, Affordable |
| Groq | api.groq.com | P2 | Ultra-fast inference |
| DeepSeek | api.deepseek.com | P3 | Code-focused |
| Cohere | api.cohere.ai | P3 | Enterprise |

### 2.4 Configuration

```toml
# ~/.sena/providers.toml

[providers.claude]
enabled = true
api_key_env = "ANTHROPIC_API_KEY"
default_model = "claude-sonnet-4-20250514"
max_tokens = 8192

[providers.openai]
enabled = true
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4o"
max_tokens = 4096

[providers.gemini]
enabled = true
api_key_env = "GOOGLE_API_KEY"
default_model = "gemini-2.0-flash"

[providers.ollama]
enabled = true
base_url = "http://localhost:11434"
default_model = "llama3.2"

[routing]
default = "claude"
fallback = ["openai", "gemini", "ollama"]
cost_optimization = true
```

---

## Phase 3: Desktop Application (Tauri 2.0)

### 3.1 Technology Stack

| Component | Technology | Reason |
|-----------|------------|--------|
| Framework | Tauri 2.0 | Small binary, Rust backend |
| Frontend | React + TypeScript | Large ecosystem, familiar |
| Styling | Tailwind CSS | Rapid development |
| State | Zustand | Simple, performant |
| Build | Vite | Fast HMR |

### 3.2 Application Features

**Core Features:**
- Dashboard with AI provider status
- Chat interface with multi-AI support
- Session management
- Network peer visualization
- Settings & configuration

**Advanced Features:**
- AI comparison mode (same prompt, multiple AIs)
- Cost tracking per provider
- Prompt history & favorites
- Export/import sessions
- Custom prompt templates

### 3.3 UI Mockup Structure

```
┌─────────────────────────────────────────────────────────────┐
│  SENA 🦁  │ Dashboard │ Chat │ Network │ Settings    [─][□][×]│
├───────────┼─────────────────────────────────────────────────┤
│           │                                                 │
│ Providers │  ┌─────────────────────────────────────────┐   │
│ ─────────│  │                                         │   │
│ ● Claude  │  │         Chat Interface                  │   │
│ ● OpenAI  │  │                                         │   │
│ ○ Gemini  │  │  User: How do I implement...           │   │
│ ● Ollama  │  │                                         │   │
│           │  │  Claude: Here's how you can...          │   │
│ Sessions  │  │                                         │   │
│ ─────────│  │                                         │   │
│ > Current │  └─────────────────────────────────────────┘   │
│   Session1│                                                 │
│   Session2│  ┌─────────────────────────────────────────┐   │
│           │  │ [Type your message...]          [Send]  │   │
│ Peers     │  └─────────────────────────────────────────┘   │
│ ─────────│                                                 │
│ 👤 Peer1  │  Provider: [Claude ▼]  Model: [Sonnet ▼]      │
│ 👤 Peer2  │                                                 │
└───────────┴─────────────────────────────────────────────────┘
```

---

## Phase 4: Cross-Platform Distribution

### 4.1 Build Targets

| Platform | Format | Tool | Size Target |
|----------|--------|------|-------------|
| macOS | .dmg, .app | Tauri bundler | < 15 MB |
| Windows | .exe, .msi | Tauri + NSIS | < 15 MB |
| Linux | .AppImage, .deb | Tauri bundler | < 20 MB |

### 4.2 Release Automation

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build Tauri App
        uses: tauri-apps/tauri-action@v0
        with:
          tagName: v__VERSION__
          releaseName: 'SENA v__VERSION__'
          releaseBody: 'See CHANGELOG.md for details'
          releaseDraft: true
```

### 4.3 Installation Methods

| Method | Command/Action |
|--------|----------------|
| macOS DMG | Download, drag to Applications |
| macOS Homebrew | `brew install sena1996/tap/sena` |
| Windows Installer | Download .msi, run installer |
| Windows Scoop | `scoop install sena` |
| Linux AppImage | Download, chmod +x, run |
| Linux apt | `apt install sena` |
| Cargo | `cargo install sena1996-ai` |

---

## Phase 5: AI Collaboration Network

### 5.1 Vision: Universal AI Collaboration

```
┌─────────────────────────────────────────────────────────────┐
│                    SENA Collaboration Hub                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│    ┌─────────┐      ┌─────────┐      ┌─────────┐          │
│    │ Claude  │──────│  SENA   │──────│ ChatGPT │          │
│    │ Session │      │  Hub    │      │ Session │          │
│    └─────────┘      └────┬────┘      └─────────┘          │
│                          │                                  │
│         ┌────────────────┼────────────────┐                │
│         │                │                │                │
│    ┌─────────┐     ┌─────────┐     ┌─────────┐            │
│    │ Gemini  │     │ Ollama  │     │  Human  │            │
│    │ Session │     │ (Local) │     │  User   │            │
│    └─────────┘     └─────────┘     └─────────┘            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Collaboration Features

**Cross-AI Features:**
- Shared context across AI providers
- AI-to-AI task delegation
- Consensus voting on solutions
- Best-response selection
- Cost-optimized routing

**Human-AI Features:**
- Multi-user sessions
- Role-based permissions
- Real-time collaboration
- Audit logging
- Session recording

### 5.3 Protocol Design

```rust
pub enum CollaborationMessage {
    // Context Sharing
    ShareContext { context: String, from: ParticipantId },
    RequestContext { topic: String },

    // Task Delegation
    DelegateTask { task: Task, to: Vec<ParticipantId> },
    TaskResponse { task_id: TaskId, response: String },

    // Consensus
    ProposeAnswer { question_id: QuestionId, answer: String },
    VoteOnAnswer { answer_id: AnswerId, vote: Vote },

    // Control
    JoinSession { participant: Participant },
    LeaveSession { participant_id: ParticipantId },
}
```

---

## Timeline Overview

| Phase | Focus | Duration | Target |
|-------|-------|----------|--------|
| Phase 1 | Foundation | 2-3 weeks | CI/CD, Tests, Logging |
| Phase 2 | Multi-AI | 3-4 weeks | Provider abstraction |
| Phase 3 | Desktop UI | 4-6 weeks | Tauri app |
| Phase 4 | Distribution | 2-3 weeks | Installers |
| Phase 5 | Collaboration | 4-6 weeks | AI network |

---

## Technology Decisions Summary

| Category | Decision | Alternative Considered |
|----------|----------|----------------------|
| GUI Framework | Tauri 2.0 | Dioxus, Iced |
| Frontend | React + TypeScript | Svelte, Vue |
| Multi-AI | Custom trait-based | LiteLLM-rs, FlyLLM |
| Packaging | Tauri bundler | cargo-bundle |
| CI/CD | GitHub Actions | GitLab CI |
| Testing | cargo test + nextest | - |
| Coverage | cargo-tarpaulin | cargo-llvm-cov |

---

## Success Metrics

| Metric | Target |
|--------|--------|
| CLI binary size | < 10 MB |
| Desktop app size | < 20 MB |
| Startup time | < 500ms |
| Test coverage | > 70% |
| CI build time | < 5 min |
| Provider latency overhead | < 50ms |

---

## References

### GUI Frameworks
- [Tauri 2.0 Documentation](https://v2.tauri.app/)
- [Tauri GitHub](https://github.com/tauri-apps/tauri)
- [Awesome Tauri](https://github.com/tauri-apps/awesome-tauri)

### Multi-AI Integration
- [LiteLLM-rs](https://crates.io/crates/litellm-rs)
- [FlyLLM](https://github.com/rodmarkun/flyllm)
- [LiteLLM Python](https://www.litellm.ai/)

### Packaging
- [Tauri Bundler](https://crates.io/crates/tauri-bundler)
- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)

---

**Sena1996 AI Tool** - Make Your AI Collaborative and Smarter

*Created by Sena1996 with Claude AI*
