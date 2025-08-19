# Claude Code Personalities - Build Automation with Just

# Variables
rust_dir := "claude-code-personalities-rust"
build_dir := "build"

# Default recipe
default: help

# Show available tasks
help:
    @printf " Claude Code Personalities - Build System\n"
    @echo ""
    @just --list

# Check dependencies
check-deps:
    @printf "$(printf '\xef\x80\x93') Checking dependencies...\n"
    @command -v rustc >/dev/null || (printf "$(printf '\xef\x81\x97') Rust not installed\n" && exit 1)
    @command -v cargo >/dev/null || (printf "$(printf '\xef\x81\x97') Cargo not installed\n" && exit 1) 
    @command -v jq >/dev/null || (printf "$(printf '\xef\x81\x97') jq not installed\n" && exit 1)
    @printf "$(printf '\xef\x80\x8c') All dependencies available\n"

# Development build and run
dev:
    @printf "$(printf '\xef\x84\xb5') Development build\n"
    cd {{rust_dir}} && cargo run -- --statusline

# Build release binary for current platform  
build:
    @printf "$(printf '\xef\x84\xb5') Building release binary\n"
    cd {{rust_dir}} && cargo build --release
    @printf "$(printf '\xef\x80\x8c') Built target/release/claude-code-personalities\n"

# Build for all platforms using a script
build-all: check-deps
    ./build-cross.sh

# Run tests
test:
    @printf "$(printf '\xef\x80\x93') Running tests...\n"
    cd {{rust_dir}} && cargo test
    @printf "$(printf '\xef\x80\x8c') All tests passed\n"

# Format code
fmt:
    @printf "$(printf '\xef\x80\x93') Formatting code...\n"
    cd {{rust_dir}} && cargo fmt
    @printf "$(printf '\xef\x80\x8c') Code formatted\n"

# Run clippy
lint:
    @printf "$(printf '\xef\x80\x93') Running clippy...\n"
    cd {{rust_dir}} && cargo clippy -- -D warnings
    @printf "$(printf '\xef\x80\x8c') No lint issues\n"

# Run tests and lints
check: test lint
    @printf "$(printf '\xef\x80\x8c') All checks passed\n"

# Clean build artifacts
clean:
    @printf "$(printf '\xef\x80\x93') Cleaning...\n"
    cd {{rust_dir}} && cargo clean
    rm -rf {{build_dir}}
    @printf "$(printf '\xef\x80\x8c') Cleaned\n"

# Install binary locally
install: build
    @printf "$(printf '\xef\x84\xb5') Installing to ~/.local/bin\n"
    @mkdir -p ~/.local/bin
    @cp {{rust_dir}}/target/release/claude-code-personalities ~/.local/bin/
    @chmod +x ~/.local/bin/claude-code-personalities
    @printf "$(printf '\xef\x80\x8c') Installed to ~/.local/bin/claude-code-personalities\n"

# Test statusline output
statusline: build
    @printf "$(printf '\xef\x80\x93') Testing statusline...\n"
    @echo '{"model":{"display_name":"Opus"},"workspace":{"current_dir":"'$(pwd)'"}}' | {{rust_dir}}/target/release/claude-code-personalities --statusline

# Test hook functionality  
test-hook: build
    @printf "$(printf '\xef\x80\x93') Testing hook...\n"
    @echo '{"session_id":"test123","tool_name":"Edit","tool_input":{"file_path":"test.js"}}' | {{rust_dir}}/target/release/claude-code-personalities --hook pre-tool
    @printf "$(printf '\xef\x80\x8c') Hook test completed\n"

# Bump version
bump-version version:
    @printf "$(printf '\xef\x80\x93') Bumping version to {{version}}\n"
    @sed -i '' 's/^version = .*/version = "{{version}}"/' {{rust_dir}}/Cargo.toml
    @echo "{{version}}" > .version
    @printf "$(printf '\xef\x80\x8c') Version updated to {{version}}\n"

# Create git tag
tag-release:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(cat .version)
    echo "🚀 Creating release tag v$VERSION"
    git add .
    git commit -m "chore: bump version to $VERSION" || true
    git tag -a "v$VERSION" -m "Release v$VERSION"
    git push origin main
    git push origin "v$VERSION"
    echo "✅ Tagged and pushed v$VERSION"

# Full release workflow
release version: (bump-version version) build-all tag-release
    @echo "✅ Release v{{version}} complete!"
    @echo ""
    @echo "GitHub Actions will build and publish automatically."

# Development server with file watching
watch:
    @echo "⚙️ Watching for changes..."
    cd {{rust_dir}} && cargo watch -x "run -- --statusline"

# Debug build and run
debug:
    @echo "⚙️ Debug build with logging..."
    cd {{rust_dir}} && RUST_LOG=debug cargo run -- --statusline