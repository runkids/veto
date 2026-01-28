#!/bin/bash
# veto uninstaller
# Usage: curl -sSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash
# Options:
#   --purge    Also remove keychain secrets (PIN, TOTP, Telegram credentials)

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${CYAN}$1${NC}"; }
success() { echo -e "${GREEN}$1${NC}"; }
warn() { echo -e "${YELLOW}$1${NC}"; }

PURGE=false
for arg in "$@"; do
    case $arg in
        --purge) PURGE=true ;;
    esac
done

VETO_DIR="${VETO_HOME:-${HOME}/.veto}"
CLAUDE_SETTINGS="${HOME}/.claude/settings.json"
OPENCODE_PLUGIN="${HOME}/.opencode/plugins/veto-gate.js"
CURSOR_HOOKS="${HOME}/.cursor/hooks.json"

# Keychain keys used by veto
KEYCHAIN_KEYS=(
    "veto.pin.hash"
    "veto.pin.salt"
    "veto.totp.secret"
    "veto.telegram.token"
)

# Check if keychain has veto secrets, return count
count_keychain_secrets() {
    local count=0

    if [[ "$OSTYPE" == "darwin"* ]]; then
        for key in "${KEYCHAIN_KEYS[@]}"; do
            security find-generic-password -s "$key" &>/dev/null && ((count++)) || true
        done
    elif command -v secret-tool &>/dev/null; then
        for key in "${KEYCHAIN_KEYS[@]}"; do
            secret-tool lookup service "$key" &>/dev/null && ((count++)) || true
        done
    fi

    echo $count
}

# Remove secrets from system keychain
remove_keychain_secrets() {
    local removed=0

    if [[ "$OSTYPE" == "darwin"* ]]; then
        for key in "${KEYCHAIN_KEYS[@]}"; do
            if security find-generic-password -s "$key" &>/dev/null; then
                security delete-generic-password -s "$key" &>/dev/null && ((removed++)) || true
            fi
        done
    elif command -v secret-tool &>/dev/null; then
        for key in "${KEYCHAIN_KEYS[@]}"; do
            secret-tool clear service "$key" &>/dev/null && ((removed++)) || true
        done
    else
        warn "Cannot detect keychain backend"
        return
    fi

    if [ $removed -gt 0 ]; then
        success "✓ Keychain secrets removed ($removed items)"
    fi
}

# Remove veto hooks from Claude Code settings.json
remove_claude_hooks() {
    if [ ! -f "$CLAUDE_SETTINGS" ]; then
        return
    fi

    # Check if jq is available
    if ! command -v jq &> /dev/null; then
        warn "jq not found, cannot auto-remove Claude Code hooks"
        warn "Please manually remove veto hooks from ${CLAUDE_SETTINGS}"
        return
    fi

    # Check if settings contains veto hooks
    if ! grep -q "veto gate" "$CLAUDE_SETTINGS" 2>/dev/null; then
        return
    fi

    info "Removing veto hooks from Claude Code..."

    # Remove hooks containing "veto gate" from PreToolUse array
    local temp_file="${CLAUDE_SETTINGS}.tmp"
    jq '
        if .hooks.PreToolUse then
            .hooks.PreToolUse |= map(
                select(
                    (.hooks // []) | all(
                        (.command // "") | contains("veto gate") | not
                    )
                )
            )
            | if .hooks.PreToolUse == [] then del(.hooks.PreToolUse) else . end
            | if .hooks == {} then del(.hooks) else . end
        else
            .
        end
    ' "$CLAUDE_SETTINGS" > "$temp_file" && mv "$temp_file" "$CLAUDE_SETTINGS"

    success "✓ Claude Code hooks removed"
}

# Remove veto plugin from OpenCode
remove_opencode_plugin() {
    if [ -f "$OPENCODE_PLUGIN" ]; then
        info "Removing OpenCode plugin..."
        rm -f "$OPENCODE_PLUGIN"
        success "✓ OpenCode plugin removed"
    fi
}

# Remove veto hooks from Cursor CLI hooks.json
remove_cursor_hooks() {
    if [ ! -f "$CURSOR_HOOKS" ]; then
        return
    fi

    if ! command -v jq &> /dev/null; then
        warn "jq not found, cannot auto-remove Cursor CLI hooks"
        warn "Please manually remove veto hooks from ${CURSOR_HOOKS}"
        return
    fi

    if ! grep -q "veto gate" "$CURSOR_HOOKS" 2>/dev/null; then
        return
    fi

    info "Removing veto hooks from Cursor CLI..."

    local temp_file="${CURSOR_HOOKS}.tmp"
    jq '
        if .hooks.beforeShellExecution then
            .hooks.beforeShellExecution |= map(
                select(
                    (.command // "") | contains("veto gate") | not
                )
            )
            | if .hooks.beforeShellExecution == [] then del(.hooks.beforeShellExecution) else . end
            | if .hooks == {} then del(.hooks) else . end
        else
            .
        end
    ' "$CURSOR_HOOKS" > "$temp_file" && mv "$temp_file" "$CURSOR_HOOKS"

    success "✓ Cursor CLI hooks removed"
}

main() {
    info "Uninstalling veto..."
    echo

    # Find and remove all veto binaries in PATH
    local binary_removed=0
    local veto_path

    while veto_path=$(which veto 2>/dev/null) && [ -n "$veto_path" ]; do
        info "Removing ${veto_path}..."
        if [ -w "$(dirname "$veto_path")" ]; then
            rm -f "$veto_path"
        else
            sudo rm -f "$veto_path"
        fi
        success "✓ Binary removed"
        ((binary_removed++))
    done

    if [ $binary_removed -eq 0 ]; then
        warn "No veto binary found in PATH"
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

    # Remove Claude Code hooks
    remove_claude_hooks

    # Remove OpenCode plugin
    remove_opencode_plugin

    # Remove Cursor CLI hooks
    remove_cursor_hooks

    # Handle keychain secrets
    local secret_count
    secret_count=$(count_keychain_secrets)

    if [ "$secret_count" -gt 0 ]; then
        if [ "$PURGE" = true ]; then
            info "Removing keychain secrets..."
            remove_keychain_secrets
        else
            echo
            warn "Keychain secrets found ($secret_count items)"
            warn "Run with --purge to remove them:"
            warn "  curl -sSL https://raw.githubusercontent.com/runkids/veto/main/uninstall.sh | bash -s -- --purge"
        fi
    fi

    echo
    success "✓ veto uninstalled successfully!"
}

main "$@"
