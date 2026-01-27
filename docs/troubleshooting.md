# Troubleshooting

## Common Issues

| Issue | Solution |
|-------|----------|
| `config not found` | Run `veto init` |
| PIN not working | Run `veto auth set-pin` to reset |
| TOTP invalid code | Check device time sync |
| Touch ID unavailable | macOS only; falls back to password |
| Telegram timeout | Increase `timeout_seconds` in config |
| Keyring errors | Check `veto doctor`; uses file fallback |

## Diagnostic Commands

### Full System Check

```bash
veto doctor
```

This checks:
- Configuration files
- Authentication methods
- Keyring/secret storage
- Claude Code integration
- Binary accessibility

### Check Specific Command

```bash
veto check -v "your command"
# Shows: Risk level, Category, Reason, Pattern matched
```

## Authentication Issues

### PIN Not Working

```bash
# Reset PIN
veto auth set-pin

# Test PIN
veto auth test pin
```

### TOTP Invalid Code

1. Check device time is synchronized
2. Ensure you're using the correct authenticator app entry
3. Try regenerating TOTP setup:

```bash
veto auth remove totp
veto auth setup-totp
```

### Touch ID Not Working

Touch ID is macOS only. If it fails:
- System will fall back to password
- Configure fallback in config:

```toml
[auth.fallback]
touchid = "pin"
```

### Telegram Timeout

1. Increase timeout in config:

```toml
[auth.telegram]
timeout_seconds = 120  # 2 minutes
```

2. Verify bot token and chat_id:

```bash
veto auth test telegram
```

## Keyring Issues

### Keyring Not Available

veto automatically falls back to encrypted file storage. Check status:

```bash
veto doctor
# Keyring Status:
#   Backend: file (fallback)
```

### Linux Keyring

On Linux, veto uses Secret Service (GNOME Keyring or KWallet). If not available:

1. Install GNOME Keyring: `apt install gnome-keyring`
2. Or use KWallet: `apt install kwalletmanager`
3. Or let veto use file fallback (automatic)

## Claude Code Integration

### Hooks Not Working

1. Verify installation:

```bash
veto doctor
# Claude Code Integration:
#   settings.json: found
#   PreToolUse hook: configured
```

2. Restart Claude Code after setup

3. Check settings.json manually:

```bash
cat ~/.claude/settings.json | jq '.hooks'
```

### Reinstall Hooks

```bash
veto setup claude --uninstall
veto setup claude
```

## FAQ

### How do I skip confirmation for a command?

Add to whitelist in `~/.veto/rules.toml`:

```toml
[whitelist]
commands = ["your-safe-command*"]
```

### Can I use veto in Docker/CI?

Yes. File-based keyring is used automatically when system keychain is unavailable.

### How do I see why a command was flagged?

```bash
veto check -v "git push -f origin main"
# Risk: HIGH
# Category: git-destructive
# Reason: Destructive git operation
# Pattern: git push*-f*
```

### Can I override auth for one command?

```bash
veto exec --auth pin "command"      # Use PIN instead of configured
veto exec --auth touchid "command"  # Use Touch ID
```

### How do I completely reset veto?

```bash
# Remove all config and secrets
rm -rf ~/.veto

# Reinitialize
veto init
```

### veto is blocking a safe command. How do I allow it?

Add the command pattern to your whitelist:

```toml
# ~/.veto/rules.toml
[whitelist]
commands = [
    "your-command*",
]
```

### How do I temporarily disable veto?

Remove the Claude Code hooks:

```bash
veto setup claude --uninstall
```

Re-enable later with `veto setup claude`.

## Getting Help

If you encounter issues not covered here:

1. Run `veto doctor` and note the output
2. Check [GitHub Issues](https://github.com/runkids/veto/issues)
3. Open a new issue with diagnostic output
