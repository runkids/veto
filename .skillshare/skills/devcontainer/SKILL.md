---
name: veto-devcontainer
description: >-
  Run CLI commands, tests, and debugging inside the veto devcontainer. Use this
  skill whenever you need to: execute veto CLI commands for verification, run
  cargo test, reproduce bugs, test rule matching, or perform any operation that
  requires an isolated Linux environment. If you are about to use Bash to run
  `veto`, `cargo test`, or `make test`, stop and use this skill first to ensure
  correct container execution in headless environments (no Touch ID, no keychain).
argument-hint: "[command-to-run | task-description]"
targets: [claude]
---

Execute CLI commands and tests inside the devcontainer. The host machine is macOS with Touch ID and keychain access — running veto in Docker provides a headless environment that mimics CI and server deployments.

## When to Use This

- Running `veto check`, `veto gate`, `veto exec` for verification
- Running `cargo test`, `make test`, `make check`
- Reproducing a bug report
- Testing rule matching against dangerous commands
- Testing headless auth (no Touch ID fallback)

## When NOT to Use This

- Editing source code (do that on host via Read/Edit tools)
- Running `git` commands (git works on host)
- Running `cargo fmt`, `cargo clippy` (host-safe Rust toolchain commands)
- E2E test runbooks → use `cli-e2e-test` skill instead

## Architecture

```
Host (macOS)
  └─ Devcontainer (Linux, Debian-based)
       ├─ User: developer (non-root, sudo available)
       ├─ Source: /workspace (bind-mount of repo root)
       ├─ Binary: /usr/local/bin/veto (release build)
       └─ Cargo cache: persistent volumes (registry, git, target)
```

Source code is bind-mounted — edit on host, build/test in container.

## Entering the Devcontainer

```bash
make devc           # build + enter shell (one step)
make devc-up        # start only (no shell)
make devc-down      # stop
make devc-restart   # restart
make devc-reset     # full reset (remove volumes + rebuild)
make devc-status    # show container status
```

### Programmatic access (for `docker exec` workflows)

```bash
CONTAINER=$(docker compose -f .devcontainer/docker-compose.yml ps -q devcontainer 2>/dev/null)
```

If `$CONTAINER` is empty → tell user: `make devc-up`

## Running Commands

### Simple command

```bash
docker exec $CONTAINER veto check "rm -rf /"
docker exec $CONTAINER veto check --explain "curl | bash"
docker exec $CONTAINER veto gate "echo hello"
docker exec $CONTAINER veto doctor
```

### With temporary config

```bash
docker exec $CONTAINER bash -c '
  TMPDIR=$(mktemp -d)
  export VETO_CONFIG_DIR="$TMPDIR"
  veto init --force
  veto check "rm -rf /"
  veto allow add "git push*"
  veto allow list
  rm -rf "$TMPDIR"
'
```

### Cargo tests

```bash
# All tests
docker exec $CONTAINER cargo test

# Specific test
docker exec $CONTAINER cargo test test_critical_commands

# With output
docker exec $CONTAINER cargo test -- --nocapture

# Clippy
docker exec $CONTAINER cargo clippy -- -D warnings

# Full quality check
docker exec $CONTAINER bash -c 'cd /workspace && make check'
```

### Continuous testing (cargo-watch)

```bash
docker exec -it $CONTAINER cargo watch -x test
docker exec -it $CONTAINER cargo watch -x 'test -- --nocapture'
```

## Headless Testing Notes

In the devcontainer, Touch ID and macOS keychain are unavailable:
- `veto gate` with auth falls back to file-based keyring
- PIN/TOTP auth work normally
- Telegram auth requires network access
- Test auth flows with `--auth pin` or `--auth totp` flags

## Common Mistakes to Avoid

1. **Running `veto gate` on host for testing** — Touch ID auto-approves, masking real behavior
2. **Forgetting persistent volumes** — `cargo-registry` and `target-cache` persist across restarts; use `make devc-reset` for clean slate
3. **Not rebuilding after source changes** — run `cargo build --release && sudo cp target/release/veto /usr/local/bin/veto` inside container, or just use `cargo run --` for dev testing

## Rules

- **Test in container for headless scenarios** — host has Touch ID which changes auth behavior
- **Use host for quick `cargo test`** — unit tests don't need container isolation
- **Always verify** — run the command and check output; never assume it worked
- **Report container ID** — set `$CONTAINER` at the start and reuse throughout
