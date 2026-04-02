# Goblin

<p align="center">
  <img src="assets/banner.png" alt="Goblin" width="100%">
</p>

<p align="center">
  <strong>The self-improving AI coding agent that gets smarter the more you use it.</strong>
</p>

<p align="center">
  <a href="https://discord.gg/goblin"><img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Discord"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License: MIT"></a>
</p>

---

**Goblin** is a chaotic-but-effective AI coding agent built on Rust for speed. It reads your codebase, writes code, fixes bugs, and — crucially — *learns* from every interaction. The more you work with Goblin, the smarter it gets about your projects, your patterns, and your preferences.

Think of it like hiring a chaotic imp who's actually really competent. Chaotic in personality, brilliant in execution.

## Features

| Feature | What it does |
|---------|--------------|
| **Self-Improving** | Goblin creates skills from experience and improves them during use |
| **Tiered Memory** | Remembers your projects, your style, your preferences across sessions |
| **Rust-Powered** | Blazing fast execution — compiled, no GC pauses, native speed |
| **MCP Integration** | Connect any Model Context Protocol server for extended capabilities |
| **Multi-Provider** | Use OpenAI, Anthropic, Nous Portal, OpenRouter, or your own endpoint |
| **Type-Safe Tools** | Exhaustive pattern matching means no missing tool handlers |
| **Semantic Search** | Understands your codebase, not just regex searches |

## Quick Start

```bash
# Install
curl -fsSL https://goblin.dev/cli | sh

# Start chatting
goblin

# Configure your AI provider
goblin model
```

## What Makes Goblin Different

Most AI coding tools are stateless — they forget everything after each conversation. Goblin *remembers*:

```
┌─────────────────────────────────────────────────────────────────┐
│                        GOBLIN BRAIN                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Session 1          Session 2          Session 3               │
│      │                   │                   │                   │
│      ▼                   ▼                   ▼                   │
│   Learns your      Remembers your      Gets even smarter         │
│   project structure project structure + style preferences        │
│      │                   │                   │                   │
│      └───────────────────┴───────────────────┘                   │
│                          │                                       │
│                          ▼                                       │
│              ┌─────────────────────┐                            │
│              │   SHARED MEMORY      │                            │
│              │  • Your patterns     │                            │
│              │  • Project quirks    │                            │
│              │  • Your preferences  │                            │
│              │  • Past solutions    │                            │
│              └─────────────────────┘                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Skills System

Goblin doesn't just use tools — it *creates new skills* from experience. After solving a complex problem, Goblin might save the pattern as a skill for future use. Skills self-improve during use.

### The DOJO

Daily review sessions where Goblin checks its own work:
- Did I follow my own rules?
- Any commits without proper documentation?
- Ops-heavy vs code-heavy balance?

Goblin patches its own rules when it finds patterns to fix.

## Architecture

```
goblin/
├── goblin_app/         # Main orchestrator - the agent loop
├── goblin_domain/      # Domain logic - tools, context, compact
├── goblin_services/    # External integrations - LLM, search
├── goblin_walker/      # Filesystem traversal & indexing
├── goblin_core/        # Memory system - tiered scopes
├── goblin_gateway/     # Multi-platform messaging
└── goblin_cron/        # Scheduled automations
```

Built on Rust for:
- **Speed**: Compiled native binary, no startup lag
- **Safety**: No GC pauses, memory-safe by default
- **Concurrency**: Async-first with tokio

## Comparison

| Tool | Speed | Self-Improving | Memory | Multi-Platform |
|------|-------|---------------|--------|----------------|
| **Goblin** | ⚡⚡⚡ | ✅ | ✅ | ✅ |
| Claude Code | ⚡⚡ | ❌ | ❌ | ❌ |
| Copilot | ⚡⚡ | ❌ | ❌ | ❌ |

## Configuration

```yaml
# goblin.yaml
model: "claude-3.5-sonnet"
goblin:
  memory:
    enabled: true
    scopes: [global, project, session]
  skills:
    auto_create: true
  dojo:
    morning_review: true
```

## Installation

```bash
# Via installer (recommended)
curl -fsSL https://goblin.dev/cli | sh

# Via cargo (requires Rust)
cargo install goblin

# Via nix
nix run github:yourname/goblin
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup.

```bash
git clone https://github.com/yourname/goblin.git
cd goblin
cargo build --release
cargo test
```

## License

MIT — see [LICENSE](LICENSE).

---

*Goblin. Chaotic. Smart. Gets the job done.*
