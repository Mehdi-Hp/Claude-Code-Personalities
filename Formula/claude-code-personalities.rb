class ClaudeCodePersonalities < Formula
  desc "Dynamic text-face personalities for Claude Code's statusline"
  homepage "https://github.com/yourusername/claude-code-personalities"
  url "https://github.com/yourusername/claude-code-personalities/archive/v1.0.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
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
  end

  def post_install
    ohai "Claude Code Personalities installed!"
    ohai "Run 'claude-personalities-setup --install' to configure Claude Code"
  end

  def caveats
    <<~EOS
      To complete the installation, run:
        claude-personalities-setup --install

      This will:
        1. Copy personality files to ~/.claude/
        2. Configure Claude Code settings
        3. Back up any existing configurations

      To uninstall later:
        claude-personalities-setup --uninstall
        brew uninstall claude-code-personalities

      Requirements:
        - Claude Code v1.0.60 or higher
        - Nerd Fonts (optional): brew install --cask font-hack-nerd-font
    EOS
  end

  test do
    # Test that jq dependency works
    assert_match "1.0.0", shell_output("echo '{\"version\":\"1.0.0\"}' | jq -r .version")
    
    # Test that the setup utility exists
    assert_predicate bin/"claude-personalities-setup", :exist?
  end
end