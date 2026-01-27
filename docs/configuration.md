# Configuration

## Directory Structure

Config files are stored in `~/.veto/`:

```
~/.veto/
├── config.toml    # Auth settings
├── rules.toml     # Custom rules (optional)
└── secrets/       # Encrypted secrets (fallback)
```

## Initialize Configuration

```bash
veto init
```

This creates default configuration files.

## config.toml — Full Example

```toml
# ============================================================
# DEFAULT AUTHENTICATION
# ============================================================
[auth]
# Fallback when no level-specific method is configured
default = "confirm"

# ============================================================
# PER-LEVEL AUTHENTICATION
# ============================================================
[auth.levels]
# Risk level → authentication method(s)
low = "confirm"                    # Simple y/n
medium = "pin"                     # PIN required
high = "touchid"                   # Biometric (macOS)
critical = ["totp", "telegram"]    # Chain: ALL must pass

# ============================================================
# FALLBACK CONFIGURATION
# ============================================================
[auth.fallback]
# When primary method unavailable, use fallback
touchid = "pin"       # Touch ID unavailable → use PIN
telegram = "totp"     # Telegram timeout → use TOTP
totp = "pin"          # TOTP not configured → use PIN

# ============================================================
# METHOD-SPECIFIC SETTINGS
# ============================================================

# PIN — stored as Argon2 hash in system keychain
[auth.pin]
enabled = true

# TOTP — Google Authenticator compatible
[auth.totp]
enabled = true
issuer = "veto"       # Shown in authenticator app

# Touch ID — macOS only
[auth.touchid]
enabled = true
prompt = "Authorize veto operation"

# Telegram — async approval via bot
[auth.telegram]
enabled = true
chat_id = "123456789"           # Your Telegram user ID
timeout_seconds = 60            # Wait time for /allow or /deny
```

## Configuration Options

### [auth]

| Key | Type | Description |
|-----|------|-------------|
| `default` | string | Default auth method when no level-specific method |

### [auth.levels]

| Key | Type | Description |
|-----|------|-------------|
| `low` | string/array | Auth for LOW risk commands |
| `medium` | string/array | Auth for MEDIUM risk commands |
| `high` | string/array | Auth for HIGH risk commands |
| `critical` | string/array | Auth for CRITICAL risk commands |

### [auth.fallback]

| Key | Type | Description |
|-----|------|-------------|
| `touchid` | string | Fallback when Touch ID unavailable |
| `telegram` | string | Fallback when Telegram times out |
| `totp` | string | Fallback when TOTP not configured |

### [auth.pin]

| Key | Type | Description |
|-----|------|-------------|
| `enabled` | bool | Enable PIN authentication |

### [auth.totp]

| Key | Type | Description |
|-----|------|-------------|
| `enabled` | bool | Enable TOTP authentication |
| `issuer` | string | Issuer name shown in authenticator app |

### [auth.touchid]

| Key | Type | Description |
|-----|------|-------------|
| `enabled` | bool | Enable Touch ID (macOS only) |
| `prompt` | string | Prompt message for Touch ID |

### [auth.telegram]

| Key | Type | Description |
|-----|------|-------------|
| `enabled` | bool | Enable Telegram authentication |
| `chat_id` | string | Your Telegram user ID |
| `timeout_seconds` | int | Timeout for approval (default: 60) |

## Verify Configuration

```bash
veto doctor
```
