# Testing Design: Risk-Driven Hybrid Strategy

Date: 2026-03-07

## Goal

Bring veto test coverage from 41% (14/34 files) to comprehensive coverage across all critical modules, using a mix of unit tests, integration tests (assert_cmd), and e2e (bash).

## Current State

- 43 unit tests (focused on rules/engine and auth/)
- 0 integration tests (no tests/ directory)
- 13 e2e scenarios (scripts/e2e.sh)
- No tests: audit.rs, config/types.rs, rules/defaults.rs, 6 command handlers

## Approach: Risk-Driven Mixed

Priority by "cost of failure":

| Priority | Module | Layer | Why |
|----------|--------|-------|-----|
| P0 | rules/defaults | unit | Wrong match = dangerous command allowed |
| P0 | rules/engine (strengthen) | unit | Bypass detection is security core |
| P1 | CLI subcommands | integration (assert_cmd) | Verify user-facing behavior |
| P1 | audit.rs | unit | Wrong audit = can't track incidents |
| P2 | config/types | unit | Parse error = whole system misbehaves |
| P2 | e2e.sh expansion | e2e | Cross-command workflows |

## Test Depth

Happy path + 2-3 edge cases per module (empty input, invalid data, file not found).

## Unit Tests

### P0: rules/defaults.rs

Test `default_rules()` with `RulesEngine` against representative commands per risk level:
- Critical: `rm -rf /`, `mkfs.ext4 /dev/sda`, `cat ~/.ssh/id_rsa`, `write_file:/etc/passwd`
- High: `rm -rf ./project`, `cat .env`, `git push --force`, `bash -c "cmd"`, `sudo rm`, `write_file:.env`
- Medium: `git push origin main`, `npm install foo`, `xargs rm`
- Low: `rm file.txt`, `curl http://example.com`
- Allow (whitelist): `ls -la`, `cargo test`, `git status`
- Edge: whitelist vs rule priority (`cat .env` — whitelist has `cat *` but high has `cat *.env*`)

### P1: audit.rs

All tests use tempdir with VETO_CONFIG_DIR isolation:
- log_and_read: write entry → read back → verify format
- read_empty: nonexistent log → empty vec
- clear_log: write → clear → confirm gone
- deny_cache_round_trip: record → was_denied = true
- deny_cache_dedup: duplicate record → only one entry
- deny_cache_miss: unrecorded → was_denied = false

### P2: config/types.rs

- config_deserialize_minimal: empty TOML → Config::default
- config_deserialize_full: complete TOML with auth/pin/totp
- auth_method_single: `"pin"` → AuthMethod::Single
- auth_method_array_compat: `["pin", "totp"]` → AuthMethod::Multiple
- risk_level_from_rules: RulesRiskLevel → ConfigRiskLevel conversion

## Integration Tests (assert_cmd)

Structure: `tests/integration/cli_{subcommand}.rs`

### cli_check.rs
- check_safe_command: exit 0, stdout ALLOW
- check_critical_blocked: exit non-zero, stdout CRITICAL
- check_high_blocked: exit non-zero
- check_medium: exit non-zero, stdout MEDIUM
- check_explain_shows_trace: stdout has pattern/category info
- check_quiet_mode: exit 0
- check_compound_command: `ls && rm -rf /` → non-zero

### cli_gate.rs
- gate_safe_allows: exit 0
- gate_claude_stdin: JSON stdin + --claude → exit 0

### cli_init.rs
- init_creates_config: exit 0, config file exists
- init_force_overwrites: double init → no error

### cli_allow.rs
- allow_list_empty: exit 0
- allow_add_and_list: add pattern → list shows it
- allow_remove: add → remove → list → gone

### cli_doctor.rs
- doctor_runs: produces diagnostic output

### cli_log.rs
- log_empty: exit 0 on empty config dir
- log_clear: exit 0

## E2E Expansion (scripts/e2e.sh)

New scenarios:
- Compound command bypass: `echo safe && rm -rf /`
- File operation rules: `veto check "write_file:/etc/passwd"`
- Git destructive: `veto check "git push --force"`
- Allowlist workflow: init → add → check (allowed) → remove → check (blocked again)
