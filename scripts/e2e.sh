#!/usr/bin/env bash
# End-to-end tests for veto in Docker sandbox
# Runs the release binary against real scenarios
set -euo pipefail

VETO="./target/release/veto"
PASS=0
FAIL=0
TOTAL=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() {
    ((PASS++))
    ((TOTAL++))
    echo -e "  ${GREEN}✓${NC} $1"
}

fail() {
    ((FAIL++))
    ((TOTAL++))
    echo -e "  ${RED}✗${NC} $1"
    echo "    Expected: $2"
    echo "    Got:      $3"
}

section() {
    echo ""
    echo -e "${YELLOW}── $1 ──${NC}"
}

# Ensure binary exists
if [ ! -f "$VETO" ]; then
    echo "Release binary not found. Run 'cargo build --release' first."
    exit 1
fi

# ─────────────────────────────────────────
section "1. Basic CLI"
# ─────────────────────────────────────────

# --help exits 0
if $VETO --help > /dev/null 2>&1; then
    pass "--help exits 0"
else
    fail "--help exits 0" "exit 0" "non-zero"
fi

# --version shows version
VERSION_OUT=$($VETO --version 2>&1)
if echo "$VERSION_OUT" | grep -q "veto"; then
    pass "--version contains 'veto'"
else
    fail "--version contains 'veto'" "contains 'veto'" "$VERSION_OUT"
fi

# ─────────────────────────────────────────
section "2. Check command (risk classification)"
# ─────────────────────────────────────────

# Safe command → exit 0
if $VETO check "echo hello" > /dev/null 2>&1; then
    pass "check 'echo hello' → safe (exit 0)"
else
    fail "check 'echo hello' → safe" "exit 0" "non-zero"
fi

# ls → safe
if $VETO check "ls -la" > /dev/null 2>&1; then
    pass "check 'ls -la' → safe (exit 0)"
else
    fail "check 'ls -la' → safe" "exit 0" "non-zero"
fi

# rm -rf / → critical (exit non-zero)
if $VETO check "rm -rf /" > /dev/null 2>&1; then
    fail "check 'rm -rf /' → blocked" "non-zero" "exit 0"
else
    pass "check 'rm -rf /' → blocked (exit non-zero)"
fi

# ─────────────────────────────────────────
section "3. Check --explain (decision trace)"
# ─────────────────────────────────────────

EXPLAIN_OUT=$($VETO check --explain "rm -rf /" 2>&1 || true)
if echo "$EXPLAIN_OUT" | grep -qi "critical\|destructive\|blocked"; then
    pass "check --explain shows risk classification"
else
    fail "check --explain shows classification" "critical/destructive/blocked" "$EXPLAIN_OUT"
fi

# ─────────────────────────────────────────
section "4. Gate command (hook mode)"
# ─────────────────────────────────────────

# Safe command through gate
if $VETO gate "echo hello" > /dev/null 2>&1; then
    pass "gate 'echo hello' → allowed (exit 0)"
else
    fail "gate 'echo hello' → allowed" "exit 0" "non-zero"
fi

# Dangerous command through gate (requires auth — skip in interactive env)
# In Docker/headless: gate blocks without auth provider → exit non-zero
# On macOS with Touch ID: gate prompts and allows → exit 0
# This test validates gate runs without crashing; auth behavior is env-dependent
GATE_OUT=$($VETO gate "rm -rf /" 2>&1 || true)
if echo "$GATE_OUT" | grep -qi "rm -rf\|critical\|touch id\|denied\|blocked\|approved"; then
    pass "gate 'rm -rf /' → processes dangerous command"
else
    fail "gate 'rm -rf /' → processes dangerous command" "recognition output" "$GATE_OUT"
fi

# ─────────────────────────────────────────
section "5. Shell wrapper bypass detection"
# ─────────────────────────────────────────

# bash -c "rm -rf /" should still be caught
if $VETO check "bash -c 'rm -rf /'" > /dev/null 2>&1; then
    fail "check 'bash -c rm -rf /' → blocked" "non-zero" "exit 0"
else
    pass "check 'bash -c rm -rf /' → blocked"
fi

# ─────────────────────────────────────────
section "6. Allowlist management"
# ─────────────────────────────────────────

# Init config first (to temp dir)
TMPDIR=$(mktemp -d)
export VETO_CONFIG_DIR="$TMPDIR"

if $VETO init --force > /dev/null 2>&1; then
    pass "init --force creates config"
else
    fail "init --force" "exit 0" "non-zero"
fi

# List allowlist (should work even if empty)
if $VETO allow list > /dev/null 2>&1; then
    pass "allow list works"
else
    fail "allow list" "exit 0" "non-zero"
fi

# Cleanup
rm -rf "$TMPDIR"
unset VETO_CONFIG_DIR

# ─────────────────────────────────────────
section "7. Doctor"
# ─────────────────────────────────────────

# Doctor should run (may warn about missing config)
DOCTOR_OUT=$($VETO doctor 2>&1 || true)
if echo "$DOCTOR_OUT" | grep -qi "veto\|config\|check\|version"; then
    pass "doctor produces diagnostic output"
else
    fail "doctor output" "diagnostic info" "$DOCTOR_OUT"
fi

# ─────────────────────────────────────────
section "8. Quiet mode"
# ─────────────────────────────────────────

# --quiet: verify exit code is correct (output suppression is best-effort)
if $VETO --quiet check "echo hello" > /dev/null 2>&1; then
    pass "--quiet check safe command → exit 0"
else
    fail "--quiet check safe command" "exit 0" "non-zero"
fi

# ─────────────────────────────────────────
# Summary
# ─────────────────────────────────────────
echo ""
echo "─────────────────────────────────────"
echo -e "Results: ${GREEN}${PASS} passed${NC}, ${RED}${FAIL} failed${NC}, ${TOTAL} total"
echo "─────────────────────────────────────"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
