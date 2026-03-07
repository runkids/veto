<p align="center">
  <h1 align="center">veto</h1>
  <p align="center">Human-in-the-loop confirmation for AI Agents</p>
</p>

<p align="center">
  <em>"I let my AI run <code>terraform destroy</code>.<br>Because I can stop it with one touch."</em>
</p>

<p align="center">
  <img src="docs/assets/VetoIntro.gif" alt="Veto Demo" width="700">
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://github.com/runkids/veto/releases"><img src="https://img.shields.io/github/v/release/runkids/veto?display_name=tag&sort=semver" alt="Release"></a>
  <a href="https://github.com/runkids/veto/releases"><img src="https://img.shields.io/github/downloads/runkids/veto/total" alt="Downloads"></a>
  <img src="https://img.shields.io/badge/rust-🦀-orange" alt="Rust">
</p>

---

npm install? **No.**
yaml config? **No.**
Just:

```bash
curl -fsSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash
veto setup claude
```

Your AI now asks before it destroys.

---

## Why

Your AI agent runs as *you*. It doesn't need `sudo` to `rm -rf ~/Documents`.

veto sits between your AI and the terminal. Dangerous command? Touch ID.
Catastrophic command? Blocked. Safe command? Passes through silently.

It's a seatbelt — not a sandbox.

---

<p align="center">
  Touch ID · PIN · TOTP · Telegram
</p>

---

<details>
<summary>How does it work?</summary>

```
+-----------+     +-----------+     +-----------+
| AI Agent  |---->| veto gate |---->| Terminal  |
+-----------+     +-----+-----+     +-----------+
                        |
                  +-----v-----+
                  | Risk Check|
                  +-----+-----+
                        |
        +---------------+---------------+
        |               |               |
        v               v               v
   +--------+      +--------+      +--------+
   |  Low   |      |  High  |      |Critical|
   | [pass] |      |[auth]  |      |[block] |
   +--------+      +--------+      +--------+
```

`veto setup claude` adds a PreToolUse hook to `~/.claude/settings.json`.
Every command Claude Code tries to run goes through veto first.

</details>

---

<p align="center">
  <a href="docs/getting-started.md"><strong>Getting Started</strong></a> ·
  <a href="docs/cookbook.md">Cookbook</a> ·
  <a href="docs/reference.md">Reference</a> ·
  <a href="docs/troubleshooting.md">Troubleshooting</a>
</p>

---

<p align="center">
  <strong>Works with</strong> Claude Code<br>
  <strong>Runs on</strong> macOS (x86/ARM) · Linux (x86/ARM)
</p>

<p align="center">
  <sub>Built with Rust.</sub>
</p>
