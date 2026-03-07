#!/usr/bin/env bash
# Run cargo test inside Docker sandbox (isolated, reproducible)
set -euo pipefail

COMPOSE_FILE="docker-compose.yml"

echo "=== veto: Docker test run ==="

# Build and run tests
docker compose -f "$COMPOSE_FILE" run --rm --build test

echo "=== All tests passed ==="
