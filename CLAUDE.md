# Claude Code Personalities

> Dynamic text-face personalities for Claude Code's statusline that change based on what Claude is doing

## Important Rules

- **NO EMOJIS**: Never use emojis in any files or output
- **Nerd Font Icons Only**: If icons are needed, use Nerd Font UTF-8 byte sequences only (e.g., `$(printf '\xef\x81\xbb')` for folder icon)

## What is This?

Claude Code Personalities is a personality system that gives Claude Code a dynamic, context-aware statusline with text-face emoticons that change based on Claude's current activity. Instead of a static prompt, you get a fun, informative statusline that shows Claude's "mood" and what it's currently working on.

### Features

- **30+ Text-Face Personalities**: From `ʕ•ᴥ•ʔ Code Wizard` to `(┛ಠДಠ)┛彡┻━┻ Frustrated Developer`
- **Context-Aware**: Personalities change based on files being edited, commands run, and errors encountered
- **Activity Tracking**: Monitors Claude's tool usage (Edit, Bash, Grep, etc.) via hooks
- **Error State Management**: Claude gets progressively more frustrated with errors
- **Nerd Font Icons**: Visual indicators for folders, activities, and status
- **Session Persistence**: Maintains personality state across a Claude Code session
- **Model-Specific Indicators**: Different icons for Opus, Sonnet, and Haiku

## How It Works

```mermaid
graph LR
    A[Claude Code] -->|Sends JSON| B[statusline.sh]
    A -->|Tool Events| C[Hooks]
    C -->|Updates State| D[/tmp/claude_activity_*.json]
    D -->|Reads State| B
    B -->|Displays| E[Terminal Statusline]
```

1. **Claude Code** calls `statusline.sh` with JSON input containing session and workspace info
2. **Hooks** intercept tool usage (PreToolUse/PostToolUse) and track Claude's activities
3. **Activity tracking** determines appropriate personality based on context
4. **State files** in `/tmp` maintain personality and activity across the session
5. **Statusline** displays personality, current directory, activity, and model

## File Structure

### Core Files

```
~/.claude/
├── statusline.sh                 # Main statusline script
├── settings.json                 # Claude Code configuration
└── hooks/
    ├── personalities_track_activity.sh        # Activity monitoring & personality assignment
    ├── personalities_reset_errors.sh          # Resets error counter on new prompts
    └── personalities_session_end.sh           # Cleanup after session ends
```

### Temporary State Files

```
/tmp/
├── claude_activity_${session_id}.json    # Current personality & activity state
└── claude_errors_${session_id}.count     # Error counter for frustration tracking
```

### Repository Structure (for development)

```
claude-code-personalities/
├── CLAUDE.md                    # This file
├── README.md                     # User-facing documentation
├── LICENSE                       # MIT License
├── install.sh                    # Quick installer script
├── scripts/
│   └── statusline.sh            # Source statusline script
├── hooks/
│   ├── personalities_track_activity.sh       # Source activity hook
│   ├── personalities_reset_errors.sh         # Source error reset hook
│   └── personalities_session_end.sh          # Source session cleanup hook
├── Formula/
│   └── claude-code-personalities.rb  # Homebrew formula
└── claude-personalities-setup    # Setup utility for Homebrew
```

## File Details

### `statusline.sh`

**Location**: `~/.claude/statusline.sh`  
**Purpose**: Main statusline display script  
**Called by**: Claude Code on every statusline update  

**Input** (via stdin):
```json
{
  "session_id": "abc-123",
  "workspace": {
    "current_dir": "/path/to/project",
    "project_dir": "/path/to/project"
  },
  "model": {
    "display_name": "Opus"
  }
}
```

**Output**: Formatted statusline with ANSI colors
```
( ˘ ³˘) Booting Up •  Pulli •  starting • [ opus]
```

**Key Variables**:
- Icons defined as UTF-8 byte sequences (e.g., `$(printf '\xef\x81\xbb')`)
- Reads personality from state file
- Determines activity icon based on current activity
- Colors: Bold for personality, gray for separators, colored model indicators

### `hooks/personalities_track_activity.sh`

**Location**: `~/.claude/hooks/personalities_track_activity.sh`  
**Purpose**: Monitors Claude's tool usage and assigns personalities  
**Called by**: PreToolUse and PostToolUse events  

**Personality Assignment Logic**:
```bash
# Frustration states (based on error count)
errors >= 5  →  (╯°□°)╯︵ ┻━┻ Table Flipper
errors >= 3  →  (┛ಠДಠ)┛彡┻━┻ Frustrated Developer

# Git operations
git command  →  ┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager

# Testing
test/spec    →  (¬_¬) Test Engineer

# Debugging
debug/grep   →  ( ͡° ͜ʖ ͡°) Mischievous Debugger
              →  (つ◉益◉)つ Bug Hunter

# File type based
*.md         →  (͡• ͜໒ ͡• ) Documentation Writer
*.jsx/*.tsx  →  ʕ•ᴥ•ʔ UI Developer
auth/security→  ಠ_ಠ Security Analyst

# Long sessions
consecutive > 10  →  ┌༼◉ل͟◉༽┐ Hyperfocused Coder
```

**State File Output**:
```json
{
  "session_id": "abc-123",
  "activity": "editing",
  "personality": "ʕ•ᴥ•ʔ Code Wizard",
  "consecutive_actions": 5,
  "error_count": 0
}
```

### `hooks/personalities_reset_errors.sh`

**Location**: `~/.claude/hooks/personalities_reset_errors.sh`  
**Purpose**: Resets error counter when user submits new prompt  
**Called by**: UserPromptSubmit event  
**Action**: Writes "0" to error count file  

### `hooks/personalities_session_end.sh`

**Location**: `~/.claude/hooks/personalities_session_end.sh`  
**Purpose**: Cleans up temporary files after session  
**Called by**: Stop event  
**Action**: Removes state and error files for the session  

### `settings.json`

**Location**: `~/.claude/settings.json`  
**Purpose**: Configures Claude Code to use personalities  

```json
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
```

## Complete Personality List

### Debugging & Testing
- `( ͡° ͜ʖ ͡°)` **Mischievous Debugger** - When debugging or using console.log
- `(つ◉益◉)つ` **Bug Hunter** - When using grep or searching for issues
- `(¬_¬)` **Test Engineer** - When running tests or working with test files
- `(╯°□°)╯` **Test Fixer** - When tests are failing

### Code Review & Quality
- `¯\_(ツ)_/¯` **Casual Code Reviewer** - During code review
- `(ㆆ_ㆆ)` **Quality Auditor** - When analyzing code quality
- `ಠ_ಠ` **Security Analyst** - Working with auth/security files

### Development & Refactoring
- `ʕ•ᴥ•ʔ` **Code Wizard / UI Developer** - General coding or React/Vue files
- `(• ε •)` **Gentle Refactorer** - When refactoring code
- `(ง'̀-'́)ง` **Dead Code Remover** - When deleting files

### Documentation & Communication
- `(͡• ͜໒ ͡• )` **Documentation Writer** - Working with README/docs
- `♥‿♥` **Config Helper** - Editing config files
- `┌༼◉ل͟◉༽┐` **Grammar Checker** - Intensive text editing

### Operations & Management
- `┗(▀̿Ĺ̯▀̿ ̿)┓` **Git Manager** - Git operations
- `( ͡ _ ͡°)ﾉ⚲` **Deployment Guard** - Deploy/Docker commands
- `⚆_⚆` **Database Expert** - Database operations
- `( ͡ _ ͡°)ノ⚡` **DevOps Engineer** - General bash operations

### Performance & Optimization
- `'(ᗒᗣᗕ)՞` **Performance Optimizer** - Performance tuning
- `★⌒ヽ( ͡° ε ͡°)` **Performance Tuner** - After optimization
- `˙ ͜ʟ˙` **Memory Manager** - Memory profiling

### Error & Frustration States
- `(┛ಠДಠ)┛彡┻━┻` **Frustrated Developer** - 3+ errors
- `(╯°□°)╯︵ ┻━┻` **Table Flipper** - 5+ errors
- `【╯°□°】╯︵ ┻━┻` **Code Berserker** - 20+ consecutive edits

### Special States
- `【≽ܫ≼】` **Research King** - Reading/searching files
- `┌༼◉ل͟◉༽┐` **Hyperfocused Coder** - 10+ consecutive actions
- `⋋| ◉ ͟ʖ ◉ |⋌` **Search Maestro** - Extended searching
- `( ˘ ³˘)` **Booting Up** - Initial startup

### Time-Based
- `( ˶˘ ³˘)☕` **Morning Engineer** - 6 AM - 12 PM
- `(つ°ヮ°)つ` **Afternoon Thinker** - 12 PM - 5 PM
- `(￣ω￣;)` **Evening Explorer** - 5 PM - 10 PM
- `˙ ͜ʟ˙` **Night Coder** - 10 PM - 6 AM

## Nerd Font Icons Reference

Icons are defined using UTF-8 byte sequences:

| Icon | UTF-8 Bytes    | Unicode | Description     |
| ---- | -------------- | ------- | --------------- |
| 📁    | `\xef\x81\xbb` | U+F07B  | Folder          |
| 💻    | `\xef\x84\xa1` | U+F121  | Code            |
| 🐛    | `\xef\x86\x88` | U+F188  | Bug             |
| 🔍    | `\xef\x80\x82` | U+F002  | Search          |
| ✏️    | `\xef\x81\x84` | U+F044  | Edit            |
| ⚡    | `\xef\x83\xa7` | U+F0E7  | Lightning/Run   |
| 👁️    | `\xef\x81\xae` | U+F06E  | Eye/Review      |
| 💡    | `\xef\x83\xab` | U+F0EB  | Lightbulb/Think |
| 🚀    | `\xef\x84\xb5` | U+F135  | Rocket          |
| ⚠️    | `\xef\x81\xb1` | U+F071  | Warning         |
| ❌    | `\xef\x81\x97` | U+F057  | Error           |
| 🔥    | `\xef\x81\xad` | U+F06D  | Fire            |
| ⚙️    | `\xef\x80\x93` | U+F013  | Gear            |
| 💻    | `\xef\x84\xa0` | U+F120  | Terminal        |

## Installation Methods

### Quick Install
```bash
curl -fsSL https://raw.githubusercontent.com/kumamaki/claude-code-personalities/main/install.sh | bash
```

### Interactive Install
```bash
git clone https://github.com/kumamaki/claude-code-personalities
cd claude-code-personalities
./install.sh
```

### Homebrew
```bash
# Option 1: Direct install
brew install kumamaki/claude-code-personalities/claude-code-personalities

# Option 2: Via tap
brew tap kumamaki/claude-code-personalities
brew install claude-code-personalities

# Then run setup
claude-personalities-setup --install
```

## Testing

### Test Statusline
```bash
echo '{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/test"}}' | ~/.claude/statusline.sh
```

### Test Hooks
```bash
# Test activity tracking
echo '{"session_id":"test","tool_name":"Edit","tool_input":{"file_path":"test.js"}}' | ~/.claude/hooks/personalities_track_activity.sh

# Check state file
cat /tmp/claude_activity_test.json
```

### Debug Mode
```bash
# Add debug output to statusline
echo "$input" > /tmp/claude_debug.json  # Add to statusline.sh

# Check what Claude Code sends
cat /tmp/claude_debug.json
```

## Troubleshooting

### Icons Not Displaying
1. Ensure Nerd Fonts are installed: `brew install --cask font-hack-nerd-font`
2. Set terminal font to a Nerd Font
3. Test with: `printf '\xef\x81\xbb'` (should show folder icon)

### Personality Not Changing
1. Check hooks are executable: `ls -la ~/.claude/hooks/`
2. Verify settings.json has hook configuration
3. Check state file exists: `ls /tmp/claude_activity_*.json`
4. Ensure jq is installed: `brew install jq`

### Always Shows "Booting Up"
- State file isn't being created/read
- Check hook permissions
- Verify session_id is being passed correctly

### Backups
All installations create timestamped backups:
```bash
~/.claude/statusline.sh.backup.20240112_143022
~/.claude/settings.json.backup.20240112_143022
```

## Configuration Options

### Environment Variables
```bash
CLAUDE_AUTO_INSTALL=true   # Skip all prompts during installation
CLAUDE_DIR=/custom/path     # Use custom Claude directory
```

### Custom Personalities
Edit `~/.claude/hooks/personalities_track_activity.sh` to add custom personalities:
```bash
elif echo "$file" | grep -qiE "\.rs$"; then
  personality="🦀 Rust Developer"
```

### Disable Features
Remove specific hooks from `settings.json` to disable features:
- Remove `PreToolUse/PostToolUse` to disable activity tracking
- Remove `UserPromptSubmit` to keep error count across prompts
- Remove `Stop` to keep state files after session

## Contributing

1. Fork the repository
2. Add new personalities to `personalities_track_activity.sh`
3. Test with various Claude Code activities
4. Submit PR with personality description

## License

MIT - See LICENSE file

## Credits

Created for Claude Code v1.0.60+  
Requires Nerd Fonts for icons  
Uses jq for JSON processing  

---

*This is a Claude Code enhancement project. For Claude Code documentation, see [docs.anthropic.com](https://docs.anthropic.com/en/docs/claude-code)*