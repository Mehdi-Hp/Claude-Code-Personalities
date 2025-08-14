#!/bin/bash

# Setup script for creating and initializing the homebrew tap repository
# This only needs to be run once when setting up the tap repo for the first time

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}${CYAN}$(printf '\xef\x84\xb5') Homebrew Tap Repository Setup${NC}\n"
echo -e "This will help you create and initialize the tap repository."
echo -e "You should have already created an empty repository on GitHub named:"
echo -e "${BOLD}homebrew-claude-code-personalities${NC}\n"

echo -ne "${YELLOW}Have you created the repository on GitHub? (y/N): ${NC}"
read -r response
if [[ "$response" != "y" ]] && [[ "$response" != "Y" ]]; then
    echo -e "\n${BLUE}Please create the repository first:${NC}"
    echo -e "${CYAN}1. Go to: https://github.com/new${NC}"
    echo -e "${CYAN}2. Name it: homebrew-claude-code-personalities${NC}"
    echo -e "${CYAN}3. Make it public${NC}"
    echo -e "${CYAN}4. Don't initialize with README${NC}"
    exit 0
fi

TAP_REPO="homebrew-claude-code-personalities"
TAP_PATH="../$TAP_REPO"

# Clone or create tap repo locally
if [ -d "$TAP_PATH" ]; then
    echo -e "${YELLOW}$(printf '\xef\x81\xb1') Tap repository already exists locally${NC}"
    cd "$TAP_PATH"
    git pull origin main 2>/dev/null || true
else
    echo -e "${BLUE}$(printf '\xef\x80\x8b') Creating local tap repository...${NC}"
    mkdir -p "$TAP_PATH"
    cd "$TAP_PATH"
    git init
    git remote add origin "git@personal:Mehdi-Hp/$TAP_REPO.git"
fi

# Copy formula
echo -e "${BLUE}$(printf '\xef\x80\x89') Copying formula...${NC}"
cp ../claude-code-personalities/Formula/claude-code-personalities.rb ./

# Create README
echo -e "${BLUE}$(printf '\xef\x81\x84') Creating README...${NC}"
cat > README.md << 'EOF'
# Homebrew Claude Code Personalities

Homebrew tap for [Claude Code Personalities](https://github.com/Mehdi-Hp/claude-code-personalities).

## 🎭 What is Claude Code Personalities?

Dynamic text-face personalities for Claude Code's statusline that change based on Claude's current activity.

## Installation

### Via Tap

```bash
brew tap Mehdi-Hp/claude-code-personalities
brew install claude-code-personalities
claude-personalities-setup --install
```

### Direct Installation

```bash
brew install Mehdi-Hp/claude-code-personalities/claude-code-personalities
claude-personalities-setup --install
```

## Uninstallation

```bash
claude-personalities-setup --uninstall
brew uninstall claude-code-personalities
brew untap Mehdi-Hp/claude-code-personalities  # if you used tap method
```

## Documentation

See the [main repository](https://github.com/Mehdi-Hp/claude-code-personalities) for full documentation.

## License

WTFPL - Do What The F*ck You Want To Public License
EOF

# Create initial commit
echo -e "${BLUE}$(printf '\xef\x80\x89') Creating initial commit...${NC}"
git add .
git commit -m "feat: initial tap repository setup

- Add claude-code-personalities formula
- Add README with installation instructions"

# Push to GitHub
echo -e "${BLUE}$(printf '\xef\x82\x93') Pushing to GitHub...${NC}"
git branch -M main
git push -u origin main || {
    echo -e "${RED}$(printf '\xef\x81\x97') Failed to push. Make sure:${NC}"
    echo -e "  1. The repository exists on GitHub"
    echo -e "  2. You have push access"
    echo -e "  3. Your SSH keys are configured"
    exit 1
}

echo
echo -e "${GREEN}${BOLD}$(printf '\xef\x80\x8c') Tap repository setup complete!${NC}\n"
echo -e "Repository: ${CYAN}https://github.com/Mehdi-Hp/$TAP_REPO${NC}"
echo
echo -e "${BOLD}Users can now install with:${NC}"
echo -e "  ${CYAN}brew tap Mehdi-Hp/claude-code-personalities${NC}"
echo -e "  ${CYAN}brew install claude-code-personalities${NC}"
echo
echo -e "${BOLD}For automated syncing:${NC}"
echo -e "  1. Generate a deploy key:"
echo -e "     ${CYAN}ssh-keygen -t ed25519 -f ~/.ssh/tap_deploy_key -N ''${NC}"
echo -e "  2. Add public key to tap repo's deploy keys (with write access):"
echo -e "     ${CYAN}https://github.com/Mehdi-Hp/$TAP_REPO/settings/keys${NC}"
echo -e "  3. Add private key as secret TAP_DEPLOY_KEY in main repo:"
echo -e "     ${CYAN}https://github.com/Mehdi-Hp/claude-code-personalities/settings/secrets/actions${NC}"