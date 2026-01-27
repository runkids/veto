<p align="center">
  <h1 align="center">veto</h1>
  <p align="center">✋ AI operation guardian — Intercept dangerous commands before AI executes them</p>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.85+-orange.svg" alt="Rust"></a>
  <a href="#supported-platforms"><img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg" alt="Platform"></a>
</p>

<p align="center">
  <strong>Intercept dangerous commands before AI executes them.</strong><br>
  Risk evaluation + multi-factor authentication for Claude Code.
</p>

---

## Why veto?

AI coding assistants execute shell commands autonomously. **veto adds a safety layer.**

| | Without veto | With veto |
|---|-------------|-----------|
| `rm -rf /` | Executes immediately | **CRITICAL** — requires strong auth |
| `git push --force` | Silent execution | **HIGH** — warns, requires approval |
| `cat .env` | Secrets exposed | **HIGH** — flags sensitive file access |
| `npm install` | Runs without notice | **MEDIUM** — logged, confirmation |
| Audit trail | None | **Every command logged** with risk level |
| Custom rules | — | **Define your own** patterns and risk levels |

---

## Custom Rules

Add your own rules in `~/.veto/rules.toml`:

```toml
# Protect sensitive files from being read
[[critical]]
category = "secrets"
patterns = [
    "cat *.env*",
    "cat *secret*",
    "cat *password*",
    "cat ~/.ssh/id_*",
    "cat *credentials*",
]
reason = "Sensitive file access"

# Block destructive database operations
[[critical]]
category = "database-destructive"
patterns = ["DROP DATABASE*", "DROP TABLE*", "TRUNCATE*"]
reason = "Destructive database operation"

# Whitelist safe commands
[whitelist]
commands = ["docker ps*", "kubectl get pods*", "git status*"]
```

[→ Full rules documentation](docs/rules.md)

---

## Supported Platforms

| OS | Architecture | Touch ID |
|----|--------------|----------|
| macOS | x86_64 / arm64 | ✅ |
| Linux | x86_64 / arm64 | ✗ |

---

## Supported AI Tools

| Tool | Status |
|------|--------|
| [Claude Code](https://claude.ai/code) | ✅ Supported |
| [Moltbot](https://clawd.bot/) | 🔜 Coming soon |

---

## Authentication Methods

| Method | Platform | Description |
|--------|----------|-------------|
| 🔐 [**PIN**](docs/authentication.md#pin) | All | 4+ digit code, Argon2 hashed |
| 📱 [**Telegram**](docs/authentication.md#telegram) | All | Remote approval via bot |
| 🔑 [**TOTP**](docs/authentication.md#totp-google-authenticator) | All | Google Authenticator compatible |
| 👆 [**Touch ID**](docs/authentication.md#touch-id-macos) | macOS | Biometric authentication |
| 💬 **Confirm** | All | Simple y/n prompt |

[→ Setup guide](docs/authentication.md)

---

## Installation

```bash
curl -sSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash
```

---

## Quick Start

```bash
veto init              # Create config in ~/.veto/
veto setup claude      # Setup Claude Code hooks
veto doctor            # Verify setup
```

Done! Restart Claude Code. High-risk commands now require Touch ID or authentication.

---

## How It Works

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   AI Assistant  │────▶│      veto       │────▶│      Shell      │
│  (Claude Code)  │     │                 │     │    (bash/zsh)   │
└─────────────────┘     │  1. Parse cmd   │     └─────────────────┘
                        │  2. Match rules │
                        │  3. Eval risk   │
                        │  4. Require auth│
                        │  5. Execute     │
                        └─────────────────┘
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [Installation](docs/installation.md) | Install guide, build from source |
| [Commands](docs/commands.md) | Full command reference, flags, exit codes |
| [Configuration](docs/configuration.md) | config.toml options |
| [Rules](docs/rules.md) | Default rules, custom rules syntax |
| [Authentication](docs/authentication.md) | Setup PIN, TOTP, Touch ID, Telegram |
| [Claude Code](docs/claude-code.md) | Claude Code integration, hooks, flow diagram |
| [Troubleshooting](docs/troubleshooting.md) | Common issues, FAQ |

---

## Development

```bash
make build      # Build debug
make test       # Run tests
make release    # Build release
make install    # Install to /usr/local/bin
make sandbox    # Docker sandbox for testing
```

---

## License

MIT — see [LICENSE](LICENSE)
