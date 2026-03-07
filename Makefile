.PHONY: help build test release install uninstall clean check fmt fmt-check lint sandbox devc devc-up devc-down devc-restart devc-reset devc-status test-docker e2e

help:
	@echo "veto - AI operation guardian"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "  Build & Run:"
	@echo "    build       Build debug binary"
	@echo "    release     Build release binary"
	@echo "    install     Install to /usr/local/bin"
	@echo "    uninstall   Remove from /usr/local/bin"
	@echo "    clean       Clean build artifacts"
	@echo ""
	@echo "  Quality:"
	@echo "    test        Run unit tests"
	@echo "    lint        Run clippy linter"
	@echo "    fmt         Format code"
	@echo "    fmt-check   Check formatting (CI)"
	@echo "    check       fmt-check + lint + test"
	@echo ""
	@echo "  Docker:"
	@echo "    test-docker   Run tests in Docker (isolated)"
	@echo "    sandbox       Enter Docker sandbox for safe testing"
	@echo "    e2e           Run e2e tests in sandbox"
	@echo ""
	@echo "  Devcontainer:"
	@echo "    devc          Start devcontainer + enter shell"
	@echo "    devc-up       Start devcontainer (no shell)"
	@echo "    devc-down     Stop devcontainer"
	@echo "    devc-restart  Restart devcontainer"
	@echo "    devc-reset    Full reset (remove volumes + rebuild)"
	@echo "    devc-status   Show devcontainer status"

# === Build & Run ===

build:
	cargo build

release:
	cargo build --release

install: release
	sudo cp target/release/veto /usr/local/bin/

uninstall:
	@./uninstall.sh

clean:
	cargo clean

# === Quality ===

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

check: fmt-check lint test

# === Docker ===

test-docker:
	./scripts/test_docker.sh

sandbox: release
	docker compose run --rm sandbox

e2e: release
	./scripts/e2e.sh

# === Devcontainer ===

devc:
	./scripts/devc.sh up && ./scripts/devc.sh shell

devc-up:
	./scripts/devc.sh up

devc-down:
	./scripts/devc.sh down

devc-restart:
	./scripts/devc.sh restart

devc-reset:
	./scripts/devc.sh reset

devc-status:
	./scripts/devc.sh status
