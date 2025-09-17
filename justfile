# Claude Code Personalities - Build Automation

# Variables
build_dir := "build"

# Default: show available commands
default:
    @just --list

# Build release binary for current platform
build:
    @printf "Building release binary...\n"
    cargo build --release
    @printf "✅ Built target/release/claude-code-personalities\n"

# Build for all platforms
build-all:
    #!/usr/bin/env bash
    set -euo pipefail
    # Check dependencies
    command -v rustc >/dev/null || (echo "❌ Rust not installed" && exit 1)
    command -v cargo >/dev/null || (echo "❌ Cargo not installed" && exit 1)
    command -v jq >/dev/null || (echo "❌ jq not installed" && exit 1)
    echo "✅ Dependencies verified"
    # Run build script
    ./build-cross.sh

# Run tests
test:
    @printf "Running tests...\n"
    cargo test
    @printf "✅ All tests passed\n"

# Run clippy linter
lint:
    @printf "Running clippy...\n"
    cargo clippy -- -D warnings
    @printf "✅ No lint issues\n"

# Clean build artifacts
clean:
    @printf "Cleaning build artifacts...\n"
    cargo clean
    rm -rf {{build_dir}}
    @printf "✅ Cleaned\n"

# Debug build with logging
debug:
    @printf "Debug build with logging...\n"
    RUST_LOG=debug cargo run -- --statusline

# Full release workflow
release version:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "🚀 Starting release v{{version}}"

    # Update version in Cargo.toml
    echo "📝 Updating version to {{version}}"
    sed -i '' 's/^version = .*/version = "{{version}}"/' Cargo.toml
    echo "{{version}}" > .version

    # Build for all platforms
    echo "🔨 Building for all platforms..."
    ./build-cross.sh

    # Commit version bump
    echo "📦 Committing version bump..."
    git add .
    git commit -m "chore: bump version to {{version}}" || true

    # Handle existing tag
    VERSION={{version}}
    if git rev-parse "v$VERSION" >/dev/null 2>&1; then
        echo "⚠️  Tag v$VERSION already exists locally. Deleting and recreating..."
        git tag -d "v$VERSION"
    fi

    # Create and push tag
    echo "🏷️  Creating tag v$VERSION..."
    git tag -a "v$VERSION" -m "Release v$VERSION"

    # Push to origin
    echo "⬆️  Pushing to GitHub..."
    git push origin main
    git push origin "v$VERSION"

    echo "✅ Release v{{version}} complete!"
    echo ""
    echo "GitHub Actions will build and publish the release automatically."