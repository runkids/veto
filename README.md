<p align="center">
  <h1 align="center">veto</h1>
  <p align="center">🛡️ Human-in-the-loop confirmation for AI Agents</p>
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
veto setup claude   # or: gemini / cursor / opencode
```

Your AI now asks before it destroys.

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

`veto setup` adds a hook to your AI tool's config:

| Tool | Config Location |
|------|-----------------|
| Claude Code | `~/.claude/settings.json` |
| Gemini CLI | `~/.gemini/settings.json` |
| Cursor CLI | `.cursor/hooks.json` |
| OpenCode | `~/.opencode/plugins/` |

</details>

---

<p align="center">
  👆 <strong>Touch ID</strong> · 🔢 <strong>PIN</strong> · 🔐 <strong>OTP</strong> · 📱 <strong>Telegram</strong>
</p>

---

<p align="center">
  <a href="docs/installation.md">Install</a> ·
  <a href="docs/claude-code.md">Claude Code</a> ·
  <a href="docs/geminicli.md">Gemini CLI</a> ·
  <a href="docs/configuration.md">Config</a> ·
  <a href="docs/rules.md">Rules</a>
</p>

---

## Philosophy

**veto is a UX layer, not a security sandbox.**

| What veto IS | What veto is NOT |
|--------------|------------------|
| Human-in-the-loop for AI agents | A replacement for OS sandboxing |
| "Are you sure?" before `rm -rf` | Comprehensive security solution |
| Pattern-based risk detection | Bulletproof malware defense |
| 10-second setup | Complex security infrastructure |

> *"But can't I just use sudo?"*
>
> `sudo` gates **privilege escalation**. AI agents run as *you* — they don't need root to `rm -rf ~/Documents`. veto gates **agent actions**, not permissions.

> *"Heuristics can be bypassed!"*
>
> True. veto catches the 99% case: AI confidently running `git push --force` or `terraform destroy`. It's a seatbelt, not an airbag — and most accidents don't need an airbag.

---

<p align="center">
  <strong>Works with</strong><br>
  Claude Code · Gemini CLI · Cursor CLI · OpenCode
</p>

<p align="center">
  <strong>Runs on</strong><br>
  macOS (x86/ARM) · Linux (x86/ARM)
</p>

---

<p align="center">
  <sub>Built with Rust.</sub>
</p>
