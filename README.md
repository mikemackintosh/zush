# Zush Prompt - High-Performance Zsh Prompt Engine

A blazingly fast, highly customizable Zsh prompt written in Rust with perfect buffer management, 24-bit true color support, and advanced templating.

## Features

- 🎨 **24-bit True Color Support**: Full RGB color with hex codes (#ff9e64)
- 📐 **Split Layout**: Left and right aligned content with automatic spacing
- 🔧 **Simplified Template Syntax**: Easy-to-read `(bold)text(/bold)` style syntax with standardized powerline symbols
- 🔧 **Handlebars Templates**: Powerful, flexible templating engine
- 🚀 **Blazing Fast**: Written in Rust for maximum performance (~10ms render time)
- 🔄 **Transient Prompts**: Simplified prompts for cleaner scrollback
- 🎨 **Tokyo Night Theme**: Beautiful default color scheme
- 📦 **Context-Aware Modules**: Python, Node, Rust, Docker detection
- 🔒 **Secure Module System**: Sandboxed, fast, extensible
- 🔌 **Easy Integration**: Simple Zsh setup with hooks
- ⚙️ **Dual Configuration**: Environment variables for behavior, TOML for appearance

## Installation

### Prerequisites

- Rust toolchain (cargo)
- Zsh shell
- A terminal with 24-bit color support
- Nerd Font (for powerline symbols) - optional but recommended

### Step-by-Step Installation

1. **Build the binary:**
```bash
cargo build --release
```

2. **Install the binary to your PATH:**
```bash
mkdir -p ~/.local/bin
cp target/release/zush-prompt ~/.local/bin/
```

3. **Create configuration directory and install themes:**
```bash
mkdir -p ~/.config/zush/themes
cp -r themes/*.toml ~/.config/zush/themes/
```

4. **Generate the shell integration script:**
```bash
zush-prompt init zsh > ~/.config/zush/zush.zsh
```

5. **Add to your `.zshrc`:**
```bash
# Optional: Configure environment variables (must be BEFORE sourcing)
export ZUSH_CURRENT_THEME="split"           # Theme to use (default: split)
export ZUSH_PROMPT_NEWLINE_BEFORE=1         # Blank line before prompt (default: 1)
export ZUSH_PROMPT_NEWLINE_AFTER=0          # Blank line after prompt (default: 0)

# Load Zush
source ~/.config/zush/zush.zsh
```

6. **Reload your shell:**
```bash
source ~/.zshrc
```

### Verify Installation

After installation, you should see a two-line prompt with your username, hostname, directory, and git info (if in a git repo).

Test that it works:
```bash
# Check the binary is accessible
which zush-prompt

# Run a command and observe the execution time in the prompt
sleep 2

# Switch themes
zush-theme minimal
```

## Configuration

Zush uses a **dual configuration system**:

### 🔧 Environment Variables (Runtime Behavior)

Set these in `~/.zshrc` **before** sourcing the integration script:

```bash
# Which theme to use
export ZUSH_CURRENT_THEME="split"           # Options: split, minimal, powerline, dcs

# Spacing configuration
export ZUSH_PROMPT_NEWLINE_BEFORE=1         # Add blank line before prompt (default: 1)
export ZUSH_PROMPT_NEWLINE_AFTER=0          # Add blank line after prompt (default: 0)

# Binary path
export ZUSH_PROMPT_BIN="$HOME/.local/bin/zush-prompt"
```

**What environment variables control:**
- Spacing and layout behavior
- Theme selection
- Binary path

### 🎨 TOML Files (Visual Appearance)

Edit theme files in `~/.config/zush/themes/` to customize appearance:

```toml
# ~/.config/zush/themes/split.toml

[colors]
# Tokyo Night color scheme
cyan = "#7dcfff"
green = "#9ece6a"
red = "#f7768e"
blue = "#7aa2f7"
magenta = "#bb9af7"
orange = "#ff9e64"

[symbols]
prompt_arrow = "❯"
git_branch = ""
segment_separator = ""

[templates]
# Main prompt template
main = """{{first_line}}
{{color colors.green symbols.prompt_arrow}} """

# Left side of first line
left = """{{color colors.blue user}}@{{color colors.blue host}}  {{color colors.magenta pwd_short}}"""

# Right side of first line
right = """{{color colors.fg_dim time}}"""

# Transient prompt (simplified)
transient = """{{color colors.cyan time}}
{{color colors.green symbols.prompt_arrow}} """
```

**What TOML files control:**
- Colors (RGB hex values)
- Symbols (Unicode characters)
- Templates (prompt layout using Handlebars)

### 📚 Configuration Documentation

- **`QUICK_START.md`** - Quick reference and common tasks
- **`CONFIGURATION.md`** - Complete configuration guide
- **`config.example.toml`** - Example configuration file

### Quick Reference

| Want to... | Edit... |
|-----------|---------|
| Change colors | Theme TOML file → `[colors]` |
| Change symbols | Theme TOML file → `[symbols]` |
| Change layout | Theme TOML file → `[templates]` |
| Add spacing | `.zshrc` → `ZUSH_PROMPT_NEWLINE_BEFORE` |
| Switch theme | `.zshrc` → `ZUSH_CURRENT_THEME` or run `zush-theme <name>` |

### Available Themes

Zush includes several built-in themes:

- **`split`** (default) - Two-line prompt with left/right aligned content
- **`minimal`** - Simple single-line prompt
- **`powerline`** - Powerline-style segments with separators
- **`dcs`** - Custom DCS theme

Switch themes:
```bash
# In .zshrc
export ZUSH_CURRENT_THEME="minimal"

# Or at runtime
zush-theme minimal
```

Create your own theme in `~/.config/zush/themes/mytheme.toml`

### Template Syntax

Templates support both traditional Handlebars syntax and a new simplified syntax for easier readability.

#### Simplified Syntax (Recommended)

The simplified syntax uses parentheses-based tags that are more concise and readable:

```toml
# Styles
(bold)Bold text(/bold)                 # Bold text
(dim)Dimmed text(/dim)                 # Dimmed text
(italic)Italic text(/italic)           # Italic text
(underline)Underlined text(/underline) # Underlined text

# Colors (hex codes required)
(fg #ff0000)Red text(/fg)              # Red foreground
(bg #000000)Black background(/bg)      # Black background

# Nested styles work naturally
(bold)(fg #00ff00)Bold green text(/fg)(/bold)

# Powerline symbols (standardized names)
(sym triangle_right)                   #
(sym pill_left)                        #
(sym git_branch)                       #
(sym circle_right)                     #
(sym angle_left)                       #

# Symbols don't need closing tags
(fg #ff0000)(sym triangle_right)(/fg)  # Red powerline separator

# Example: Powerline segment
(bg #f38ba8)(fg #11111b) user (/fg)(/bg)(fg #f38ba8)(sym triangle_right)(/fg)
```

**Path Formatting:**

The `format_path` helper provides multiple ways to format directory paths:

```handlebars
# Different path formatting modes
{{format_path pwd "full"}}      # Full path: ~/projects/zush/zuper-shell-prompt/zush-prompt-rust
{{format_path pwd "last"}}      # Last segment only: …/zush-prompt-rust
{{format_path pwd "first:1"}}   # First char per segment: ~/p/z/z/zush-prompt-rust
{{format_path pwd "first:3"}}   # First 3 chars per segment: ~/pro/zus/zup/zush-prompt-rust
{{format_path pwd "depth:2"}}   # Deepest 2 directories: ~/zuper-shell-prompt/zush-prompt-rust
{{format_path pwd "ellipsis"}}  # Base + ellipsis + current: ~/…/zush-prompt-rust
```

**Available modes:**
- `"full"` - Complete path (default)
- `"last"` - Only the last directory segment with ellipsis prefix
- `"first:N"` - Abbreviate each segment to first N characters (preserves last segment)
- `"depth:N"` - Show only the deepest N directories
- `"ellipsis"` - Show first and last segments with ellipsis in between

**Available Powerline Symbols:**

| Symbol Names | Character | Description |
|-------------|-----------|-------------|
| `triangle_right`, `tri_right`, `arrow_right` |  | Solid right arrow |
| `triangle_left`, `tri_left`, `arrow_left` |  | Solid left arrow |
| `pill_right`, `flame_right`, `round_right` |  | Right rounded/flame |
| `pill_left`, `flame_left`, `round_left` |  | Left rounded/flame |
| `angle_right`, `thin_right` |  | Thin right angle |
| `angle_left`, `thin_left` |  | Thin left angle |
| `circle_right`, `semicircle_right` |  | Right semicircle |
| `circle_left`, `semicircle_left` |  | Left semicircle |
| `git_branch`, `branch` |  | Git branch icon |
| `lock` |  | Lock icon |
| `folder` |  | Folder icon |
| `home` |  | Home icon |

#### Traditional Handlebars Syntax

```handlebars
# Color and styling
{{color colors.blue "text"}}           # Foreground color
{{bg colors.red "text"}}               # Background color
{{bold "text"}}                        # Bold text
{{dim "text"}}                         # Dimmed text
{{italic "text"}}                      # Italic text
{{underline "text"}}                   # Underlined text
{{reset}}                              # Reset all formatting

# Conditionals
{{#if condition}}...{{/if}}            # If statement
{{#if cond}}...{{else}}...{{/if}}      # If-else
{{#if (eq a b)}}...{{/if}}             # Equality check
{{#if (gt a b)}}...{{/if}}             # Greater than
{{#if (lt a b)}}...{{/if}}             # Less than

# Math operations
{{div value divisor}}                  # Division
{{mul value factor}}                   # Multiplication
```

**Note:** Both syntaxes can be mixed in the same template. The simplified syntax is preprocessed before Handlebars rendering.

### Available Template Variables

**System Info:**
- `user` - Username
- `host` - Hostname
- `pwd` - Full current directory path
- `pwd_short` - Current directory with `~` for home
- `shell` - Shell name (zsh)
- `ssh` - "true" if in SSH session
- `virtual_env` - Active Python virtual environment name
- `terminal_width` - Current terminal width in columns

**Git Info:**
- `git_branch` - Current git branch name
- `git_staged` - Number of staged files
- `git_modified` - Number of modified files
- `git_added` - Number of added files
- `git_deleted` - Number of deleted files
- `git_renamed` - Number of renamed files
- `git_untracked` - Number of untracked files
- `git_conflicted` - Number of conflicted files

**Command Info:**
- `exit_code` - Last command exit code
- `execution_time` - Command duration in milliseconds
- `execution_time_s` - Command duration in seconds
- `time` - Current time (HH:MM:SS)
- `jobs` - Number of background jobs
- `history_number` - Command history number

**Pre-rendered Content:**
- `first_line` - Pre-rendered first line (for split theme)
- `colors.*` - Access to your color definitions
- `symbols.*` - Access to your symbol definitions

**Modules:**
- `modules` - Array of active modules (Python, Node, Rust, Docker)
- `modules[].id` - Module identifier (e.g., "python", "node")
- `modules[].content` - Rendered module content (e.g., "🐍 myenv")

## Command Line Usage

```bash
# Generate Zsh integration script (required for initial setup)
zush-prompt init zsh > ~/.config/zush/zush.zsh

# Print example configuration
zush-prompt config

# Render prompt with context (used internally by shell hooks)
zush-prompt --theme split --template main --format zsh prompt \
  --context '{"user":"john","pwd":"/home/john"}' \
  --exit-code 0 \
  --execution-time 1.5

# Render transient prompt
zush-prompt --theme split --template transient --format raw prompt

# Test different formats
zush-prompt --format raw    # Raw ANSI output
zush-prompt --format zsh    # Zsh-escaped output
zush-prompt --format debug  # Debug information
```

## Advanced Features

### Transient Prompts

Previous prompts are automatically simplified to save vertical space in scrollback:

**Before running a command:**
```
duppster@macbook  ~/projects/zush  main ●2 ✚1          123ms  14:30:45
❯ cargo build
```

**After command executes (what you see in scrollback):**
```
14:30:45
❯ cargo build
[command output here]
```

This keeps your terminal history clean and readable while still showing full context when typing.

### Dynamic Terminal Width

The prompt automatically adapts to terminal width changes:
- Native Rust-based terminal size detection
- Automatic spacing calculation between left and right content
- No manual configuration needed
- Handles resize events gracefully

### Git Integration

Automatic git status detection with detailed information:
- Current branch name
- Staged files count
- Modified files count
- Untracked files count
- All displayed with color-coded symbols

### Execution Time Tracking

Commands are automatically timed with millisecond precision:
- Color-coded based on duration (green < 1s, yellow < 5s, red ≥ 5s)
- Displayed in the prompt after command completion
- Helps identify slow commands

### Module System

Zush features a **secure, context-aware module system** that automatically detects and displays relevant development environment information. Modules only appear when relevant to your current directory.

**Key Features:**
- 🔒 **Secure**: Sandboxed filesystem access (only pwd and home directories)
- ⚡ **Fast**: 200ms caching, minimal overhead (~1-2ms per active module)
- 🎯 **Context-Aware**: Only displays when relevant files are detected
- 🔧 **Built-in Modules**: Python, Node.js, Rust, Docker

#### Built-in Modules

**Python Module** (`python`)
- Detects active virtual environments (`VIRTUAL_ENV`)
- Project markers: `pyproject.toml`, `requirements.txt`, `setup.py`, `Pipfile`, `.python-version`
- Shows: Virtual environment name or package name
- Example: `🐍 myenv`

**Node.js Module** (`node`)
- Detects: `package.json`, `.nvmrc`, `.node-version`, `node_modules/`
- Shows: Package name from `package.json`
- Example: `⬢ my-app`

**Rust Module** (`rust`)
- Detects: `Cargo.toml`, `Cargo.lock`, `rust-toolchain`
- Shows: Package name from `Cargo.toml`
- Example: `🦀 zush-prompt`

**Docker Module** (`docker`)
- Detects: `Dockerfile`, `docker-compose.yml`, `.devcontainer/`
- Shows: Which Docker files are present
- Example: `🐳 Dockerfile+compose`

#### Using Modules in Templates

Modules are available in templates via the `{{modules}}` array:

```handlebars
{{#each modules}}
  {{color colors.cyan this.content}}
{{/each}}
```

Example template integration:
```handlebars
# Show modules in your prompt
{{#if modules}}
  {{#each modules}}
    [{{this.content}}]
  {{/each}}
{{/if}}
```

Each module provides:
- `id` - Module identifier (e.g., "python", "node")
- `content` - Rendered module content (e.g., "🐍 myenv")

#### Module Configuration

Modules can be configured in your theme TOML file:

```toml
[modules.python]
enabled = true
symbol = "🐍"
show_version = false

[modules.node]
enabled = true
symbol = "⬢"
show_version = false

[modules.rust]
enabled = true
symbol = "🦀"
show_version = false

[modules.docker]
enabled = true
symbol = "🐳"
show_context = false
```

#### Performance Characteristics

- **Detection**: ~0.1ms per module (file existence checks)
- **Rendering**: ~1-2ms when active
- **Caching**: 200ms (prevents redundant checks)
- **Timeout**: 100ms maximum per module (prevents hanging)
- **Security**: Sandboxed filesystem, no arbitrary code execution

#### Creating Custom Modules

The module system is designed to be **easily extensible** while maintaining security and performance. New modules can be added by implementing the `Module` trait in Rust.

**Example workflow:**
1. Detect project markers (files, directories, environment variables)
2. Extract relevant information (version, name, status)
3. Render formatted output
4. Register module in registry

All modules are compiled into the binary for maximum security and performance - no runtime code loading or eval.

## Troubleshooting

### Colors not displaying correctly

Ensure your terminal supports 24-bit true colors:
```bash
printf "\x1b[38;2;255;100;0mTRUECOLOR\x1b[0m\n"
```

If you see "TRUECOLOR" in orange, 24-bit colors are working. If not:
- iTerm2: Preferences → Profiles → Terminal → Report Terminal Type → `xterm-256color`
- Terminal.app: Should work by default on macOS
- Other terminals: Set `TERM=xterm-256color` in your `.zshrc`

### Symbols not showing (boxes or question marks)

You need a Nerd Font installed:
1. Download from [Nerd Fonts](https://www.nerdfonts.com/)
2. Install the font (e.g., "JetBrainsMono Nerd Font")
3. Set it as your terminal font
4. Restart your terminal

### Prompt not updating

Check that hooks are properly installed:
```bash
add-zsh-hook -L | grep zush
```

You should see:
- `zush_preexec` in the preexec list
- `zush_precmd` in the precmd list

If missing, try:
```bash
source ~/.config/zush/zush.zsh
```

### Execution time not showing

Ensure the `zsh/datetime` module is loaded:
```bash
zmodload zsh/datetime
echo $EPOCHREALTIME
```

If empty, the module isn't loaded. This should be automatic in the integration script.

### Transient prompts creating buffer issues on resize

This is a known limitation. True transient prompts (that modify already-rendered prompts) are fundamentally incompatible with buffer stability during terminal resize. This is a tradeoff documented in `CONFIGURATION.md`.

To minimize issues:
- Avoid resizing the terminal frequently
- Use full-screen terminal windows
- Consider disabling transient prompts if this is a major issue

### Theme not found

Ensure the theme file exists:
```bash
ls ~/.config/zush/themes/
```

If themes are missing, copy them from the project:
```bash
cp -r themes/*.toml ~/.config/zush/themes/
```

### Performance issues / slow prompt

Check what's taking time:
```bash
# Time the prompt rendering
time zush-prompt --theme split --template main --format zsh prompt \
  --context '{"pwd":"'$PWD'","user":"'$USER'"}' \
  --exit-code 0
```

If slow:
- Ensure you're using the release build (`cargo build --release`)
- Check if git operations are slow (large repos)
- Verify terminal size detection isn't failing

## Architecture

```
src/
├── buffer/      # Terminal buffer management
├── color/       # 24-bit color system
├── template/    # Handlebars template engine
├── segments/    # Data collectors (git, time, etc.)
├── modules/     # Context-aware modules (Python, Node, Rust, Docker)
├── config/      # Configuration management
└── main.rs      # CLI and Zsh integration
```

## Building from Source

### Quick Build

```bash
# Build for your current platform
cargo build --release

# Binary will be in target/release/zush-prompt
```

### Build for All Platforms

We provide scripts to build for multiple platforms:

```bash
# Build for all supported platforms
./build-all.sh

# Binaries will be in dist/ directory
# - dist/zush-prompt-macos-universal (both Intel & Apple Silicon)
# - dist/zush-prompt-linux-x86_64
# - dist/zush-prompt-linux-aarch64
```

### Create Release Archives

```bash
# After building, create release archives
./create-release.sh

# Creates versioned archives in releases/v0.1.0/
# - Includes binary, themes, documentation, and install script
```

### Cross-Platform Build Requirements

**macOS builds from macOS:**
- No additional requirements
- Automatically builds universal binary (Intel + Apple Silicon)

**Linux builds:**
- Install `cross` tool: `cargo install cross`
- Builds using Docker containers (no additional setup needed)

**Supported platforms:**
- macOS (Intel x86_64)
- macOS (Apple Silicon ARM64)
- macOS (Universal - both architectures)
- Linux (x86_64)
- Linux (ARM64/aarch64)
- Linux (x86_64 static - musl)

See **`BUILD.md`** for detailed cross-compilation instructions.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development

```bash
# Run in debug mode
cargo run -- init zsh

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## License

MIT License - See LICENSE file for details

## Comparison with Other Prompts

| Feature | Zush | Oh My Posh | Starship | Powerlevel10k |
|---------|------|------------|----------|---------------|
| Language | Rust | Go | Rust | Zsh |
| 24-bit colors | ✅ | ✅ | ✅ | ✅ |
| True transient prompts | ✅ | ✅ | ❌ | ✅ |
| Template engine | Handlebars | Go templates | TOML | Zsh functions |
| Native terminal sizing | ✅ | ✅ | ✅ | ✅ |
| Dual configuration system | ✅ | ❌ | ❌ | ⚠️ |
| Context-aware modules | ✅ | ✅ | ✅ | ⚠️ |
| Sandboxed module system | ✅ | ❌ | ❌ | ❌ |
| Git status integration | ✅ | ✅ | ✅ | ✅ |
| Execution time tracking | ✅ | ✅ | ✅ | ✅ |
| Performance | ~10ms | ~15-20ms | ~15-25ms | Excellent |

**Note:** True transient prompts (rewriting previous prompts) have a known limitation with buffer stability on terminal resize. This affects Zush, Oh My Posh, and Powerlevel10k.

## Documentation

- **`README.md`** (this file) - Overview, installation, and feature reference
- **`QUICK_START.md`** - Quick reference guide with common tasks
- **`CONFIGURATION.md`** - Complete configuration reference
- **`BUILD.md`** - Cross-platform build instructions
- **`config.example.toml`** - Fully annotated example configuration

## Project Structure

```
zush-prompt-rust/
├── src/
│   ├── buffer/          # Terminal buffer management
│   ├── color/           # 24-bit color system (Tokyo Night)
│   ├── template/        # Handlebars template engine
│   ├── segments/        # Data collectors (git, time, etc.)
│   ├── modules/         # Context-aware modules
│   │   ├── mod.rs       # Module trait and sandboxed context
│   │   ├── python.rs    # Python/venv detection
│   │   ├── node.rs      # Node.js detection
│   │   ├── rust_lang.rs # Rust/Cargo detection
│   │   ├── docker.rs    # Docker detection
│   │   └── registry.rs  # Module registry and caching
│   ├── config/          # Configuration management
│   └── main.rs          # CLI and Zsh integration
├── themes/              # Built-in theme files
│   ├── split.toml       # Two-line split layout (default)
│   ├── minimal.toml     # Simple single-line
│   ├── powerline.toml   # Powerline-style segments
│   └── dcs.toml         # Custom DCS theme
├── CONFIGURATION.md     # Complete configuration guide
├── QUICK_START.md       # Quick reference
└── config.example.toml  # Example configuration
```

## Acknowledgments

- Tokyo Night color scheme by Folke Lemaitre
- Nerd Fonts for powerline symbols
- Inspired by Oh My Posh and Starship