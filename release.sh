#!/bin/bash

# Claude Code Personalities Release Script
# Usage: ./release.sh [version]
# Example: ./release.sh 1.2.0

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Get version from argument or prompt
VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
    echo -ne "${BLUE}Enter version (e.g., 1.1.0): ${NC}"
    read VERSION
fi

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}✗ Error: Version must be in format X.Y.Z${NC}"
    exit 1
fi

TAG="v$VERSION"

echo -e "${BOLD}${CYAN}🚀 Releasing Claude Code Personalities $TAG${NC}\n"

# Step 1: Check for uncommitted changes
echo -e "${BLUE}📋 Checking for uncommitted changes...${NC}"
if [[ -n $(git status -s) ]]; then
    echo -e "${YELLOW}⚠️  You have uncommitted changes:${NC}"
    git status -s
    echo
    echo -ne "${YELLOW}Continue anyway? (y/N): ${NC}"
    read -r response
    if [[ "$response" != "y" ]] && [[ "$response" != "Y" ]]; then
        echo -e "${RED}Release cancelled${NC}"
        exit 1
    fi
fi

# Step 2: Update version in files
echo -e "${BLUE}📝 Updating version in files...${NC}"

# Update install.sh
if grep -q 'VERSION="[0-9.]*"' install.sh; then
    sed -i '' "s/VERSION=\"[0-9.]*\"/VERSION=\"$VERSION\"/" install.sh
    echo -e "  ${GREEN}✓${NC} Updated install.sh"
fi

# Update claude-personalities-setup
if grep -q 'VERSION="[0-9.]*"' claude-personalities-setup; then
    sed -i '' "s/VERSION=\"[0-9.]*\"/VERSION=\"$VERSION\"/" claude-personalities-setup
    echo -e "  ${GREEN}✓${NC} Updated claude-personalities-setup"
fi

# Update Formula version (not the URL yet)
if grep -q 'version "[0-9.]*"' Formula/claude-code-personalities.rb 2>/dev/null; then
    sed -i '' "s/version \"[0-9.]*\"/version \"$VERSION\"/" Formula/claude-code-personalities.rb
    echo -e "  ${GREEN}✓${NC} Updated Formula version"
fi

# Commit version updates if any changes
if [[ -n $(git status -s) ]]; then
    echo -e "${BLUE}💾 Committing version updates...${NC}"
    git add -A
    git commit -m "chore: bump version to $VERSION"
    echo -e "  ${GREEN}✓${NC} Committed version changes"
fi

# Step 3: Push latest changes
echo -e "${BLUE}📤 Pushing to GitHub...${NC}"
git push origin main
echo -e "  ${GREEN}✓${NC} Pushed to main branch"

# Step 4: Create and push tag
echo -e "${BLUE}🏷️  Creating tag $TAG...${NC}"
if git tag -l | grep -q "^$TAG$"; then
    echo -e "${YELLOW}⚠️  Tag $TAG already exists. Delete it first? (y/N): ${NC}"
    read -r response
    if [[ "$response" == "y" ]] || [[ "$response" == "Y" ]]; then
        git tag -d "$TAG"
        git push origin --delete "$TAG" 2>/dev/null || true
    else
        echo -e "${RED}Release cancelled${NC}"
        exit 1
    fi
fi

git tag -a "$TAG" -m "Release $TAG - Claude Code Personalities

Enhanced installer with visual improvements and better user experience."
git push origin "$TAG"
echo -e "  ${GREEN}✓${NC} Created and pushed tag $TAG"

# Step 5: Create GitHub release
echo -e "${BLUE}📦 Creating GitHub release...${NC}"

# Create release notes
RELEASE_NOTES="## 🎭 Claude Code Personalities $VERSION

### Installation

#### Quick Install
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash
\`\`\`

#### Homebrew
\`\`\`bash
# Direct install
brew install Mehdi-Hp/claude-code-personalities/claude-code-personalities

# Or with tap
brew tap Mehdi-Hp/claude-code-personalities
brew install claude-code-personalities
\`\`\`

### Documentation

See [CLAUDE.md](https://github.com/Mehdi-Hp/claude-code-personalities/blob/main/CLAUDE.md) for:
- Complete personality list (30+ personalities)
- Customization guide
- Technical details
"

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo -e "${YELLOW}⚠️  GitHub CLI (gh) not found. Install with: brew install gh${NC}"
    echo -e "${YELLOW}   Please create the release manually at:${NC}"
    echo -e "${CYAN}   https://github.com/Mehdi-Hp/claude-code-personalities/releases/new${NC}"
else
    gh release create "$TAG" \
        --title "$TAG - Claude Code Personalities" \
        --notes "$RELEASE_NOTES" \
        --latest
    echo -e "  ${GREEN}✓${NC} Created GitHub release"
fi

# Step 6: Calculate SHA256 for Homebrew
echo -e "${BLUE}🔐 Calculating SHA256 for Homebrew formula...${NC}"
echo -e "  ${CYAN}Waiting for GitHub to process the release...${NC}"
sleep 5

# Try to get SHA256
MAX_RETRIES=3
RETRY_COUNT=0
SHA256=""

while [[ $RETRY_COUNT -lt $MAX_RETRIES ]] && [[ -z "$SHA256" ]]; then
    SHA256=$(curl -sL "https://github.com/Mehdi-Hp/claude-code-personalities/archive/$TAG.tar.gz" | shasum -a 256 | cut -d' ' -f1)
    
    if [[ -z "$SHA256" ]] || [[ "$SHA256" == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" ]]; then
        # Empty file hash, GitHub hasn't processed it yet
        SHA256=""
        RETRY_COUNT=$((RETRY_COUNT + 1))
        if [[ $RETRY_COUNT -lt $MAX_RETRIES ]]; then
            echo -e "  ${YELLOW}Retrying in 5 seconds... ($RETRY_COUNT/$MAX_RETRIES)${NC}"
            sleep 5
        fi
    fi
done

if [[ -z "$SHA256" ]]; then
    echo -e "${YELLOW}⚠️  Could not calculate SHA256 automatically${NC}"
    echo -e "${YELLOW}   Please run this command manually after GitHub processes the release:${NC}"
    echo -e "${CYAN}   curl -sL https://github.com/Mehdi-Hp/claude-code-personalities/archive/$TAG.tar.gz | shasum -a 256${NC}"
else
    echo -e "  ${GREEN}✓${NC} SHA256: ${BOLD}$SHA256${NC}"
    
    # Step 7: Update Formula with SHA256
    echo -e "${BLUE}🍺 Updating Homebrew formula...${NC}"
    sed -i '' "s/sha256 \".*\"/sha256 \"$SHA256\"/" Formula/claude-code-personalities.rb
    sed -i '' "s|archive/v[0-9.]*\.tar\.gz|archive/$TAG.tar.gz|" Formula/claude-code-personalities.rb
    echo -e "  ${GREEN}✓${NC} Updated formula with SHA256"
    
    # Step 8: Commit and push formula update
    echo -e "${BLUE}📤 Pushing formula update...${NC}"
    git add Formula/claude-code-personalities.rb
    git commit -m "chore: update formula SHA256 for $TAG release"
    git push origin main
    echo -e "  ${GREEN}✓${NC} Pushed formula update"
fi

# Success message
echo
echo -e "${GREEN}${BOLD}✅ Release $TAG completed successfully!${NC}"
echo
echo -e "${BOLD}Users can now install with:${NC}"
echo
echo -e "${CYAN}Direct install:${NC}"
echo "  brew install Mehdi-Hp/claude-code-personalities/claude-code-personalities"
echo
echo -e "${CYAN}Or with tap:${NC}"
echo "  brew tap Mehdi-Hp/claude-code-personalities"
echo "  brew install claude-code-personalities"
echo
echo -e "${CYAN}View release at:${NC}"
echo "  https://github.com/Mehdi-Hp/claude-code-personalities/releases/tag/$TAG"