# Claude Code Personalities

> Dynamic text-face personalities for Claude Code's statusline that change based on what Claude is doing

## Important Rules

- **NO EMOJIS**: Never use emojis in any files or output
- **Nerd Font Icons Only**: If icons are needed, use Nerd Font UTF-8 byte sequences only (e.g., `\u{f07b}` for folder icon)

## What is This?

Claude Code Personalities is a personality system that gives Claude Code a dynamic, context-aware statusline with text-face emoticons that change based on Claude's current activity. Instead of a static prompt, you get a fun, informative statusline that shows Claude's "mood" and what it's currently working on.

**Rust Implementation**: The project is implemented in pure Rust for lightning-fast performance, better error handling, and zero external dependencies. No shell scripts needed!

### Features

- **30+ Text-Face Personalities**: From `ʕ•ᴥ•ʔ Code Wizard` to `(┛ಠДಠ)┛彡┻━┻ Frustrated Developer`
- **Context-Aware**: Personalities change based on files being edited, commands run, and errors encountered
- **Interactive Configuration**: Use `config` command to customize what appears in statusline
- **Activity Tracking**: Monitors Claude's tool usage (Edit, Bash, Grep, etc.) via hooks
- **Error State Management**: Claude gets progressively more frustrated with errors
- **Nerd Font Icons**: Visual indicators for folders, activities, and status
- **Session Persistence**: Maintains personality state across a Claude Code session
- **Model-Specific Indicators**: Different icons for Opus, Sonnet, and Haiku

## How It Works

```mermaid
graph LR
    A[Claude Code] -->|JSON Input| B[claude-code-personalities --statusline]
    A -->|Hook Events| C[claude-code-personalities --hook]
    C -->|Updates State| D[/tmp/claude_session_*.json]
    D -->|Reads State| B
    B -->|Displays| E[Terminal Statusline]
```

1. **Claude Code** calls the binary with `--statusline` and passes JSON input containing session and workspace info
2. **Hook system** intercepts tool usage (PreToolUse/PostToolUse) and calls binary with `--hook` to track Claude's activities
3. **State management** updates personality and activity based on context (files, tools, errors)
4. **Persistent state** maintained in `/tmp/claude_session_*.json` files across the session
5. **Statusline display** shows personality, current directory, activity, and model

## Architecture

### Pure Rust Binary

The entire system is contained in a single Rust binary that operates in different modes:

```
claude-code-personalities
├── --statusline          # Generate statusline output (called by Claude Code)
├── --hook <type>         # Handle hook events (pre-tool, post-tool, session-end)
├── install               # Install and configure Claude Code
├── config                # Interactive configuration menu
├── status                # Check installation status
├── update                # Update to latest version
└── uninstall             # Remove personalities
```

### File Structure

```
~/.claude/
├── claude-code-personalities           # Main Rust binary
├── settings.json                       # Claude Code configuration (modified by installer)
└── personalities_config.json          # User preferences (created by config command)

/tmp/
└── claude_session_<session_id>.json    # Session state (personality, activity, error count)
```

### Configuration in settings.json

The installer modifies Claude Code's `~/.claude/settings.json` to add:

```json
{
  "statusLine": {
    "type": "command", 
    "command": "/Users/user/.claude/claude-code-personalities",
    "args": ["--statusline"],
    "padding": 0
  },
  "hooks": {
    "PreToolUse": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "/Users/user/.claude/claude-code-personalities",
        "args": ["--hook", "pre-tool"]
      }]
    }],
    "PostToolUse": [{
      "matcher": "*", 
      "hooks": [{
        "type": "command",
        "command": "/Users/user/.claude/claude-code-personalities",
        "args": ["--hook", "post-tool"]
      }]
    }],
    "Stop": [{
      "hooks": [{
        "type": "command",
        "command": "/Users/user/.claude/claude-code-personalities",
        "args": ["--hook", "session-end"]
      }]
    }]
  }
}
```

## How Personalities Work

### Personality Assignment Logic

The Rust binary analyzes tool usage and context to determine personalities:

```rust
// Error-based frustration (highest priority)
if error_count >= 5 { "(╯°□°)╯︵ ┻━┻ Table Flipper" }
else if error_count >= 3 { "(┛ಠДಠ)┛彡┻━┻ Frustrated Developer" }

// Tool-based personalities  
else if tool == "Bash" && args.contains("git") { "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager" }
else if tool == "Grep" || activity == "searching" { "( ͡° ͜ʖ ͡°) Search Detective" }
else if tool == "Edit" && file.ends_with(".md") { "(͡• ͜໒ ͡• ) Documentation Writer" }
else if tool == "Edit" && file.ends_with(".rs") { "\ue7a8 Rust Developer" }

// Activity-based fallbacks
else if activity == "editing" { "ʕ•ᴥ•ʔ Code Wizard" }
else if activity == "reading" { "(⌐■_■) File Inspector" }
else { "(｡◕‿◕｡) Helpful Assistant" }
```

### Session State Format

State is stored in `/tmp/claude_session_<session_id>.json`:

```json
{
  "session_id": "abc-123",
  "activity": "editing", 
  "personality": "ʕ•ᴥ•ʔ Code Wizard",
  "consecutive_actions": 5,
  "error_count": 0,
  "last_updated": "2024-08-27T10:30:00Z"
}
```

## Installation Process

The `claude-code-personalities install` command:

1. **Copies binary** to `~/.claude/claude-code-personalities`
2. **Backs up** existing `settings.json` (timestamped)
3. **Merges configuration** into `settings.json` (preserves existing hooks)
4. **Creates default config** at `~/.claude/personalities_config.json`
5. **Validates setup** by testing statusline and hooks

## Configuration

### Interactive Configuration

```bash
claude-code-personalities config
```

Opens a multi-select menu to toggle:
- Show Personality (text faces)
- Show Activity (current action)
- Show Current File
- Show Current Directory  
- Show Model Indicator
- Show Error Indicators
- Use Nerd Font Icons
- Use ANSI Colors
- Theme Selection

Settings saved to `~/.claude/personalities_config.json`:

```json
{
  "show_personality": true,
  "show_activity": true, 
  "show_current_file": false,
  "show_directory": true,
  "show_model": true,
  "show_errors": true,
  "use_icons": true,
  "use_colors": true,
  "theme": "Dark"
}
```

### Custom Personalities

Personalities are defined in Rust code (`src/statusline/personality.rs`). To add custom ones, modify the personality determination logic and rebuild:

```rust
else if file_path.ends_with(".py") {
    "\ue73c Python Developer".to_string()
}
```

## Nerd Font Icons Reference

Icons are defined using UTF-8 sequences:

| UTF-8 Bytes    | Unicode | Description     |
| -------------- | ------- | --------------- |
| `\xef\x81\xbb` | U+F07B  | Folder          |
| `\xef\x84\xa1` | U+F121  | Code            |
| `\xef\x86\x88` | U+F188  | Bug             |
| `\xef\x80\x82` | U+F002  | Search          |
| `\xef\x81\x84` | U+F044  | Edit            |
| `\xef\x83\xa7` | U+F0E7  | Lightning/Run   |
| `\xef\x81\xae` | U+F06E  | Eye/Review      |
| `\xef\x83\xab` | U+F0EB  | Lightbulb/Think |
| `\xef\x84\xb5` | U+F135  | Rocket          |
| `\xef\x81\xb1` | U+F071  | Warning         |
| `\xef\x81\x97` | U+F057  | Error           |
| `\xef\x81\xad` | U+F06D  | Fire            |
| `\xef\x80\x93` | U+F013  | Gear            |
| `\xef\x84\xa0` | U+F120  | Terminal        |

## Testing

### Test Statusline Generation
```bash
# Simulate Claude Code input
echo '{"session_id":"test","model":{"display_name":"Opus"},"workspace":{"current_dir":"/project"}}' | claude-code-personalities --statusline
```

### Test Hook Processing
```bash  
# Simulate tool usage hook
echo '{"session_id":"test","tool_name":"Edit","tool_input":{"file_path":"main.rs"}}' | claude-code-personalities --hook pre-tool
```

### Debug Mode
```bash
# Check what's in the state file
cat /tmp/claude_session_test.json

# Check installation status
claude-code-personalities status
```

## Troubleshooting

### Icons Not Displaying
1. Install Nerd Fonts: `brew install --cask font-hack-nerd-font`
2. Set terminal font to a Nerd Font
3. Test: `printf '\xef\x81\xbb'` should show folder icon

### Personality Not Changing
1. Check installation: `claude-code-personalities status`
2. Verify hooks in `~/.claude/settings.json`
3. Check state file exists: `ls /tmp/claude_session_*.json`
4. Test hook manually with echo command above

### Always Shows Default Personality
- State file isn't being created/updated
- Check that hooks are configured correctly
- Ensure session_id is being passed from Claude Code

### Update Issues
1. Check binary location: `which claude-code-personalities`
2. Run status check: `claude-code-personalities status`
3. Reinstall if needed: `claude-code-personalities install --force`

### Performance Issues
The Rust implementation is designed for speed:
- Binary startup: ~1ms
- State file I/O: ~0.1ms  
- Personality calculation: ~0.01ms
- Total statusline generation: <2ms

If experiencing slowness, check:
- Disk space in `/tmp`
- File permissions on state files
- Multiple concurrent hook executions

## Development

### Building from Source
```bash
git clone https://github.com/Mehdi-Hp/claude-code-personalities
cd claude-code-personalities
cargo build --release

# Binary at target/release/claude-code-personalities
```

### Adding New Personalities
1. Edit `src/statusline/personality.rs`
2. Add new personality patterns to `determine_personality()`
3. Add corresponding kaomoji to `src/kaomoji/`
4. Rebuild and test

### Contributing
- Follow existing code patterns
- Add tests for new functionality
- Update documentation
- Use `cargo fmt` and `cargo clippy`

## Architecture Benefits

The pure Rust implementation provides:

- **Performance**: 10-100x faster than shell scripts
- **Reliability**: Better error handling and recovery
- **Maintainability**: Single codebase vs scattered scripts
- **Portability**: Same binary works across platforms
- **Security**: No shell injection vulnerabilities
- **Dependencies**: Zero external runtime dependencies

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.