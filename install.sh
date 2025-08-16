#!/bin/bash

# Claude Code Personalities - Stage 1: CLI Tool Installer
# Stage 1: curl -fsSL [url] | bash (installs CLI tool only)
# Stage 2: claude-code-personalities install (configures Claude Code)

set -e

GITHUB_REPO="Mehdi-Hp/claude-code-personalities"
LOCAL_BIN="$HOME/.local/bin"
BIN_PATH="$LOCAL_BIN/claude-code-personalities"
TEMP_DIR=$(mktemp -d)

# Nerd Font icons
ICON_FOLDER=$(printf '\xef\x81\xbb')
ICON_CODE=$(printf '\xef\x84\xa1')
ICON_GEAR=$(printf '\xef\x80\x93')
ICON_ROCKET=$(printf '\xef\x84\xb5')
ICON_CHECK=$(printf '\xef\x80\x8c')
ICON_DOWNLOAD=$(printf '\xef\x80\x99')
ICON_TERMINAL=$(printf '\xef\x84\xa0')

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
ITALIC='\033[3m'
NC='\033[0m'

# Helper functions
print_success() {
    echo -e "  ${GREEN}${ICON_CHECK}${NC} $1"
}

print_error() {
    echo -e "  ${RED}✗${NC} $1"
}

print_info() {
    echo -e "  ${CYAN}${ICON_DOWNLOAD}${NC} $1"
}

print_warning() {
    echo -e "  ${YELLOW}⚠${NC} $1"
}

# Clean up on exit
trap "rm -rf $TEMP_DIR" EXIT

# Header
clear
echo
echo -e "${BOLD}${CYAN}   ╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}   ║                                                           ║${NC}"
echo -e "${BOLD}${CYAN}   ║           ${NC}${BOLD}( ꩜ ᯅ ꩜;)⁭⁭ ${MAGENTA}Claude Code Personalities${NC}             ${BOLD}${CYAN}║${NC}"
echo -e "${BOLD}${CYAN}   ║                                                           ║${NC}"
echo -e "${BOLD}${CYAN}   ╚═══════════════════════════════════════════════════════════╝${NC}"
echo
echo -e "   ${ICON_ROCKET} ${ITALIC}Give Claude Code dynamic personalities that change based${NC}"
echo -e "   ${ITALIC}on what it's doing - from debugging to git management!${NC}"
echo
echo


# Dim gray divider
divider="${DIM}$(printf '%.0s─' $(seq 1 60))${NC}"
echo -e "  $divider"
echo -e "  ${BOLD}${BLUE}Installing${NC} ${BOLD}CLI Tool${NC}"
echo -e "  $divider"
echo

# Check dependencies
print_info "Checking dependencies..."
if ! command -v curl &> /dev/null; then
    print_error "curl is required but not installed"
    echo "Install with: brew install curl"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    print_error "jq is required but not installed"
    echo "Install with: brew install jq"
    exit 1
fi

print_success "Dependencies verified"

# Download latest release
echo
print_info "Downloading latest version..."

RELEASE_INFO=$(curl -sL "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
LATEST_VERSION=$(echo "$RELEASE_INFO" | jq -r '.tag_name' | sed 's/^v//')
TARBALL_URL=$(echo "$RELEASE_INFO" | jq -r '.tarball_url')

if [[ -z "$LATEST_VERSION" ]] || [[ "$LATEST_VERSION" == "null" ]]; then
    print_error "Failed to get latest version from GitHub"
    exit 1
fi

echo -e "    ${CYAN}Latest version: ${BOLD}v$LATEST_VERSION${NC}"
echo

# Download and extract
print_info "Extracting release files..."
curl -sL "$TARBALL_URL" | tar xz -C "$TEMP_DIR" --strip-components=1

# Create local bin directory if needed
if [[ ! -d "$LOCAL_BIN" ]]; then
    print_info "Creating $LOCAL_BIN directory..."
    mkdir -p "$LOCAL_BIN"
fi

# Install the CLI tool
print_info "Installing CLI tool..."

if [[ -f "$TEMP_DIR/bin/claude-code-personalities" ]]; then
    cp "$TEMP_DIR/bin/claude-code-personalities" "$BIN_PATH"
else
    print_error "CLI tool not found in release"
    exit 1
fi

chmod +x "$BIN_PATH"
print_success "CLI tool installed to $BIN_PATH"

# Check PATH
echo
if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
    print_warning "$LOCAL_BIN is not in your PATH"
    echo
    echo "    Add this to your shell config (.bashrc, .zshrc, etc.):"
    echo -e "    ${CYAN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    echo
    echo "    Then reload your shell:"
    echo -e "    ${CYAN}source ~/.zshrc${NC}  # or source ~/.bashrc"
    echo
fi

# Success message with next steps
echo
echo -e "  $divider"
echo -e "  ${BOLD}${GREEN}Claude Code Personalities Installed${NC} ${ICON_CHECK}"
echo -e "  $divider"
echo
echo

echo -e "  ${BOLD}${GREEN}Next Steps${NC}"
echo
echo -e "    ${CYAN}claude-code-personalities install${NC}"
echo
echo

echo -e "  ${BOLD}Available Commands:${NC}"
echo -e "    ${CYAN}claude-code-personalities install${NC}       - Configure Claude Code"
echo -e "    ${CYAN}claude-code-personalities status${NC}        - Check installation status"  
echo -e "    ${CYAN}claude-code-personalities help${NC}          - Show all commands"
echo