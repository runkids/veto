---
name: veto-cli-e2e-test
description: >-
  Run isolated E2E tests from ai_docs/tests runbooks against the veto release
  binary. Use this skill whenever the user asks to: run an E2E test, execute a
  test runbook, validate a feature end-to-end, create a new runbook, or test
  CLI behavior in the sandbox. If you need to run a multi-step CLI validation
  sequence (init → check → gate → verify), this is the skill. Prefer this over
  ad-hoc command sequences for any test that follows a runbook or needs
  reproducible results.
argument-hint: "[runbook-name | new]"
targets: [claude]
---

Run isolated E2E tests against the veto release binary. $ARGUMENTS specifies runbook name or "new".

## Flow

### Phase 0: Environment Check

1. Ensure release binary exists:
   ```bash
   cargo build --release
   ```

2. Verify binary works:
   ```bash
   ./target/release/veto --version
   ```

### Phase 1: Detect Scope

1. Read all `*_runbook.md` files under `ai_docs/tests/`
2. Identify recent changes:
   ```bash
   git diff --name-only HEAD~3
   ```
3. Match changes to relevant runbooks:
   - `src/rules/` → `e2e_core_runbook.md` (risk classification tests)
   - `src/auth/` → auth-related runbooks
   - `src/commands/allow.rs` → allowlist runbooks
   - `src/cli/` → CLI flag/subcommand runbooks

### Phase 2: Select Tests

- **Option A**: Run existing runbook (match to recent changes)
- **Option B**: Run `scripts/e2e.sh` (automated 13-test suite)
- **Option C**: Generate new runbook for untested feature
- If $ARGUMENTS specifies a runbook, skip to Phase 3

### Phase 3: Execute

#### Running automated e2e suite:

```bash
make e2e
```

#### Running in Docker sandbox (isolated):

```bash
make sandbox
# Inside sandbox:
./scripts/e2e.sh
```

#### Running specific runbook manually:

1. Read the selected runbook `.md` file
2. Execute each step sequentially
3. After each step, verify conditions in the Expected block
4. Mark each step PASS / FAIL

#### Generating new runbook:

1. Read `git diff HEAD~3` to find changed files in `src/`
2. Read changed files to understand new/modified functionality
3. Verify all CLI flags exist:
   ```bash
   # Check flag exists in cli/mod.rs
   grep -n "flag_name" src/cli/mod.rs
   ```
4. Generate new runbook to `ai_docs/tests/<slug>_runbook.md`
5. Run the quality checklist before executing

### Phase 4: Report

Output summary:
```
── E2E Test Report ──

Runbook:  {runbook name}
Duration: {time}

Step 1: {description}  PASS
Step 2: {description}  PASS
Step 3: {description}  FAIL ← {error detail}
...

Result: {N}/{total} passed
```

If any FAIL → distinguish:
- **Runbook bug**: wrong flag, stale assertion → fix runbook
- **Real bug**: CLI misbehavior → analyze cause, provide fix

## Runbook Quality Checklist

Before executing a newly generated runbook:

- [ ] **All CLI subcommands exist** — verify in `src/cli/mod.rs`
- [ ] **All flags exist** — grep `src/cli/mod.rs` for each `--flag`
- [ ] **Exit codes match** — `check` returns non-zero for blocked commands
- [ ] **Auth context** — `gate` requires auth; in headless env, behavior differs from macOS
- [ ] **Config isolation** — use `VETO_CONFIG_DIR` or `--force` to avoid polluting real config
- [ ] **Binary path** — use `./target/release/veto`, not system-installed version

## Runbook Template

```markdown
# CLI E2E Runbook: <Title>

<One-line summary of what this validates.>

**Origin**: <version> — <why this runbook exists>

## Scope

- <bullet list of behaviors being validated>

## Environment

- Release binary: `./target/release/veto`
- Optional: Docker sandbox (`make sandbox`)

## Steps

### 1. Setup: <description>

\```bash
<commands>
\```

**Expected**: <what should happen>

### 2. <Action>: <description>
...

## Pass Criteria

- All steps marked PASS
- <additional criteria>
```

## Rules

- **Always build release first** — `cargo build --release`
- **Use config isolation** — set `VETO_CONFIG_DIR` to temp dir for stateful tests
- **Don't abort on failure** — record FAIL, continue to next step, summarize at end
- **Verify every step** — never skip Expected checks
- **veto = ./target/release/veto** — use the release binary, not debug
