#!/usr/bin/env bash
# Per-start initialization (runs every container start)
set -euo pipefail

echo "=== veto devcontainer: start ==="

# Ensure PATH includes cargo bin
if ! grep -q 'cargo/bin' /home/developer/.bashrc 2>/dev/null; then
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> /home/developer/.bashrc
fi

# Ensure RUST_BACKTRACE is set for development
if ! grep -q 'RUST_BACKTRACE' /home/developer/.bashrc 2>/dev/null; then
    echo 'export RUST_BACKTRACE=1' >> /home/developer/.bashrc
fi

# Rebuild if binary is stale (source changed since last build)
if [ -f /usr/local/bin/veto ]; then
    BINARY_TIME=$(stat -c %Y /usr/local/bin/veto 2>/dev/null || echo 0)
    NEWEST_SRC=$(find /workspace/src -name '*.rs' -printf '%T@\n' 2>/dev/null | sort -rn | head -1 | cut -d. -f1 || echo 0)
    if [ "$NEWEST_SRC" -gt "$BINARY_TIME" ] 2>/dev/null; then
        echo "Source changed, rebuilding..."
        cd /workspace && cargo build --release 2>&1 && sudo cp target/release/veto /usr/local/bin/veto
    fi
fi

echo "=== Ready! ==="
echo "  cargo watch -x test    # continuous testing"
echo "  cargo watch -x run     # hot-reload"
echo "  cargo clippy            # lint"
echo "  cargo fmt               # format"
