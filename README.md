# Zush Prompt - High-Performance Zsh Prompt Engine

A blazingly fast, highly customizable Zsh prompt written in Rust with perfect buffer management, 24-bit true color support, and advanced templating.

## Features

- 🎨 **24-bit True Color Support**: Full RGB color with hex codes (#ff9e64)
- 📐 **Perfect Buffer Management**: No corruption on terminal resize
- 🎯 **Three-Section Layout**: Left, center, and right prompt alignment
- 🔧 **Handlebars Templates**: Powerful, flexible templating engine
- 🚀 **Blazing Fast**: Written in Rust for maximum performance
- 🔄 **Transient Prompts**: Simplified prompts for command history
- 🎨 **Tokyo Night Theme**: Beautiful default color scheme
- 📦 **Modular Segments**: Git, time, execution time, and more
- 🔌 **Easy Integration**: Simple Zsh setup with hooks

## Installation

### Quick Install

```bash
chmod +x install.sh
./install.sh
```

### Manual Installation

1. Build the project:
```bash
cargo build --release
```

2. Copy the binary to your PATH:
```bash
cp target/release/zush-prompt ~/.local/bin/
```

3. Create configuration directory:
```bash
mkdir -p ~/.config/zush
cp config.example.toml ~/.config/zush/config.toml
```

4. Add to your `.zshrc`:
```bash
# Generate the integration script
zush-prompt init zsh > ~/.config/zush/zush.zsh

# Add to .zshrc
echo 'source ~/.config/zush/zush.zsh' >> ~/.zshrc
```

## Configuration

The configuration file is located at `~/.config/zush/config.toml`.

### Basic Configuration

```toml
[colors]
# Define your color palette (hex codes)
background = "#1a1b26"
foreground = "#c0caf5"
red = "#f7768e"
green = "#9ece6a"
blue = "#7aa2f7"

[symbols]
# Customize symbols
prompt_arrow = "❯"
git_branch = ""
success = "✓"
error = "✖"

[behavior]
transient_prompt = true
show_execution_time_threshold = 2.0
```

### Template Syntax

Templates use Handlebars syntax with custom helpers:

```handlebars
{{color colors.blue "text"}}           # Colored text
{{bg colors.red "text"}}                # Background color
{{bold "text"}}                         # Bold text
{{dim "text"}}                          # Dimmed text
{{italic "text"}}                       # Italic text
{{underline "text"}}                    # Underlined text

{{#if condition}}...{{/if}}            # Conditional rendering
{{#if_eq val1 val2}}...{{/if_eq}}      # Equality check
{{truncate text 20}}                    # Truncate text
{{pad_left text 10}}                    # Left padding
{{pad_right text 10}}                   # Right padding
{{center text 20}}                      # Center text
```

### Available Variables

Variables available in templates:

- `pwd` - Current directory
- `pwd_short` - Current directory with ~ for home
- `user` - Username
- `host` - Hostname
- `git_branch` - Current git branch
- `git_dirty` - Whether git repo has changes
- `ssh` - Whether in SSH session
- `exit_code` - Last command exit code
- `execution_time` - Last command execution time
- `jobs` - Number of background jobs
- `virtual_env` - Python virtual environment
- `time` - Current time
- `date` - Current date

### Example Templates

#### Minimal Prompt
```toml
[templates.minimal]
template = "{{color colors.cyan pwd_short}} {{color colors.blue symbols.prompt_arrow}} "
```

#### Two-Line Prompt
```toml
[templates.twoline]
template = """
╭─ {{user}}@{{host}} in {{color colors.magenta pwd}}
╰─{{color colors.blue symbols.prompt_arrow}} """
```

#### Powerline Style
```toml
[templates.powerline]
template = """
{{bg colors.blue}}{{color colors.black user}}{{reset}}{{color colors.blue}}{{bg colors.magenta}}{{reset}}
{{color colors.blue symbols.prompt_arrow}} """
```

## Command Line Usage

```bash
# Show help
zush-prompt --help

# Generate Zsh integration script
zush-prompt init zsh

# Print example configuration
zush-prompt config

# Render a specific template
zush-prompt --template minimal

# Debug output
zush-prompt --format debug

# Use custom config file
zush-prompt --config /path/to/config.toml
```

## Advanced Features

### Transient Prompts

Previous command prompts are automatically simplified to save space:

```bash
# Full prompt when typing
user in ~/projects/zush on main ✓
❯ cargo build

# Simplified after execution
14:23:05 | cargo build
❯
```

### Custom Segments

Add custom data collectors in your templates:

```toml
[templates.custom]
template = """
{{#if k8s_context}}
  K8s: {{color colors.blue k8s_context}}
{{/if}}
{{#if aws_profile}}
  AWS: {{color colors.orange aws_profile}}
{{/if}}
"""
```

### Performance Optimization

The prompt is optimized for speed:
- Zero external commands in hot path
- Efficient terminal size detection
- Cached git information
- Compiled templates
- Minimal allocations

## Troubleshooting

### Colors not working
Ensure your terminal supports 24-bit colors:
```bash
printf "\x1b[38;2;255;100;0mTRUECOLOR\x1b[0m\n"
```

### Prompt not updating
Check that hooks are properly installed:
```bash
add-zsh-hook -L | grep zush
```

### Buffer corruption
This implementation uses proper ANSI escape sequences and Zsh escaping to prevent buffer issues.

## Architecture

```
src/
├── buffer/      # Terminal buffer management
├── color/       # 24-bit color system
├── template/    # Handlebars template engine
├── segments/    # Data collectors (git, time, etc.)
├── config/      # Configuration management
└── main.rs      # CLI and Zsh integration
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

MIT License - See LICENSE file for details

## Comparison with Other Prompts

| Feature | Zush | Oh My Posh | Starship | Powerlevel10k |
|---------|------|------------|----------|---------------|
| Language | Rust | Go | Rust | Zsh |
| 24-bit colors | ✅ | ✅ | ✅ | ✅ |
| Buffer stability | ✅ | ⚠️ | ✅ | ⚠️ |
| Template engine | Handlebars | Custom | TOML | Zsh |
| Transient prompts | ✅ | ✅ | ✅ | ✅ |
| Three-section layout | ✅ | ✅ | ❌ | ✅ |
| Performance | Excellent | Good | Excellent | Good |

## Acknowledgments

- Tokyo Night color scheme
- Powerline fonts
- Inspired by Oh My Posh and Starship