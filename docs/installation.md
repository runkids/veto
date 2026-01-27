# Installation

## Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash
```

This script:
1. Detects your OS and architecture
2. Downloads the appropriate binary
3. Installs to `~/.local/bin/veto`
4. Adds to PATH if needed

## Uninstall

```bash
curl -sSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash
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
