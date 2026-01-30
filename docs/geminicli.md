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

Gemini CLI invokes `BeforeTool` hooks before executing tools:

1. **Shell commands** (`run_shell_command`) - veto evaluates command risk
2. **File operations** (`write_file`, `edit_file`, `replace_in_file`) - veto checks file path sensitivity

If an operation needs verification, veto responds with a message instructing the
assistant to ask you for a PIN/TOTP/confirmation and retry with the appropriate
`VETO_*` prefix.

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
            "name": "veto-gate-shell",
            "type": "command",
            "command": "veto gate --gemini",
            "timeout": 90000,
            "description": "Security gate for shell commands"
          }
        ]
      },
      {
        "matcher": "write_file|edit_file|replace_in_file",
        "hooks": [
          {
            "name": "veto-gate-file",
            "type": "command",
            "command": "veto gate --gemini --file-op",
            "timeout": 30000,
            "description": "Security gate for file write operations"
          }
        ]
      }
    ]
  }
}
```

## Managing Hooks

Use Gemini CLI's built-in commands to manage hooks:

```bash
/hooks panel           # View all hooks
/hooks enable-all      # Enable all hooks
/hooks disable-all     # Disable all hooks
/hooks enable veto-gate-shell   # Enable specific hook
/hooks disable veto-gate-file   # Disable specific hook
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
