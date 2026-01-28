<p align="center">
  <h1 align="center">veto</h1>
  <p align="center">✋ AI operation guardian — intercept dangerous commands before AI executes them</p>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.85+-orange.svg" alt="Rust"></a>
  <a href="#platforms"><img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg" alt="Platform"></a>
</p>

<p align="center">
  <strong>Risk evaluation + authentication gate for shell commands.</strong><br>
  Built for Claude Code & OpenCode, also usable as a CLI.
</p>

<p align="center">
  <img src="docs/assets/VetoIntro.gif" alt="Veto Demo" width="600">
</p>

---

## Why veto?

AI coding assistants can execute shell commands autonomously. **veto adds a risk-based gate**:

- Evaluate risk level (`ALLOW` → `CRITICAL`) using built-in + custom rules
- For higher risk, require authentication (Touch ID / PIN / TOTP / Telegram / confirm)
- Keep an audit trail of evaluations

---

## Quick Start

Install:

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash
```

Prefer to inspect installers before running them? See [Installation](docs/installation.md).

Setup for your AI tool:

```bash
veto init
veto setup claude    # For Claude Code
veto setup opencode  # For OpenCode
veto doctor
```

Restart your AI tool. High-risk commands will now require verification.

---

## Examples

Check command risk level:

```bash
veto check "rm -rf node_modules"
# Risk: MEDIUM

veto check -v "git push -f origin main"
# Risk: HIGH
# Category: git-destructive
# Reason: Destructive git operation
```

Execute with authentication:

```bash
veto exec "rm -rf node_modules"
# Prompts for confirmation based on risk level
```

When AI runs a dangerous command:

```
┌──────────────┐      ┌─────────────┐      ┌─────────────┐
│ AI: rm -rf / │─────▶│ veto gate   │─────▶│ Touch ID /  │
│              │      │ (intercept) │      │ PIN prompt  │
└──────────────┘      └─────────────┘      └─────────────┘
```

---

## Install / Upgrade / Uninstall

Install (script downloads the correct binary):

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash
```

Enable AI tool integration (optional):

```bash
veto init
veto setup claude    # For Claude Code
veto setup opencode  # For OpenCode
```

Upgrade:

```bash
veto upgrade --check
veto upgrade
```

Reinstall hooks/plugins:

```bash
veto setup claude    # Reinstall Claude Code hooks
veto setup opencode  # Reinstall OpenCode plugin
```

Uninstall:

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash
```

Full uninstall (including keychain secrets):

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash -s -- --purge
```

Remove AI tool integration only:

```bash
veto setup claude --uninstall    # Remove Claude Code hooks
veto setup opencode --uninstall  # Remove OpenCode plugin
```

Full details: [Installation](docs/installation.md) and [Claude Code integration](docs/claude-code.md).

---

## Security Model (Threat Model Boundaries)

What veto helps with:

- Accidental destructive commands (e.g. recursive deletes, force pushes)
- Automation running higher-risk commands without explicit approval
- Visibility: an audit trail of command evaluations

What veto does NOT protect against:

- Bypass: running commands outside `veto` / without Claude Code hooks enabled
- A compromised host (malware, root compromise) or a compromised user account
- Malicious-but-benign-looking commands that don't match your rules
- "Approved but harmful": once you approve, veto will allow the command

Audit log privacy note: the audit log records command strings. Treat it as sensitive if your commands include secrets.

---

## Custom Rules

Add your own rules in `~/.veto/rules.toml`:

```toml
[[critical]]
category = "secrets"
patterns = ["cat *.env*", "cat ~/.ssh/id_*"]
reason = "Sensitive file access"

[whitelist]
commands = ["git status*", "docker ps*"]
```

Full rules syntax: [Rules](docs/rules.md).

---

## Platforms

| OS | Architecture | Touch ID |
|----|--------------|----------|
| macOS | x86_64 / arm64 | ✅ |
| Linux | x86_64 / arm64 | ✗ |

---

## Authentication Methods

- 🔐 PIN, 🔑 TOTP, 📱 Telegram, 👆 Touch ID (macOS), 💬 confirm
- Setup: [Authentication](docs/authentication.md)

---

## Audit Log

Every evaluation is logged to `~/.veto/audit.log`.

```bash
veto log
veto log -n 10
veto log --filter DENIED
veto log -f
veto log --clear
```

---

## Documentation

- [Installation](docs/installation.md)
- [Commands](docs/commands.md)
- [Configuration](docs/configuration.md)
- [Rules](docs/rules.md)
- [Authentication](docs/authentication.md)
- [Claude Code Integration](docs/claude-code.md)
- [OpenCode Integration](docs/opencode.md)
- [Troubleshooting](docs/troubleshooting.md)

---

## Development

```bash
make build
make test
make release
make install
make sandbox
```

---

## License

MIT — see [LICENSE](LICENSE)
