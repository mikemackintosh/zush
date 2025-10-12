#!/bin/bash
set -e

VERSION=${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')}

echo "=========================================="
echo "Creating Release Archives"
echo "Version: $VERSION"
echo "=========================================="
echo ""

# Create release directory
RELEASE_DIR="releases/v${VERSION}"
mkdir -p "$RELEASE_DIR"

# Function to create archive for a binary
create_archive() {
    local binary=$1
    local platform=$2
    
    if [ -f "dist/$binary" ]; then
        echo "Creating archive for $platform..."
        
        # Create temporary directory with contents
        local temp_dir="temp_release"
        mkdir -p "$temp_dir"
        
        # Copy binary
        cp "dist/$binary" "$temp_dir/zush-prompt"
        chmod +x "$temp_dir/zush-prompt"
        
        # Copy themes
        mkdir -p "$temp_dir/themes"
        cp themes/*.toml "$temp_dir/themes/" 2>/dev/null || true
        
        # Copy documentation
        cp README.md "$temp_dir/"
        cp CONFIGURATION.md "$temp_dir/" 2>/dev/null || true
        cp QUICK_START.md "$temp_dir/" 2>/dev/null || true
        cp config.example.toml "$temp_dir/" 2>/dev/null || true
        
        # Create installation script
        cat > "$temp_dir/install.sh" << 'INSTALL_EOF'
#!/bin/bash
set -e

echo "Installing Zush..."

# Install binary
mkdir -p ~/.local/bin
cp zush-prompt ~/.local/bin/
chmod +x ~/.local/bin/zush-prompt

# Install themes
mkdir -p ~/.config/zush/themes
cp themes/*.toml ~/.config/zush/themes/

# Generate integration script
~/.local/bin/zush-prompt init zsh > ~/.config/zush/zush.zsh

echo ""
echo "✓ Zush installed successfully!"
echo ""
echo "Add this to your ~/.zshrc:"
echo "  source ~/.config/zush/zush.zsh"
echo ""
echo "Then reload your shell:"
echo "  source ~/.zshrc"
echo ""
INSTALL_EOF
        chmod +x "$temp_dir/install.sh"
        
        # Create archive
        local archive_name="zush-prompt-${VERSION}-${platform}.tar.gz"
        tar -czf "$RELEASE_DIR/$archive_name" -C "$temp_dir" .
        
        # Cleanup
        rm -rf "$temp_dir"
        
        echo "✓ Created: $RELEASE_DIR/$archive_name"
    else
        echo "⚠ Skipping $platform (binary not found: dist/$binary)"
    fi
}

# Create archives for each platform
create_archive "zush-prompt-macos-universal" "macos-universal"
create_archive "zush-prompt-macos-arm64" "macos-arm64"
create_archive "zush-prompt-macos-x86_64" "macos-x86_64"
create_archive "zush-prompt-linux-x86_64" "linux-x86_64"
create_archive "zush-prompt-linux-aarch64" "linux-aarch64"
create_archive "zush-prompt-linux-x86_64-static" "linux-x86_64-static"

# Create checksums for release
echo ""
echo "Creating checksums for release..."
cd "$RELEASE_DIR"
shasum -a 256 *.tar.gz > SHA256SUMS
cd - > /dev/null

echo ""
echo "=========================================="
echo "Release Summary"
echo "=========================================="
echo ""
ls -lh "$RELEASE_DIR"
echo ""
echo "Release archives created in: $RELEASE_DIR"
echo ""
echo "To verify checksums:"
echo "  cd $RELEASE_DIR && shasum -c SHA256SUMS"
echo ""
echo "To create a GitHub release:"
echo "  gh release create v${VERSION} $RELEASE_DIR/*.tar.gz $RELEASE_DIR/SHA256SUMS"
echo ""
