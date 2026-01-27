#!/bin/bash
# veto installer
# Usage: curl -sSL https://raw.githubusercontent.com/runkids/veto/main/install.sh | bash

set -e

REPO="runkids/veto"
INSTALL_DIR="${VETO_INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${CYAN}$1${NC}"; }
success() { echo -e "${GREEN}$1${NC}"; }
warn() { echo -e "${YELLOW}$1${NC}"; }
error() { echo -e "${RED}$1${NC}" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin) echo "apple-darwin" ;;
        Linux)  echo "unknown-linux-gnu" ;;
        *)      error "Unsupported OS: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get latest version
get_latest_version() {
    curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    info "Installing veto..."
    echo

    OS=$(detect_os)
    ARCH=$(detect_arch)
    VERSION=$(get_latest_version)

    if [ -z "$VERSION" ]; then
        error "Failed to get latest version. Check your internet connection."
    fi

    FILENAME="veto-${ARCH}-${OS}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

    info "Detected: ${ARCH}-${OS}"
    info "Version: ${VERSION}"
    info "Installing to: ${INSTALL_DIR}"
    echo

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    # Download
    info "Downloading ${FILENAME}..."
    if ! curl -sSL "$URL" -o "${TMP_DIR}/${FILENAME}"; then
        error "Failed to download. URL: ${URL}"
    fi

    # Extract
    info "Extracting..."
    tar -xzf "${TMP_DIR}/${FILENAME}" -C "$TMP_DIR"

    # Install
    info "Installing..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMP_DIR}/veto" "${INSTALL_DIR}/veto"
    else
        warn "Need sudo to install to ${INSTALL_DIR}"
        sudo mv "${TMP_DIR}/veto" "${INSTALL_DIR}/veto"
    fi
    chmod +x "${INSTALL_DIR}/veto"

    echo
    success "✓ veto ${VERSION} installed successfully!"
    echo
    info "Run 'veto init' to create config files"
    info "Run 'veto doctor' to verify installation"
}

main "$@"
