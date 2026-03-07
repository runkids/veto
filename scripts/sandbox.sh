#!/usr/bin/env bash
# Sandbox management for safe command testing
set -euo pipefail

COMPOSE_FILE="docker-compose.yml"
SERVICE="sandbox"
CONTAINER="veto-sandbox"

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  up      Start sandbox (build release first)"
    echo "  down    Stop sandbox"
    echo "  shell   Enter running sandbox"
    echo "  reset   Stop + remove sandbox"
}

case "${1:-help}" in
    up)
        echo "Building release binary..."
        cargo build --release
        echo "Starting sandbox..."
        docker compose -f "$COMPOSE_FILE" run --rm "$SERVICE"
        ;;
    down)
        docker compose -f "$COMPOSE_FILE" stop "$SERVICE"
        ;;
    shell)
        docker exec -it "$CONTAINER" bash
        ;;
    reset)
        docker compose -f "$COMPOSE_FILE" rm -sf "$SERVICE"
        ;;
    help|*)
        usage
        ;;
esac
