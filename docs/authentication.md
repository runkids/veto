# Authentication

## Available Methods

| Method | Type | Platform | Security Level |
|--------|------|----------|----------------|
| `confirm` | Sync | All | Low — y/n prompt |
| `pin` | Sync | All | Medium — Argon2 hashed |
| `totp` | Sync | All | High — RFC 6238, 6-digit |
| `touchid` | Sync | macOS | High — Biometric |
| `telegram` | Async | All | High — Remote approval |

## Setup Methods

### PIN

```bash
veto auth set-pin
# Enter new PIN (minimum 4 characters): ****
# Confirm PIN: ****
# ✓ PIN configured successfully!
```

### TOTP (Google Authenticator)

```bash
veto auth setup-totp
# Scan QR code with authenticator app
# Enter 6-digit code to verify: 123456
# ✓ TOTP configured successfully!
```

Compatible apps:
- Google Authenticator
- Authy
- 1Password
- Any RFC 6238 compatible app

### Touch ID (macOS)

No setup required. Uses system authentication.

```bash
veto auth test touchid
# Touch ID / password verification required
# [Touch ID prompt appears]
# ✓ Authentication successful!
```

### Telegram

```bash
veto auth setup-telegram
# 1. Create bot via @BotFather
# 2. Enter bot token: ***
# 3. Get chat_id via @userinfobot
# ✓ Telegram bot token stored!
```

Then configure `~/.veto/config.toml`:

```toml
[auth.telegram]
enabled = true
chat_id = "YOUR_CHAT_ID"
timeout_seconds = 60
```

#### Telegram Bot Setup

1. Open Telegram and search for `@BotFather`
2. Send `/newbot` and follow the prompts
3. Copy the bot token
4. Get your chat_id from `@userinfobot`
5. Run `veto auth setup-telegram` and enter the token

## Verify Setup

```bash
veto auth list
# Configured authentication methods:
#   ✓ confirm - configured
#   ✓ pin - configured
#   ✓ totp - configured
#   ✓ touchid - configured
#   ✗ telegram - not configured

veto auth test pin
# Enter PIN: ****
# ✓ Authentication successful!
```

## Authentication Configuration

### Per-Level Authentication

Configure different auth methods for different risk levels:

```toml
[auth.levels]
low = "confirm"                    # Simple y/n
medium = "pin"                     # PIN required
high = "touchid"                   # Biometric (macOS)
critical = ["totp", "telegram"]    # Chain: ALL must pass
```

### Fallback Configuration

When primary method is unavailable:

```toml
[auth.fallback]
touchid = "pin"       # Touch ID unavailable → use PIN
telegram = "totp"     # Telegram timeout → use TOTP
totp = "pin"          # TOTP not configured → use PIN
```

### Authentication Chains

For critical operations, require multiple factors:

```toml
[auth.levels]
critical = ["totp", "telegram"]  # Must pass BOTH
```

Execution flow:
```
1. User runs: veto exec "rm -rf /"
2. veto: Risk = CRITICAL
3. veto: Enter TOTP code: [user enters]
4. veto: Waiting for Telegram approval...
5. Telegram: [user sends /allow]
6. veto: ✓ Executing command
```

## Secret Storage

veto stores secrets using system keychain with automatic fallback:

| Backend | Platform | Location |
|---------|----------|----------|
| Keychain | macOS | macOS Keychain |
| Secret Service | Linux | GNOME Keyring / KWallet |
| File (fallback) | All | `~/.veto/secrets/*.enc` |

### Security Details

| Component | Algorithm |
|-----------|-----------|
| PIN | Argon2id + random salt |
| TOTP | HMAC-SHA1 (RFC 6238) |
| File encryption | AES-256-GCM + PBKDF2 (100k iterations) |

### Check Backend Status

```bash
veto doctor
# Keyring Status:
#   Backend: system
#   PIN configured: yes
#   TOTP configured: yes
#   Telegram configured: no
#   Keyring test: ✓ (write/read OK)
```

## Managing Authentication

### Remove a Method

```bash
veto auth remove pin
veto auth remove totp
veto auth remove telegram
```

### Reset PIN

```bash
veto auth set-pin
# Will overwrite existing PIN
```

### Test Methods

```bash
veto auth test confirm
veto auth test pin
veto auth test totp
veto auth test touchid
veto auth test telegram
```
