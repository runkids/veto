# Rules Configuration

## Overview

veto uses rules to evaluate command risk levels. Rules are checked in order:

1. **Whitelist** — Always allow (overrides all other rules)
2. **Critical** — Highest risk, strongest auth required
3. **High** — High risk operations
4. **Medium** — Moderate risk
5. **Low** — Low risk, light confirmation
6. **Default** — Commands not matching any rule

## Default Rules (Built-in)

### CRITICAL — System destruction, credential exposure

```
Category: destructive
  rm -rf /
  rm -rf /*
  rm -rf ~
  mkfs*
  dd if=* of=/dev/*

Category: credentials
  *AWS_SECRET*
  *PRIVATE_KEY*
  cat ~/.ssh/id_*
  cat *id_rsa*
```

### HIGH — Force operations, secrets access

```
Category: rm-recursive-force
  rm -rf *
  rm -fr *

Category: secrets
  cat *.env*
  cat .env
  cat *secret*
  cat *password*

Category: git-destructive
  git push*--force*
  git push*-f*
  git reset --hard*
  git clean -fd*
```

### MEDIUM — Recursive delete, remote git, package install

```
Category: rm-recursive
  rm -r *
  rm -R *

Category: git
  git push*
  git merge*
  git rebase*

Category: install
  npm install*
  pip install*
  cargo install*
  brew install*
  apt install*
```

### LOW — Simple delete, network

```
Category: rm
  rm *

Category: network
  curl*
  wget*
```

### ALLOW — Whitelisted safe commands

```
ls*, pwd, echo *, cat *, head *, tail *, grep *, find *
which *, whoami, date
cargo build*, cargo test*, cargo check*, cargo fmt*, cargo clippy*
npm run*, npm test*
git status*, git log*, git diff*, git branch*, git show*
```

## Custom Rules — `~/.veto/rules.toml`

```toml
# ============================================================
# WHITELIST — Always allow (overrides all other rules)
# ============================================================
[whitelist]
commands = [
    "ls*",
    "pwd",
    "echo *",
    "cargo build*",
    "cargo test*",
    "git status*",
    "git log*",
    "git diff*",
    # Add your safe commands here
    "docker ps*",
    "docker logs*",
    "kubectl get*",
]

# ============================================================
# CRITICAL — Requires strongest authentication
# ============================================================
[[critical]]
category = "database-drop"
patterns = [
    "drop database*",
    "DROP DATABASE*",
    "dropdb*",
]
reason = "Database destruction"

[[critical]]
category = "production-deploy"
patterns = [
    "*deploy*prod*",
    "*production*deploy*",
]
reason = "Production deployment"

# ============================================================
# HIGH — Requires strong authentication
# ============================================================
[[high]]
category = "docker-destructive"
patterns = [
    "docker rm -f*",
    "docker system prune*",
    "docker volume rm*",
]
reason = "Docker resource deletion"

[[high]]
category = "k8s-destructive"
patterns = [
    "kubectl delete*",
    "kubectl drain*",
]
reason = "Kubernetes destructive operation"

# ============================================================
# MEDIUM — Requires confirmation
# ============================================================
[[medium]]
category = "docker-build"
patterns = [
    "docker build*",
    "docker-compose up*",
]
reason = "Docker operation"

[[medium]]
category = "database-modify"
patterns = [
    "psql*",
    "mysql*",
    "mongosh*",
]
reason = "Database access"

# ============================================================
# LOW — Logged, light confirmation
# ============================================================
[[low]]
category = "ssh"
patterns = [
    "ssh *",
    "scp *",
]
reason = "Remote connection"
```

## Rule Pattern Syntax

| Pattern | Matches |
|---------|---------|
| `rm *` | `rm` followed by anything |
| `*secret*` | Contains "secret" anywhere |
| `git push*-f*` | `git push` with `-f` anywhere after |
| `cat ~/.ssh/id_*` | Exact path prefix |

### Pattern Examples

| Pattern | Example Matches |
|---------|-----------------|
| `rm -rf *` | `rm -rf node_modules`, `rm -rf /tmp/test` |
| `*password*` | `echo $PASSWORD`, `cat password.txt` |
| `git push*--force*` | `git push origin main --force` |
| `docker rm*` | `docker rm container1`, `docker rm -f all` |

## Rule Evaluation Order

1. Check whitelist first — if matches, ALLOW
2. Check critical rules — if matches, CRITICAL
3. Check high rules — if matches, HIGH
4. Check medium rules — if matches, MEDIUM
5. Check low rules — if matches, LOW
6. No match — use default (usually ALLOW)

## Debugging Rules

```bash
veto check -v "your command"
# Risk: HIGH
# Category: git-destructive
# Reason: Destructive git operation
# Pattern: git push*-f*
```
