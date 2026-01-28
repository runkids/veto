# Installation

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash
```

Prefer to inspect the installer first?

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/install.sh -o install.sh
less install.sh
bash install.sh
```

This script:
1. Detects your OS and architecture
2. Downloads the appropriate binary
3. Installs to `~/.local/bin/veto`
4. Adds to PATH if needed

## Next Steps

Initialize config:

```bash
veto init
```

If you use Claude Code, enable hooks:

```bash
veto setup claude
```

If you use Gemini CLI, enable hooks:

```bash
veto setup gemini
```

If you use Cursor CLI, enable hooks:

```bash
veto setup cursor
```

Verify everything is wired correctly:

```bash
veto doctor
```

Claude-specific details: [Claude Code integration](claude-code.md).
Gemini-specific details: [Gemini CLI integration](geminicli.md).
Cursor-specific details: [Cursor CLI integration](cursorcli.md).

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash
```

This removes:
- veto binary (from PATH)
- Config directory (`~/.veto`)
- Claude Code hooks
- Gemini CLI hooks
- Cursor CLI hooks
- OpenCode plugin

Keychain secrets (PIN, TOTP, Telegram) are preserved by default. To remove everything:

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash -s -- --purge
```

## Build from Source

### Prerequisites

- Rust 1.85+
- macOS: Xcode Command Line Tools (for Touch ID support)

### Steps

```bash
git clone https://github.com/runkids/veto.git
cd veto
cargo build --release
```

The binary will be at `target/release/veto`.

### Install Locally

```bash
# Using make
make install

# Or manually
cp target/release/veto ~/.local/bin/
```

## Platform-Specific Notes

### macOS

Touch ID authentication is supported natively. No additional setup required.

### Linux

- Touch ID is not available
- Uses Secret Service (GNOME Keyring / KWallet) for secret storage
- Falls back to encrypted file storage if keyring unavailable

## Verify Installation

```bash
veto --version
veto doctor
```

## Homebrew (Future)

```bash
# Coming soon
brew install runkids/tap/veto
```
