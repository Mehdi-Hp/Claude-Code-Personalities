#!/bin/bash

# Release script for Claude Code Personalities - Rust Edition
# Uses Makefile for building and GitHub Actions for deployment

set -e

# Icons
ICON_ROCKET=$(printf '\xef\x84\xb5')
ICON_CHECK=$(printf '\xef\x80\x8c')
ICON_STAR=$(printf '\xef\x80\x85')
ICON_GIT=$(printf '\xef\x84\xa6')
ICON_TAG=$(printf '\xef\x81\x92')
ICON_WARNING=$(printf '\xef\x81\xb1')

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}${CYAN}${ICON_ROCKET} Claude Code Personalities Release Script${NC}"
echo ""

# Step 1: Pre-flight checks
echo -e "${BLUE}${ICON_GIT} Pre-flight checks...${NC}"

# Check if on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "${RED}Error: Not on main branch (currently on $CURRENT_BRANCH)${NC}"
    echo "Please switch to main branch: git checkout main"
    exit 1
fi
echo -e "  ${GREEN}${ICON_CHECK}${NC} On main branch"

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: You have uncommitted changes${NC}"
    echo "Please commit or stash your changes first"
    exit 1
fi
echo -e "  ${GREEN}${ICON_CHECK}${NC} Working directory clean"

# Check dependencies
if ! command -v just &> /dev/null; then
    echo -e "${RED}Error: just is required${NC}"
    exit 1
fi

if ! just check-deps; then
    echo -e "${RED}Error: Missing dependencies${NC}"
    exit 1
fi

# Pull latest changes
echo -e "${BLUE}${ICON_GIT} Pulling latest changes...${NC}"
git pull origin main
echo -e "  ${GREEN}${ICON_CHECK}${NC} Up to date with origin"

# Step 2: Version management
echo ""
echo -e "${BLUE}${ICON_TAG} Version Management${NC}"
CURRENT_VERSION=$(cat .version 2>/dev/null || echo "0.0.0")
echo -e "Current version: ${CYAN}$CURRENT_VERSION${NC}"
echo ""
echo "What type of release is this?"
echo "  1) Patch (bug fixes)"
echo "  2) Minor (new features, backwards compatible)"
echo "  3) Major (breaking changes)"
echo -n "Select (1-3): "
read -n 1 RELEASE_TYPE
echo ""

# Parse current version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Increment version based on type
case $RELEASE_TYPE in
    1)
        PATCH=$((PATCH + 1))
        ;;
    2)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    3)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    *)
        echo -e "${RED}Invalid selection${NC}"
        exit 1
        ;;
esac

VERSION="$MAJOR.$MINOR.$PATCH"
TAG="v$VERSION"

echo ""
echo -e "New version: ${GREEN}$VERSION${NC}"
echo -n "Continue with release? (Y/n): "
read -n 1 CONFIRM
echo ""

if [[ -z "$CONFIRM" ]] || [[ "$CONFIRM" =~ ^[Yy]$ ]]; then
    # Continue with release
    :
else
    echo "Release cancelled"
    exit 0
fi

# Step 3: Run tests
echo ""
echo -e "${BLUE}${ICON_CHECK} Running tests and lints...${NC}"
just check
echo -e "  ${GREEN}${ICON_CHECK}${NC} All checks passed"

# Step 4: Build all binaries
echo ""
echo -e "${BLUE}${ICON_ROCKET} Building binaries for all platforms...${NC}"
just build-all
echo -e "  ${GREEN}${ICON_CHECK}${NC} All binaries built"

# Show built binaries
echo ""
echo -e "${BOLD}Built binaries:${NC}"
ls -la build/

# Step 5: Update version and create release
echo ""
echo -e "${BLUE}${ICON_TAG} Creating release...${NC}"

# Use justfile to handle version bumping and tagging
just release "$VERSION"

echo ""
echo -e "${BOLD}${GREEN}${ICON_STAR} Release $TAG complete!${NC}"
echo ""
echo -e "${CYAN}GitHub Actions will build and publish the release automatically.${NC}"
echo ""
echo -e "${CYAN}Installation (once published):${NC}"
echo "  curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash"
echo ""