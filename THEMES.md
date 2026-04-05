# Zush Prompt Theme System

The Zush prompt now supports a flexible theme system that allows you to easily switch between different prompt styles without modifying your entire configuration.

## Quick Start

### Switching Themes

**Method 1: Configuration file** (permanent)
```toml
# ~/.config/zush/config.toml
theme = "minimal"  # Choose: dcs, minimal, powerline
```

**Method 2: Command line** (temporary)
```bash
# Use a built-in theme
zush-prompt --theme powerline prompt

# Use a custom theme file
zush-prompt --theme ~/my-theme.toml prompt
```

**Method 3: Shell alias** (convenience)
```bash
# Add to ~/.zshrc
alias prompt-minimal='ZUSH_PROMPT_BIN="zush-prompt --theme minimal"'
alias prompt-powerline='ZUSH_PROMPT_BIN="zush-prompt --theme powerline"'
alias prompt-dcs='ZUSH_PROMPT_BIN="zush-prompt --theme dcs"'
```

## Built-in Themes

### DCS (Default)
```bash
zush-prompt --theme dcs prompt
```
- **Style**: Oh My Posh DCS theme port
- **Features**: Three-line layout, powerline segments, Tokyo Night colors
- **Best for**: Users who want a feature-rich, visually distinctive prompt

### Minimal
```bash
zush-prompt --theme minimal prompt
```
- **Style**: Clean, simple single-line
- **Features**: Minimal colors, simple indicators
- **Best for**: Users who prefer less visual noise

### Powerline
```bash
zush-prompt --theme powerline prompt
```
- **Style**: Classic powerline segments
- **Features**: Connected segments with arrows
- **Best for**: Traditional powerline users

## Creating Custom Themes

### Step 1: Create theme file
```bash
# Copy an existing theme as a starting point
cp ~/.config/zush/themes/minimal.toml ~/.config/zush/themes/mytheme.toml
```

### Step 2: Edit the theme
```toml
# ~/.config/zush/themes/mytheme.toml
name = "My Custom Theme"
description = "My personalized prompt"

[colors]
fg = "#ffffff"
bg = "#000000"
accent = "#00ff00"
error = "#ff0000"
path = "#00ffff"

[symbols]
prompt_arrow = "→"
git_branch = "⎇"
error_symbol = "✗"

[templates]
main = """{{#if (ne exit_code 0)}}{{color colors.error}}{{symbols.error_symbol}}{{else}}{{color colors.accent}}✓{{/if}} {{color colors.path}}{{pwd_short}}{{reset}} {{color colors.accent}}{{symbols.prompt_arrow}}{{reset}} """
```

### Step 3: Use your theme
```bash
# Test it
zush-prompt --theme mytheme prompt

# Make it permanent
echo 'theme = "mytheme"' >> ~/.config/zush/config.toml
```

## Theme File Structure

```toml
# Metadata
name = "Theme Name"
description = "Theme description"
author = "Your Name"

# Color palette (hex format)
[colors]
fg = "#c0caf5"           # Foreground
bg = "#1a1b26"           # Background
success = "#9ece6a"      # Success indicator
error = "#f7768e"        # Error indicator
# ... add as many as needed

# Symbols and icons
[symbols]
prompt_arrow = "❯"       # Main prompt indicator
git_branch = ""         # Git branch icon
segment_separator = ""  # Powerline separator
# ... add as many as needed

# Templates (Handlebars syntax)
[templates]
main = "..."             # Main prompt
right = "..."            # Right-side prompt (if supported)
transient = "..."        # Simplified transient prompt
continuation = "..."     # Multi-line continuation
```

## Template Variables

Available in all templates:
- `{{user}}` - Username
- `{{host}}` - Hostname
- `{{pwd}}` - Full path
- `{{pwd_short}}` - Abbreviated path
- `{{git_branch}}` - Git branch name
- `{{git_dirty}}` - Boolean for uncommitted changes
- `{{exit_code}}` - Last command exit code
- `{{execution_time}}` - Command execution time
- `{{execution_time_ms}}` - Execution time in ms
- `{{time}}` - Current time
- `{{ssh}}` - Boolean for SSH session

## Template Helpers

- `{{color colors.name}}` - Set text color
- `{{bg colors.name}}` - Set background color
- `{{bold text}}` - Bold text
- `{{dim text}}` - Dimmed text
- `{{italic text}}` - Italic text
- `{{underline text}}` - Underlined text
- `{{reset}}` - Reset all formatting

## Advanced Features

### Override Theme Settings

You can override specific theme settings without modifying the theme file:

```toml
# ~/.config/zush/config.toml
theme = "dcs"

[overrides]
colors.fg = "#ffffff"              # Override foreground color
symbols.prompt_arrow = "→"         # Change prompt arrow
```

### Conditional Formatting

Use Handlebars conditionals for dynamic prompts:

```handlebars
{{#if (ne exit_code 0)}}
  {{color colors.red}}Error: {{exit_code}}{{reset}}
{{else}}
  {{color colors.green}}✓{{reset}}
{{/if}}
```

### Custom Path to Theme

```bash
# Use absolute path
zush-prompt --theme /path/to/custom-theme.toml prompt

# Use relative path
zush-prompt --theme ./my-theme.toml prompt
```

## Testing Themes

### Preview all themes
```bash
# Run the theme gallery script
./test_themes.sh
```

### Test specific theme
```bash
# Test with sample context
zush-prompt --theme mytheme --format raw prompt \
  --context '{"pwd":"~/projects","git_branch":"main","user":"test"}' \
  --exit-code 0
```

### Debug template rendering
```bash
zush-prompt --theme mytheme --format debug prompt
```

## Troubleshooting

### Theme not loading
1. Check file exists: `ls ~/.config/zush/themes/`
2. Verify TOML syntax: `cat ~/.config/zush/themes/mytheme.toml`
3. Test directly: `zush-prompt --theme mytheme prompt`

### Powerline characters not showing
1. Install powerline fonts or Nerd Fonts
2. Configure terminal to use appropriate font
3. Use ASCII alternatives in theme symbols

### Colors not working
1. Ensure terminal supports true color (24-bit)
2. Check color hex format: `#rrggbb`
3. Verify color names match between `[colors]` and templates

## Sharing Themes

To share your theme with others:
1. Upload the `.toml` file to GitHub/GitLab
2. Others can download and place in `~/.config/zush/themes/`
3. Include screenshots and font requirements in README

## Example Theme Gallery

See `~/.config/zush/themes/` for examples:
- `dcs.toml` - Feature-rich with powerline
- `minimal.toml` - Clean and simple
- `powerline.toml` - Classic powerline style

Each theme file is self-contained and portable!