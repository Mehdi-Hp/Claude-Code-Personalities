#!/bin/bash

# Cross-compilation script for Claude Code Personalities
set -euo pipefail

echo "🚀 Cross-compiling for all platforms..."

# Clean build dir
rm -rf build
mkdir -p build

# Get version
cd claude-code-personalities-rust
VERSION=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.name == "claude-code-personalities") | .version')
echo "Version: $VERSION"

# Targets to build (macOS for local testing, GitHub Actions will build Linux)
TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

# Add Linux targets if Docker is available OR if in CI environment
if command -v docker &>/dev/null && docker version &>/dev/null; then
    TARGETS+=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
    )
    echo "Docker detected - building for all platforms"
elif [[ "${CI:-false}" == "true" ]] || [[ "${GITHUB_ACTIONS:-false}" == "true" ]]; then
    TARGETS+=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
    )
    echo "CI environment detected - building for all platforms"
else
    echo "Local environment without Docker - building macOS binaries only"
fi

# Install cross if needed for Linux targets
if [[ " ${TARGETS[*]} " =~ "linux" ]]; then
    command -v cross >/dev/null || cargo install cross --git https://github.com/cross-rs/cross
fi

# Add all targets
for target in "${TARGETS[@]}"; do
    rustup target add $target 2>/dev/null || true
done

# Build each target
for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    
    if [[ "$target" == *"linux"* ]]; then
        # Use cross for all Linux targets
        cross build --release --target $target
    else
        # Use regular cargo for macOS targets
        cargo build --release --target $target
    fi
    
    # Copy to build dir with proper naming
    platform_name=$(echo $target | sed 's/unknown-//g' | sed 's/gnu//g' | sed 's/-$//')
    cp target/$target/release/claude-code-personalities ../build/claude-code-personalities-$platform_name
    chmod +x ../build/claude-code-personalities-$platform_name
    
    # Show file size
    size=$(du -h ../build/claude-code-personalities-$platform_name | cut -f1)
    echo "✅ $platform_name ($size)"
done

cd ..
echo ""
echo "✅ All binaries built in build/"
ls -la build/