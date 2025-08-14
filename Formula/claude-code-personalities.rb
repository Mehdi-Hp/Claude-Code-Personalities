class ClaudeCodePersonalities < Formula
  desc "Dynamic text-face personalities for Claude Code's statusline"
  homepage "https://github.com/Mehdi-Hp/claude-code-personalities"
  url "https://github.com/Mehdi-Hp/claude-code-personalities/archive/v1.2.0.tar.gz"
  sha256 "ccf59e67a72387b1ca4b83409afa15682d61d44eb1dc4d62bffa011cdac5553b"
  license "WTFPL"

  depends_on "jq"

  def install
    # Install scripts
    (share/"claude-code-personalities").install "scripts/statusline.sh"
    
    # Install hooks
    (share/"claude-code-personalities/hooks").install Dir["hooks/*.sh"]
    
    # Install setup utility
    bin.install "claude-personalities-setup"
    
    # Make scripts executable
    chmod 0755, share/"claude-code-personalities/statusline.sh"
    chmod 0755, Dir[share/"claude-code-personalities/hooks/*.sh"]
    
    # Create persistent uninstall script
    (etc/"claude-code-personalities").mkpath
    (etc/"claude-code-personalities/uninstall.sh").write uninstall_script
    (etc/"claude-code-personalities/uninstall.sh").chmod 0755
  end

  def post_install
    ohai "Starting Claude Code Personalities setup..."
    system bin/"claude-personalities-setup", "--install"
  end

  def caveats
    <<~EOS
      Installation complete! Claude Code has been configured with personalities.

      To uninstall and restore original settings:
        brew uninstall claude-code-personalities
        (Settings will be automatically restored)

      Requirements:
        - Claude Code v1.0.60 or higher
        - Nerd Fonts (optional): brew install --cask font-hack-nerd-font
    EOS
  end

  def post_uninstall
    # Run uninstall script if it exists
    uninstall_script_path = "#{HOMEBREW_PREFIX}/etc/claude-code-personalities/uninstall.sh"
    system uninstall_script_path if File.exist?(uninstall_script_path)
  end

  test do
    # Test that jq dependency works
    assert_match "1.1.0", shell_output("echo '{\"version\":\"1.1.0\"}' | jq -r .version")
    
    # Test that the setup utility exists
    assert_predicate bin/"claude-personalities-setup", :exist?
  end

  private

  def uninstall_script
    <<~BASH
      #!/bin/bash
      # Claude Code Personalities Uninstall Script
      # Auto-restores backups when uninstalling via brew
      
      CLAUDE_DIR="$HOME/.claude"
      HOOKS_DIR="$CLAUDE_DIR/hooks"
      
      echo "Restoring original Claude Code settings..."
      
      # Restore statusline.sh from backup
      if ls "$CLAUDE_DIR"/statusline.sh.backup.* 1> /dev/null 2>&1; then
        LATEST_BACKUP=$(ls -t "$CLAUDE_DIR"/statusline.sh.backup.* 2>/dev/null | head -1)
        if [ -f "$LATEST_BACKUP" ]; then
          mv "$LATEST_BACKUP" "$CLAUDE_DIR/statusline.sh"
          echo "$(printf '\\xef\\x80\\x8c') Restored statusline.sh"
        fi
      fi
      
      # Restore settings.json from backup
      if ls "$CLAUDE_DIR"/settings.json.backup.* 1> /dev/null 2>&1; then
        LATEST_BACKUP=$(ls -t "$CLAUDE_DIR"/settings.json.backup.* 2>/dev/null | head -1)
        if [ -f "$LATEST_BACKUP" ]; then
          mv "$LATEST_BACKUP" "$CLAUDE_DIR/settings.json"
          echo "$(printf '\\xef\\x80\\x8c') Restored settings.json"
        fi
      fi
      
      # Remove personality hooks
      rm -f "$HOOKS_DIR"/personalities_*.sh 2>/dev/null
      
      # Clean up any remaining backup files
      rm -f "$CLAUDE_DIR"/*.backup.* 2>/dev/null
      
      # Remove the uninstall script itself
      rm -rf "#{HOMEBREW_PREFIX}/etc/claude-code-personalities" 2>/dev/null
      
      echo "$(printf '\\xef\\x80\\x8c') Claude Code settings restored to original state"
    BASH
  end
end