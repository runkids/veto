.PHONY: help build test dev sandbox release clean doctor init check

# Default target
help:
	@echo "veto - AI operation guardian"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Development:"
	@echo "  dev        Enter interactive Docker development shell"
	@echo "  build      Build debug binary in Docker"
	@echo "  test       Run all tests in Docker"
	@echo "  test-unit  Run unit tests only"
	@echo "  release    Build release binary in Docker"
	@echo "  clean      Clean build artifacts and Docker volumes"
	@echo ""
	@echo "Testing:"
	@echo "  check      Run cargo check"
	@echo "  clippy     Run clippy linter"
	@echo "  fmt        Format code"
	@echo "  fmt-check  Check code formatting"
	@echo ""
	@echo "Utilities:"
	@echo "  sandbox    Enter safe sandbox environment"
	@echo "  doctor     Run veto doctor"
	@echo "  init       Run veto init"
	@echo ""
	@echo "Local (requires Rust installed):"
	@echo "  local-build   Build locally"
	@echo "  local-test    Test locally"
	@echo "  local-release Build release locally"

# =============================================================================
# Docker Development
# =============================================================================

dev:
	docker-compose run --rm dev

build:
	docker-compose run --rm test cargo build

test:
	docker-compose run --rm test cargo test

test-unit:
	docker-compose run --rm test cargo test --lib

release:
	docker-compose run --rm test cargo build --release

clean:
	docker-compose down -v
	rm -rf target/

# =============================================================================
# Code Quality
# =============================================================================

check:
	docker-compose run --rm test cargo check

clippy:
	docker-compose run --rm test cargo clippy -- -D warnings

fmt:
	docker-compose run --rm test cargo fmt

fmt-check:
	docker-compose run --rm test cargo fmt -- --check

# =============================================================================
# Utilities
# =============================================================================

sandbox:
	docker-compose run --rm sandbox

doctor:
	docker-compose run --rm test cargo run -- doctor

init:
	docker-compose run --rm test cargo run -- init

# =============================================================================
# Local Development (without Docker)
# =============================================================================

local-build:
	cargo build

local-test:
	cargo test

local-release:
	cargo build --release

# =============================================================================
# CI Pipeline
# =============================================================================

ci: fmt-check check clippy test
	@echo "CI checks passed!"
