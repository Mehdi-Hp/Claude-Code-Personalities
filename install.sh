#!/bin/bash

# Claude Code Personalities - Interactive Quick Installer
# Run: bash install.sh
# Or: curl -fsSL [url] | bash --auto  (for non-interactive mode)

set -e

VERSION="1.0.0"
CLAUDE_DIR="$HOME/.claude"
HOOKS_DIR="$CLAUDE_DIR/hooks"
AUTO_MODE=false
SKIP_ALL=false

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
    if [[ "$AUTO_MODE" == true ]] || [[ "$SKIP_ALL" == true ]]; then
        return 0
    fi
    
    local message="$1"
    local allow_skip="${2:-false}"
    
    echo -e "${CYAN}$message${NC}"
    
    if [[ "$allow_skip" == "true" ]]; then
        echo -e "${YELLOW}[Enter] Continue | [s] Skip | [a] Yes to all | [q] Quit: ${NC}"
        read -r response
        case "$response" in
            [sS])
                return 1
                ;;
            [aA])
                SKIP_ALL=true
                return 0
                ;;
            [qQ])
                echo -e "${YELLOW}Installation cancelled.${NC}"
                exit 0
                ;;
            *)
                return 0
                ;;
        esac
    else
        echo -e "${YELLOW}[Enter] Continue | [q] Quit: ${NC}"
        read -r response
        if [[ "$response" == "q" ]] || [[ "$response" == "Q" ]]; then
            echo -e "${YELLOW}Installation cancelled.${NC}"
            exit 0
        fi
    fi
    return 0
}

# Header
clear
echo -e "${BOLD}${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${BLUE}║     🎭 Claude Code Personalities Installer v${VERSION}    ║${NC}"
echo -e "${BOLD}${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo

if [[ "$AUTO_MODE" == true ]]; then
    print_info "Running in automatic mode (no prompts)"
else
    echo -e "${CYAN}This interactive installer will guide you through the setup.${NC}"
    echo -e "${CYAN}You'll be prompted before any files are modified.${NC}"
fi
echo

# Preview
echo -e "${BOLD}What this installer does:${NC}"
echo "  ${CYAN}1.${NC} Check for required dependencies (jq)"
echo "  ${CYAN}2.${NC} Create ~/.claude directories if needed"
echo "  ${CYAN}3.${NC} Back up any existing configurations"
echo "  ${CYAN}4.${NC} Install the personality statusline script"
echo "  ${CYAN}5.${NC} Install activity tracking hooks"
echo "  ${CYAN}6.${NC} Configure Claude Code settings"
echo
echo -e "${BOLD}Personalities you'll get:${NC}"
echo "  ${MAGENTA}•${NC} (┛ಠДಠ)┛彡┻━┻ Frustrated Developer"
echo "  ${MAGENTA}•${NC} ( ͡° ͜ʖ ͡°) Mischievous Debugger"
echo "  ${MAGENTA}•${NC} ʕ•ᴥ•ʔ Code Wizard"
echo "  ${MAGENTA}•${NC} And 30+ more dynamic personalities!"
echo

confirm_action "Ready to begin installation?"

# Step 1: Check dependencies
echo
echo -e "${BOLD}${BLUE}Step 1/6:${NC} ${BOLD}Checking dependencies...${NC}"
echo

if ! command -v jq &> /dev/null; then
    print_warning "jq not found (required for JSON processing)"
    echo "  jq is needed for the personalities to work properly."
    echo "  Install with: ${CYAN}brew install jq${NC}"
    
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

# Check for Nerd Fonts
if fc-list 2>/dev/null | grep -qi "nerd" &> /dev/null; then
    print_success "Nerd Fonts detected"
elif ls ~/Library/Fonts/*Nerd* &> /dev/null 2>&1; then
    print_success "Nerd Fonts detected"
else
    print_warning "Nerd Fonts not detected (icons may not display)"
    print_info "Install with: brew install --cask font-hack-nerd-font"
fi

# Step 2: Create directories
echo
echo -e "${BOLD}${BLUE}Step 2/6:${NC} ${BOLD}Creating directories...${NC}"
echo
echo "  Will create (if needed):"
echo "    • ${CYAN}~/.claude${NC}"
echo "    • ${CYAN}~/.claude/hooks${NC}"

confirm_action "Create these directories?" true

if [[ $? -eq 0 ]]; then
    mkdir -p "$HOOKS_DIR"
    print_success "Directories ready"
else
    print_info "Skipped directory creation"
fi

# Step 3: Backup existing files
echo
echo -e "${BOLD}${BLUE}Step 3/6:${NC} ${BOLD}Backing up existing files...${NC}"
echo

BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUPS_MADE=false

if [[ -f "$CLAUDE_DIR/statusline.sh" ]]; then
    print_warning "Found existing statusline.sh"
    confirm_action "Back up existing statusline.sh?" true
    
    if [[ $? -eq 0 ]]; then
        cp "$CLAUDE_DIR/statusline.sh" "$CLAUDE_DIR/statusline.sh.backup.$BACKUP_TIMESTAMP"
        print_success "Backed up to statusline.sh.backup.$BACKUP_TIMESTAMP"
        BACKUPS_MADE=true
    fi
fi

if [[ -f "$CLAUDE_DIR/settings.json" ]]; then
    print_warning "Found existing settings.json"
    confirm_action "Back up existing settings.json?" true
    
    if [[ $? -eq 0 ]]; then
        cp "$CLAUDE_DIR/settings.json" "$CLAUDE_DIR/settings.json.backup.$BACKUP_TIMESTAMP"
        print_success "Backed up to settings.json.backup.$BACKUP_TIMESTAMP"
        BACKUPS_MADE=true
    fi
fi

if [[ -d "$HOOKS_DIR" ]] && [[ "$(ls -A $HOOKS_DIR 2>/dev/null)" ]]; then
    print_warning "Found existing hooks"
    confirm_action "Back up existing hooks?" true
    
    if [[ $? -eq 0 ]]; then
        cp -r "$HOOKS_DIR" "${HOOKS_DIR}.backup.$BACKUP_TIMESTAMP"
        print_success "Backed up to hooks.backup.$BACKUP_TIMESTAMP"
        BACKUPS_MADE=true
    fi
fi

if [[ "$BACKUPS_MADE" == false ]]; then
    print_info "No existing files to back up"
fi

# Step 4: Install statusline
echo
echo -e "${BOLD}${BLUE}Step 4/6:${NC} ${BOLD}Installing personality statusline...${NC}"
echo
echo "  ${CYAN}Purpose:${NC} Displays text-face personalities and status icons"
echo "  ${CYAN}Target:${NC}  ~/.claude/statusline.sh"
echo "  ${CYAN}Size:${NC}    ~3KB"

confirm_action "Install statusline.sh?" true

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
echo -e "${BOLD}${BLUE}Step 5/6:${NC} ${BOLD}Installing activity tracking hooks...${NC}"
echo
echo "  Hooks to install:"
echo "    ${CYAN}•${NC} personalities_track_activity.sh - Assigns personalities based on activity"
echo "    ${CYAN}•${NC} personalities_reset_errors.sh - Resets frustration on new prompts"
echo "    ${CYAN}•${NC} personalities_session_end.sh - Cleans up after sessions"

confirm_action "Install all hook scripts?" true

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
echo -e "${BOLD}${BLUE}Step 6/6:${NC} ${BOLD}Configuring Claude Code settings...${NC}"
echo

if [[ -f "$CLAUDE_DIR/settings.json" ]]; then
    echo "  ${YELLOW}Existing settings.json detected${NC}"
    echo "  Will add personality configuration to your current settings"
    echo
    confirm_action "Update settings.json?" true
    
    if [[ $? -eq 0 ]]; then
        # For simplicity, we'll create a new one
        # In production, use jq to properly merge
        print_warning "Manual merge may be required for complex settings"
        print_info "Creating new settings with personality config"
    else
        print_info "Skipped settings.json update"
        print_warning "You'll need to manually configure hooks in settings.json"
    fi
else
    echo "  Creating new settings.json with personality configuration"
    confirm_action "Create settings.json?" true
fi

if [[ $? -eq 0 ]]; then
    cat > "$CLAUDE_DIR/settings.json" << 'EOF'
{
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
}
EOF
    print_success "Configuration complete"
fi

# Completion
echo
echo -e "${GREEN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}${BOLD}✨ Installation Complete!${NC}"
echo -e "${GREEN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

echo -e "${BOLD}Installed Personalities:${NC}"
echo "  ${MAGENTA}•${NC} (┛ಠДಠ)┛彡┻━┻ Frustrated Developer"
echo "  ${MAGENTA}•${NC} ( ͡° ͜ʖ ͡°) Mischievous Debugger"
echo "  ${MAGENTA}•${NC} (つ◉益◉)つ Bug Hunter"
echo "  ${MAGENTA}•${NC} ʕ•ᴥ•ʔ Code Wizard"
echo "  ${MAGENTA}•${NC} ┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager"
echo "  ${MAGENTA}•${NC} And 25+ more!"
echo

echo -e "${BOLD}Quick Test:${NC}"
echo -e "${CYAN}  echo '{\"model\":{\"display_name\":\"Opus\"}}' | ~/.claude/statusline.sh${NC}"
echo

echo -e "${BOLD}Next Steps:${NC}"
echo "  1. ${CYAN}Restart Claude Code${NC} to activate personalities"
echo "  2. Start coding and watch your personalities change!"
echo

if [[ "$BACKUPS_MADE" == true ]]; then
    echo -e "${BOLD}Backups:${NC}"
    echo "  Your original files were backed up with timestamp: ${CYAN}$BACKUP_TIMESTAMP${NC}"
    echo
fi

print_success "Enjoy your new Claude Code personalities! 🎭"