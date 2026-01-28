# Claude Code Integration

## Quick Setup

```bash
veto init             # Create config (first time)
veto setup claude      # Install hooks
veto doctor            # Verify setup
```

Restart Claude Code. Done!

To remove: `veto setup claude --uninstall`

## How It Works

### Basic Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Claude Code + veto Flow                          │
└─────────────────────────────────────────────────────────────────────────────┘

  AI wants to run                PreToolUse Hook                    Result
  ─────────────                  ───────────────                    ──────

  ┌─────────────┐               ┌─────────────────┐
  │ Claude Code │──── Bash ────▶│  veto gate      │
  │   (AI)      │    command    │   --claude      │
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
                   ▼             ▼             ▼
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
  │ Claude Code │───────▶│  veto gate      │───────▶│  macOS Dialog   │
  │   (AI)      │        │  --claude       │        │  or Touch ID    │
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
                                        ┌────────────┐              ┌─────────────────┐
                                        │  exit 0    │              │ JSON output     │
                                        │  Command   │              │ continue: false │
                                        │  Executes  │              │ AI stops        │
                                        └────────────┘              └─────────────────┘
```

### Authentication Flow (pin/totp)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PIN/TOTP Authentication                               │
└─────────────────────────────────────────────────────────────────────────────┘

  Step 1: AI tries command
  ───────────────────────

  ┌─────────────┐        ┌─────────────────┐        ┌─────────────────────────┐
  │ Claude Code │───────▶│  veto gate      │───────▶│ "Ask user for PIN..."  │
  │   (AI)      │        │  --claude       │        │  exit 2                 │
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
  │ Claude Code │───────▶│  VETO_PIN=1234  │───────▶│  Verify PIN            │
  │   (AI)      │        │  veto gate ...  │        │                         │
  └─────────────┘        └─────────────────┘        └────────────┬────────────┘
                                                                 │
                                                   ┌─────────────┴─────────────┐
                                                   │                           │
                                                   ▼                           ▼
                                            ┌────────────┐              ┌─────────────────┐
                                            │  Correct   │              │  Wrong PIN      │
                                            │  PIN       │              │                 │
                                            └─────┬──────┘              └─────┬───────────┘
                                                  │                           │
                                                  ▼                           ▼
                                            ┌────────────┐              ┌─────────────────┐
                                            │  exit 0    │              │ JSON output     │
                                            │  Command   │              │ continue: false │
                                            │  Executes  │              │ AI stops        │
                                            └────────────┘              └─────────────────┘
```

## Key Behaviors

| User Action | veto Output | Claude Code Behavior |
|-------------|-------------|---------------------|
| ✓ Approve (Touch ID/PIN) | `exit 0` | Command executes |
| ✗ Cancel | `{"continue":false}` | **AI stops completely** |
| ✗ Verification failed | `{"continue":false}` | **AI stops completely** |

When a command requires verification but no credentials were provided (common in Claude mode for `pin` / `totp` / `confirm`):
- veto exits non-zero and prints an instruction telling the AI to ask you for a code and retry with `VETO_PIN=...` / `VETO_TOTP=...` / `VETO_CONFIRM=yes`.

When user cancels authentication:
- veto outputs JSON with `"permissionDecision": "deny"` and `"continue": false`
- Claude Code receives this and **stops all processing**
- AI will **not** ask "What should I do instead?" or retry

## Denied Commands

If you explicitly deny a dialog/touchid/telegram prompt, veto will block retries
for the same command in hook modes. To retry after explicit approval, add:

```bash
VETO_FORCE=yes <command>
```

## Manual Configuration

If you prefer to configure manually, add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "veto gate --claude"
          }
        ]
      }
    ]
  }
}
```

## veto shell (Alternative)

Instead of hooks, you can use the interactive shell:

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

## Debugging

### Check Hook Status

```bash
veto doctor
# Claude Code Integration:
#   settings.json: found
#   PreToolUse hook: configured
#   veto binary: accessible
```

### Test Gate Command

```bash
echo '{"tool_input":{"command":"ls -la"}}' | veto gate --claude
# ALLOW commands should exit 0 (often with no output)

echo '{"tool_input":{"command":"rm -rf /"}}' | veto gate --claude
# Should require verification (and may instruct Claude to ask you for a code)
```

### View Risk Level

```bash
veto check -v "git push -f origin main"
# Risk: HIGH
# Category: git-destructive
# Reason: Destructive git operation
# Pattern: git push*-f*
```

## Passing Auth Non-Interactively

For automated scenarios, you can pass authentication directly:

```bash
veto gate --claude --totp 123456
veto gate --claude --pin 1234
```

This is primarily useful inside the Claude Code hook (Claude provides stdin JSON with the command).

For a manual test, include stdin JSON:

```bash
echo '{"tool_input":{"command":"rm -rf /"}}' | veto gate --claude --totp 123456
```

## Other AI Tools

veto supports multiple AI coding assistants:

| Tool | Integration | Command |
|------|-------------|---------|
| Claude Code | ✅ Supported | `veto setup claude` |
| Gemini CLI | ✅ Supported | `veto setup gemini` |
| OpenCode | ✅ Supported | `veto setup opencode` |
| Cursor CLI | ✅ Supported | `veto setup cursor` |

See [Gemini CLI Integration](geminicli.md), [OpenCode Integration](opencode.md), and [Cursor CLI Integration](cursorcli.md) for setup details.
