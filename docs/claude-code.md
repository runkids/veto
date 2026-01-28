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
                       │               │               ▼
                       │               │      ┌─────────────────┐
                       │               │      │ Authentication  │
                       │               │      │ (Touch ID/PIN/  │
                       │               │      │  TOTP/Dialog)   │
                       │               │      └────────┬────────┘
                       │               │               │
                       │               │        ┌──────┴──────┐
                       │               │        │             │
                       │               │        ▼             ▼
                       │               │   ┌────────┐   ┌────────┐
                       │               │   │Approved│   │Cancelled│
                       │               │   └───┬────┘   └───┬────┘
                       │               │       │            │
                       ▼               ▼       ▼            ▼
                 ┌──────────────────────────────┐    ┌──────────────┐
                 │         exit 0               │    │ JSON output  │
                 │     Command Executes         │    │ continue:    │
                 │                              │    │   false      │
                 └──────────────────────────────┘    └──────┬───────┘
                                                           │
                                                           ▼
                                                    ┌──────────────┐
                                                    │ AI Stops     │
                                                    │ Completely   │
                                                    │ (no retry)   │
                                                    └──────────────┘
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

Currently veto supports Claude Code. More integrations coming soon:

- [Moltbot](https://clawd.bot/) — 🔜 Coming soon
