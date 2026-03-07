---
name: veto-codebase-audit
description: >-
  Cross-validate CLI flags, rules, tests, and integrations for consistency
  across the veto codebase. Use this skill whenever the user asks to: audit the
  codebase, check for consistency issues, find undocumented flags, verify test
  coverage, validate rule definitions, or check integration completeness. This
  is a read-only audit — it reports issues but never modifies files. Use after
  large refactors, before releases, or whenever you suspect code has drifted.
targets: [claude]
---

Read-only consistency audit across the veto codebase. $ARGUMENTS specifies focus area (e.g., "rules", "flags", "tests", "auth") or omit for full audit.

**Scope**: This skill only READS and REPORTS. It does not modify any files.

## Audit Dimensions

Run all dimensions in parallel where possible. For each, produce a summary table.

### 1. CLI Flag Audit

Compare every flag/subcommand in `src/cli/mod.rs` against README and `--help` output.

```bash
# Extract all Clap derive attributes
grep -n '#\[arg\|#\[command' src/cli/mod.rs

# Check README documents them
grep -n 'veto ' README.md
```

Report:
- **UNDOCUMENTED**: Flag exists in code but not in README
- **STALE**: Flag documented but not in code
- **OK**: Flag matches

### 2. Rule Coverage Audit

Validate rules in `src/rules/defaults.rs`:

```bash
# Count rules per category
grep -c 'category:' src/rules/defaults.rs

# Check each rule pattern is tested
grep -rn '#\[test\]' src/rules/
```

Report per risk level (critical, high, medium, low):
- **TESTED**: Rule pattern has matching test case
- **UNTESTED**: Rule exists but no test covers it
- **ORPHAN**: Test references pattern not in defaults

### 3. Auth Method Audit

Cross-check auth methods across:
- `src/auth/manager.rs` (dispatch)
- `src/cli/mod.rs` (--auth flag options)
- `src/commands/auth.rs` (auth subcommands)

```bash
grep -n 'auth' src/auth/manager.rs
grep -n 'auth' src/cli/mod.rs
```

Report:
- **COMPLETE**: Auth method registered in all 3 locations
- **PARTIAL**: Missing from one or more locations
- **DEAD**: Registered but implementation missing

### 4. Integration Audit

Check `src/commands/setup.rs` integrations:

```bash
# List setup subcommands
grep -n 'Setup\|setup' src/cli/mod.rs

# Check each has implementation
ls src/commands/setup.rs
```

Verify each integration (Claude, Gemini, OpenCode, Cursor) has:
- Setup command handler
- Gate mode support (`--claude`, `--gemini`, etc.)
- Uninstall support

Report:
- **COMPLETE**: Integration fully wired
- **PARTIAL**: Missing setup/gate/uninstall
- **STUB**: Defined but not implemented

### 5. Test Coverage

For each module:

```bash
# Find modules with tests
grep -rn '#\[cfg(test)\]' src/

# Find integration test files
ls tests/ 2>/dev/null
```

Report:
- **COVERED**: Module has test section with cases
- **PARTIAL**: Test section exists but few cases
- **MISSING**: No tests for this module

### 6. Shell Wrapper Bypass Audit

Verify bypass detection in `src/rules/engine.rs` covers known vectors:

| Vector | Expected |
|--------|----------|
| `bash -c` | Detected |
| `sh -c` | Detected |
| `eval` | Detected |
| `sudo` | Detected |
| `xargs` | Detected |
| `env` | Detected |
| pipe to `sh`/`bash` | Detected |

```bash
grep -n 'bash\|eval\|sudo\|xargs' src/rules/engine.rs
```

Report:
- **DETECTED**: Bypass vector is handled
- **MISSING**: Known vector not covered

## Output Format

```
== veto Codebase Audit ==

### CLI Flags (N issues)
| Subcommand | Flag       | Status       |
|------------|------------|--------------|
| check      | --explain  | OK           |
| gate       | --cursor   | UNDOCUMENTED |

### Rules (N issues)
| Level    | Category        | Patterns | Tested |
|----------|-----------------|----------|--------|
| critical | destructive     | 7        | YES    |
| high     | rm-recursive    | 3        | NO     |

### Auth Methods (N issues)
| Method   | Manager | CLI | Commands | Status   |
|----------|---------|-----|----------|----------|
| pin      | Yes     | Yes | Yes      | COMPLETE |
| telegram | Yes     | Yes | Yes      | COMPLETE |

### Integrations (N issues)
| Tool     | Setup | Gate | Uninstall | Status   |
|----------|-------|------|-----------|----------|
| claude   | Yes   | Yes  | Yes       | COMPLETE |
| cursor   | Yes   | Yes  | Yes       | COMPLETE |

### Test Coverage (N issues)
| Module       | Status  | Notes           |
|--------------|---------|-----------------|
| rules/engine | COVERED | 12 test cases   |
| auth/totp    | PARTIAL | missing edge    |

### Bypass Detection (N issues)
| Vector   | Status   |
|----------|----------|
| bash -c  | DETECTED |
| eval     | DETECTED |

== Summary: X OK / Y issues found ==
```

## Rules

- **Read-only** — never modify files, only report
- **Evidence-based** — every finding must include file path and line number
- **No false positives** — verify with grep before flagging
- **Scope $ARGUMENTS** — if user specifies "rules", only run dimension 2
