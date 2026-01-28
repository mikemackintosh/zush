# Zush Themes - Quick Start Guide

This guide will help you get started with Zush themes and create your own custom themes.

## Installation

### Step 1: Install Example Themes

Copy the example themes to your config directory:

```bash
# Create themes directory if it doesn't exist
mkdir -p ~/.config/zush/themes

# Copy all example themes
cp examples/themes/*.toml ~/.config/zush/themes/

# Or copy individual themes
cp examples/themes/dcs.toml ~/.config/zush/themes/
cp examples/themes/minimal.toml ~/.config/zush/themes/
```

### Step 2: Enable Theme Switching

Add to your `~/.zshrc`:

```bash
# Load the theme switcher
source /path/to/zush-prompt-rust/zush-theme.zsh

# Set default theme (optional, before init)
export ZUSH_THEME="dcs"

# Initialize Zush prompt
eval "$(zush-prompt init zsh)"
```

## Usage

### List Available Themes

```bash
# Basic list
zush-theme list

# List with previews
zush-theme list --preview
```

### Preview All Themes

```bash
# Detailed preview with multiple scenarios
zush-theme preview

# Compact single-line preview
zush-theme preview --compact
```

### Switch Themes

```bash
# Switch to a theme
zush-theme dcs
zush-theme minimal
zush-theme powerline
zush-theme nord

# Quick aliases
zt minimal          # Short form
zt-dcs             # Predefined alias
```

### Reset to Default

```bash
zush-theme reset
```

## Available Example Themes

### 1. DCS (Default)
**File:** `dcs.toml`
**Style:** Oh My Posh inspired, multi-line with powerline segments
**Colors:** Catppuccin Mocha palette
**Features:**
- Time display with clock icon
- Lambda symbol segment
- Directory with folder icon
- Comprehensive git status
- Error indicators
- Transient prompt

**Best for:** Power users who want maximum information

### 2. Minimal
**File:** `minimal.toml`
**Style:** Clean, simple single-line
**Colors:** Vibrant but minimal (blues, greens, purples)
**Features:**
- Current directory
- Git branch indicator
- Exit code on errors
- Fast rendering

**Best for:** Users who prefer simplicity and speed

### 3. Powerline
**File:** `powerline.toml`
**Style:** Classic powerline with connected segments
**Colors:** Bold segment colors (blue, purple, teal, orange)
**Features:**
- User and host segments
- Directory segment
- Git status with detailed indicators
- Powerline arrows connecting segments
- Professional look

**Best for:** Traditional powerline fans

### 4. Catppuccin
**File:** `catppuccin.toml`
**Style:** Catppuccin Mocha powerline
**Colors:** Full Catppuccin Mocha palette (pastels)
**Features:**
- User segment
- Directory with folder icon
- Git status with color-coded changes
- Time display
- Smooth color transitions
- Nerd Font icons

**Best for:** Catppuccin theme lovers

### 5. Nord
**File:** `nord.toml`
**Style:** Clean two-line with Nord colors
**Colors:** Nord arctic palette (blues and cool colors)
**Features:**
- Folder icon and directory
- Git branch with change indicators
- Exit code display
- Calming color scheme

**Best for:** Nord theme enthusiasts

### 6. Tokyo Night
**File:** `tokyonight.toml`
**Style:** Multi-line with vibrant neon colors
**Colors:** Tokyo Night palette (vibrant blues, cyans, magentas)
**Features:**
- Time display with icon
- Lambda symbol
- Directory with folder icon
- Git status with indicators
- Neon aesthetic

**Best for:** Developers who love vibrant colors

## Creating Custom Themes

### Method 1: Copy and Modify

```bash
# Copy an existing theme as a template
cp ~/.config/zush/themes/minimal.toml ~/.config/zush/themes/mytheme.toml

# Edit the theme
nano ~/.config/zush/themes/mytheme.toml

# Test it
zush-theme mytheme
```

### Method 2: Create from Scratch

Create `~/.config/zush/themes/mytheme.toml`:

```toml
name = "mytheme"
description = "My custom theme"
author = "Your Name"
version = "1.0.0"

[colors]
primary = "#3b82f6"
success = "#10b981"
error = "#ef4444"
warning = "#f59e0b"
directory = "#06b6d4"
git = "#8b5cf6"

[symbols]
prompt_arrow = "→"
git_branch = ""
folder = ""

[templates]
main = """{{color colors.directory ""}} {{color colors.directory pwd_short}} {{#if git_branch}}{{color colors.git ""}} {{color colors.git git_branch}} {{/if}}{{#if (eq exit_code 0)}}{{color colors.success "→"}} {{else}}{{color colors.error "✗"}} {{/if}}"""

transient = """{{color colors.primary "→"}} """
```

## Theme Structure Reference

### Metadata Section
```toml
name = "theme-name"           # Theme identifier
description = "Description"   # Short description
author = "Your Name"          # Theme author
version = "1.0.0"             # Semantic version
```

### Colors Section
```toml
[colors]
# Define colors as hex codes (#rrggbb)
primary = "#3b82f6"
success = "#10b981"
error = "#ef4444"
# Add as many as you need
```

### Symbols Section
```toml
[symbols]
# Define symbols and icons
prompt_arrow = "❯"
git_branch = ""
folder = ""
# Requires Nerd Fonts for icons
```

### Templates Section
```toml
[templates]
main = """..."""        # Main prompt template
transient = """..."""   # Simplified transient prompt
right = """..."""       # Right-side prompt (optional)
```

## Template Variables

Available variables:
- `{{user}}` - Current username
- `{{host}}` - Hostname
- `{{pwd}}` - Full path
- `{{pwd_short}}` - Abbreviated path
- `{{git_branch}}` - Git branch name
- `{{git_staged}}` - Staged files count
- `{{git_modified}}` - Modified files count
- `{{git_added}}` - Added files count
- `{{git_deleted}}` - Deleted files count
- `{{git_renamed}}` - Renamed files count
- `{{git_untracked}}` - Untracked files count
- `{{git_conflicted}}` - Conflicted files count
- `{{exit_code}}` - Last command exit code
- `{{execution_time}}` - Execution time
- `{{time}}` - Current time (HH:MM:SS)
- `{{ssh}}` - Boolean for SSH session

## Template Helpers

### Color Helpers
```handlebars
{{color colors.name "text"}}              # Set color and display text
{{bg colors.name}}                        # Set background color
{{fg colors.name}}                        # Set foreground color
{{segment bg_color fg_color "text"}}     # Colored segment
{{reset}}                                 # Reset formatting
```

### Conditionals
```handlebars
{{#if (eq exit_code 0)}}
  Success prompt
{{else}}
  Error prompt
{{/if}}

{{#if git_branch}}
  Show git info
{{/if}}

{{#if (gt git_modified 0)}}
  Show modified count: {{git_modified}}
{{/if}}
```

### Comparison Operators
- `eq` - Equal to
- `ne` - Not equal to
- `gt` - Greater than
- `gte` - Greater than or equal to
- `lt` - Less than
- `lte` - Less than or equal to

## Tips and Best Practices

### 1. Start Simple
Begin with a minimal theme and add features gradually. Don't overwhelm yourself with complexity.

### 2. Test Different Scenarios
Always test your theme in different states:
```bash
# Test with git repo
cd ~/some-git-repo
zush-theme mytheme

# Test error state
false
# Your prompt should show error indicator

# Test outside git repo
cd ~
```

### 3. Use Color Palettes
Choose colors from established palettes for cohesive themes:
- [Catppuccin](https://github.com/catppuccin/catppuccin)
- [Nord](https://www.nordtheme.com/)
- [Tokyo Night](https://github.com/enkia/tokyo-night-vscode-theme)
- [Dracula](https://draculatheme.com/)
- [Gruvbox](https://github.com/morhetz/gruvbox)

### 4. Preview Before Finalizing
Use the preview command to see your theme in different scenarios:
```bash
zush-theme preview
```

### 5. Keep Transient Simple
The transient prompt appears for previous commands. Keep it minimal:
```toml
transient = """{{color colors.dim "❯"}} """
```

### 6. Use Nerd Fonts
For best results, install and use a Nerd Font:
- [Nerd Fonts Download](https://www.nerdfonts.com/)
- Recommended: FiraCode Nerd Font, JetBrains Mono Nerd Font

### 7. Share Your Themes
If you create a great theme, share it! Others might love it too.

## Troubleshooting

### Icons Not Showing
**Problem:** Icons appear as boxes or question marks
**Solution:**
1. Install a Nerd Font
2. Configure your terminal to use it
3. Or replace icons with ASCII alternatives

### Colors Look Wrong
**Problem:** Colors don't match your expectations
**Solution:**
1. Ensure terminal supports 24-bit color
2. Check hex format is `#rrggbb`
3. Test in different lighting conditions

### Theme Not Loading
**Problem:** Theme doesn't appear or errors occur
**Solution:**
1. Check file exists: `ls ~/.config/zush/themes/`
2. Validate TOML syntax online
3. Look for error messages
4. Test directly: `zush-prompt --theme mytheme prompt`

### Prompt Too Long
**Problem:** Prompt wraps or takes too much space
**Solution:**
1. Use `pwd_short` instead of `pwd`
2. Reduce number of segments
3. Use symbols instead of text
4. Consider single-line layout

## Advanced Features

### Conditional Segments
Show segments only when relevant:
```handlebars
{{#if ssh}}
  {{color colors.warning " SSH "}}
{{/if}}

{{#if git_branch}}
  {{color colors.git ""}} {{git_branch}}
{{/if}}
```

### Dynamic Colors
Change colors based on state:
```handlebars
{{#if (gt git_modified 0)}}
  {{color colors.warning git_modified}}
{{else}}
  {{color colors.success "✓"}}
{{/if}}
```

### Multi-line Prompts
Create multi-line layouts:
```handlebars
main = """Line 1 with info
Line 2 with more info
{{color colors.primary "❯"}} """
```

## Resources

- [Theme Examples](./themes/) - Browse all example themes
- [Zush Documentation](../zush-prompt-rust/THEMES.md) - Complete reference
- [Handlebars Guide](https://handlebarsjs.com/guide/) - Template syntax
- [Color Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes) - Inspiration

## Getting Help

If you need help:
1. Check existing themes for examples
2. Use `zush-theme preview` to debug
3. Read error messages carefully
4. Test template syntax incrementally

Happy theming!
