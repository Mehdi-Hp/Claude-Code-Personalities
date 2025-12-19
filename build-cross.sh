#!/bin/bash

# Simple local build script for Claude Code Personalities
# For cross-platform releases, use GitHub Actions matrix builds instead
set -euo pipefail

# Nerd Font icons
ICON_ROCKET=$(printf '\xef\x84\xb5')
ICON_CHECK=$(printf '\xef\x91\x84')
ICON_PACKAGE=$(printf '\xef\x91\xb7')

echo "$ICON_ROCKET Building Claude Code Personalities locally..."

# Clean build dir
rm -rf build
mkdir -p build

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')
echo "Version: $VERSION"

# Build for current platform
echo "Building for current platform..."
cargo build --release

# Copy to build dir
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    platform="macos-$(uname -m)"
    binary_name="claude-code-personalities-$platform"
else
    # Linux or other
    platform="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
    binary_name="claude-code-personalities-$platform"
fi

cp target/release/claude-code-personalities build/$binary_name
chmod +x build/$binary_name

# Generate checksum (cross-platform)
cd build
if [[ "$OSTYPE" == "darwin"* ]]; then
    shasum -a 256 $binary_name > $binary_name.sha256
else
    sha256sum $binary_name > $binary_name.sha256
fi

# Show results
size=$(du -h $binary_name | cut -f1)
echo "$ICON_CHECK Built: $binary_name ($size)"
echo ""
echo "$ICON_PACKAGE Local build complete! For multi-platform releases, use:"
echo "   git tag v$VERSION && git push origin v$VERSION"
echo ""
ls -la