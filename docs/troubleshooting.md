# Troubleshooting

## First Step: Run Doctor

```bash
veto doctor
```

This checks config, auth methods, keyring, Claude Code hooks, and binary access.

Healthy output:

```
veto Doctor
  Configuration:
    config.toml: found
  Claude Code Integration:
    settings.json: found
    PreToolUse hook: configured
    veto binary: accessible
  Keyring Status:
    Backend: system
    Keyring test: write/read OK
```

---

## Common Issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| `config not found` | No config file | `veto init` |
| PIN not working | Wrong or corrupted PIN | `veto auth set-pin` |
| TOTP invalid code | Device clock out of sync | Sync device time |
| Touch ID unavailable | Not macOS, or hardware issue | Falls back to password; set `[auth.fallback] touchid = "pin"` |
| Telegram timeout | Bot not started or slow network | Increase `timeout_seconds`; verify with `veto auth test telegram` |
| Keyring errors | No system keychain available | veto auto-falls back to file storage |
| Command keeps getting blocked after denial | Deny cache active | Override with `VETO_FORCE=yes` |

---

## Authentication

### Reset PIN

```bash
veto auth set-pin    # Overwrites existing
veto auth test pin   # Verify
```

### TOTP Not Working

1. Check device time is synchronized
2. Regenerate:

```bash
veto auth remove totp
veto auth setup-totp
```

### Telegram Not Responding

1. Make sure you tapped **Start** on your bot
2. Verify bot token and chat_id: `veto auth test telegram`
3. Increase timeout:

```toml
[auth.telegram]
timeout_seconds = 120
```

---

## Claude Code Hooks

### Hooks Not Working

1. Run `veto doctor` — check "PreToolUse hook: configured"
2. Restart Claude Code after running `veto setup claude`
3. Inspect manually: `cat ~/.claude/settings.json | jq '.hooks'`

### Reinstall Hooks

```bash
veto setup claude --uninstall
veto setup claude
```

---

## Keyring

### Linux: No System Keyring

veto falls back to encrypted file storage automatically. To use a system keyring:

- GNOME: `apt install gnome-keyring`
- KDE: `apt install kwalletmanager`

Check status: `veto doctor` → "Keyring Status"

---

## FAQ

### "But can't I just use sudo?"

`sudo` gates **privilege escalation**. AI agents run as *you* — they don't need root to `rm -rf ~/Documents`. veto gates **agent actions**, not permissions.

### "Heuristics can be bypassed!"

True. veto catches the 99% case: AI confidently running `git push --force` or `terraform destroy`. It's a seatbelt, not an airbag — and most accidents don't need an airbag.

### How do I skip confirmation for a command?

Add to whitelist in `~/.veto/rules.toml`:

```toml
[whitelist]
commands = ["your-safe-command*"]
```

### How do I see why a command was flagged?

```bash
veto check -v "git push -f origin main"
```

### Can I use veto in Docker/CI?

Yes. File-based keyring is used automatically.

### How do I completely reset veto?

```bash
rm -rf ~/.veto
veto init
```

### How do I temporarily disable veto?

```bash
veto setup claude --uninstall
# Re-enable later:
veto setup claude
```

---

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash
```

Removes: binary, `~/.veto/`, Claude Code hooks.
Keychain secrets preserved. To remove everything:

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash -s -- --purge
```

---

## Getting Help

1. Run `veto doctor` and copy the output
2. Check [GitHub Issues](https://github.com/runkids/veto/issues)
3. Open a new issue with diagnostic output
