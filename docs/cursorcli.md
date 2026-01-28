# Cursor CLI Integration

## Quick Setup

```bash
veto init             # Create config (first time)
veto setup cursor     # Install hook
veto doctor           # Verify setup
```

Restart Cursor CLI. Done!

To remove: `veto setup cursor --uninstall`

## How It Works

Cursor CLI runs `beforeShellExecution` hooks before executing shell commands.
The hook calls `veto gate --cursor` which returns JSON like:

```json
{"continue": true, "permission": "allow"}
```

If verification is required, veto returns a denial:

```json
{"continue": false, "permission": "deny", "user_message": "..."}
```

## Hook Location (Global)

The global Cursor CLI hooks file is:

```
~/.cursor/hooks.json
```

veto writes a `beforeShellExecution` hook pointing to `veto gate --cursor`.

## Recommended Authentication

- **Recommended:** `dialog` or `touchid` (fully enforced by veto)
- **Confirm:** denied in Cursor CLI (no re-prompt). Use dialog/touchid instead
- **PIN/TOTP:** not supported inside Cursor CLI hooks
  - Run in a terminal with `VETO_PIN=<code>` / `VETO_TOTP=<code>`
  - Or switch auth to `dialog`/`touchid` in `~/.veto/config.toml`

If a command is explicitly rejected (dialog/touchid/telegram), Cursor CLI will not re-prompt.
To retry after an explicit user approval, run with `VETO_FORCE=yes`.

## Debugging

```bash
cat ~/.cursor/hooks.json
veto gate --cursor <<<'{"command":"rm -rf test","cwd":"/tmp"}'
```
