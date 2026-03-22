
<p align="center">
  <a href="https://github.com/dawsonblock/Electro/stargazers"><img src="https://img.shields.io/github/stars/dawsonblock/Electro?style=for-the-badge&color=F5A623&logo=github&logoColor=white" alt="Stars"></a>&nbsp;
  <a href="https://discord.gg/3ux2c5xz"><img src="https://img.shields.io/badge/Discord-Community-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Discord"></a>&nbsp;
  <img src="https://img.shields.io/badge/License-MIT-A3E635?style=for-the-badge" alt="MIT">&nbsp;
  <img src="https://img.shields.io/badge/v3.2.0-Stable-06B6D4?style=for-the-badge" alt="Version">&nbsp;
  <img src="https://img.shields.io/badge/Rust-1.82+-E34F26?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
</p>

<h1 align="center">Electro</h1>

<p align="center">
  Cloud-native AI agent runtime, written in Rust.<br>
  <sub>Deploy once. Runs forever. Thinks on a budget.</sub>
</p>

---

## What is Electro

Electro is a full agent runtime—not just an LLM wrapper. It treats the language model as a finite brain with a strict token budget, managing context building, tool execution loops, persistent memory, and multi-channel deployment from a single binary. It runs headless on a $5 VPS and stays up with four-layer panic resilience.

## Capabilities

- **Multi-channel deployment**: Telegram, Discord, Slack, CLI, or interactive TUI
- **6 AI providers**: Anthropic, OpenAI, Google Gemini, xAI Grok, OpenRouter, and ChatGPT via OAuth—no API key needed for Codex
- **Built-in tools**: Shell, vision browser (screenshot → click), file ops, web fetch, git, MCP client
- **λ-Memory**: Exponential decay memory with hash-based recall; 95% cross-session accuracy
- **Blueprints**: Structured, replayable recipes that capture learned procedures without extra LLM calls
- **Many Tems**: Stigmergic swarm intelligence for parallel task execution
- **Eigen-Tune**: Self-tuning distillation that trains local models from LLM interactions

## Quick Start

```bash
# Clone and build
git clone https://github.com/dawsonblock/Electro.git && cd Electro
cargo build --release
```

**Mode 1 — Interactive TUI**

```bash
./target/release/electro tui
```

First run launches an arrow-key setup wizard. Paste any supported API key—the provider is detected automatically.

**Mode 2 — Server (persistent channel bot)**

```bash
export TELEGRAM_BOT_TOKEN="your-token"
./target/release/electro start
```

Replace `TELEGRAM_BOT_TOKEN` with `DISCORD_BOT_TOKEN` or `SLACK_BOT_TOKEN` for those channels.

## State and Configuration

Electro stores state in `~/.electro/`:

- `electro.toml` — main configuration
- `memory/` — SQLite + Markdown memory backends
- `vault/` — ChaCha20-Poly1305 encrypted secrets
- `blueprints/` — saved procedure recipes
- `logs/` — structured tracing output

Run `electro config validate` to check your configuration. See [docs/configuration.md](docs/configuration.md) for the full schema.

## Security Model

- **Deny-by-default access control**: Empty allowlists block all users until explicitly provisioned
- **Workspace isolation**: All file operations sandboxed via `resolve_safe_path`—no path escapes
- **Network isolation**: Browser and web_fetch restricted to public web or explicit allowlist
- **Secrets at rest**: ChaCha20-Poly1305 vault with `vault://` URI scheme; AES-256-GCM one-time key encryption on first setup
- **Credential hygiene**: API keys auto-stripped from chat; secret output filtered on replies

See [docs/security.md](docs/security.md) for details.

## Documentation

| Guide | Description |
|-------|-------------|
| [docs/](docs/) | Full documentation index |
| [docs/configuration.md](docs/configuration.md) | Config schema, environment variables |
| [docs/security.md](docs/security.md) | Security architecture, threat model |
| [docs/channels.md](docs/channels.md) | Telegram, Discord, Slack setup |
| [docs/providers.md](docs/providers.md) | AI provider configuration |
| [docs/tools.md](docs/tools.md) | Built-in tools, MCP servers |
| [docs/memory.md](docs/memory.md) | λ-Memory, SQLite, Markdown backends |
| [docs/architecture.md](docs/architecture.md) | Deep dive on crates, traits, message flow |

Join the community at https://discord.gg/3ux2c5xz

## Workspace Layout

```
electro (binary)
├─ electro-core        — Traits, types, config, errors
├─ electro-agent       — Agentic core, λ-Memory, blueprints
├─ electro-hive        — Swarm intelligence, Many Tems
├─ electro-distill     — Eigen-Tune self-tuning
├─ electro-providers   — Anthropic, OpenAI, Gemini, Grok, OpenRouter
├─ electro-codex-oauth — ChatGPT Plus/Pro via OAuth PKCE
├─ electro-tui         — Interactive terminal UI
├─ electro-channels    — Telegram, Discord, Slack, CLI
├─ electro-memory      — SQLite, Markdown, λ-Memory
├─ electro-vault        — ChaCha20-Poly1305 secrets
├─ electro-tools       — Shell, browser, file ops, web fetch, git
├─ electro-mcp         — MCP client, 14-server registry
├─ electro-gateway     — HTTP server, health, dashboard
└─ electro-observable  — OpenTelemetry tracing
```

## Development

```bash
cargo check --workspace                              # Quick compile check
cargo test --workspace                               # Run tests
cargo clippy --workspace --all-targets --all-features -- -D warnings  # Lint
cargo fmt --all                                      # Format
cargo build --release                                # Release binary
```

Requires **Rust 1.82+** and Chrome/Chromium (for the browser tool).

## Status

Electro is production-ready (v3.2.0). The project follows a monthly release cadence with full regression testing (1,638 tests, zero warnings).

---

<p align="center">MIT License</p>
