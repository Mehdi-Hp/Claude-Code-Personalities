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
ICON_CHECK=$(printf '\xef\x91\x84')
ICON_SUCCESS=$(printf '\xef\x93\xb5')
ICON_DOWNLOAD=$(printf '\xef\x80\x99')
ICON_GEAR=$(printf '\xef\x80\x93')
ICON_ERROR=$(printf '\xef\x81\x97')
ICON_WARNING=$(printf '\xef\x81\xb1')

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
ORANGE='\033[38;2;255;165;0m'  # RGB(255, 165, 0)
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
echo -e "${BOLD}${YELLOW}   ╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${YELLOW}   ║                                                           ║${NC}"
echo -e "${BOLD}${YELLOW}   ║           ${BOLD}${NC}૮ ․ ․ ྀིა  Claude Code Personalities              ║"
echo -e "${BOLD}${YELLOW}   ║                                                           ║${NC}"
echo -e "${BOLD}${YELLOW}   ╚═══════════════════════════════════════════════════════════╝${NC}"
echo
echo -e "   ${ICON_ROCKET} ${ITALIC}Lightning-fast Claude Code statusline with intelligent activity tracking${NC}"
echo
echo

# Detect platform
PLATFORM=$(detect_platform)

if [[ "$PLATFORM" == "unsupported" ]]; then
    print_error "Unsupported platform: $(uname -s) $(uname -m)"
    echo
    echo "Supported platforms:"
    echo "  - macOS (Intel/Apple Silicon)"
    echo "  - Linux (x86_64/ARM64)"
    exit 1
fi

print_success "Platform detected: $PLATFORM"

# Check dependencies
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
RELEASE_INFO=$(curl -sL "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
LATEST_VERSION=$(echo "$RELEASE_INFO" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' | sed 's/^v//')

if [[ -z "$LATEST_VERSION" ]] || [[ "$LATEST_VERSION" == "null" ]]; then
    print_error "Failed to get latest version from GitHub"
    exit 1
fi

# Show section header with version
echo
# Dim gray divider
divider="${DIM}$(printf '%.0s─' $(seq 1 60))${NC}"
echo -e "  $divider"
echo -e "  ${BOLD}${ORANGE}Installing the Binary v$LATEST_VERSION${NC}"
echo -e "  $divider"
echo

# Construct binary name and download URL
MAPPED_PLATFORM=$(map_platform_to_binary_name "$PLATFORM")
BINARY_NAME="claude-code-personalities-${MAPPED_PLATFORM}"
DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/v$LATEST_VERSION/$BINARY_NAME"

if ! curl -sfL "$DOWNLOAD_URL" -o "$TEMP_DIR/claude-code-personalities"; then
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
if [[ ! -f "$TEMP_DIR/claude-code-personalities" ]]; then
    print_error "Downloaded binary file is missing"
    echo "This indicates a download failure. Please try again or check network connectivity."
    exit 1
elif [[ ! -s "$TEMP_DIR/claude-code-personalities" ]]; then
    print_error "Downloaded binary is empty (0 bytes)"
    echo "This indicates:"
    echo "  - Server returned empty response (possibly 404 or network error)"
    echo "  - GitHub release is corrupted or still building"
    echo "  - CDN caching issue"
    echo
    echo "Expected URL: $DOWNLOAD_URL"
    echo "Try again in a few minutes or download manually from GitHub releases."
    exit 1
fi

# Create local bin directory if needed
if [[ ! -d "$LOCAL_BIN" ]]; then
    mkdir -p "$LOCAL_BIN"
fi

# Install the binary
cp "$TEMP_DIR/claude-code-personalities" "$BIN_PATH"
chmod +x "$BIN_PATH"

# Verify the copy was successful
if [[ ! -f "$BIN_PATH" ]] || [[ ! -s "$BIN_PATH" ]]; then
    print_error "Binary copy verification failed"
    echo "The binary was not copied correctly to $BIN_PATH"
    exit 1
fi

# Verify sizes match
SOURCE_SIZE=$(stat -f%z "$TEMP_DIR/claude-code-personalities" 2>/dev/null || stat -c%s "$TEMP_DIR/claude-code-personalities" 2>/dev/null || echo "0")
TARGET_SIZE=$(stat -f%z "$BIN_PATH" 2>/dev/null || stat -c%s "$BIN_PATH" 2>/dev/null || echo "0")

if [[ "$SOURCE_SIZE" != "$TARGET_SIZE" ]] || [[ "$SOURCE_SIZE" == "0" ]]; then
    print_error "Binary copy verification failed"
    echo "Source size: $SOURCE_SIZE bytes"
    echo "Target size: $TARGET_SIZE bytes"
    echo "The copy operation may have been incomplete or corrupted."
    exit 1
fi

print_success "Binary installed to $BIN_PATH"

# Verify installation
if ! "$BIN_PATH" --version &> /dev/null; then
    print_error "Binary installed but execution failed"
    echo "This indicates:"
    echo "  - Platform compatibility issue (wrong architecture)"
    echo "  - Missing system dependencies"
    echo "  - Corrupted binary during copy operation"
    echo
    echo "Binary location: $BIN_PATH"
    echo "Binary size: $(ls -lh "$BIN_PATH" | awk '{print $5}') bytes"
    echo
    echo "Please try downloading a fresh copy from GitHub releases."
    exit 1
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
    echo -e "    ${YELLOW}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    echo
    echo "    Then reload your shell:"
    echo -e "    ${YELLOW}source ~/.zshrc${NC}  # or source ~/.bashrc"
    echo
fi

# Success message with next steps
echo
echo -e "  $divider"
echo -e "  ${ICON_SUCCESS} ${BOLD}${NC} Claude Code Personalities Installed${NC}"
echo -e "  $divider"
echo
echo

# Auto-initialize Claude Code if possible
echo

if command -v claude &> /dev/null; then
    echo -e "  • Claude Code detected - setting up personalities..."
    
    if "$BIN_PATH" init --non-interactive 2>/dev/null; then
        print_success "Configuration completed"
        echo
        echo -e "  ${BOLD}${ORANGE}Ready to Use!${NC}"
    else
        print_warning "Auto-configuration failed"
        echo
        echo -e "  ${BOLD}${ORANGE}Manual Setup Required${NC}"
        echo -e "    Run: ${NC}claude-code-personalities init${NC}"
    fi
else
    print_info "Claude Code not detected in PATH"
    echo
    echo -e "  ${BOLD}${ORANGE}Next Steps${NC}"
    echo -e "    1. Install Claude Code if you haven't already"
    echo -e "    2. Run: ${NC}claude-code-personalities init${NC}"
fi

echo
echo

echo -e "  ${BOLD}${ORANGE}Available Commands:${NC}"
echo -e "    ${NC}claude-code-personalities init${NC}          ${DIM}- Configure Claude Code${NC}"
echo -e "    ${NC}claude-code-personalities status${NC}        ${DIM}- Check installation status${NC}"  
echo -e "    ${NC}claude-code-personalities --help${NC}        ${DIM}- Show all commands${NC}"
echo