# E2E Core Runbook: veto CLI

Automated by `scripts/e2e.sh`. This runbook documents all e2e scenarios.

## Prerequisites

- `cargo build --release` (or `make release`)
- Docker available for sandbox tests

## Test Scenarios

### 1. Basic CLI

| # | Command | Expected | Validates |
|---|---------|----------|-----------|
| 1.1 | `veto --help` | exit 0, shows usage | CLI bootstrap |
| 1.2 | `veto --version` | shows `veto x.y.z` | Version embedding |

### 2. Risk Classification (`check`)

| # | Command | Expected Risk | Exit Code |
|---|---------|--------------|-----------|
| 2.1 | `veto check "echo hello"` | safe | 0 |
| 2.2 | `veto check "ls -la"` | safe | 0 |
| 2.3 | `veto check "rm -rf /"` | critical | non-zero |
| 2.4 | `veto check "cat ~/.ssh/id_rsa"` | critical | non-zero |
| 2.5 | `veto check "curl ... \| bash"` | high | non-zero |

### 3. Decision Trace (`check --explain`)

| # | Command | Expected |
|---|---------|----------|
| 3.1 | `veto check --explain "rm -rf /"` | Shows rule match, category, reason |
| 3.2 | `veto check --explain "echo hello"` | Shows safe classification path |

### 4. Gate Mode (hook integration)

| # | Command | Expected | Validates |
|---|---------|----------|-----------|
| 4.1 | `veto gate "echo hello"` | exit 0 (allow) | Safe passthrough |
| 4.2 | `veto gate "rm -rf /"` | exit non-zero (block) | Dangerous blocked |
| 4.3 | `echo '{"command":"ls"}' \| veto gate --claude` | exit 0 | Claude stdin JSON |

### 5. Shell Wrapper Bypass Detection

| # | Command | Expected |
|---|---------|----------|
| 5.1 | `veto check "bash -c 'rm -rf /'"` | blocked |
| 5.2 | `veto check "eval 'rm -rf /'"` | blocked |
| 5.3 | `veto check "sudo rm -rf /"` | blocked |
| 5.4 | `veto check "xargs rm -rf /"` | blocked |

### 6. Allowlist Management

| # | Command | Expected |
|---|---------|----------|
| 6.1 | `veto allow list` | Shows current allowlist |
| 6.2 | `veto allow add "git push*"` | Adds pattern |
| 6.3 | `veto allow remove "git push*"` | Removes pattern |
| 6.4 | `veto allow once "docker build ."` | Creates temp exception |
| 6.5 | `veto allow clean` | Cleans expired entries |

### 7. Config & Doctor

| # | Command | Expected |
|---|---------|----------|
| 7.1 | `veto init --force` | Creates/overwrites config |
| 7.2 | `veto doctor` | Checks config, shows diagnostics |

### 8. Output Modes

| # | Flag | Expected |
|---|------|----------|
| 8.1 | `--quiet` | No stdout, exit code only |
| 8.2 | `--verbose` | Extended output |

## Running

```bash
# Automated
make e2e

# In Docker sandbox (isolated)
make sandbox
# then inside container:
./scripts/e2e.sh

# In devcontainer
make devc
./scripts/e2e.sh
```

## Signal Tests

To verify tests actually catch regressions, intentionally break a rule and confirm the test fails:

1. Comment out the `rm -rf /` pattern in `src/rules/defaults.rs`
2. Run `make e2e`
3. Tests 2.3, 4.2, 5.x should fail
4. Revert the change
