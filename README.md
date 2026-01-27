<p align="center">
  <h1 align="center">veto</h1>
  <p align="center">AI operation guardian — verify before execute</p>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.82+-orange.svg" alt="Rust"></a>
</p>

<p align="center">
  <a href="#installation">Install</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#commands">Commands</a> •
  <a href="#configuration">Config</a> •
  <a href="#how-it-works">How It Works</a>
</p>

---

## Why veto?

AI coding assistants (Claude Code, Cursor, Codex) can execute shell commands autonomously. While powerful, this creates risk — a misunderstood instruction could lead to destructive operations.

| Without veto | With veto |
|--------------|-----------|
| AI runs `rm -rf /` directly | veto detects CRITICAL risk, requires confirmation |
| `git push --force` happens silently | veto warns about destructive git operation |
| Secrets in `.env` exposed via `cat` | veto flags HIGH risk for secrets access |
| No audit trail | Risk level logged for every command |

**veto** acts as a guardian layer between AI tools and your shell, evaluating command risk and requiring appropriate authentication before execution.

---

## Installation

### From Source (Docker)

```bash
git clone https://github.com/runkids/veto.git
cd veto
docker-compose run --rm test cargo build --release
# Binary at: target/release/veto
```

### From Source (Local)

```bash
git clone https://github.com/runkids/veto.git
cd veto
cargo build --release
cp target/release/veto /usr/local/bin/
```

---

## Quick Start

```bash
# 1. Initialize configuration
veto init

# 2. Check a command's risk level
veto check "git push origin main"
# Output: Risk: MEDIUM

# 3. Execute with verification
veto exec "git push origin main"
# Prompts for confirmation before executing
```

---

## Commands

| Command | Description |
|---------|-------------|
| `veto check <cmd>` | Evaluate risk level without executing (exit codes: 0=ALLOW, 1=LOW, 2=MEDIUM, 3=HIGH, 4=CRITICAL) |
| `veto exec <cmd>` | Verify risk and authenticate before executing |
| `veto init` | Create default config files in `~/.config/veto/` |
| `veto doctor` | Verify installation and configuration status |
| `veto shell` | Start interactive shell wrapper (coming soon) |

### Global Flags

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Show detailed risk information (category, reason, matched pattern) |
| `-q, --quiet` | Suppress output, return exit code only |

---

## How It Works

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   AI Assistant  │────▶│      veto       │────▶│      Shell      │
│ (Claude, Cursor)│     │                 │     │    (bash/zsh)   │
└─────────────────┘     │  1. Parse cmd   │     └─────────────────┘
                        │  2. Match rules │
                        │  3. Eval risk   │
                        │  4. Require auth│
                        │  5. Execute     │
                        └─────────────────┘
```

### Risk Levels

| Level | Exit Code | Auth Required | Examples |
|-------|-----------|---------------|----------|
| ALLOW | 0 | No | `ls`, `pwd`, `cargo build` |
| LOW | 1 | Confirm | `curl`, `wget` |
| MEDIUM | 2 | Confirm | `git push`, `npm install` |
| HIGH | 3 | Confirm + Warning | `cat .env`, `git push --force` |
| CRITICAL | 4 | Confirm + Warning | `rm -rf /`, `mkfs` |

---

## Configuration

Configuration files are stored in `~/.config/veto/`:

### config.toml

```toml
[auth]
# Default authentication method
default = "confirm"

# Per-level authentication (optional)
# [auth.levels]
# low = "confirm"
# medium = "confirm"
# high = "pin"
# critical = ["pin", "confirm"]

# TouchID (macOS, coming soon)
# [auth.touchid]
# enabled = true
# prompt = "Authorize veto operation"
```

### rules.toml

```toml
# Whitelist - always allow these commands
[whitelist]
commands = [
    "ls*",
    "pwd",
    "cargo build*",
    "git status*",
]

# Custom critical rules
[[critical]]
category = "my-critical"
patterns = ["danger-cmd*"]
reason = "Custom critical operation"

# Custom high-risk rules
[[high]]
category = "my-high"
patterns = ["risky-cmd*"]
reason = "Custom high-risk operation"
```

---

## Integration with AI Tools

### Claude Code

Add to your Claude Code hooks or wrapper:

```bash
# Instead of direct execution
bash -c "dangerous command"

# Use veto
veto exec "dangerous command"
```

### Script Integration

```bash
# Check risk level programmatically
veto check -q "rm -rf /"
case $? in
    0) echo "Safe" ;;
    1|2) echo "Needs review" ;;
    3|4) echo "Dangerous!" ;;
esac
```

---

## FAQ

### How do I add custom rules?

Edit `~/.config/veto/rules.toml` and add patterns under the appropriate risk level section.

### Can I disable confirmation for certain commands?

Add them to the `[whitelist]` section in `rules.toml`.

### What if veto blocks a legitimate command?

Use `veto check -v "command"` to see why it was flagged, then either:
1. Add it to whitelist
2. Confirm when prompted

### How do I reset configuration?

```bash
veto init --force
```

---

## Development

### Docker Commands

```bash
# Run all tests
docker-compose run --rm test cargo test

# Run specific tests
docker-compose run --rm test cargo test rules
docker-compose run --rm test cargo test auth

# Build release
docker-compose run --rm test cargo build --release

# Interactive development shell
docker-compose run --rm dev
```

### Interactive Development

進入 Docker 容器進行互動式測試：

```bash
# 啟動開發環境
docker-compose run --rm dev

# 現在你在容器內的 /app 目錄
# 可以執行以下命令：

cargo build                           # 編譯
cargo test                            # 測試
./target/debug/veto --help           # 查看說明
./target/debug/veto check "ls"       # 檢查命令風險
./target/debug/veto check -v "rm -rf /" # 詳細風險資訊
./target/debug/veto init             # 初始化設定
./target/debug/veto doctor           # 診斷狀態

# 離開容器
exit
```

### Sandbox Testing

安全測試危險命令（唯讀環境）：

```bash
docker-compose run --rm sandbox
# 此環境為唯讀，可安全測試危險命令的風險評估
```

---

## License

MIT License - see [LICENSE](LICENSE) for details.
