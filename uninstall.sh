#!/bin/bash
# veto uninstaller
# Usage: curl -sSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${CYAN}$1${NC}"; }
success() { echo -e "${GREEN}$1${NC}"; }
warn() { echo -e "${YELLOW}$1${NC}"; }

INSTALL_DIR="${VETO_INSTALL_DIR:-/usr/local/bin}"
VETO_DIR="${VETO_HOME:-${HOME}/.veto}"

main() {
    info "Uninstalling veto..."
    echo

    # Remove binary
    if [ -f "${INSTALL_DIR}/veto" ]; then
        info "Removing ${INSTALL_DIR}/veto..."
        if [ -w "$INSTALL_DIR" ]; then
            rm -f "${INSTALL_DIR}/veto"
        else
            sudo rm -f "${INSTALL_DIR}/veto"
        fi
        success "✓ Binary removed"
    else
        warn "Binary not found at ${INSTALL_DIR}/veto"
    fi

    # Remove veto directory (config + secrets)
    if [ -d "$VETO_DIR" ]; then
        info "Removing veto directory ${VETO_DIR}..."
        rm -rf "$VETO_DIR"
        success "✓ Veto directory removed"
    fi

    # Also remove legacy config directory if exists
    if [ -d "${HOME}/.config/veto" ]; then
        info "Removing legacy config directory ${HOME}/.config/veto..."
        rm -rf "${HOME}/.config/veto"
        success "✓ Legacy config removed"
    fi

    echo
    success "✓ veto uninstalled successfully!"
    echo
    warn "Note: Secrets stored in system Keychain are not removed."
    warn "Use 'security delete-generic-password -s veto' to remove them manually."
}

main "$@"
