#!/usr/bin/env bash

# Zush Prompt Installation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Installation directories
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/zush}"
ZSH_CONFIG="${ZSH_CONFIG:-$HOME/.zshrc}"

echo -e "${BLUE}🚀 Installing Zush Prompt...${NC}"

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed. Please install Rust first:${NC}"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the project
echo -e "${YELLOW}📦 Building Zush Prompt...${NC}"
cargo build --release

# Create directories
echo -e "${YELLOW}📁 Creating directories...${NC}"
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"

# Copy binary
echo -e "${YELLOW}📋 Installing binary...${NC}"
cp target/release/zush-prompt "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/zush-prompt"

# Copy config
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    echo -e "${YELLOW}📝 Installing default configuration...${NC}"
    cp config.example.toml "$CONFIG_DIR/config.toml"
else
    echo -e "${BLUE}ℹ️  Configuration already exists, skipping...${NC}"
fi

# Add to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}🔧 Adding $INSTALL_DIR to PATH...${NC}"
    echo "" >> "$ZSH_CONFIG"
    echo "# Zush Prompt" >> "$ZSH_CONFIG"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$ZSH_CONFIG"
fi

# Generate Zsh integration
echo -e "${YELLOW}🔗 Generating Zsh integration...${NC}"
"$INSTALL_DIR/zush-prompt" init zsh > "$CONFIG_DIR/zush.zsh"

# Add to .zshrc
if ! grep -q "source.*zush.zsh" "$ZSH_CONFIG" 2>/dev/null; then
    echo -e "${YELLOW}✏️  Adding Zush to .zshrc...${NC}"
    echo "" >> "$ZSH_CONFIG"
    echo "# Zush Prompt Integration" >> "$ZSH_CONFIG"
    echo "source $CONFIG_DIR/zush.zsh" >> "$ZSH_CONFIG"
else
    echo -e "${BLUE}ℹ️  Zush already in .zshrc, skipping...${NC}"
fi

# Success message
echo -e "${GREEN}✅ Zush Prompt installed successfully!${NC}"
echo ""
echo -e "${BLUE}To get started:${NC}"
echo "  1. Edit your configuration: $CONFIG_DIR/config.toml"
echo "  2. Reload your shell: source ~/.zshrc"
echo "  3. Or start a new terminal session"
echo ""
echo -e "${BLUE}Available commands:${NC}"
echo "  zush-prompt config        # Print example configuration"
echo "  zush-prompt init zsh      # Print Zsh integration script"
echo "  zush-prompt --help        # Show help"
echo ""
echo -e "${YELLOW}⚡ Enjoy your new prompt!${NC}"