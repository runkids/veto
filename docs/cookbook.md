# Cookbook

Recipes for common tasks. Each one is self-contained.

---

## Basics

### Skip verification for safe commands

Your team runs `terraform plan` constantly. No need to authenticate every time.

Add to `~/.veto/rules.toml`:

```toml
[whitelist]
commands = [
    "terraform plan*",
    "docker ps*",
    "kubectl get*",
]
```

Verify: `veto check "terraform plan"` → ALLOW

---

### Set up PIN

Use PIN instead of (or alongside) Touch ID. Works on all platforms.

```bash
veto auth set-pin
# Enter new PIN (minimum 4 characters): ****
# Confirm PIN: ****
```

Set PIN as default auth:

```toml
# ~/.veto/config.toml
[auth]
default = "pin"
```

Verify: `veto auth test pin`

---

### Set up TOTP

Use Google Authenticator, Authy, or any RFC 6238 app.

```bash
veto auth setup-totp
# Scan QR code with authenticator app
# Enter 6-digit code to verify: 123456
```

Verify: `veto auth test totp`

---

### View what's been blocked

```bash
# All entries
veto log

# Last 5
veto log -n 5

# Only denied commands
veto log --filter DENIED

# Live tail
veto log -f
```

Log format: `[timestamp] RESULT RISK auth_method "command"`

---

## Intermediate

### Create custom rules

Mark specific commands as CRITICAL — strongest auth required.

Add to `~/.veto/rules.toml`:

```toml
[[critical]]
category = "database-drop"
patterns = [
    "drop database*",
    "DROP DATABASE*",
    "dropdb*",
]
reason = "Database destruction"
```

Verify:

```bash
veto check -v "drop database production"
# Risk: CRITICAL
# Category: database-drop
# Reason: Database destruction
```

---

### Approve remotely via Telegram

Get approval requests on your phone. Useful when you're away from the keyboard.

**Setup (3 minutes):**

1. Message **@BotFather** on Telegram → `/newbot` → name it → copy the **bot token**
2. Message **@userinfobot** → copy your **chat ID**
3. Configure veto:

```bash
veto auth setup-telegram
# Enter bot token: [paste]
```

4. Add to `~/.veto/config.toml`:

```toml
[auth.telegram]
enabled = true
chat_id = "123456789"
timeout_seconds = 60
```

5. **Important:** Open your bot in Telegram and tap **Start**

Verify: `veto auth test telegram` — you should receive a message.

Reply `/allow` or `/deny` to approve or reject commands.

---

### Use veto in Docker/CI

veto works without a system keychain. It falls back to encrypted file storage automatically.

No special setup needed. Just install and run.

---

### Configure auth per risk level

Different auth strength for different risk levels:

```toml
# ~/.veto/config.toml
[auth.levels]
low = "confirm"          # y/n prompt
medium = "pin"           # PIN
high = "touchid"         # biometric
critical = "telegram"    # remote approval

[auth.fallback]
touchid = "pin"          # Touch ID unavailable → PIN
telegram = "totp"        # Telegram timeout → TOTP
```

---

## Advanced

### Integrate veto check into scripts

Use exit codes to build custom workflows:

```bash
veto check -q "dangerous command"
case $? in
    0) echo "ALLOW" ;;
    1) echo "LOW" ;;
    2) echo "MEDIUM" ;;
    3) echo "HIGH" ;;
    4) echo "CRITICAL" ;;
esac
```

---

### Challenge-Response authentication

Prevent AI from reusing credentials. veto generates a one-time code that only you can see.

Enable per rule:

```toml
# ~/.veto/rules.toml
[[critical]]
category = "destructive"
patterns = ["rm -rf *"]
reason = "Recursive force delete"
challenge = true
```

Flow:
1. AI triggers the rule → veto generates a 4-digit code
2. Code appears as a macOS notification (or Telegram message)
3. AI can't see it — asks you for the code
4. You provide it → AI retries with `VETO_RESPONSE=<PIN><code>` (e.g., `12344827`)

Properties: 4 digits, 60-second expiry, single-use, command-bound.

---

### Temporarily allow a blocked command

If veto blocks a command you actually want to run:

```bash
# Allow once (interactive — select from recent denials)
veto allow

# Allow with TTL
veto allow "docker system prune*" --ttl 1h
```

Or override a denied command:

```bash
VETO_FORCE=yes <command>
```
