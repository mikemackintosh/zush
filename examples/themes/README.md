# Zush Theme Gallery

This directory contains example themes for the Zush prompt system. Each theme is a standalone TOML file that can be used as-is or customized to your preferences.

## Quick Start

### Installing Themes

Copy any theme to your themes directory:

```bash
# Install a single theme
cp examples/themes/dcs.toml ~/.config/zush/themes/

# Install all example themes
cp examples/themes/*.toml ~/.config/zush/themes/
```

### Using Themes

```bash
# List all available themes
zush-theme list

# Preview all themes
zush-theme preview

# Switch to a theme
zush-theme dcs
zush-theme minimal
zush-theme powerline
```

## Available Themes

### DCS (Default)
**Style:** Oh My Posh inspired, multi-line with powerline segments
**Colors:** Catppuccin-inspired modern palette
**Best For:** Users who want a feature-rich, visually distinctive prompt

Features:
- Three-line layout
- Time display with icon
- Lambda symbol
- Directory with folder icon
- Comprehensive git status with icons
- Transient prompt support

### Minimal
**Style:** Clean single-line prompt
**Colors:** Vibrant but minimal color scheme
**Best For:** Users who prefer simplicity and less visual noise

Features:
- Single-line design
- Current directory
- Git branch indicator
- Exit code display on errors
- Fast and lightweight

### Powerline
**Style:** Classic powerline with connected segments
**Colors:** Bold segment colors with smooth transitions
**Best For:** Traditional powerline enthusiasts

Features:
- User and host segments
- Directory segment
- Git status with detailed indicators
- Powerline arrows connecting all segments
- Clean transient mode

### Catppuccin
**Style:** Catppuccin Mocha powerline
**Colors:** Full Catppuccin Mocha palette
**Best For:** Catppuccin theme lovers

Features:
- Beautiful pastel colors
- Smooth color transitions
- User, directory, git, and time segments
- Comprehensive git status
- Nerd Font icons throughout

## Theme Structure

Each theme file follows this structure:

```toml
# Theme metadata
name = "theme-name"
description = "Theme description"
author = "Your Name"
version = "1.0.0"

# Color definitions (hex format)
[colors]
primary = "#3b82f6"
success = "#10b981"
error = "#ef4444"
# ... more colors

# Symbol definitions
[symbols]
prompt_arrow = "❯"
git_branch = ""
# ... more symbols

# Template definitions (Handlebars syntax)
[templates]
main = """..."""       # Main prompt
transient = """..."""  # Simplified transient prompt
right = """..."""      # Right-side prompt (optional)
```

## Creating Custom Themes

1. **Start with a base theme:**
   ```bash
   cp examples/themes/minimal.toml ~/.config/zush/themes/mytheme.toml
   ```

2. **Edit the colors:**
   Change the hex color values in the `[colors]` section

3. **Customize symbols:**
   Replace icons in the `[symbols]` section (requires Nerd Fonts)

4. **Modify templates:**
   Edit the Handlebars templates in `[templates]`

5. **Test your theme:**
   ```bash
   zush-theme mytheme
   ```

## Template Variables

Available in all templates:

- `{{user}}` - Username
- `{{host}}` - Hostname
- `{{pwd}}` - Full path
- `{{pwd_short}}` - Abbreviated path (~/ and parent directories)
- `{{git_branch}}` - Current git branch
- `{{git_staged}}` - Number of staged files
- `{{git_modified}}` - Number of modified files
- `{{git_added}}` - Number of added files
- `{{git_deleted}}` - Number of deleted files
- `{{git_renamed}}` - Number of renamed files
- `{{git_untracked}}` - Number of untracked files
- `{{git_conflicted}}` - Number of conflicted files
- `{{exit_code}}` - Last command exit code
- `{{execution_time}}` - Command execution time
- `{{execution_time_ms}}` - Execution time in milliseconds
- `{{time}}` - Current time (HH:MM:SS)
- `{{ssh}}` - Boolean indicating SSH session

## Template Helpers

### Color Helpers
- `{{color colors.name "text"}}` - Set foreground color
- `{{bg colors.name}}` - Set background color
- `{{segment bg_color fg_color "text"}}` - Colored segment
- `{{reset}}` - Reset all formatting

### Text Formatting
- `{{bold "text"}}` - Bold text
- `{{dim "text"}}` - Dimmed text
- `{{italic "text"}}` - Italic text
- `{{underline "text"}}` - Underlined text

### Conditionals
```handlebars
{{#if (eq exit_code 0)}}
  Success!
{{else}}
  Error: {{exit_code}}
{{/if}}

{{#if git_branch}}
  On branch: {{git_branch}}
{{/if}}

{{#if (gt git_modified 0)}}
  {{git_modified}} files modified
{{/if}}
```

## Font Requirements

Most themes use Nerd Font icons. Install a Nerd Font for best results:

- [Nerd Fonts](https://www.nerdfonts.com/)
- Recommended: FiraCode Nerd Font, JetBrains Mono Nerd Font, Cascadia Code Nerd Font

Configure your terminal to use the installed font.

## Tips

1. **Start Simple:** Begin with the minimal theme and add features gradually
2. **Test Colors:** Use online tools like [Coolors](https://coolors.co/) to design color palettes
3. **Preview Before Switching:** Always run `zush-theme preview` to see themes before applying
4. **Use Conditionals:** Show information only when relevant (e.g., git info only in repos)
5. **Keep Transient Simple:** Transient prompts should be minimal for better terminal scrollback

## Troubleshooting

**Icons not showing?**
- Install a Nerd Font
- Configure your terminal to use it
- Or replace Unicode icons with ASCII alternatives

**Colors look wrong?**
- Ensure your terminal supports 24-bit true color
- Check that hex colors are in `#rrggbb` format
- Test with `zush-prompt --theme themename prompt`

**Theme not loading?**
- Verify TOML syntax with an online validator
- Check file is in `~/.config/zush/themes/`
- Run `zush-theme list` to see if it appears

## Contributing

To share your theme:
1. Create a `.toml` file following the structure above
2. Add a preview image/screenshot
3. Submit to the Zush themes repository

## Resources

- [Zush Documentation](../zush-prompt-rust/THEMES.md)
- [Handlebars Template Guide](https://handlebarsjs.com/guide/)
- [Catppuccin Colors](https://github.com/catppuccin/catppuccin)
- [Nord Colors](https://www.nordtheme.com/)
- [Tokyo Night Colors](https://github.com/enkia/tokyo-night-vscode-theme)
