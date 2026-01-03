#!/usr/bin/env bash
#
# Zush Prompt Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/mikemackintosh/zush/main/scripts/install.sh | bash
#

set -e

# Configuration
REPO="mikemackintosh/zush"
INSTALL_DIR="${ZUSH_INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${ZUSH_CONFIG_DIR:-$HOME/.config/zush}"
THEMES_DIR="$CONFIG_DIR/themes"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
DIM='\033[2m'
NC='\033[0m'

info() { echo -e "${BLUE}$1${NC}"; }
success() { echo -e "${GREEN}$1${NC}"; }
warn() { echo -e "${YELLOW}$1${NC}"; }
error() { echo -e "${RED}$1${NC}" >&2; }

# Detect platform
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Darwin)  os="macos" ;;
        Linux)   os="linux" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)       error "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)             error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# Get latest release version
get_latest_version() {
    if command -v curl &>/dev/null; then
        curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/'
    elif command -v wget &>/dev/null; then
        wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one."
        exit 1
    fi
}

# Download file
download() {
    local url="$1" dest="$2"
    if command -v curl &>/dev/null; then
        curl -fsSL "$url" -o "$dest"
    else
        wget -q "$url" -O "$dest"
    fi
}

main() {
    echo ""
    info "Installing Zush Prompt..."
    echo ""

    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: $platform"

    # Get version
    local version
    if [ -n "$ZUSH_VERSION" ]; then
        version="$ZUSH_VERSION"
        info "Using specified version: $version"
    else
        info "Fetching latest version..."
        version=$(get_latest_version)
        if [ -z "$version" ]; then
            error "Failed to determine latest version"
            exit 1
        fi
        info "Latest version: $version"
    fi

    # Create directories
    info "Creating directories..."
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$THEMES_DIR"

    # Build download URL
    local archive_name="zush-prompt-${platform}.tar.gz"
    local download_url="https://github.com/$REPO/releases/download/v${version}/${archive_name}"

    # Download binary
    info "Downloading $archive_name..."
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    if ! download "$download_url" "$tmp_dir/archive.tar.gz"; then
        error "Failed to download from: $download_url"
        echo ""
        warn "Available platforms:"
        echo "  - macos-arm64 (Apple Silicon)"
        echo "  - macos-x86_64 (Intel Mac)"
        echo "  - linux-x86_64"
        echo "  - linux-arm64"
        exit 1
    fi

    # Extract binary
    info "Extracting..."
    tar -xzf "$tmp_dir/archive.tar.gz" -C "$tmp_dir"

    # Install binary
    info "Installing binary to $INSTALL_DIR..."
    mv "$tmp_dir/zush-prompt" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/zush-prompt"

    # Download themes
    info "Downloading themes..."
    local themes_url="https://github.com/$REPO/releases/download/v${version}/themes.tar.gz"
    if download "$themes_url" "$tmp_dir/themes.tar.gz" 2>/dev/null; then
        tar -xzf "$tmp_dir/themes.tar.gz" -C "$THEMES_DIR"
        success "Themes installed to $THEMES_DIR"
    else
        # Fallback: download themes from main branch
        info "Downloading themes from repository..."
        for theme in minimal split powerline dcs starship catppuccin; do
            download "https://raw.githubusercontent.com/$REPO/main/themes/${theme}.toml" \
                     "$THEMES_DIR/${theme}.toml" 2>/dev/null || true
        done
        success "Themes installed"
    fi

    # Generate init script
    info "Generating shell integration..."
    "$INSTALL_DIR/zush-prompt" init zsh > "$CONFIG_DIR/zush.zsh"

    # Download theme switcher
    download "https://raw.githubusercontent.com/$REPO/main/zush-theme.zsh" \
             "$CONFIG_DIR/zush-theme.zsh" 2>/dev/null || true

    # Check if PATH needs updating
    local needs_path=0
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        needs_path=1
    fi

    echo ""
    success "Installation complete!"
    echo ""

    # Print setup instructions
    info "Add this to your ~/.zshrc:"
    echo ""
    echo -e "${DIM}# Zush Prompt${NC}"
    if [ $needs_path -eq 1 ]; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
    echo "export ZUSH_CURRENT_THEME=\"split\"  # or: minimal, powerline, dcs, starship, catppuccin"
    echo "source $CONFIG_DIR/zush.zsh"
    echo "[ -f $CONFIG_DIR/zush-theme.zsh ] && source $CONFIG_DIR/zush-theme.zsh"
    echo ""

    info "Then reload your shell:"
    echo "  source ~/.zshrc"
    echo ""

    info "Theme switching:"
    echo "  zush-theme minimal    # Switch to minimal theme"
    echo "  zush-theme list       # List all themes"
    echo ""
}

main "$@"
