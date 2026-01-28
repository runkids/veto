# Gemini CLI Integration

## Quick Setup

```bash
veto setup gemini
```

Restart Gemini CLI. Done!

To remove:

```bash
veto setup gemini --uninstall
```

## How It Works

Gemini CLI invokes a `BeforeTool` hook before running `run_shell_command`.
The hook calls `veto gate --gemini` to evaluate risk and enforce authentication.

If a command needs verification, veto responds with a message instructing the
assistant to ask you for a PIN/TOTP/confirmation and retry the command with
the appropriate `VETO_*` prefix.

## Manual Configuration

Add this to `~/.gemini/settings.json`:

```json
{
  "hooks": {
    "BeforeTool": [
      {
        "matcher": "run_shell_command",
        "hooks": [
          {
            "type": "command",
            "command": "veto gate --gemini"
          }
        ]
      }
    ]
  }
}
```

## Verify

```bash
veto doctor
```

## Retry with Credentials

When Gemini CLI asks for a code, retry the command with an environment prefix:

```bash
VETO_PIN=1234 rm -rf node_modules
VETO_TOTP=123456 git push -f origin main
VETO_CONFIRM=yes rm -rf /tmp/example
```

## Denied Commands

If you explicitly deny a dialog/touchid/telegram prompt, veto will block retries
for the same command in hook modes. To retry after explicit approval, add:

```bash
VETO_FORCE=yes <command>
```
