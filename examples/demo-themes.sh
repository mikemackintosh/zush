#!/bin/bash
# Quick demo script to showcase Zush themes
# Usage: ./demo-themes.sh

set -e

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║          Zush Themes Installation Demo              ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

# Check if zush-prompt is available
if ! command -v zush-prompt &> /dev/null; then
    echo "❌ Error: zush-prompt not found in PATH"
    echo "Please install zush-prompt first"
    exit 1
fi

# Check if themes directory exists
THEMES_DIR="$HOME/.config/zush/themes"
if [ ! -d "$THEMES_DIR" ]; then
    echo "📁 Creating themes directory: $THEMES_DIR"
    mkdir -p "$THEMES_DIR"
fi

# Copy example themes
echo "📦 Installing example themes..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

theme_count=0
for theme_file in "$SCRIPT_DIR/themes"/*.toml; do
    if [ -f "$theme_file" ]; then
        theme_name=$(basename "$theme_file")
        cp "$theme_file" "$THEMES_DIR/"
        echo "  ✓ Installed: $theme_name"
        ((theme_count++))
    fi
done

echo ""
echo "✅ Successfully installed $theme_count themes!"
echo ""

# Show available themes
echo "╔══════════════════════════════════════════════════════╗"
echo "║              Available Themes                        ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

for theme_file in "$THEMES_DIR"/*.toml; do
    if [ -f "$theme_file" ]; then
        name=$(basename "$theme_file" .toml)
        description=$(grep '^description' "$theme_file" 2>/dev/null | sed 's/description = "\(.*\)"/\1/' || echo "")
        echo "  • $name"
        if [ -n "$description" ]; then
            echo "    $description"
        fi
    fi
done

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║                 Next Steps                           ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
echo "1. Add to your ~/.zshrc:"
echo "   source $SCRIPT_DIR/../zush-prompt-rust/zush-theme.zsh"
echo "   source <(zush-prompt init zsh)"
echo ""
echo "2. Reload your shell:"
echo "   source ~/.zshrc"
echo ""
echo "3. Try the themes:"
echo "   zush-theme list              # List all themes"
echo "   zush-theme list --preview    # List with previews"
echo "   zush-theme preview           # Full preview of all themes"
echo "   zush-theme minimal           # Switch to minimal theme"
echo "   zush-theme dcs               # Switch to DCS theme"
echo ""
echo "4. Quick aliases:"
echo "   zt list                      # Quick list"
echo "   zt minimal                   # Quick switch"
echo ""
echo "📚 Documentation:"
echo "   $SCRIPT_DIR/THEMES_GUIDE.md"
echo "   $SCRIPT_DIR/themes/README.md"
echo ""
echo "Happy theming! 🎨"
echo ""
