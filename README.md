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
- **Context-Aware Modules** - Python, Node.js, Go, Ruby, Rust, Docker detection
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

### One-Line Install

```bash
curl -fsSL https://raw.githubusercontent.com/mikemackintosh/zush/main/scripts/install.sh | bash
```

Then add to your `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
export ZUSH_CURRENT_THEME="split"  # or: minimal, powerline, dcs, starship, catppuccin
source ~/.config/zush/zush.zsh
[ -f ~/.config/zush/zush-theme.zsh ] && source ~/.config/zush/zush-theme.zsh
```

Reload your shell:
```bash
source ~/.zshrc
```

### Alternative: Build from Source

```bash
# Requires Rust
cargo build --release
make install
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
# Powerline segment colors (Nord-inspired)
bg_user = "#5e81ac"        # Blue
bg_pwd = "#88c0d0"         # Cyan
bg_git = "#a3be8c"         # Green
fg_light = "#eceff4"       # Almost white
fg_dark = "#2e3440"        # Almost black
red = "#bf616a"
green = "#a3be8c"
cyan = "#88c0d0"

[symbols]
sep = ""                  # Powerline arrow
git = ""                  # Git icon
folder = ""               # Folder icon

[templates]
main = """{{first_line}}
{{#if (eq exit_code 0)}}{{color colors.green "❯"}} {{else}}{{color colors.red "["}}{{exit_code}}{{color colors.red "] ❯"}} {{/if}}"""

left = """{{segment colors.bg_user colors.fg_light " "}}{{segment colors.bg_user colors.fg_light user}}{{segment colors.bg_user colors.fg_light "@"}}{{segment colors.bg_user colors.fg_light host}}..."""

right = """{{bg colors.bg_time}}{{fg colors.fg_light}}  {{time}} {{reset}}"""

transient = """{{#if (eq exit_code 0)}}{{color colors.green "❯"}} {{else}}{{color colors.red "❯"}} {{/if}}"""
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

- **`split`** (default) - Two-line prompt with left/right aligned content, Nord colors
- **`minimal`** - Simple single-line prompt
- **`powerline`** - Powerline-style segments with separators
- **`dcs`** - Oh My Posh-inspired theme with Catppuccin colors
- **`catppuccin`** - Catppuccin Mocha powerline theme
- **`starship`** - Starship-inspired minimal prompt with Tokyo Night colors

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

Zush includes a secure, context-aware module system that automatically detects development environments and displays relevant information in your prompt.

**Core Features:**
- 🔒 **Sandboxed filesystem access** - Modules can only read allowed paths
- ⚡ **200ms intelligent caching** - Fast repeated renders
- 🎯 **Context-aware** - Only appears when relevant to your current directory
- 🔐 **Secure by design** - No arbitrary code execution, no shell commands in detection logic

### Available Modules

#### 🐍 Python
**Detection:**
- Virtual environment via `$VIRTUAL_ENV`
- `pyproject.toml`, `requirements.txt`, `setup.py`
- `Pipfile`, `poetry.lock`
- `.python-version`
- `.venv/` or `venv/` directories

**Display:**
- Shows virtual environment name when active (e.g., `🐍 myenv`)
- Falls back to "python" if in project but no venv
- Optional: Display Python version with `show_version`

**Example output:** `🐍 myproject-venv` or `🐍 python v3.11.5`

---

#### ⬢ Node.js
**Detection:**
- `package.json`
- `.nvmrc`, `.node-version`
- `node_modules/` directory

**Display:**
- Shows package name from `package.json` (e.g., `⬢ my-app`)
- Falls back to "node" if no package.json
- Optional: Display Node.js version with `show_version`

**Example output:** `⬢ express-api` or `⬢ node v18.17.0`

---

#### 🐹 Go
**Detection:**
- `go.mod`, `go.sum` (Go modules)
- `Gopkg.toml`, `Gopkg.lock` (dep)
- `.go-version`
- `glide.yaml`
- **Inside `$GOPATH/src/`** (supports multiple paths with `:` separator)

**Display:**
- Shows module name from `go.mod` (last path component)
- Falls back to "go" if no go.mod
- Optional: Display Go version with `show_version`

**Example output:** `🐹 myapp` or `🐹 cli-tool v1.21.5`

---

#### 💎 Ruby
**Detection:**
- `Gemfile`, `Gemfile.lock`
- `Rakefile`
- `.ruby-version`, `.ruby-gemset`
- `config.ru` (Rack apps)
- `.gemspec` files

**Display:**
- Shows gem/project name from directory or Gemfile
- Falls back to "ruby"
- Shows version by default (disable with `show_version: false`)

**Example output:** `💎 rails-app v3.2.2`

---

#### 🦀 Rust
**Detection:**
- `Cargo.toml`, `Cargo.lock`
- `rust-toolchain`, `rust-toolchain.toml`

**Display:**
- Shows package name from `Cargo.toml` `[package]` section
- Falls back to "rust" if no Cargo.toml
- Optional: Display rustc version with `show_version`

**Example output:** `🦀 zush-prompt` or `🦀 my-crate v1.73.0`

---

#### 🐳 Docker
**Detection:**
- `Dockerfile`
- `docker-compose.yml`, `docker-compose.yaml`
- `.dockerignore`
- `.devcontainer/` directory

**Display:**
- Shows detected file types (e.g., `🐳 Dockerfile+compose`)
- Optionally shows Docker context if not "default"
- Falls back to "docker" if detection unclear

**Example output:** `🐳 Dockerfile+compose` or `🐳 compose (prod-cluster)`

---

### Using Modules in Templates

Modules are available as an array in the `modules` variable:

```handlebars
{{#each modules}}
  {{color colors.cyan this.content}}
{{/each}}
```

**Module object properties:**
- `id` - Module identifier (e.g., "python", "node", "go")
- `content` - Rendered module output (e.g., "🐍 myenv", "⬢ my-app")

**Example: Segment-style modules**
```handlebars
{{#each modules}}
  {{bg colors.bg_module}}{{fg colors.fg_light}} {{this.content}} {{reset}}
{{/each}}
```

**Example: Inline modules with separators**
```handlebars
{{#each modules}}
  {{color colors.blue "["}}{{this.content}}{{color colors.blue "]"}}
{{/each}}
```

### Module Configuration

All modules are **enabled by default** and automatically appear when relevant. To customize individual modules, you would need to create custom module configurations (feature coming soon).

**Current customization via environment:**
- Modules automatically detect their environment
- Ruby and Go show versions by default
- Python, Node, Rust, and Docker hide versions by default (faster rendering)

### Performance Characteristics

- **Detection time:** <1ms per module (filesystem checks only)
- **Caching:** 200ms per module (reuses results for rapid prompts)
- **Version queries:** ~10-50ms when enabled (runs `python --version`, etc.)
- **Recommendation:** Disable version display for faster prompts, or enable for development workflows

### Security Model

The module system is designed with security as a priority:

1. **Sandboxed filesystem:** Modules can only access current directory and home directory
2. **No arbitrary execution:** Detection uses only filesystem checks and controlled command execution
3. **Size limits:** File reads limited to 1MB maximum
4. **Timeout protection:** Module renders timeout after 100ms to prevent hanging
5. **No network access:** Modules operate entirely on local filesystem and installed binaries

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
| Language modules | 6 (Python, Node, Go, Ruby, Rust, Docker) | 40+ | 40+ | Limited |
| Sandboxed modules | Yes | No | No | No |
| Module caching | 200ms intelligent | None | Filesystem-based | Limited |

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Nord color scheme for the default split theme
- Catppuccin color scheme by the Catppuccin team
- Tokyo Night color scheme by Folke Lemaitre (starship theme)
- Nerd Fonts for powerline symbols
- Inspired by Oh My Posh and Starship
