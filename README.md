# Zush Prompt

**Zush** (Zuper-shell, like "super-shell") - A blazingly fast, highly customizable Zsh prompt engine written in Rust with 24-bit true color support, advanced templating, and perfect buffer management.

## Key Features

- **24-bit True Color** - Full RGB with hex codes
- **Blazing Fast** - ~10ms render time, written in Rust
- **Dual Configuration** - Environment variables for behavior, TOML for appearance
- **Smart Layout** - Left/right aligned content with automatic spacing
- **Transient Prompts** - Simplified prompts for cleaner scrollback
- **Gradient Text** - Color gradients across any text
- **Powerful Templates** - Handlebars with simplified syntax
- **Git Integration** - Automatic status detection with detailed information
- **Context-Aware Modules** - Python, Node, Rust, Docker detection
- **Execution Timing** - Millisecond-precision command tracking
- **Secure & Sandboxed** - Safe module system with no arbitrary code execution

## Preview

```
 15:30:45                                     main ●2 ✚1          123ms
◆ λ ◗ ~/projects/zush ◗ /
❯
```

After command executes (transient prompt in scrollback):
```
15:30:45
❯ cargo build
[command output here]
```

## Quick Start

### Installation

1. **Build and install:**
```bash
cargo build --release
mkdir -p ~/.local/bin
cp target/release/zush-prompt ~/.local/bin/
```

2. **Set up configuration:**
```bash
mkdir -p ~/.config/zush/themes
cp -r themes/*.toml ~/.config/zush/themes/
zush-prompt init zsh > ~/.config/zush/zush.zsh
```

3. **Add to `.zshrc`:**
```bash
# Optional: Configure behavior (before sourcing)
export ZUSH_CURRENT_THEME="split"           # Default theme
export ZUSH_PROMPT_NEWLINE_BEFORE=1         # Blank line before prompt

# Load Zush
source ~/.config/zush/zush.zsh
```

4. **Reload:**
```bash
source ~/.zshrc
```

### Verify Installation

```bash
# Check binary is accessible
which zush-prompt

# Switch themes
zush-theme minimal

# Run a command and observe execution time
sleep 2
```

## Configuration

Zush uses a **dual configuration system**:

### Environment Variables (Runtime Behavior)

Set in `.zshrc` **before** sourcing the integration script:

```bash
export ZUSH_CURRENT_THEME="split"           # Which theme to use
export ZUSH_PROMPT_NEWLINE_BEFORE=1         # Add blank line before prompt
export ZUSH_PROMPT_NEWLINE_AFTER=0          # Add blank line after prompt
export ZUSH_PROMPT_BIN="~/.local/bin/zush-prompt"
```

### TOML Files (Visual Appearance)

Edit theme files in `~/.config/zush/themes/` to customize colors, symbols, and layout:

```toml
[colors]
cyan = "#7dcfff"
green = "#9ece6a"
red = "#f7768e"
blue = "#7aa2f7"

[symbols]
prompt_arrow = "❯"
git_branch = ""

[templates]
main = """{{first_line}}
{{color colors.green symbols.prompt_arrow}} """

left = """{{color colors.blue user}}@{{host}}  {{color colors.magenta pwd_short}}"""
right = """{{color colors.fg_dim time}}"""

transient = """{{color colors.cyan time}}
{{color colors.green symbols.prompt_arrow}} """
```

### Quick Reference

| Want to... | Edit... |
|-----------|---------|
| Change colors | Theme TOML → `[colors]` |
| Change symbols | Theme TOML → `[symbols]` |
| Change layout | Theme TOML → `[templates]` |
| Add spacing | `.zshrc` → `ZUSH_PROMPT_NEWLINE_BEFORE` |
| Switch theme | `.zshrc` → `ZUSH_CURRENT_THEME` or `zush-theme <name>` |

## Built-in Themes

- **`split`** (default) - Two-line prompt with left/right aligned content
- **`minimal`** - Simple single-line prompt
- **`powerline`** - Powerline-style segments with separators
- **`dcs`** - Custom DCS theme
- **`splug`** - Tokyo Night powerline theme

Switch themes:
```bash
zush-theme minimal
```

Create your own in `~/.config/zush/themes/mytheme.toml`

## Template Features

### Available Helpers

**Colors & Styling:**
- `{{color colors.blue "text"}}` - Foreground color
- `{{bg colors.red "text"}}` - Background color
- `{{gradient colors.teal colors.blue "text"}}` - Color gradients
- `{{bold "text"}}`, `{{dim "text"}}`, `{{italic "text"}}`, `{{underline "text"}}`

**Path Formatting:**
```handlebars
{{format_path pwd "last"}}      # …/zush-prompt-rust
{{format_path pwd "depth:2"}}   # ~/zuper-shell-prompt/zush-prompt-rust
{{format_path pwd "first:1"}}   # ~/p/z/z/zush-prompt-rust
```

**Conditionals & Logic:**
```handlebars
{{#if (eq exit_code 0)}}
  {{color colors.green "✓"}}
{{else}}
  {{color colors.red "✗"}}
{{/if}}
```

See **[TEMPLATE_HELPERS.md](TEMPLATE_HELPERS.md)** for complete reference.

### Available Variables

**System:** `user`, `host`, `pwd`, `pwd_short`, `ssh`, `virtual_env`, `terminal_width`
**Git:** `git_branch`, `git_staged`, `git_modified`, `git_added`, `git_deleted`, `git_untracked`
**Command:** `exit_code`, `execution_time`, `time`, `jobs`, `history_number`
**Content:** `first_line`, `colors.*`, `symbols.*`, `modules`

## Module System

Zush includes a secure, context-aware module system that automatically detects development environments:

- **Python** - Detects virtual environments and Python projects
- **Node.js** - Shows package name from `package.json`
- **Rust** - Displays Cargo project name
- **Docker** - Indicates Docker/compose files

**Features:**
- Sandboxed filesystem access
- 200ms caching for performance
- Only appears when relevant
- No arbitrary code execution

Use in templates:
```handlebars
{{#each modules}}
  {{color colors.cyan this.content}}
{{/each}}
```

## Advanced Features

### Transient Prompts

Previous prompts are automatically simplified to save vertical space in scrollback. This keeps your terminal history clean while showing full context when typing.

**Note:** True transient prompts have a known limitation with buffer stability on terminal resize. This is a fundamental tradeoff that affects all prompts with this feature (Oh My Posh, Powerlevel10k).

### Dynamic Terminal Width

The prompt automatically adapts to terminal width changes with native Rust-based detection and automatic spacing calculation.

### Git Integration

Automatic git status detection with:
- Branch name display
- Staged, modified, untracked file counts
- Color-coded indicators

### Execution Time Tracking

Commands are automatically timed with millisecond precision and color-coded based on duration.

## Troubleshooting

### Colors not displaying

Ensure 24-bit color support:
```bash
printf "\x1b[38;2;255;100;0mTRUECOLOR\x1b[0m\n"
```

If not orange, set `TERM=xterm-256color` in `.zshrc`

### Symbols showing as boxes

Install a [Nerd Font](https://www.nerdfonts.com/) and set it as your terminal font.

### Prompt not updating

Check hooks are installed:
```bash
add-zsh-hook -L | grep zush
```

Should show `zush_preexec` and `zush_precmd`. If missing:
```bash
source ~/.config/zush/zush.zsh
```

### Theme not found

Copy themes from the project:
```bash
cp -r themes/*.toml ~/.config/zush/themes/
```

### Performance issues

Ensure you're using the release build:
```bash
cargo build --release
```

Time the prompt rendering:
```bash
time zush-prompt --theme split --template main --format zsh prompt \
  --context '{"pwd":"'$PWD'","user":"'$USER'"}' --exit-code 0
```

## Documentation

- **[README.md](README.md)** - This file - overview and quick start
- **[TEMPLATE_HELPERS.md](TEMPLATE_HELPERS.md)** - Complete template helper reference
- **[CONFIGURATION.md](CONFIGURATION.md)** - Environment variables vs TOML configuration
- **[QUICK_START.md](QUICK_START.md)** - Quick reference guide
- **[THEMES.md](THEMES.md)** - Theme creation and customization
- **[BUILD.md](BUILD.md)** - Cross-platform build instructions
- **[PERFORMANCE.md](PERFORMANCE.md)** - Performance characteristics

## Architecture

```
src/
├── buffer/      # Terminal buffer management
├── color/       # 24-bit color system
├── template/    # Handlebars template engine with custom helpers
├── segments/    # Data collectors (git, time, system info)
├── modules/     # Context-aware modules (Python, Node, Rust, Docker)
├── config/      # Configuration management
└── main.rs      # CLI and Zsh integration
```

## Building from Source

### Quick Build
```bash
cargo build --release
# Binary: target/release/zush-prompt
```

### Multi-Platform Build
```bash
./build-all.sh
# Binaries in dist/ for macOS (universal), Linux (x86_64, aarch64)
```

### Create Release
```bash
./create-release.sh
# Creates versioned archives in releases/v0.1.0/
```

See **[BUILD.md](BUILD.md)** for detailed cross-compilation instructions.

## Contributing

Contributions welcome! Please submit issues and pull requests.

**Development:**
```bash
cargo run -- init zsh
cargo test
cargo fmt --check
cargo clippy
```

## Comparison

| Feature | Zush | Oh My Posh | Starship | Powerlevel10k |
|---------|------|------------|----------|---------------|
| Language | Rust | Go | Rust | Zsh |
| Performance | ~10ms | ~15-20ms | ~15-25ms | Excellent |
| 24-bit colors | Yes | Yes | Yes | Yes |
| Transient prompts | Yes | Yes | No | Yes |
| Gradient text | Yes | No | No | Partial |
| Template engine | Handlebars | Go templates | TOML | Zsh |
| Dual config system | Yes | No | No | Partial |
| Sandboxed modules | Yes | No | No | No |

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Tokyo Night color scheme by Folke Lemaitre
- Nerd Fonts for powerline symbols
- Inspired by Oh My Posh and Starship
