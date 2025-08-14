#!/bin/bash

# Sync formula to homebrew tap repository
# Usage: ./sync-tap.sh [--push]

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

TAP_REPO="homebrew-claude-code-personalities"
TAP_PATH="../$TAP_REPO"
FORMULA_FILE="Formula/claude-code-personalities.rb"

echo -e "${BOLD}${CYAN}$(printf '\xef\x81\x9f') Syncing Formula to Tap Repository${NC}\n"

# Check if formula exists
if [ ! -f "$FORMULA_FILE" ]; then
    echo -e "${RED}$(printf '\xef\x81\x97') Error: Formula file not found at $FORMULA_FILE${NC}"
    exit 1
fi

# Check if tap repo exists
if [ ! -d "$TAP_PATH" ]; then
    echo -e "${YELLOW}$(printf '\xef\x81\xb1') Tap repository not found locally${NC}"
    echo -e "${BLUE}Cloning tap repository...${NC}"
    cd ..
    git clone "git@github.com:Mehdi-Hp/$TAP_REPO.git" || {
        echo -e "${RED}$(printf '\xef\x81\x97') Failed to clone tap repository${NC}"
        echo -e "${YELLOW}Have you created the repository on GitHub?${NC}"
        echo -e "${CYAN}Create it at: https://github.com/new${NC}"
        exit 1
    }
    cd - > /dev/null
fi

# Navigate to tap repo
cd "$TAP_PATH"

# Update tap repo
echo -e "${BLUE}$(printf '\xef\x81\x9f') Updating tap repository...${NC}"
git pull origin main 2>/dev/null || true

# Copy formula (tap repo has formula at root, not in Formula/ directory)
echo -e "${BLUE}$(printf '\xef\x80\x89') Copying formula...${NC}"
cp "../claude-code-personalities/$FORMULA_FILE" ./claude-code-personalities.rb

# Check for changes
if [[ -z $(git status -s) ]]; then
    echo -e "${GREEN}$(printf '\xef\x80\x8c') Tap repository already up to date${NC}"
    exit 0
fi

# Show changes
echo -e "${BLUE}Changes detected:${NC}"
git diff --stat

# Commit changes
echo -e "${BLUE}$(printf '\xef\x80\x89') Committing changes...${NC}"
git add claude-code-personalities.rb
FORMULA_VERSION=$(grep -E "version \"[0-9.]+\"" claude-code-personalities.rb | sed 's/.*version "\([^"]*\)".*/\1/')
git commit -m "chore: sync formula from main repository (v$FORMULA_VERSION)"

# Push if requested
if [[ "$1" == "--push" ]]; then
    echo -e "${BLUE}$(printf '\xef\x82\x93') Pushing to GitHub...${NC}"
    git push origin main
    echo -e "${GREEN}$(printf '\xef\x80\x8c') Formula synced and pushed successfully!${NC}"
else
    echo -e "${YELLOW}$(printf '\xef\x81\xb1') Changes committed locally. Run with --push to push to GitHub${NC}"
fi

cd - > /dev/null

echo
echo -e "${GREEN}${BOLD}$(printf '\xef\x80\x8c') Sync complete!${NC}"