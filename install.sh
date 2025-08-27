#!/bin/bash

# Claude Code Personalities - Installation Script (Rust Edition)
# Downloads and installs the Rust binary for the current platform

set -e

GITHUB_REPO="Mehdi-Hp/claude-code-personalities"
LOCAL_BIN="$HOME/.local/bin"
BIN_PATH="$LOCAL_BIN/claude-code-personalities"
TEMP_DIR=$(mktemp -d)

# Icons
ICON_ROCKET=$(printf '\xef\x84\xb5')
ICON_CHECK=$(printf '\xef\x80\x8c')
ICON_DOWNLOAD=$(printf '\xef\x80\x99')
ICON_GEAR=$(printf '\xef\x80\x93')
ICON_ERROR=$(printf '\xef\x81\x97')
ICON_WARNING=$(printf '\xef\x81\xb1')

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
    echo -e "  ${RED}${ICON_ERROR}${NC} $1"
}

print_info() {
    echo -e "  ${CYAN}${ICON_DOWNLOAD}${NC} $1"
}

print_warning() {
    echo -e "  ${YELLOW}${ICON_WARNING}${NC} $1"
}

# Clean up on exit
trap "rm -rf $TEMP_DIR" EXIT

# Detect platform
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)
    
    case "$os" in
        Darwin)
            case "$arch" in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64) echo "aarch64-apple-darwin" ;;
                *) echo "unsupported" ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64) echo "x86_64-linux" ;;
                aarch64) echo "aarch64-linux" ;;
                arm64) echo "aarch64-linux" ;;
                *) echo "unsupported" ;;
            esac
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

# Map detected platform to release binary name format
map_platform_to_binary_name() {
    case "$1" in
        x86_64-apple-darwin) echo "macos-x86_64" ;;
        aarch64-apple-darwin) echo "macos-aarch64" ;;
        x86_64-linux) echo "linux-x86_64" ;;
        aarch64-linux) echo "linux-aarch64" ;;
        *) echo "$1" ;;
    esac
}

# Header
clear
echo
echo -e "${BOLD}${CYAN}   ╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}   ║                                                           ║${NC}"
echo -e "${BOLD}${CYAN}   ║           ${NC}${BOLD}૮ ․ ․ ྀིა ${MAGENTA}Claude Code Personalities${NC}             ${BOLD}${CYAN}║${NC}"
echo -e "${BOLD}${CYAN}   ║                                                           ║${NC}"
echo -e "${BOLD}${CYAN}   ╚═══════════════════════════════════════════════════════════╝${NC}"
echo
echo -e "   ${ICON_ROCKET} ${ITALIC}Lightning-fast Rust binary (~1ms statusline generation)${NC}"
echo -e "   ${ITALIC}with zero dependencies and intelligent activity tracking${NC}"
echo
echo

# Dim gray divider
divider="${DIM}$(printf '%.0s─' $(seq 1 60))${NC}"
echo -e "  $divider"
echo -e "  ${BOLD}${BLUE}Installing${NC} ${BOLD}Rust Binary${NC}"
echo -e "  $divider"
echo

# Detect platform
print_info "Detecting platform..."
PLATFORM=$(detect_platform)

if [[ "$PLATFORM" == "unsupported" ]]; then
    print_error "Unsupported platform: $(uname -s) $(uname -m)"
    echo
    echo "Supported platforms:"
    echo "  - macOS (Intel/Apple Silicon)"
    echo "  - Linux (x86_64/ARM64)"
    exit 1
fi

print_success "Detected platform: $PLATFORM"

# Check dependencies
print_info "Checking dependencies..."
if ! command -v curl &> /dev/null; then
    print_error "curl is required but not installed"
    echo "Install with: brew install curl"
    exit 1
fi

if ! command -v grep &> /dev/null || ! command -v sed &> /dev/null; then
    print_error "grep and sed are required but not found"
    exit 1
fi

print_success "Dependencies verified"

# Download latest release
echo
print_info "Downloading latest version..."

RELEASE_INFO=$(curl -sL "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
LATEST_VERSION=$(echo "$RELEASE_INFO" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' | sed 's/^v//')

if [[ -z "$LATEST_VERSION" ]] || [[ "$LATEST_VERSION" == "null" ]]; then
    print_error "Failed to get latest version from GitHub"
    exit 1
fi

echo -e "    ${CYAN}Latest version: ${BOLD}v$LATEST_VERSION${NC}"

# Construct binary name and download URL
MAPPED_PLATFORM=$(map_platform_to_binary_name "$PLATFORM")
BINARY_NAME="claude-code-personalities-${MAPPED_PLATFORM}"
DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/v$LATEST_VERSION/$BINARY_NAME"

print_info "Downloading binary..."
if ! curl -sL "$DOWNLOAD_URL" -o "$TEMP_DIR/claude-code-personalities"; then
    print_error "Failed to download binary from GitHub releases"
    echo
    echo "Expected URL: $DOWNLOAD_URL"
    echo "This might indicate:"
    echo "  - The release is still building"
    echo "  - Your platform is not yet supported"
    echo "  - Network connectivity issues"
    exit 1
fi

print_success "Binary downloaded"

# Verify downloaded binary
if [[ ! -f "$TEMP_DIR/claude-code-personalities" ]] || [[ ! -s "$TEMP_DIR/claude-code-personalities" ]]; then
    print_error "Downloaded binary is missing or empty"
    exit 1
fi

# Create local bin directory if needed
if [[ ! -d "$LOCAL_BIN" ]]; then
    print_info "Creating $LOCAL_BIN directory..."
    mkdir -p "$LOCAL_BIN"
fi

# Install the binary
print_info "Installing binary..."
cp "$TEMP_DIR/claude-code-personalities" "$BIN_PATH"
chmod +x "$BIN_PATH"

print_success "Binary installed to $BIN_PATH"

# Verify installation
print_info "Verifying installation..."
if ! "$BIN_PATH" --version &> /dev/null; then
    print_warning "Binary installed but version check failed"
    echo "This might indicate a platform compatibility issue"
else
    VERSION_OUTPUT=$("$BIN_PATH" --version 2>/dev/null || echo "unknown")
    print_success "Installation verified: $VERSION_OUTPUT"
fi

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
echo -e "  ${ICON_CHECK} ${BOLD}${GREEN}Claude Code Personalities Installed${NC}"
echo -e "  $divider"
echo
echo

echo -e "  ${BOLD}${GREEN}Next Steps${NC}"

echo -e "    ${CYAN}claude-code-personalities install${NC}"
echo
echo

echo -e "  ${BOLD}Available Commands:${NC}"
echo -e "    ${CYAN}claude-code-personalities install${NC}       - Configure Claude Code"
echo -e "    ${CYAN}claude-code-personalities status${NC}        - Check installation status"  
echo -e "    ${CYAN}claude-code-personalities --help${NC}        - Show all commands"
echo