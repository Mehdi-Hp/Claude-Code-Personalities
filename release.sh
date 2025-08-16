#!/bin/bash

# Release script for Claude Code Personalities
# This script automates the release process

set -e

# Icons (same ones used in the personality system)
ICON_ROCKET=$(printf '\xef\x84\xb5')
ICON_CHECK=$(printf '\xef\x80\x8c')
ICON_STAR=$(printf '\xef\x80\x85')
ICON_PACKAGE=$(printf '\xef\x86\x87')
ICON_GIT=$(printf '\xef\x84\xa6')
ICON_TAG=$(printf '\xef\x81\x92')
ICON_UPLOAD=$(printf '\xef\x82\x93')
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

# Step 1: Check if on main branch
echo -e "${BLUE}${ICON_GIT} Checking branch...${NC}"
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "${RED}Error: Not on main branch (currently on $CURRENT_BRANCH)${NC}"
    echo "Please switch to main branch: git checkout main"
    exit 1
fi
echo -e "  ${GREEN}${ICON_CHECK}${NC} On main branch"

# Step 2: Check for uncommitted changes
echo -e "${BLUE}${ICON_GIT} Checking for uncommitted changes...${NC}"
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: You have uncommitted changes${NC}"
    echo "Please commit or stash your changes first"
    exit 1
fi
echo -e "  ${GREEN}${ICON_CHECK}${NC} Working directory clean"

# Step 3: Pull latest changes
echo -e "${BLUE}${ICON_GIT} Pulling latest changes...${NC}"
git pull origin main
echo -e "  ${GREEN}${ICON_CHECK}${NC} Up to date with origin"

# Step 4: Determine version
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
echo -n "Continue with release? (y/N): "
read -n 1 CONFIRM
echo ""

if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
    echo "Release cancelled"
    exit 0
fi

# Step 5: Update version files
echo ""
echo -e "${BLUE}${ICON_PACKAGE} Updating version files...${NC}"
echo "$VERSION" > .version
sed -i '' "s/VERSION=\"[0-9.]*\"/VERSION=\"$VERSION\"/" scripts/claude-code-personalities-statusline.sh
sed -i '' "s/VERSION=\"[0-9.]*\"/VERSION=\"$VERSION\"/" bin/claude-code-personalities
echo -e "  ${GREEN}${ICON_CHECK}${NC} Updated version to $VERSION"

# Step 6: Commit version changes
echo -e "${BLUE}${ICON_GIT} Committing version changes...${NC}"
git add .version scripts/claude-code-personalities-statusline.sh bin/claude-code-personalities
git commit -m "chore: bump version to $VERSION"
echo -e "  ${GREEN}${ICON_CHECK}${NC} Committed version changes"

# Step 7: Create and push tag
echo -e "${BLUE}${ICON_TAG} Creating tag $TAG...${NC}"
git tag -a "$TAG" -m "Release $TAG"
echo -e "  ${GREEN}${ICON_CHECK}${NC} Tag created"

echo -e "${BLUE}${ICON_UPLOAD} Pushing changes and tag...${NC}"
git push origin main
git push origin "$TAG"
echo -e "  ${GREEN}${ICON_CHECK}${NC} Pushed to GitHub"

# Step 8: Create release notes
echo ""
echo -e "${BLUE}${ICON_PACKAGE} Creating GitHub release...${NC}"

# Get commits since last tag
LAST_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
if [[ -n "$LAST_TAG" ]]; then
    COMMITS=$(git log "$LAST_TAG"..HEAD --pretty=format:"- %s" --no-merges)
else
    COMMITS=$(git log --pretty=format:"- %s" --no-merges -10)
fi

RELEASE_NOTES="## Changes

$COMMITS

## Installation

\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash
\`\`\`

## Updating

\`\`\`bash
claude-code-personalities update
\`\`\`
"

# Check if gh CLI is available
if command -v gh &> /dev/null; then
    echo "$RELEASE_NOTES" | gh release create "$TAG" \
        --title "Release $TAG" \
        --notes-file - \
        --latest
    echo -e "  ${GREEN}${ICON_CHECK}${NC} GitHub release created"
else
    echo -e "${YELLOW}${ICON_WARNING}  GitHub CLI (gh) not found. Install with: brew install gh${NC}"
    echo ""
    echo "Please create the release manually at:"
    echo "https://github.com/Mehdi-Hp/claude-code-personalities/releases/new"
    echo ""
    echo "Tag: $TAG"
    echo ""
    echo "Release notes:"
    echo "$RELEASE_NOTES"
fi

echo ""
echo -e "${BOLD}${GREEN}${ICON_STAR} Release $TAG complete!${NC}"
echo ""
echo -e "${CYAN}Installation:${NC}"
echo "  curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash"
echo ""
echo -e "${CYAN}Update:${NC}"
echo "  claude-code-personalities update"
echo ""