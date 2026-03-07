#!/usr/bin/env bash
# Post-create initialization (runs once after container build)
set -euo pipefail

echo "=== veto devcontainer: post-create setup ==="

# Build debug binary
echo "Building veto (debug)..."
cargo build 2>&1

# Build release binary for sandbox testing
echo "Building veto (release)..."
cargo build --release 2>&1

# Install to user PATH (ENV in Dockerfile ensures ~/.local/bin is in PATH for all shells)
mkdir -p ~/.local/bin
install -m 755 target/release/veto ~/.local/bin/veto

# Run tests to verify environment
echo "Running tests..."
cargo test 2>&1

# Add help alias to bashrc
cat >> ~/.bashrc << 'BASHRC'

# veto dev shortcuts
help() { make -C /workspace help; }
alias t='cargo test'
alias tw='cargo watch -x test'
alias r='cargo run --'
alias rr='cargo run --release --'
alias ck='make check'
alias e2e='make e2e'
BASHRC

# Mark initialization complete
touch ~/.devcontainer-initialized

echo "=== Setup complete! ==="
echo "  veto is available at ~/.local/bin/veto"
echo "  Type 'help' for available commands"
