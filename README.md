# Claude Code Personalities

> Dynamic text-face personalities for Claude Code's statusline that change based on what Claude is doing

![Version](https://img.shields.io/badge/version-1.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Claude Code](https://img.shields.io/badge/Claude%20Code-v1.0.60+-purple)

## What is This?

Claude Code Personalities gives your Claude Code assistant a dynamic, context-aware statusline with 30+ text-face emoticons that change based on Claude's current activity. Watch Claude transform from `ʕ•ᴥ•ʔ Code Wizard` when coding to `(┛ಠДಠ)┛彡┻━┻ Frustrated Developer` when encountering errors!

### Features

- **30+ Dynamic Personalities** - From calm to frustrated, each with unique text-faces
- **Context-Aware** - Personalities change based on files, commands, and errors
- **Activity Tracking** - Monitors tool usage (Edit, Bash, Grep, etc.)
- **Error State Management** - Claude gets progressively more frustrated with errors
- **Nerd Font Icons** - Beautiful visual indicators for status and activity
- **Session Persistence** - Maintains state across your Claude Code session
- **Model Indicators** - Different icons for Opus, Sonnet, and Haiku

## Quick Start

### ✨ Enhanced Installer (v1.1.0)

Our installer now features:
- **📊 Progress Tracking** - Visual progress bar showing installation steps
- **🎨 Beautiful UI** - Colorful, formatted output with box drawings
- **🔍 Dependency Checking** - Automatic detection of Claude Code and jq
- **💾 Smart Backups** - Timestamped backups of existing configurations
- **🎯 Icon Testing** - Verify Nerd Fonts are properly installed

### One-Line Install

```bash
curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash
```

### Interactive Install

```bash
git clone https://github.com/Mehdi-Hp/claude-code-personalities
cd claude-code-personalities
./install.sh
```


### Homebrew Install

```bash
brew tap Mehdi-Hp/claude-code
brew install claude-code-personalities
claude-personalities-setup --install
```

## How It Works

Claude Code Personalities uses hooks and a custom statusline to:

1. **Monitor Claude's activities** through PreToolUse/PostToolUse hooks
2. **Assign personalities** based on context (file types, commands, errors)
3. **Display dynamic statusline** with personality, activity, and model info
4. **Track frustration levels** as errors accumulate
5. **Reset on new prompts** to start fresh with each task

## Sample Personalities

| Personality | Face | When It Appears |
|------------|------|-----------------|
| **Code Wizard** | `ʕ•ᴥ•ʔ` | General coding |
| **Frustrated Developer** | `(┛ಠДಠ)┛彡┻━┻` | 3+ errors |
| **Table Flipper** | `(╯°□°)╯︵ ┻━┻` | 5+ errors |
| **Mischievous Debugger** | `( ͡° ͜ʖ ͡°)` | Debugging |
| **Bug Hunter** | `(つ◉益◉)つ` | Using grep/search |
| **Git Manager** | `┗(▀̿Ĺ̯▀̿ ̿)┓` | Git operations |
| **Test Engineer** | `(¬_¬)` | Running tests |
| **Documentation Writer** | `(͡• ͜໒ ͡• )` | Writing docs |
| **UI Developer** | `ʕ•ᴥ•ʔ` | React/Vue files |
| **Security Analyst** | `ಠ_ಠ` | Auth/security files |
| **Hyperfocused Coder** | `┌༼◉ل͟◉༽┐` | 10+ consecutive actions |

[See all 30+ personalities →](CLAUDE.md#complete-personality-list)

## Requirements

- **Claude Code** v1.0.60 or higher
- **jq** for JSON processing (`brew install jq`)
- **Nerd Fonts** for icons (optional but recommended)
  ```bash
  brew install --cask font-hack-nerd-font
  ```

## Installation Details

The installer will:

1. Check for required dependencies (jq)
2. Create `~/.claude` directories if needed
3. Back up any existing configurations
4. Install the personality statusline script
5. Install activity tracking hooks
6. Configure Claude Code settings

All existing files are backed up with timestamps before modification.

## Testing

### Test the Statusline

```bash
echo '{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/test"}}' | ~/.claude/statusline.sh
```

Expected output: `( ˘ ³˘) Booting Up • [folder] test • [rocket] starting • [gear] opus]`

### Check Installation

```bash
# Verify files are installed
ls -la ~/.claude/statusline.sh
ls -la ~/.claude/hooks/

# Test hook execution
echo '{"session_id":"test","tool_name":"Edit","tool_input":{"file_path":"test.js"}}' | ~/.claude/hooks/personalities_track_activity.sh
```

## Troubleshooting

### Icons Not Displaying?

1. Install Nerd Fonts: `brew install --cask font-hack-nerd-font`
2. Set your terminal font to a Nerd Font
3. Test with: `printf '\xef\x81\xbb'` (should show folder icon)

### Personality Not Changing?

1. Check hooks are executable: `ls -la ~/.claude/hooks/`
2. Verify settings.json has hook configuration
3. Check state file exists: `ls /tmp/claude_activity_*.json`
4. Ensure jq is installed: `which jq`

### Always Shows "Booting Up"?

- Session ID might not be passed correctly
- Check hook permissions: `chmod +x ~/.claude/hooks/*.sh`
- Verify Claude Code is v1.0.60+

## Customization

### Add Your Own Personalities

Edit `~/.claude/hooks/personalities_track_activity.sh` to add custom personalities:

```bash
elif echo "$file" | grep -qiE "\.rs$"; then
  personality="(◕ᴗ◕) Rust Developer"
elif echo "$file" | grep -qiE "\.go$"; then
  personality="(⌐■_■) Gopher"
```

### Adjust Frustration Levels

Modify error thresholds in `personalities_track_activity.sh`:

```bash
if (( errors >= 10 )); then
  personality="【╯°□°】╯︵ ┻━┻ VOLCANIC RAGE"
elif (( errors >= 5 )); then
  personality="(╯°□°)╯︵ ┻━┻ Table Flipper"
```

## Uninstall

### Using Homebrew

```bash
claude-personalities-setup --uninstall
brew uninstall claude-code-personalities
```

### Manual Uninstall

```bash
# Remove personality files
rm ~/.claude/statusline.sh
rm -rf ~/.claude/hooks/

# Restore original settings if backed up
mv ~/.claude/settings.json.backup.* ~/.claude/settings.json
```

## Technical Details

For implementation details, architecture, and complete personality list, see [CLAUDE.md](CLAUDE.md).

## Contributing

We welcome contributions! To add new personalities:

1. Fork the repository
2. Add personalities to `hooks/personalities_track_activity.sh`
3. Test with various Claude Code activities
4. Submit a PR with description

## Support

- **Issues**: [GitHub Issues](https://github.com/Mehdi-Hp/claude-code-personalities/issues)
- **Documentation**: [CLAUDE.md](CLAUDE.md)
- **Claude Code**: [Official Docs](https://docs.anthropic.com/en/docs/claude-code)

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

Created for Claude Code by the community. Special thanks to:
- Anthropic for Claude Code
- Nerd Fonts project for icons
- jq for JSON processing

---

*Enhance your Claude Code experience with dynamic personalities!*