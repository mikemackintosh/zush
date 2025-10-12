#!/bin/bash
set -e

echo "=========================================="
echo "Building Zush for all platforms"
echo "=========================================="
echo ""

# Create dist directory
mkdir -p dist

# Check if we're on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "✓ Running on macOS, can build macOS targets natively"
    
    # Install targets if not already installed
    echo "Installing macOS targets..."
    rustup target add aarch64-apple-darwin 2>/dev/null || true
    rustup target add x86_64-apple-darwin 2>/dev/null || true
    
    # Build for Apple Silicon
    echo ""
    echo "Building for macOS (Apple Silicon - M1/M2/M3)..."
    cargo build --release --target aarch64-apple-darwin
    cp target/aarch64-apple-darwin/release/zush-prompt dist/zush-prompt-macos-arm64
    echo "✓ Built: dist/zush-prompt-macos-arm64"
    
    # Build for Intel
    echo ""
    echo "Building for macOS (Intel x86_64)..."
    cargo build --release --target x86_64-apple-darwin
    cp target/x86_64-apple-darwin/release/zush-prompt dist/zush-prompt-macos-x86_64
    echo "✓ Built: dist/zush-prompt-macos-x86_64"
    
    # Create universal binary
    echo ""
    echo "Creating universal macOS binary..."
    lipo -create \
        target/x86_64-apple-darwin/release/zush-prompt \
        target/aarch64-apple-darwin/release/zush-prompt \
        -output dist/zush-prompt-macos-universal
    echo "✓ Built: dist/zush-prompt-macos-universal"
    
    # Verify universal binary
    echo ""
    echo "Verifying universal binary:"
    lipo -info dist/zush-prompt-macos-universal
else
    echo "ℹ Running on Linux, building for current architecture only"
    cargo build --release
    cp target/release/zush-prompt dist/zush-prompt-linux-$(uname -m)
    echo "✓ Built: dist/zush-prompt-linux-$(uname -m)"
fi

# Check if cross tool is available for Linux builds
echo ""
if command -v cross &> /dev/null; then
    echo "✓ cross tool found, building Linux targets..."
    
    echo ""
    echo "Building for Linux x86_64..."
    cross build --release --target x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/release/zush-prompt dist/zush-prompt-linux-x86_64
    echo "✓ Built: dist/zush-prompt-linux-x86_64"
    
    echo ""
    echo "Building for Linux ARM64..."
    cross build --release --target aarch64-unknown-linux-gnu
    cp target/aarch64-unknown-linux-gnu/release/zush-prompt dist/zush-prompt-linux-aarch64
    echo "✓ Built: dist/zush-prompt-linux-aarch64"
    
    echo ""
    echo "Building static Linux binary (musl)..."
    cross build --release --target x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-musl/release/zush-prompt dist/zush-prompt-linux-x86_64-static
    echo "✓ Built: dist/zush-prompt-linux-x86_64-static"
else
    echo "⚠ cross tool not found - skipping Linux cross-compilation"
    echo "  Install with: cargo install cross"
    echo "  This allows building Linux binaries from macOS and vice versa"
fi

echo ""
echo "=========================================="
echo "Build Summary"
echo "=========================================="
echo ""
ls -lh dist/
echo ""

# Calculate sizes and create checksums
echo "Creating checksums..."
cd dist
shasum -a 256 zush-prompt-* > SHA256SUMS
cd ..
echo "✓ Created: dist/SHA256SUMS"

echo ""
echo "=========================================="
echo "Build Complete!"
echo "=========================================="
echo ""
echo "Binaries are in the dist/ directory"
echo "To install locally:"
echo "  cp dist/zush-prompt-macos-universal ~/.local/bin/zush-prompt"
echo ""
echo "To create release archives:"
echo "  ./create-release.sh"
echo ""
