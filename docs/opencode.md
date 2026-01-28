# OpenCode Integration

## Quick Setup

```bash
veto init              # Create config (first time)
veto setup opencode    # Install plugin
veto doctor            # Verify setup
```

Restart OpenCode. Done!

To remove: `veto setup opencode --uninstall`

## How It Works

### Basic Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            OpenCode + veto Flow                              │
└─────────────────────────────────────────────────────────────────────────────┘

  AI wants to run            tool.execute.before               Result
  ─────────────              ───────────────────               ──────

  ┌─────────────┐               ┌─────────────────┐
  │  OpenCode   │─── Bash ─────▶│  veto gate      │
  │    (AI)     │   command     │   --opencode    │
  └─────────────┘               └────────┬────────┘
                                         │
                                         ▼
                                ┌─────────────────┐
                                │ Evaluate Risk   │
                                │ Level           │
                                └────────┬────────┘
                                         │
                        ┌────────────────┼────────────────┐
                        │                │                │
                        ▼                ▼                ▼
                 ┌────────────┐  ┌────────────┐  ┌────────────┐
                 │   ALLOW    │  │ LOW/MEDIUM │  │HIGH/CRITICAL│
                 │            │  │            │  │            │
                 └─────┬──────┘  └─────┬──────┘  └─────┬──────┘
                       │               │               │
                       ▼               ▼               ▼
                 ┌───────────────────────────────────────────┐
                 │          Check config.toml                │
                 │          [auth.levels]                    │
                 └─────────────────┬─────────────────────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    │              │              │
                    ▼              ▼              ▼
              ┌──────────┐  ┌──────────┐  ┌──────────┐
              │ No auth  │  │ dialog/  │  │ pin/totp │
              │ needed   │  │ touchid  │  │ confirm  │
              └────┬─────┘  └────┬─────┘  └────┬─────┘
                   │             │             │
                   │             ▼             ▼
                   │       ┌──────────┐  ┌────────────────┐
                   │       │ System   │  │ AI asks user   │
                   │       │ Popup    │  │ for code       │
                   │       └────┬─────┘  └───────┬────────┘
                   │             │               │
                   ▼             ▼               ▼
              ┌─────────────────────────────────────────────┐
              │                  exit 0                     │
              │              Command Executes               │
              └─────────────────────────────────────────────┘
```

### Authentication Flow (dialog/touchid)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Dialog/TouchID Authentication                           │
└─────────────────────────────────────────────────────────────────────────────┘

  ┌─────────────┐        ┌─────────────────┐        ┌─────────────────┐
  │  OpenCode   │───────▶│  veto gate      │───────▶│  macOS Dialog   │
  │  (AI)       │        │  --opencode     │        │  or Touch ID    │
  └─────────────┘        └─────────────────┘        └────────┬────────┘
                                                             │
                                               ┌─────────────┴─────────────┐
                                               │                           │
                                               ▼                           ▼
                                        ┌────────────┐              ┌────────────┐
                                        │  Approve   │              │   Cancel   │
                                        │  (click)   │              │  (click)   │
                                        └─────┬──────┘              └─────┬──────┘
                                              │                           │
                                              ▼                           ▼
                                        ┌────────────┐              ┌────────────┐
                                        │  exit 0    │              │  exit 2    │
                                        │  Command   │              │ STOP_RETRY │
                                        │  Executes  │              │ AI stops   │
                                        └────────────┘              └────────────┘
```

### Authentication Flow (pin/totp)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PIN/TOTP Authentication                               │
└─────────────────────────────────────────────────────────────────────────────┘

  Step 1: AI tries command
  ───────────────────────

  ┌─────────────┐        ┌─────────────────┐        ┌─────────────────────────┐
  │  OpenCode   │───────▶│  veto gate      │───────▶│ "Ask user for PIN..."  │
  │  (AI)       │        │  --opencode     │        │  exit 2                 │
  └─────────────┘        └─────────────────┘        └─────────────────────────┘
                                                              │
                                                              ▼
                                                    ┌─────────────────────────┐
                                                    │ AI asks user in chat:   │
                                                    │ "Please provide PIN"    │
                                                    └─────────────────────────┘

  Step 2: User provides code, AI retries
  ──────────────────────────────────────

  ┌─────────────┐        ┌─────────────────┐        ┌─────────────────────────┐
  │  OpenCode   │───────▶│  VETO_PIN=1234  │───────▶│  Verify PIN            │
  │  (AI)       │        │  veto gate ...  │        │                         │
  └─────────────┘        └─────────────────┘        └────────────┬────────────┘
                                                                 │
                                                   ┌─────────────┴─────────────┐
                                                   │                           │
                                                   ▼                           ▼
                                            ┌────────────┐              ┌────────────┐
                                            │  Correct   │              │  Wrong     │
                                            │  PIN       │              │  PIN       │
                                            └─────┬──────┘              └─────┬──────┘
                                                  │                           │
                                                  ▼                           ▼
                                            ┌────────────┐              ┌────────────┐
                                            │  exit 0    │              │  exit 2    │
                                            │  Command   │              │  Blocked   │
                                            │  Executes  │              │            │
                                            └────────────┘              └────────────┘
```

## Plugin Location

The plugin is installed at:
```
~/.config/opencode/plugins/veto-gate.js
```

## Authentication Methods

| Method | Behavior in OpenCode |
|--------|---------------------|
| `dialog` | macOS dialog popup - AI cannot bypass |
| `touchid` | Touch ID prompt - AI cannot bypass |
| `pin` | AI asks user for PIN, retry with `VETO_PIN=<code>` |
| `totp` | AI asks user for TOTP, retry with `VETO_TOTP=<code>` |
| `confirm` | AI asks user, retry with `VETO_CONFIRM=yes` |
| `telegram` | Telegram bot approval |

**Recommended for OpenCode:** `dialog` or `touchid` (AI cannot bypass these)

## Configuration

Edit `~/.veto/config.toml`:

```toml
[auth]
default = "dialog"  # Recommended for OpenCode

[auth.levels]
low = []              # No auth needed
medium = []           # No auth needed
high = ["dialog"]     # Require dialog confirmation
critical = ["touchid"] # Require Touch ID
```

## Debugging

### Check Plugin Status

```bash
ls ~/.config/opencode/plugins/veto-gate.js
```

### Test Gate Command

```bash
# Should show dialog popup
veto gate --opencode "rm -rf test"

# With PIN auth configured
veto gate --opencode "rm -rf test"
# [veto] HIGH command blocked. Ask user in chat for their PIN code...
```

### View Risk Level

```bash
veto check -v "git push -f origin main"
# Risk: HIGH
# Category: git-destructive
# Reason: Destructive git operation
```

## Differences from Claude Code

| Feature | Claude Code | OpenCode |
|---------|-------------|----------|
| Integration | `~/.claude/settings.json` | `~/.config/opencode/plugins/` |
| Hook format | JSON stdin/stdout | ES module plugin |
| Flag | `--claude` | `--opencode` |
| Cancel behavior | JSON `continue: false` | `throw Error` + tracking |

## Troubleshooting

### AI doesn't ask for PIN/TOTP

Some AI models may not follow the instruction to ask users for codes. Use `dialog` or `touchid` instead - these don't depend on AI cooperation.

### AI keeps retrying after denial

The plugin tracks denied commands and blocks retries with:
```
[veto] BLOCKED. This command was rejected. DO NOT RETRY.
```

### Plugin not loading

1. Check file exists: `ls ~/.config/opencode/plugins/veto-gate.js`
2. Restart OpenCode
3. Check OpenCode console for errors
