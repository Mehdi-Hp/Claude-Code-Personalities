#!/bin/bash

# Claude Code Personalities - Interactive Quick Installer
# Run: bash install.sh
# Or: curl -fsSL [url] | bash --auto  (for non-interactive mode)

set -e

VERSION="1.0.0"
CLAUDE_DIR="$HOME/.claude"
HOOKS_DIR="$CLAUDE_DIR/hooks"
AUTO_MODE=false

# Nerd Font icons (same ones used in the personality system)
ICON_FOLDER=$(printf '\xef\x81\xbb')
ICON_CODE=$(printf '\xef\x84\xa1')
ICON_BUG=$(printf '\xef\x86\x88')
ICON_SEARCH=$(printf '\xef\x80\x82')
ICON_EDIT=$(printf '\xef\x81\x84')
ICON_RUN=$(printf '\xef\x83\xa7')
ICON_GEAR=$(printf '\xef\x80\x93')
ICON_ROCKET=$(printf '\xef\x84\xb5')

# Check for auto mode
for arg in "$@"; do
    case $arg in
        --auto|-a)
            AUTO_MODE=true
            ;;
        --help|-h)
            echo "Claude Code Personalities Installer v$VERSION"
            echo "Usage: bash install.sh [OPTIONS]"
            echo "Options:"
            echo "  --auto, -a    Run in automatic mode (no prompts)"
            echo "  --help, -h    Show this help message"
            exit 0
            ;;
    esac
done

# Also check environment variable
if [[ "$CLAUDE_AUTO_INSTALL" == "true" ]]; then
    AUTO_MODE=true
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

# Helper functions
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

confirm_action() {
    if [[ "$AUTO_MODE" == true ]]; then
        return 0
    fi
    
    local message="$1"
    
    echo -e "${CYAN}$message${NC}"
    echo
    echo -e "${YELLOW}[Enter] Yes | [q] Quit: ${NC}"
    read -n 1 -r response
    echo # Add newline after single char input
    
    if [[ "$response" == "q" ]] || [[ "$response" == "Q" ]]; then
        echo -e "${YELLOW}Installation cancelled.${NC}"
        exit 0
    fi
    
    return 0
}

# Header
clear
echo -e "${BOLD}${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${BLUE}║  ( ꩜ ᯅ ꩜;)⁭⁭ Claude Code Personalities Installer v${VERSION}  ║${NC}"
echo -e "${BOLD}${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo

if [[ "$AUTO_MODE" == true ]]; then
    print_info "Running in automatic mode (no prompts)"
else
    echo -e "${CYAN}This interactive installer will guide you through the setup.${NC}"
    echo -e "${CYAN}You'll be prompted before any files are modified.${NC}"
fi
echo

# Preview
echo -e "${CYAN}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "${BOLD}What this installer does:${NC}"
echo -e "${CYAN}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "  ${CYAN}1.${NC} Check for Claude Code and jq dependencies"
echo -e "  ${CYAN}2.${NC} Test icon rendering (and provide Nerd Fonts info if needed)"
echo -e "  ${CYAN}3.${NC} Back up any existing configurations"
echo -e "  ${CYAN}4.${NC} Install the personality statusline script"
echo -e "  ${CYAN}5.${NC} Install activity tracking hooks"
echo -e "  ${CYAN}6.${NC} Configure Claude Code settings"
echo -e "${CYAN}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo

confirm_action "Ready to begin installation?"

# Step 1: Check dependencies
echo
echo -e "${CYAN}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "${BOLD}${BLUE}Step 1/6:${NC} ${BOLD}Checking dependencies...${NC}"
echo -e "${CYAN}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo

# Check if Claude Code is installed
if [[ ! -d "$CLAUDE_DIR" ]]; then
    print_error "Claude Code is not installed (missing ~/.claude directory)"
    echo
    echo "Please install Claude Code first:"
    echo -e "  ${CYAN}https://docs.anthropic.com/en/docs/claude-code${NC}"
    echo
    exit 1
else
    print_success "Claude Code is installed"
fi

# Create hooks directory if it doesn't exist (silently)
mkdir -p "$HOOKS_DIR"

if ! command -v jq &> /dev/null; then
    print_warning "jq not found (required for JSON processing)"
    echo "  jq is needed for the personalities to work properly."
    echo -e "  Install with: ${CYAN}brew install jq${NC}"
    
    if [[ "$AUTO_MODE" == false ]]; then
        echo
        echo -e "${YELLOW}Options:${NC}"
        echo "  [c] Continue without jq (limited functionality)"
        echo "  [q] Quit and install jq first"
        echo -ne "${YELLOW}Your choice: ${NC}"
        read -r response
        if [[ "$response" == "q" ]] || [[ "$response" == "Q" ]]; then
            echo
            echo "Please install jq with: brew install jq"
            echo "Then run this installer again."
            exit 1
        fi
    fi
else
    print_success "jq is installed"
fi

# Step 2: Icon rendering test
echo
echo -e "${MAGENTA}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "${BOLD}${BLUE}Step 2/6:${NC} ${BOLD}Testing icon rendering...${NC}"
echo -e "${MAGENTA}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo
echo "  The personalities use special icons. Let's test if they render correctly:"
echo "  (These are the actual icons you'll see in your Claude Code statusline)"
echo
printf "  %s %s %s %s %s %s %s %s\n" "$ICON_FOLDER" "$ICON_EDIT" "$ICON_SEARCH" "$ICON_RUN" "$ICON_BUG" "$ICON_GEAR" "$ICON_CODE" "$ICON_ROCKET"
echo
echo "  If you see boxes, question marks, or garbled text instead of icons,"
echo "  you should install Nerd Fonts for the best experience:"
echo -e "  ${CYAN}https://www.nerdfonts.com/font-downloads${NC}"
echo -e "  ${CYAN}brew install --cask font-hack-nerd-font${NC}"
echo
echo -e "${CYAN}Do the icons above display correctly?${NC}"
echo
echo -e "${YELLOW}[Enter] Yes, looks good | [q] Quit: ${NC}"
read -n 1 -r font_response
echo

if [[ "$font_response" == "q" ]] || [[ "$font_response" == "Q" ]]; then
    echo -e "${YELLOW}Installation cancelled.${NC}"
    echo
    echo "If icons didn't display correctly, install Nerd Fonts with:"
    echo -e "  ${CYAN}brew install --cask font-hack-nerd-font${NC}"
    echo -e "  ${CYAN}https://www.nerdfonts.com/font-downloads${NC}"
    exit 0
else
    print_success "Great! Proceeding with installation"
fi

# Step 3: Backup existing files
echo
echo -e "${YELLOW}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "${BOLD}${BLUE}Step 3/6:${NC} ${BOLD}Backing up existing files...${NC}"
echo -e "${YELLOW}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo

BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUPS_MADE=false
SKIP_BACKUPS=false

# Check what files exist that could be backed up
files_to_backup=0
[[ -f "$CLAUDE_DIR/statusline.sh" ]] && ((files_to_backup++))
[[ -f "$CLAUDE_DIR/settings.json" ]] && ((files_to_backup++))
[[ -d "$HOOKS_DIR" ]] && [[ "$(ls -A $HOOKS_DIR 2>/dev/null)" ]] && ((files_to_backup++))

if [[ $files_to_backup -gt 0 ]]; then
    echo "  Found existing files that can be backed up:"
    [[ -f "$CLAUDE_DIR/statusline.sh" ]] && echo -e "    • ${CYAN}statusline.sh${NC}"
    [[ -f "$CLAUDE_DIR/settings.json" ]] && echo -e "    • ${CYAN}settings.json${NC}"
    [[ -d "$HOOKS_DIR" ]] && [[ "$(ls -A $HOOKS_DIR 2>/dev/null)" ]] && echo -e "    • ${CYAN}hooks/${NC}"
    echo
    echo -e "${CYAN}Back up existing files?${NC}"
    echo
    echo -e "${YELLOW}[Enter] Yes, back up | [s] Skip backups | [q] Quit: ${NC}"
    read -n 1 -r backup_response
    echo
    
    if [[ "$backup_response" == "q" ]] || [[ "$backup_response" == "Q" ]]; then
        echo -e "${YELLOW}Installation cancelled.${NC}"
        exit 0
    elif [[ "$backup_response" == "s" ]] || [[ "$backup_response" == "S" ]]; then
        print_info "Skipping backups - existing files will be overwritten"
        SKIP_BACKUPS=true
    else
        # Perform backups
        if [[ -f "$CLAUDE_DIR/statusline.sh" ]]; then
            cp "$CLAUDE_DIR/statusline.sh" "$CLAUDE_DIR/statusline.sh.backup.$BACKUP_TIMESTAMP"
            print_success "Backed up statusline.sh.backup.$BACKUP_TIMESTAMP"
            BACKUPS_MADE=true
        fi
        
        if [[ -f "$CLAUDE_DIR/settings.json" ]]; then
            cp "$CLAUDE_DIR/settings.json" "$CLAUDE_DIR/settings.json.backup.$BACKUP_TIMESTAMP"
            print_success "Backed up settings.json.backup.$BACKUP_TIMESTAMP"
            BACKUPS_MADE=true
        fi
        
        if [[ -d "$HOOKS_DIR" ]] && [[ "$(ls -A $HOOKS_DIR 2>/dev/null)" ]]; then
            cp -r "$HOOKS_DIR" "${HOOKS_DIR}.backup.$BACKUP_TIMESTAMP"
            print_success "Backed up hooks.backup.$BACKUP_TIMESTAMP"
            BACKUPS_MADE=true
        fi
    fi
else
    print_info "No existing files to back up"
fi

# Step 4: Install statusline
echo
echo -e "${GREEN}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "${BOLD}${BLUE}Step 4/6:${NC} ${BOLD}Installing personality statusline...${NC}"
echo -e "${GREEN}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo
echo -e "  ${CYAN}Purpose:${NC} Displays text-face personalities and status icons"
echo -e "  ${CYAN}Target:${NC}  ~/.claude/statusline.sh"
echo -e "  ${CYAN}Size:${NC}    ~3KB"

confirm_action "Install statusline.sh?"

if [[ $? -eq 0 ]]; then
    # Determine the script directory
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    # Copy from source if we're running from the repo
    if [[ -f "$SCRIPT_DIR/scripts/statusline.sh" ]]; then
        cp "$SCRIPT_DIR/scripts/statusline.sh" "$CLAUDE_DIR/statusline.sh"
    else
        # Fallback: try to download from GitHub
        curl -fsSL "https://raw.githubusercontent.com/yourusername/claude-code-personalities/main/scripts/statusline.sh" \
             -o "$CLAUDE_DIR/statusline.sh" 2>/dev/null || {
            print_error "Could not find or download statusline.sh"
            exit 1
        }
    fi
    
    chmod +x "$CLAUDE_DIR/statusline.sh"
    print_success "Installed statusline.sh"
else
    print_info "Skipped statusline installation"
fi

# Step 5: Install hooks
echo
echo -e "${BLUE}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "${BOLD}${BLUE}Step 5/6:${NC} ${BOLD}Installing activity tracking hooks...${NC}"
echo -e "${BLUE}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo
echo "  Hooks to install:"
echo -e "    ${CYAN}•${NC} personalities_track_activity.sh - Assigns personalities based on activity"
echo -e "    ${CYAN}•${NC} personalities_reset_errors.sh - Resets frustration on new prompts"
echo -e "    ${CYAN}•${NC} personalities_session_end.sh - Cleans up after sessions"

confirm_action "Install all hook scripts?"

if [[ $? -eq 0 ]]; then
    # Determine the script directory
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    # Install personalities_track_activity.sh
    print_info "Installing personalities_track_activity.sh..."
    if [[ -f "$SCRIPT_DIR/hooks/personalities_track_activity.sh" ]]; then
        cp "$SCRIPT_DIR/hooks/personalities_track_activity.sh" "$HOOKS_DIR/personalities_track_activity.sh"
    else
        curl -fsSL "https://raw.githubusercontent.com/yourusername/claude-code-personalities/main/hooks/personalities_track_activity.sh" \
             -o "$HOOKS_DIR/personalities_track_activity.sh" 2>/dev/null || {
            print_error "Could not find or download personalities_track_activity.sh"
        }
    fi
    chmod +x "$HOOKS_DIR/personalities_track_activity.sh"
    print_success "Installed personalities_track_activity.sh"

    # Install personalities_reset_errors.sh
    print_info "Installing personalities_reset_errors.sh..."
    if [[ -f "$SCRIPT_DIR/hooks/personalities_reset_errors.sh" ]]; then
        cp "$SCRIPT_DIR/hooks/personalities_reset_errors.sh" "$HOOKS_DIR/personalities_reset_errors.sh"
    else
        curl -fsSL "https://raw.githubusercontent.com/yourusername/claude-code-personalities/main/hooks/personalities_reset_errors.sh" \
             -o "$HOOKS_DIR/personalities_reset_errors.sh" 2>/dev/null || {
            print_error "Could not find or download personalities_reset_errors.sh"
        }
    fi
    chmod +x "$HOOKS_DIR/personalities_reset_errors.sh"
    print_success "Installed personalities_reset_errors.sh"

    # Install personalities_session_end.sh
    print_info "Installing personalities_session_end.sh..."
    if [[ -f "$SCRIPT_DIR/hooks/personalities_session_end.sh" ]]; then
        cp "$SCRIPT_DIR/hooks/personalities_session_end.sh" "$HOOKS_DIR/personalities_session_end.sh"
    else
        curl -fsSL "https://raw.githubusercontent.com/yourusername/claude-code-personalities/main/hooks/personalities_session_end.sh" \
             -o "$HOOKS_DIR/personalities_session_end.sh" 2>/dev/null || {
            print_error "Could not find or download personalities_session_end.sh"
        }
    fi
    chmod +x "$HOOKS_DIR/personalities_session_end.sh"
    print_success "Installed personalities_session_end.sh"
else
    print_info "Skipped hooks installation"
fi

# Step 6: Update settings.json
echo
echo -e "${MAGENTA}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo -e "${BOLD}${BLUE}Step 6/6:${NC} ${BOLD}Configuring Claude Code settings...${NC}"
echo -e "${MAGENTA}✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦ ✦${NC}"
echo

# Define the personality configuration
PERSONALITY_CONFIG='{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/statusline.sh",
    "padding": 0
  },
  "hooks": {
    "PreToolUse": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/personalities_track_activity.sh"
      }]
    }],
    "PostToolUse": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/personalities_track_activity.sh"
      }]
    }],
    "UserPromptSubmit": [{
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/personalities_reset_errors.sh"
      }]
    }],
    "Stop": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/personalities_session_end.sh"
      }]
    }]
  }
}'

if [[ -f "$CLAUDE_DIR/settings.json" ]]; then
    echo -e "  ${YELLOW}Existing settings.json detected${NC}"
    echo "  Will merge personality configuration with your current settings"
    echo
    confirm_action "Update settings.json?"
    
    if [[ $? -eq 0 ]]; then
        # Check if jq is available for proper JSON merging
        if command -v jq &> /dev/null; then
            print_info "Using jq to merge settings..."
            
            # Create a temporary file for the merged config
            TEMP_SETTINGS=$(mktemp)
            
            # Merge the configurations, with personality config taking precedence for statusLine and hooks
            jq -s '.[0] * .[1]' "$CLAUDE_DIR/settings.json" <(echo "$PERSONALITY_CONFIG") > "$TEMP_SETTINGS" 2>/dev/null
            
            if [[ $? -eq 0 ]] && [[ -s "$TEMP_SETTINGS" ]]; then
                # Verify the JSON is valid
                if jq empty "$TEMP_SETTINGS" 2>/dev/null; then
                    mv "$TEMP_SETTINGS" "$CLAUDE_DIR/settings.json"
                    print_success "Successfully merged personality configuration with existing settings"
                else
                    rm -f "$TEMP_SETTINGS"
                    print_error "Failed to create valid JSON during merge"
                    print_info "Creating new settings.json with personality config (backup was created)"
                    echo "$PERSONALITY_CONFIG" | jq '.' > "$CLAUDE_DIR/settings.json"
                fi
            else
                rm -f "$TEMP_SETTINGS"
                print_warning "Failed to merge configurations"
                print_info "Creating new settings.json with personality config (backup was created)"
                echo "$PERSONALITY_CONFIG" | jq '.' > "$CLAUDE_DIR/settings.json"
            fi
        else
            # Fallback: No jq available, use basic approach
            print_warning "jq not found - cannot merge settings automatically"
            echo
            echo "  Options:"
            echo "  1. Your existing settings have been backed up"
            echo "  2. We'll create a new settings.json with personality config"
            echo "  3. You can manually merge your backup after installation"
            echo
            confirm_action "Replace settings.json with personality config?"
            
            if [[ $? -eq 0 ]]; then
                echo "$PERSONALITY_CONFIG" > "$CLAUDE_DIR/settings.json"
                print_success "Created new settings.json with personality config"
                print_info "Your original settings are in: settings.json.backup.$BACKUP_TIMESTAMP"
                print_info "You may want to manually merge any custom settings from the backup"
            else
                print_info "Skipped settings.json update"
                print_warning "You'll need to manually configure hooks in settings.json"
            fi
        fi
    else
        print_info "Skipped settings.json update"
        print_warning "You'll need to manually configure hooks in settings.json"
    fi
else
    echo "  Creating new settings.json with personality configuration"
    confirm_action "Create settings.json?"
    
    if [[ $? -eq 0 ]]; then
        # Create new settings.json
        if command -v jq &> /dev/null; then
            echo "$PERSONALITY_CONFIG" | jq '.' > "$CLAUDE_DIR/settings.json"
        else
            echo "$PERSONALITY_CONFIG" > "$CLAUDE_DIR/settings.json"
        fi
        print_success "Created settings.json with personality config"
    else
        print_info "Skipped settings.json creation"
        print_warning "You'll need to manually create and configure settings.json"
    fi
fi

# Completion
echo
echo -e "${GREEN}✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨${NC}"
echo -e "${GREEN}${BOLD}🎉 Installation Complete! 🎉${NC}"
echo -e "${GREEN}✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨ ✨${NC}"
echo

echo -e "${BOLD}Installed Personalities:${NC}"
echo -e "  ${MAGENTA}•${NC} Frustrated Developer - Gets angry with errors"
echo -e "  ${MAGENTA}•${NC} Mischievous Debugger - When debugging code"
echo -e "  ${MAGENTA}•${NC} Bug Hunter - When searching with grep"
echo -e "  ${MAGENTA}•${NC} Code Wizard - General coding mode"
echo -e "  ${MAGENTA}•${NC} Git Manager - During git operations"
echo -e "  ${MAGENTA}•${NC} And 25+ more context-aware personalities!"
echo

echo -e "${BOLD}Next Steps:${NC}"
echo -e "  1. ${CYAN}Restart Claude Code${NC} to activate personalities"
echo "  2. Start coding and watch your personalities change!"
echo

if [[ "$BACKUPS_MADE" == true ]]; then
    echo -e "${BOLD}Backups:${NC}"
    echo -e "  Your original files were backed up with timestamp: ${CYAN}$BACKUP_TIMESTAMP${NC}"
    echo
fi

print_success "Enjoy your new Claude Code personalities! 🎭"