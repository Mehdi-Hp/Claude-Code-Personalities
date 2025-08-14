class ClaudeCodePersonalities < Formula
  desc "Dynamic text-face personalities for Claude Code's statusline"
  homepage "https://github.com/Mehdi-Hp/claude-code-personalities"
  url "https://github.com/Mehdi-Hp/claude-code-personalities/archive/v1.0.3.tar.gz"
  sha256 "e55aee51cc109ac77f62fd5f13a2675a3634f419903dd6adf144daf77ce61d41"  # Will be updated by release script
  license "WTFPL"
  version "1.0.3"

  depends_on "jq"

  def install
    # Install scripts
    (share/"claude-code-personalities").install "scripts/statusline.sh"
    
    # Install version file
    (share/"claude-code-personalities").install ".version"
    
    # Install hooks
    (share/"claude-code-personalities/hooks").install Dir["hooks/*.sh"]
    
    # Install setup utility
    bin.install "claude-code-personalities"
    
    # Make scripts executable
    chmod 0755, share/"claude-code-personalities/statusline.sh"
    chmod 0755, Dir[share/"claude-code-personalities/hooks/*.sh"]
  end

  def caveats
    <<~EOS
      Claude Code Personalities installed!

      To set up personalities, run:
        claude-code-personalities --install

      To uninstall and restore original settings:
        claude-code-personalities --uninstall
        brew uninstall claude-code-personalities

      Requirements:
        - Claude Code v1.0.60 or higher
        - Nerd Fonts (optional): brew install --cask font-hack-nerd-font

      For help:
        claude-code-personalities --help
    EOS
  end

  test do
    # Test that jq dependency works
    assert_match "1.1.0", shell_output("echo '{\"version\":\"1.1.0\"}' | jq -r .version")
    
    # Test that the setup utility exists
    assert_predicate bin/"claude-code-personalities", :exist?
  end
end