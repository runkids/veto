# Reference

Complete lookup for CLI, configuration, and rules.

---

## CLI

### Commands

| Command | Description |
|---------|-------------|
| `veto check <cmd>` | Evaluate risk (exit code = risk level) |
| `veto exec <cmd>` | Verify + authenticate + execute |
| `veto gate <cmd>` | Verify only (for hooks, no execute) |
| `veto shell` | Interactive protected shell |
| `veto init` | Create default config |
| `veto doctor` | Diagnose installation |
| `veto upgrade` | Self-update to latest version |
| `veto log` | View audit log |
| `veto setup claude` | Setup / `--uninstall` remove Claude Code hooks |
| `veto auth set-pin` | Set/update PIN |
| `veto auth setup-totp` | Setup TOTP |
| `veto auth setup-telegram` | Setup Telegram bot |
| `veto auth list` | Show configured methods |
| `veto auth test <method>` | Test authentication |
| `veto auth remove <method>` | Remove method |
| `veto allow` | Temporarily allow a command |

### Flags

| Flag | Applies to | Description |
|------|-----------|-------------|
| `-v, --verbose` | check, exec | Show category, reason, pattern |
| `-q, --quiet` | check | Exit code only |
| `--auth <method>` | exec, gate | Override auth method |
| `--claude` | gate | Read command from Claude Code stdin JSON |
| `--file-op` | gate | File operation mode (Write/Edit hooks) |
| `--totp <code>` | gate | Pass TOTP code directly |
| `--pin <code>` | gate | Pass PIN directly |
| `--check` | upgrade | Only check for updates |
| `--force` | upgrade | Force reinstall |
| `-n, --tail <N>` | log | Show last N entries |
| `-f, --follow` | log | Follow log in real-time |
| `--filter <R>` | log | Filter by ALLOWED/DENIED/BLOCKED |
| `--clear` | log | Clear the audit log |

### Exit Codes

| Code | Level | Examples |
|------|-------|----------|
| 0 | ALLOW | `ls`, `pwd`, `cargo build` |
| 1 | LOW | `curl`, `wget` |
| 2 | MEDIUM | `git push`, `npm install` |
| 3 | HIGH | `cat .env`, `git push -f` |
| 4 | CRITICAL | `rm -rf /`, `mkfs` |

---

## Configuration

Directory: `~/.veto/`

```
~/.veto/
├── config.toml    # Auth settings
├── rules.toml     # Custom rules (optional)
├── audit.log      # Command audit trail
└── secrets/       # Encrypted secrets (fallback)
```

### config.toml

```toml
[auth]
default = "touchid"              # "pin" on Linux

[auth.levels]
low = "pin"
medium = "pin"
high = "touchid"
critical = "telegram"

[auth.fallback]
touchid = "pin"
telegram = "totp"
totp = "pin"

[auth.pin]
enabled = true

[auth.totp]
enabled = true
issuer = "veto"

[auth.touchid]
enabled = true
prompt = "Veto: Approve running this command?"

[auth.telegram]
enabled = true
chat_id = "123456789"
timeout_seconds = 60
```

### Config Fields

| Section | Key | Type | Description |
|---------|-----|------|-------------|
| `[auth]` | `default` | string | Default auth method |
| `[auth.levels]` | `low/medium/high/critical` | string | Auth per risk level |
| `[auth.fallback]` | `touchid/telegram/totp` | string | Fallback when primary unavailable |
| `[auth.pin]` | `enabled` | bool | Enable PIN |
| `[auth.totp]` | `enabled` | bool | Enable TOTP |
| `[auth.totp]` | `issuer` | string | Issuer in authenticator app |
| `[auth.touchid]` | `enabled` | bool | Enable Touch ID (macOS) |
| `[auth.touchid]` | `prompt` | string | Touch ID prompt message |
| `[auth.telegram]` | `enabled` | bool | Enable Telegram |
| `[auth.telegram]` | `chat_id` | string | Telegram user ID |
| `[auth.telegram]` | `timeout_seconds` | int | Approval timeout (default: 60) |

---

## Rules

### Built-in Rules

**CRITICAL** — system destruction, credential exposure:

| Category | Patterns |
|----------|----------|
| destructive | `rm -rf /`, `rm -rf /*`, `rm -rf ~`, `mkfs*`, `dd if=* of=/dev/*` |
| credentials | `*AWS_SECRET*`, `*PRIVATE_KEY*`, `cat ~/.ssh/id_*`, `cat *id_rsa*` |

**HIGH** — force operations, secrets access:

| Category | Patterns |
|----------|----------|
| rm-recursive-force | `rm -rf *`, `rm -fr *` |
| secrets | `cat *.env*`, `cat .env`, `cat *secret*`, `cat *password*` |
| git-destructive | `git push*--force*`, `git push*-f*`, `git reset --hard*`, `git clean -fd*` |

**MEDIUM** — recursive delete, remote git, package install:

| Category | Patterns |
|----------|----------|
| rm-recursive | `rm -r *`, `rm -R *` |
| git | `git push*`, `git merge*`, `git rebase*` |
| install | `npm install*`, `pip install*`, `cargo install*`, `brew install*`, `apt install*` |

**LOW** — simple delete, network:

| Category | Patterns |
|----------|----------|
| rm | `rm *` |
| network | `curl*`, `wget*` |

**ALLOW** — whitelisted safe commands:

`ls*`, `pwd`, `echo *`, `cat *`, `head *`, `tail *`, `grep *`, `find *`, `which *`, `whoami`, `date`, `cargo build*`, `cargo test*`, `cargo check*`, `cargo fmt*`, `cargo clippy*`, `npm run*`, `npm test*`, `git status*`, `git log*`, `git diff*`, `git branch*`, `git show*`

### Custom Rules (rules.toml)

```toml
[whitelist]
commands = ["your-safe-command*"]

[[critical]]
category = "name"
patterns = ["pattern*"]
reason = "Why it's critical"
challenge = true           # Optional: enable challenge-response
```

Sections: `[whitelist]`, `[[critical]]`, `[[high]]`, `[[medium]]`, `[[low]]`

### Pattern Syntax

| Pattern | Matches |
|---------|---------|
| `rm *` | `rm` followed by anything |
| `*secret*` | Contains "secret" anywhere |
| `git push*-f*` | `git push` with `-f` anywhere after |
| `cat ~/.ssh/id_*` | Exact path prefix |

### Evaluation Order

1. Whitelist → ALLOW (overrides all)
2. Critical → CRITICAL
3. High → HIGH
4. Medium → MEDIUM
5. Low → LOW
6. No match → default (ALLOW)

---

## Claude Code Hook

### How It Works

```
Claude Code ─── Bash command ───▶ veto gate --claude
                                        |
                                  Evaluate Risk
                                        |
                            +-----------+-----------+
                            |           |           |
                         ALLOW      LOW/MED    HIGH/CRIT
                            |           |           |
                         exit 0    check auth   require auth
                                        |           |
                                     exit 0      exit 0
                                  (or deny)    (or deny)
```

### Manual Hook Configuration

`~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{
          "type": "command",
          "command": "veto gate --claude",
          "timeout": 90
        }]
      },
      {
        "matcher": "Write|Edit",
        "hooks": [{
          "type": "command",
          "command": "veto gate --claude --file-op",
          "timeout": 30
        }]
      }
    ]
  }
}
```

Timeout is in **seconds** (not milliseconds).

### Key Behaviors

| Action | veto Output | Claude Code |
|--------|-------------|-------------|
| Approve | `exit 0` | Command executes |
| Cancel | `{"continue":false}` | AI stops completely |
| No credentials provided | `exit 2` + instruction | AI asks user for code |

### Passing Auth Non-Interactively

```bash
echo '{"tool_input":{"command":"rm -rf /"}}' | veto gate --claude --pin 1234
echo '{"tool_input":{"command":"rm -rf /"}}' | veto gate --claude --totp 123456
```

---

## Secret Storage

| Backend | Platform | Location |
|---------|----------|----------|
| Keychain | macOS | macOS Keychain |
| Secret Service | Linux | GNOME Keyring / KWallet |
| File (fallback) | All | `~/.veto/secrets/*.enc` |

| Component | Algorithm |
|-----------|-----------|
| PIN | Argon2id + random salt |
| TOTP | HMAC-SHA1 (RFC 6238) |
| File encryption | AES-256-GCM + PBKDF2 (100k iterations) |

---

## Build from Source

Prerequisites: Rust 1.85+, macOS: Xcode Command Line Tools

```bash
git clone https://github.com/runkids/veto.git
cd veto
cargo build --release
cp target/release/veto ~/.local/bin/
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `VETO_HOME` | Override config directory (default: `~/.veto`) |
| `VETO_PIN` | Pass PIN for non-interactive auth |
| `VETO_TOTP` | Pass TOTP code for non-interactive auth |
| `VETO_CONFIRM` | Set to `yes` to auto-confirm |
| `VETO_FORCE` | Set to `yes` to override a previously denied command |
| `VETO_RESPONSE` | Challenge-response code (format: `<PIN><challenge>`) |

---

## Audit Log

Format: `[timestamp] RESULT RISK auth_method "command"`

```
[2026-01-28 10:04:42] ALLOWED HIGH PIN "rm -rf video/out/"
[2026-01-28 03:31:39] DENIED LOW - "rm video/out/test.gif"
```

| Field | Values |
|-------|--------|
| RESULT | `ALLOWED`, `DENIED` |
| RISK | `CRITICAL`, `HIGH`, `MEDIUM`, `LOW` |
| auth_method | `Telegram`, `PIN`, `TOTP`, `Touch ID`, `confirm`, `-` |
