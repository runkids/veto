---
name: veto-implement-feature
description: >-
  Implement a feature or fix using TDD workflow in the veto Rust codebase. Use
  this skill whenever the user asks to: add a new rule category, implement a CLI
  subcommand, add an auth method, modify the risk engine, add a new integration
  (setup command), or write Rust code for veto. This skill enforces test-first
  development and follows veto's module conventions. If the request involves
  writing Rust code and tests, use this skill.
argument-hint: "[spec-file-path | feature description]"
targets: [claude]
---

Implement a feature following TDD workflow. $ARGUMENTS is a spec file path or a plain-text feature description.

**Scope**: This skill writes Rust code and tests. It does NOT update docs (update README manually if needed).

## Workflow

### Step 1: Understand Requirements

If $ARGUMENTS is a file path:
1. Read the spec file
2. Extract acceptance criteria and edge cases
3. Identify affected modules

If $ARGUMENTS is a description:
1. Search existing code for related functionality
2. Identify the right module to extend
3. Confirm scope with user before proceeding

### Step 2: Identify Affected Files

Map the feature to veto's module structure:

```
src/
├── main.rs                    # Entry point
├── cli/mod.rs                 # Clap CLI definitions
├── config/
│   ├── mod.rs                 # Config module root
│   ├── types.rs               # Config structs
│   └── loader.rs              # Config file loading
├── rules/
│   ├── mod.rs                 # Rules module root
│   ├── types.rs               # Rule, Rules, Whitelist structs
│   ├── engine.rs              # Risk evaluation engine
│   └── defaults.rs            # Built-in rule patterns
├── executor/
│   ├── mod.rs                 # Command execution
│   └── shell.rs               # Shell interaction
├── auth/
│   ├── mod.rs                 # Auth module root
│   ├── manager.rs             # Auth method dispatch
│   ├── pin.rs                 # PIN auth
│   ├── totp.rs                # TOTP auth
│   ├── touchid.rs             # macOS Touch ID
│   ├── telegram.rs            # Telegram bot auth
│   ├── keyring.rs             # System keychain
│   ├── challenge.rs           # Challenge generation
│   ├── confirm.rs             # Confirmation prompts
│   └── dialog.rs              # Dialog-based auth
├── commands/
│   ├── mod.rs                 # Command dispatch
│   ├── init.rs                # veto init
│   ├── doctor.rs              # veto doctor
│   ├── setup.rs               # veto setup (integrations)
│   ├── shell.rs               # veto shell
│   ├── auth.rs                # veto auth
│   ├── upgrade.rs             # veto upgrade
│   ├── log.rs                 # veto log
│   └── allow.rs               # veto allow
└── audit.rs                   # Audit logging
```

### Step 3: Write Failing Tests First (RED)

Write tests in the relevant module's `#[cfg(test)] mod tests`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_basic_case() {
        // Setup
        let rules = default_rules();
        let engine = RuleEngine::new(&rules);

        // Act
        let result = engine.evaluate("some command");

        // Assert
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }
}
```

For integration tests, use `assert_cmd` in `tests/`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_check_blocks_dangerous_command() {
    Command::cargo_bin("veto")
        .unwrap()
        .args(&["check", "rm -rf /"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("CRITICAL"));
}
```

Verify tests fail:
```bash
cargo test test_feature_basic_case
```

### Step 4: Implement (GREEN)

Write minimal code to make tests pass:

1. Follow existing patterns in each module
2. Use `colored` crate for terminal output
3. Use `thiserror` for error types
4. Register new CLI commands in `cli/mod.rs` (Clap derive)
5. Wire command dispatch in `commands/mod.rs`

Key patterns:
- **Rules**: Add patterns to `defaults.rs`, types to `types.rs`, logic to `engine.rs`
- **Auth**: Add method to `auth/`, register in `manager.rs`
- **Commands**: Add handler in `commands/`, register in `cli/mod.rs`
- **Config**: Add fields to `types.rs`, parsing to `loader.rs`

Verify tests pass:
```bash
cargo test
```

### Step 5: Quality Check

```bash
make check  # cargo fmt --check + cargo clippy -D warnings + cargo test
```

Fix any issues before proceeding.

### Step 6: E2E Verification

If the feature affects CLI-visible behavior:
1. Update `scripts/e2e.sh` with new test cases
2. Run `make e2e` to verify

### Step 7: Report

1. List all created/modified files
2. Confirm each acceptance criterion is met with test evidence
3. Show `cargo test` output as proof

## Rules

- **Test-first** — always write failing test before implementation
- **Minimal code** — only write what's needed to pass tests
- **Follow patterns** — match existing code style in each module
- **3-strike rule** — if a test fails 3 times after fixes, stop and report
- **No speculative features** — only implement what's in the spec/description
- **Verify with `make check`** — must pass fmt, clippy, and all tests
