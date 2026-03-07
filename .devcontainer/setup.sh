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

# Install to PATH for easy access
sudo cp target/release/veto /usr/local/bin/veto

# Run tests to verify environment
echo "Running tests..."
cargo test 2>&1

# Mark initialization complete
touch ~/.devcontainer-initialized

echo "=== Setup complete! ==="
echo "  veto is available at /usr/local/bin/veto"
echo "  Run 'cargo watch -x test' for continuous testing"
echo "  Run 'cargo watch -x run' for hot-reload development"
