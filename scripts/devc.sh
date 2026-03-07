#!/usr/bin/env bash
# Devcontainer lifecycle management
set -euo pipefail

COMPOSE_FILE=".devcontainer/docker-compose.yml"
SERVICE="devcontainer"
CONTAINER="veto-devcontainer"

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  up        Start devcontainer"
    echo "  down      Stop devcontainer"
    echo "  shell     Enter running devcontainer"
    echo "  restart   Restart devcontainer"
    echo "  reset     Full reset (remove volumes + rebuild)"
    echo "  status    Show container status"
    echo "  logs      Tail container logs"
}

case "${1:-help}" in
    up)
        echo "Starting devcontainer..."
        docker compose -f "$COMPOSE_FILE" up -d --build
        echo "Running setup..."
        docker exec "$CONTAINER" bash -c '\
            cd /workspace && \
            touch src/main.rs && \
            cargo build --locked --release 2>&1 && \
            install -m 755 target/release/veto ~/.local/bin/veto && \
            echo "veto $(veto --version 2>&1)" && \
            echo "✓ Dev environment ready"'
        ;;
    down)
        echo "Stopping devcontainer..."
        docker compose -f "$COMPOSE_FILE" down
        ;;
    shell)
        if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER}$"; then
            echo "Container not running. Run '$0 up' first."
            exit 1
        fi
        docker exec -it "$CONTAINER" bash
        ;;
    restart)
        docker compose -f "$COMPOSE_FILE" restart
        ;;
    reset)
        echo "Full reset: removing containers and volumes..."
        docker compose -f "$COMPOSE_FILE" down -v
        echo "Rebuilding..."
        docker compose -f "$COMPOSE_FILE" up -d --build
        echo "Running setup..."
        docker exec "$CONTAINER" bash -c '\
            cd /workspace && \
            touch src/main.rs && \
            cargo build --locked --release 2>&1 && \
            install -m 755 target/release/veto ~/.local/bin/veto && \
            echo "veto $(veto --version 2>&1)" && \
            echo "✓ Dev environment ready"'
        ;;
    status)
        docker compose -f "$COMPOSE_FILE" ps
        ;;
    logs)
        docker compose -f "$COMPOSE_FILE" logs -f "$SERVICE"
        ;;
    help|*)
        usage
        ;;
esac
