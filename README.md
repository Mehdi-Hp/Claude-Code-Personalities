# ૮ ․ ․ ྀིა Claude Code Personalities

> Dynamic text-face personalities for Claude Code's statusline

Give your Claude Code assistant a dynamic, context-aware statusline with 30+ text-face emoticons that change based on Claude's current activity.  
Watch Claude transform from `ʕ•ᴥ•ʔ Code Wizard` when coding to `(┛ಠДಠ)┛彡┻━┻ Frustrated Developer` when encountering errors!

![Claude Code Personalities Screenshot](screenshot.png)

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash
```

### Homebrew

```bash
# Option 1: Direct install
brew install Mehdi-Hp/claude-code-personalities/claude-code-personalities

# Option 2: Via tap
brew tap Mehdi-Hp/claude-code-personalities
brew install claude-code-personalities

# Then run setup
claude-code-personalities --install
```

## Uninstallation

To remove personalities and restore original settings:

```bash
# Restore original Claude Code settings
claude-code-personalities --uninstall

# Remove from Homebrew
brew uninstall claude-code-personalities
```

## Requirements

- Claude Code v1.0.60+
- jq (`brew install jq`)
- Nerd Fonts for icons (`brew install --cask font-hack-nerd-font`)

## Documentation

For detailed information, customization options, and the complete personality list, see [CLAUDE.md](CLAUDE.md).

## License

WTFPL - Do what you want with it.