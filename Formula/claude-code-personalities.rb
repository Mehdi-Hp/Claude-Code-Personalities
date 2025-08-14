class ClaudeCodePersonalities < Formula
  desc "Dynamic text-face personalities for Claude Code's statusline"
  homepage "https://github.com/Mehdi-Hp/claude-code-personalities"
  url "https://github.com/Mehdi-Hp/claude-code-personalities/archive/v1.3.5.tar.gz"
  sha256 "59f8ad235e9f8b312f4a3cc64ff81d39511593ca3e7699e70e1d693b0fb389a7"
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

  def caveats
    <<~EOS
      Claude Code Personalities installed!

      To set up personalities, run:
        claude-personalities-setup --install

      To uninstall and restore original settings:
        claude-personalities-setup --uninstall
        brew uninstall claude-code-personalities

      Requirements:
        - Claude Code v1.0.60 or higher
        - Nerd Fonts (optional): brew install --cask font-hack-nerd-font

      For help:
        claude-personalities-setup --help
    EOS
  end

  test do
    # Test that jq dependency works
    assert_match "1.1.0", shell_output("echo '{\"version\":\"1.1.0\"}' | jq -r .version")
    
    # Test that the setup utility exists
    assert_predicate bin/"claude-personalities-setup", :exist?
  end
end