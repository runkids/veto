# Getting Started

Get veto running in 5 minutes.

## Prerequisites

- macOS or Linux
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed

## 1. Install

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash
```

Prefer to inspect first?

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/install.sh -o install.sh
less install.sh
bash install.sh
```

## 2. Initialize

```bash
veto init
```

Creates `~/.veto/config.toml` with sensible defaults.
macOS uses Touch ID. Linux uses PIN (you'll be prompted to set one).

## 3. Connect to Claude Code

```bash
veto setup claude
```

This adds a PreToolUse hook to `~/.claude/settings.json`.
Restart Claude Code for the hook to take effect.

## 4. Verify

```bash
veto doctor
```

You should see:

```
veto Doctor
  Configuration:
    config.toml: found
  Claude Code Integration:
    settings.json: found
    PreToolUse hook: configured
    veto binary: accessible
```

## 5. Try It

Open Claude Code and ask it to run something risky:

> "Delete the node_modules folder"

veto will intercept `rm -rf node_modules` and ask you to authenticate.
Approve with Touch ID (or PIN on Linux) — the command runs.
Deny — Claude Code stops completely. No retry, no "what should I do instead?".

That's it. veto is protecting you.

## Next Steps

- [Cookbook](cookbook.md) — whitelist safe commands, set up PIN/TOTP, custom rules
- [Reference](reference.md) — all CLI flags, config options, built-in rules
