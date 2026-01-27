# Commands Reference

## Core Commands

| Command | Description |
|---------|-------------|
| `veto check <cmd>` | Evaluate risk (exit code = risk level) |
| `veto exec <cmd>` | Verify + authenticate + execute |
| `veto gate <cmd>` | Verify only (for hooks, no execute) |
| `veto shell` | Interactive protected shell |
| `veto init` | Create default config |
| `veto doctor` | Diagnose installation |
| `veto upgrade` | Self-update to latest version |

## Setup Commands

| Command | Description |
|---------|-------------|
| `veto setup claude` | Setup Claude Code hooks |
| `veto setup claude --uninstall` | Remove Claude Code hooks |

## Authentication Commands

| Command | Description |
|---------|-------------|
| `veto auth set-pin` | Set/update PIN |
| `veto auth setup-totp` | Setup TOTP (QR code) |
| `veto auth setup-telegram` | Setup Telegram bot |
| `veto auth list` | Show configured methods |
| `veto auth test <method>` | Test authentication |
| `veto auth remove <method>` | Remove method |

## Flags

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Show category, reason, pattern |
| `-q, --quiet` | Exit code only (for scripts) |
| `--auth <method>` | Override auth method (exec, gate) |
| `--claude` | Read command from Claude Code stdin JSON (gate only) |
| `--totp <code>` | Pass TOTP code directly (gate only) |
| `--pin <code>` | Pass PIN directly (gate only) |
| `--check` | Only check for updates (upgrade only) |
| `--force` | Force reinstall even if latest (upgrade only) |

## Exit Codes

| Level | Code | Default Auth | Examples |
|-------|------|--------------|----------|
| ALLOW | 0 | None | `ls`, `pwd`, `cargo build` |
| LOW | 1 | confirm | `curl`, `wget` |
| MEDIUM | 2 | confirm | `git push`, `npm install` |
| HIGH | 3 | configurable | `cat .env`, `git push -f` |
| CRITICAL | 4 | configurable | `rm -rf /`, `mkfs` |

## Usage Examples

### Check Command Risk

```bash
# Simple check
veto check "rm -rf node_modules"

# Verbose output
veto check -v "git push -f origin main"
# Risk: HIGH
# Category: git-destructive
# Reason: Destructive git operation
# Pattern: git push*-f*

# Quiet mode for scripts
veto check -q "dangerous command"
echo $?  # 0-4 based on risk level
```

### Execute with Authentication

```bash
# Standard execution
veto exec "rm -rf node_modules"

# Override authentication method
veto exec --auth pin "command"
veto exec --auth touchid "command"
```

### Script Integration

```bash
veto check -q "dangerous command"
case $? in
    0) echo "ALLOW — safe" ;;
    1) echo "LOW — log it" ;;
    2) echo "MEDIUM — warn user" ;;
    3) echo "HIGH — require approval" ;;
    4) echo "CRITICAL — block or strong auth" ;;
esac
```

### Interactive Shell

```bash
veto shell
# ╔════════════════════════════════════════╗
# ║         Veto Protected Shell           ║
# ╚════════════════════════════════════════╝

veto ~/project $ rm -rf node_modules
# Risk: MEDIUM
# Reason: Recursive delete
# Allow this operation? [y/N]
```

Shell built-ins: `cd`, `pwd`, `help`, `exit`

### Self-Update

```bash
# Check for updates
veto upgrade --check
# Current version: 0.1.0
# Latest version: 0.1.1
# Update available: 0.1.0 → 0.1.1
# Run 'veto upgrade' to install the update.

# Install latest version
veto upgrade

# Force reinstall (even if already latest)
veto upgrade --force
```
