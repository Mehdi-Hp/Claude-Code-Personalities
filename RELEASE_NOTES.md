# Release v1.1.0 - Enhanced Installer & Visual Improvements

## 🎉 What's New

### Enhanced Installer Experience
- **Beautiful Visual Design**: New bordered UI with progress tracking
- **Step-by-Step Installation**: Clear 6-step process with visual indicators
- **Smart Dependency Checking**: Automatic detection of Claude Code and jq
- **Intelligent Backups**: Timestamped backups of existing configurations
- **Icon Testing**: Built-in Nerd Font verification

### Improved Statusline Display
- **Proper Model Display**: Shows capitalized model names with versions (e.g., "[✴︎ Opus 4.1]")
- **Smart Path Trimming**: Long file paths are intelligently truncated to show only relevant parts
- **Command Simplification**: Shows just command names instead of full commands
- **Consistent Capitalization**: All activities now properly capitalized (Starting, Editing, etc.)

### Visual Refinements
- **North Star Icon**: New model indicator using Nerd Font's North Star (✴︎)
- **Simplified Output**: Removed unnecessary decorative elements
- **Dim Gray Dividers**: Subtle visual separation in installer
- **Clean Completion Screen**: Minimal, focused next steps

## 📦 Installation

### One-Line Install
```bash
curl -fsSL https://raw.githubusercontent.com/Mehdi-Hp/claude-code-personalities/main/install.sh | bash
```

### Homebrew (Coming Soon)
```bash
brew tap Mehdi-Hp/claude-code
brew install claude-code-personalities
```

## 🔧 Release Checklist

### Before Creating GitHub Release
- [x] Update version to 1.1.0 in all files
- [x] Replace placeholder GitHub username with Mehdi-Hp
- [x] Verify installer works correctly
- [x] Test statusline display
- [x] Update documentation

### Creating the Release
1. **Create Git Tag**:
   ```bash
   git tag -a v1.1.0 -m "Release v1.1.0 - Enhanced Installer & Visual Improvements"
   git push origin v1.1.0
   ```

2. **Create GitHub Release**:
   - Go to https://github.com/Mehdi-Hp/claude-code-personalities/releases/new
   - Choose tag: v1.1.0
   - Release title: "v1.1.0 - Enhanced Installer & Visual Improvements"
   - Copy this release notes content
   - Publish release

3. **Update Homebrew Formula**:
   After creating the release, calculate SHA256:
   ```bash
   curl -L https://github.com/Mehdi-Hp/claude-code-personalities/archive/v1.1.0.tar.gz | shasum -a 256
   ```
   Then update the formula with the actual SHA256 hash.

## 🐛 Bug Fixes
- Fixed model display showing lowercase without version
- Fixed path overflow in statusline
- Fixed command clutter in activity display
- Fixed inconsistent capitalization throughout

## 📝 Notes
- Requires Claude Code v1.0.60+
- Requires jq for JSON processing
- Nerd Fonts recommended for proper icon display

## 🙏 Credits
Created for Claude Code by Mehdi-Hp