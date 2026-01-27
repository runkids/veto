.PHONY: help build test release install uninstall clean sandbox

help:
	@echo "veto - AI operation guardian"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "  build     Build debug binary"
	@echo "  test      Run tests"
	@echo "  release   Build release binary"
	@echo "  install   Install to /usr/local/bin"
	@echo "  uninstall Remove from /usr/local/bin"
	@echo "  clean     Clean build artifacts"
	@echo "  sandbox   Enter Docker sandbox for safe testing"

build:
	cargo build

test:
	cargo test

release:
	cargo build --release

install: release
	sudo cp target/release/veto /usr/local/bin/

uninstall:
	@./uninstall.sh

clean:
	cargo clean

sandbox: release
	docker-compose run --rm sandbox
