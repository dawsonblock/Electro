# ELECTRO — Setup for Pros

You know what you're doing. Here's what you need.

## Requirements

- Rust 1.82+ (or Docker)
- Chrome/Chromium (optional, for browser tool)
- Telegram bot token via [@BotFather](https://t.me/BotFather)

## Build

```bash
git clone https://github.com/nagisanzenin/electro.git && cd electro
cargo build --release   # ~2.5min cold, 9.6 MB binary
```

## Authentication — Pick Your Poison

### Codex OAuth (ChatGPT Plus/Pro)

```bash
electro auth login                    # browser flow
electro auth login --headless         # headless (paste redirect URL)
electro auth login --output ./o.json  # export token for containers
electro auth status                   # check expiry
```

Tokens last ~10 days. Stored at `~/.electro/oauth.json`. Auto-detected at startup.

### API Key

No auth flow needed. Start the bot, paste any supported key in Telegram. Auto-detected:

| Prefix | Provider |
|--------|----------|
| `sk-ant-` | Anthropic |
| `sk-` | OpenAI |
| `AIzaSy` | Gemini |
| `xai-` | Grok |
| `sk-or-` | OpenRouter |

Or use the OTK secure setup link (AES-256-GCM encrypted client-side).

## Run

```bash
export TELEGRAM_BOT_TOKEN="..."
electro start                          # foreground
electro start -d                       # daemon (logs: ~/.electro/electro.log)
electro start -d --log /var/log/sk.log # custom log path
electro stop                           # graceful shutdown
```

## Configuration

Config file: `electro.toml` (project root) or `~/.electro/electro.toml`.

```toml
[provider]
name = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-sonnet-4-6"

[agent]
max_spend_usd = 5.0   # 0.0 = unlimited (default)

[channel.telegram]
enabled = true
token = "${TELEGRAM_BOT_TOKEN}"
allowlist = []         # Empty no longer auto-whitelists without ELECTRO_ALLOW_FIRST_USER_BOOTSTRAP=1
file_transfer = true

[memory]
backend = "sqlite"

[security]
sandbox = "mandatory"
# Set to 1 to allow the first user to message the bot to become admin. Disabled by default.
# ELECTRO_ALLOW_FIRST_USER_BOOTSTRAP=1
# Comma-separated list of allowed domains for browser and web_fetch. Empty means all public web.
# ELECTRO_PUBLIC_WEB_ALLOWLIST="github.com,docs.rs"
```

Environment variables expand via `${VAR}` syntax. Full schema: `crates/electro-core/src/types/config.rs`.

## Docker

```bash
# Authenticate on host
electro auth login --output ./oauth.json

# Or set API key as env var
echo "ANTHROPIC_API_KEY=sk-ant-..." > .env
```

```yaml
# docker-compose.yml
services:
  electro:
    build: .
    environment:
      - TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN}
    volumes:
      - ./oauth.json:/root/.electro/oauth.json      # Codex OAuth
      - ./electro.toml:/root/.electro/electro.toml   # config
      - electro-data:/root/.electro                   # persistent state
    restart: unless-stopped

volumes:
  electro-data:
```

`TELEGRAM_BOT_TOKEN` env var auto-injects into Telegram config. No need to duplicate it in `electro.toml`.

## VPS Deployment (systemd)

```bash
# Build on server (or cross-compile and scp the binary)
cargo build --release
sudo cp target/release/electro /usr/local/bin/

# Create systemd service
sudo tee /etc/systemd/system/electro.service << 'EOF'
[Unit]
Description=ELECTRO AI Agent
After=network.target

[Service]
Type=simple
User=electro
Environment=TELEGRAM_BOT_TOKEN=your-token
Environment=ANTHROPIC_API_KEY=your-key
ExecStart=/usr/local/bin/electro start
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now electro
journalctl -u electro -f  # tail logs
```

Minimum VPS: 512 MB RAM, 1 vCPU. Idles at 15 MB RSS. >:3

## Multi-Provider — Switch Live

Swap providers mid-conversation with `/model`:

```
/model                    # list available models
/model gpt-5.4           # switch to GPT-5.4
/model claude-sonnet-4-6 # switch to Claude
```

Or just say "Switch to GPT-5.2" — natural language works too.

Credentials stored at `~/.electro/credentials.toml`. The agent reads and edits this file itself.

## MCP Servers — Extend at Runtime

```
/mcp add fetch npx -y @modelcontextprotocol/server-fetch
/mcp add github npx -y @modelcontextprotocol/server-github
/mcp                    # list connected servers
/mcp remove fetch       # disconnect
```

The agent also self-extends — it searches the 14-server built-in registry by capability when it needs something it doesn't have.

Config: `~/.electro/mcp.toml`

## Key Paths

| Path | Purpose |
|------|---------|
| `~/.electro/` | Home directory (all persistent state) |
| `~/.electro/credentials.toml` | Provider API keys (encrypted) |
| `~/.electro/oauth.json` | Codex OAuth tokens |
| `~/.electro/memory.db` | SQLite memory backend |
| `~/.electro/allowlist.toml` | User whitelist |
| `~/.electro/custom-tools/` | Agent-authored script tools |
| `~/.electro/mcp.toml` | MCP server configuration |
| `~/.electro/electro.log` | Daemon log (with `-d`) |

## Updating

```bash
electro update   # git pull + cargo build --release
# or manually:
git pull && cargo build --release
```

## Compilation Gates

All four pass before anything touches main. No exceptions.

```bash
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --workspace    # 1,378 tests, 0 failures
```

## Design Documents

| Document | Topic |
|----------|-------|
| [COGNITIVE_ARCHITECTURE.md](docs/design/COGNITIVE_ARCHITECTURE.md) | The Finite Brain Model — context as working memory |
| [BLUEPRINT_SYSTEM.md](docs/design/BLUEPRINT_SYSTEM.md) | Blueprint procedural memory vision |
| [BLUEPRINT_MATCHING_V2.md](docs/design/BLUEPRINT_MATCHING_V2.md) | Zero-extra-LLM-call matching architecture |
| [BLUEPRINT_IMPLEMENTATION.md](docs/design/BLUEPRINT_IMPLEMENTATION.md) | Step-by-step implementation plan |
| [OTK_SECURE_KEY_SETUP.md](docs/OTK_SECURE_KEY_SETUP.md) | AES-256-GCM encrypted onboarding |
| [BENCHMARK_REPORT.md](docs/benchmarks/BENCHMARK_REPORT.md) | Performance benchmarks vs OpenClaw/ZeroClaw |
