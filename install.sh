#!/bin/bash

# Claude Code Personalities Installer
# One-line install: curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash

set -e

GITHUB_REPO="Mehdi-Hp/claude-code-personalities"
# Install to user's local bin directory (no sudo needed)
LOCAL_BIN="$HOME/.local/bin"
BIN_PATH="$LOCAL_BIN/claude-code-personalities"
TEMP_DIR=$(mktemp -d)

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

print_error() {
    echo -e "\033[0;31m✗\033[0m $1"
}

# Clean up on exit
trap "rm -rf $TEMP_DIR" EXIT

echo -e "${BOLD}${BLUE}Claude Code Personalities Installer${NC}"
echo ""

# Check dependencies
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

# Download latest release
print_info "Downloading latest version..."

RELEASE_INFO=$(curl -sL "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
LATEST_VERSION=$(echo "$RELEASE_INFO" | jq -r '.tag_name' | sed 's/^v//')
TARBALL_URL=$(echo "$RELEASE_INFO" | jq -r '.tarball_url')

if [[ -z "$LATEST_VERSION" ]] || [[ "$LATEST_VERSION" == "null" ]]; then
    print_error "Failed to get latest version from GitHub"
    exit 1
fi

print_info "Latest version: v$LATEST_VERSION"

# Download and extract
curl -sL "$TARBALL_URL" | tar xz -C "$TEMP_DIR" --strip-components=1

# Create local bin directory if it doesn't exist
if [[ ! -d "$LOCAL_BIN" ]]; then
    print_info "Creating $LOCAL_BIN directory..."
    mkdir -p "$LOCAL_BIN"
fi

# Install the command (no sudo needed!)
print_info "Installing claude-code-personalities command..."

if [[ -f "$TEMP_DIR/bin/claude-code-personalities" ]]; then
    cp "$TEMP_DIR/bin/claude-code-personalities" "$BIN_PATH"
else
    # Fallback to old location
    cp "$TEMP_DIR/claude-code-personalities" "$BIN_PATH"
fi

chmod +x "$BIN_PATH"

print_success "Command installed to $BIN_PATH"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
    print_warning "$LOCAL_BIN is not in your PATH"
    echo ""
    echo "Add this to your shell config file (.bashrc, .zshrc, etc.):"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then reload your shell or run:"
    echo "  source ~/.bashrc  # or ~/.zshrc"
    echo ""
fi

# Run the actual installation
echo ""
print_info "Running installation..."
echo ""

"$BIN_PATH" install

echo ""
echo -e "${BOLD}${GREEN}Installation complete!${NC}"
echo ""
echo "Available commands:"
echo "  claude-code-personalities status        - Check installation status"
echo "  claude-code-personalities check-update  - Check for updates"
echo "  claude-code-personalities update        - Update to latest version"
echo "  claude-code-personalities help          - Show all commands"
echo ""
echo -e "${CYAN}Restart Claude Code to see the personalities!${NC}"
