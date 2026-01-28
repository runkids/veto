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

If verification is required, veto can return a denial or an approval request:

```json
{"continue": false, "permission": "deny", "user_message": "..."}
```

```json
{"continue": true, "permission": "ask", "user_message": "...", "agent_message": "..."}
```

## Hook Location (Global)

The global Cursor CLI hooks file is:

```
~/.cursor/hooks.json
```

veto writes a `beforeShellExecution` hook pointing to `veto gate --cursor`.

## Recommended Authentication

- **Recommended:** `dialog` or `touchid` (fully enforced by veto)
- **Confirm:** returns a Cursor approval prompt (`permission: ask`)
- **PIN/TOTP:** not supported inside Cursor CLI hooks
  - Run in a terminal with `VETO_PIN=<code>` / `VETO_TOTP=<code>`
  - Or switch auth to `dialog`/`touchid` in `~/.veto/config.toml`

## Debugging

```bash
cat ~/.cursor/hooks.json
veto gate --cursor <<<'{"command":"rm -rf test","cwd":"/tmp"}'
```
