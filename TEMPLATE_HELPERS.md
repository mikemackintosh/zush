# Zush Prompt - Template Helpers Reference

Complete guide to all available template helpers for creating custom themes.

## Color Helpers

### `{{color}}` - Foreground Color (Auto-Reset)
Apply 24-bit RGB color to text. **Automatically resets after text.**

**Syntax:**
```handlebars
{{color "hex_color" "text"}}
{{color colors.variable_name "text"}}
{{color r g b "text"}}
```

**Examples:**
```handlebars
{{color "#ff0000" "Red Text"}}
{{color colors.blue "Blue Text"}}
{{color 255 0 0 "RGB Red Text"}}
```

### `{{fg}}` - Foreground Color (No Reset) **NEW!**
Set foreground color without resetting. **Use with `{{bg}}` for powerline segments.**

**Syntax:**
```handlebars
{{fg "hex_color"}}
{{fg colors.variable_name}}
```

**Examples:**
```handlebars
{{fg "#ff0000"}}Red text continues{{reset}}
{{bg colors.blue}}{{fg colors.white}}Text{{reset}}
```

### `{{bg}}` - Background Color (No Reset)
Apply background color (24-bit RGB). **Does not auto-reset when used alone.**

**Syntax:**
```handlebars
{{bg "hex_color"}}              // Set background (NO RESET)
{{bg "hex_color" "text"}}       // Background with text (AUTO RESET)
{{bg colors.variable_name}}     // From color palette (NO RESET)
```

**Examples:**
```handlebars
{{bg "#1e1e2e"}}Text with dark bg{{reset}}
{{bg colors.blue " Segment "}}
{{bg "#89b4fa"}}{{fg "#000000"}}Text{{reset}}
```

### `{{segment}}` - Powerline Segment Helper **NEW!**
Set both background AND foreground colors simultaneously. **No auto-reset.**

**Syntax:**
```handlebars
{{segment "bg_color" "fg_color" "text"}}
{{segment colors.bg colors.fg "text"}}
```

**Examples:**
```handlebars
{{segment "#2e7de9" "#ffffff" " user "}}
{{segment colors.bg_blue colors.fg_white " segment "}}
```

**Powerline Pattern:**
```handlebars
{{segment colors.bg1 colors.fg " text "}}{{bg colors.bg2}}{{fg colors.bg1}}{{reset}}
#                                        ^next bg    ^arrow color
```

### `{{gradient}}` - Color Gradient Text **NEW!**
Create smooth color gradients across text. Each character gets a progressively interpolated color from start to end.

**Syntax:**
```handlebars
{{gradient "start_hex" "end_hex" "text"}}
{{gradient colors.start colors.end "text"}}
```

**Examples:**
```handlebars
{{gradient "#1abc9c" "#7aa2f7" "USERNAME"}}
{{gradient colors.teal colors.blue "gradient text"}}
{{gradient colors.orange colors.purple "~/projects/zush"}}
```

**How it works:**
- Interpolates RGB values linearly across each character
- First character gets start color, last character gets end color
- Middle characters get smoothly interpolated colors
- Automatically resets color after text

**Visual Example:**
```handlebars
# In your theme:
[colors]
teal = "#1abc9c"
blue = "#7aa2f7"
orange = "#ff9e64"
purple = "#9d7cd8"

[templates]
main = """{{gradient colors.teal colors.blue user}} in {{gradient colors.orange colors.purple pwd_short}}
❯ """
```

**Use cases:**
- Usernames with smooth color transitions
- Directory paths with gradient effects
- Eye-catching titles or headers
- Rainbow-style text effects

**Tips:**
- Works best with contrasting colors for visible gradient
- Longer text shows smoother transitions
- Combine with bold for more vibrant effect: `{{bold (gradient colors.red colors.yellow "text")}}`
- Use color variables for consistency across your theme

## Text Styling Helpers

### `{{bold}}` - Bold Text
Make text bold/bright.

**Syntax:**
```handlebars
{{bold "text"}}
{{bold variable_name}}
```

**Examples:**
```handlebars
{{bold "Important"}}
{{bold user}}
```

### `{{dim}}` - Dimmed Text
Make text dimmed/faint (50% brightness).

**Syntax:**
```handlebars
{{dim "text"}}
{{dim variable_name}}
```

**Examples:**
```handlebars
{{dim time}}
{{dim "secondary info"}}
```

### `{{italic}}` - Italic Text
Make text italic (if terminal supports it).

**Syntax:**
```handlebars
{{italic "text"}}
```

**Examples:**
```handlebars
{{italic "Note: "}}
```

### `{{underline}}` - Underlined Text
Underline text.

**Syntax:**
```handlebars
{{underline "text"}}
```

**Examples:**
```handlebars
{{underline "important"}}
```

### `{{reset}}` - Reset All Styles
Clear all colors and styles.

**Syntax:**
```handlebars
{{reset}}
```

**Examples:**
```handlebars
{{color colors.red "Error"}}{{reset}} Back to normal
{{bg "#ff0000"}}Background{{reset}}
```

## Layout Helpers

### `{{truncate}}` - Truncate Long Text
Truncate text to maximum length with "...".

**Syntax:**
```handlebars
{{truncate text max_length}}
```

**Examples:**
```handlebars
{{truncate pwd_short 30}}
{{truncate git_branch 15}}
```

### `{{pad_left}}` - Left Padding
Pad text with spaces on the left.

**Syntax:**
```handlebars
{{pad_left text width}}
```

**Examples:**
```handlebars
{{pad_left time 10}}        # "  18:30:00"
{{pad_left exit_code 3}}    # "  0"
```

### `{{pad_right}}` - Right Padding
Pad text with spaces on the right.

**Syntax:**
```handlebars
{{pad_right text width}}
```

**Examples:**
```handlebars
{{pad_right user 15}}
```

### `{{center}}` - Center Text
Center text within a given width.

**Syntax:**
```handlebars
{{center text width}}
```

**Examples:**
```handlebars
{{center "Title" 40}}
```

## Complete Theme Example

Here's a theme using multiple helpers:

```toml
[colors]
blue = "#89b4fa"
green = "#a6e3a1"
red = "#f38ba8"
gray = "#6c7086"
bg_dark = "#1e1e2e"

[templates]
main = """{{bg colors.bg_dark}}{{bold (color colors.blue user)}} {{reset}}\
{{color colors.gray "in"}} \
{{color colors.blue pwd_short}} \
{{#if git_branch}}\
  {{dim "on"}} {{color colors.green git_branch}} \
{{/if}}
{{#if (eq exit_code 0)}}\
  {{color colors.green "❯"}}{{reset}} \
{{else}}\
  {{color colors.red "✗"}} {{color colors.red exit_code}} {{color colors.red "❯"}}{{reset}} \
{{/if}}"""
```

## Combining Helpers

You can combine helpers for complex effects:

### Foreground + Background
```handlebars
{{bg colors.blue}}{{color colors.black " Text "}}{{reset}}
```

### Bold + Color
```handlebars
{{bold (color colors.red "ERROR")}}
```

### Multiple Styles in Sequence
```handlebars
{{bg colors.bg_user}}{{bold (color colors.fg_light user)}}{{reset}}
```

## Available Context Variables

These variables are available in templates:

| Variable | Type | Description |
|----------|------|-------------|
| `pwd` | string | Full path to current directory |
| `pwd_short` | string | Abbreviated path (~) |
| `user` | string | Current username |
| `host` | string | Full hostname |
| `shell` | string | Shell name (zsh) |
| `git_branch` | string | Current git branch (empty if not in repo) |
| `ssh` | boolean | True if in SSH session |
| `virtual_env` | string | Python venv name (if active) |
| `jobs` | number | Number of background jobs |
| `exit_code` | number | Last command exit code |
| `execution_time` | number | Command duration (milliseconds) |
| `execution_time_s` | number | Command duration (seconds) |
| `time` | string | Current time (HH:MM:SS) |
| `colors` | object | Color palette from theme |
| `symbols` | object | Symbol definitions from theme |

## ANSI Escape Code Reference

If you need raw escape codes:

| Code | Effect |
|------|--------|
| `\x1b[0m` | Reset all |
| `\x1b[1m` | Bold/Bright |
| `\x1b[2m` | Dim |
| `\x1b[3m` | Italic |
| `\x1b[4m` | Underline |
| `\x1b[38;2;R;G;Bm` | 24-bit foreground color |
| `\x1b[48;2;R;G;Bm` | 24-bit background color |

## Best Practices

### 1. Always Reset
Always use `{{reset}}` after colors/styles to avoid bleeding:
```handlebars
{{color colors.red "Error"}}{{reset}}
```

### 2. Use Color Variables
Define colors in `[colors]` section, not hardcoded:
```toml
[colors]
error = "#ef4444"

[templates]
main = """{{color colors.error "Failed"}}"""
```

### 3. Handle Empty Variables
Use conditionals for optional content:
```handlebars
{{#if git_branch}}
  {{color colors.green git_branch}}
{{/if}}
```

### 4. Proper Spacing
Be careful with spaces in templates:
```handlebars
{{color colors.blue user}} in {{color colors.cyan pwd}}
#                        ^space^space
```

### 5. Background Segments
For powerline-style segments:
```handlebars
{{bg colors.bg1}}{{color colors.fg " text "}}{{reset}}{{color colors.bg1 ""}}{{reset}}
```

## Testing Your Theme

Test directly with the command line:

```bash
# Test main template
zush-prompt --theme mytheme --format raw prompt \
  --context '{"pwd":"~/test","user":"demo","git_branch":"main"}' \
  --exit-code 0

# Test with error
zush-prompt --theme mytheme --format raw prompt \
  --context '{"pwd":"~/test","user":"demo"}' \
  --exit-code 1

# Debug mode
zush-prompt --theme mytheme --format debug prompt
```

## Common Patterns

### Conditional Colors
```handlebars
{{#if (eq exit_code 0)}}
  {{color colors.green "✓"}}
{{else}}
  {{color colors.red "✗"}}
{{/if}}
```

### Perfect Powerline Segments
```handlebars
# Segment 1 (blue background, white text)
{{segment colors.bg_blue colors.fg_white " user "}}

# Transition arrow + Segment 2 (purple background, white text)
{{bg colors.bg_purple}}{{fg colors.bg_blue}}{{segment colors.bg_purple colors.fg_white " host "}}

# Final arrow (no more segments)
{{reset}}{{fg colors.bg_purple}}{{reset}}
```

### Complete Powerline Example
```handlebars
{{segment "#2e7de9" "#ffffff" " demo "}}\
{{bg "#7847bd"}}{{fg "#2e7de9"}}\
{{segment "#7847bd" "#ffffff" " mac "}}\
{{bg "#0f766e"}}{{fg "#7847bd"}}\
{{segment "#0f766e" "#ffffff" " ~/projects "}}\
{{reset}}{{fg "#0f766e"}}{{reset}}
```

### Time-Based Dim
```handlebars
{{dim time}} {{color colors.blue pwd}}
```

### Bold Usernames
```handlebars
{{bold (color colors.cyan user)}}
```

### Gradient Effects
```handlebars
# Gradient username
{{gradient colors.teal colors.blue user}}

# Gradient path with bold
{{bold (gradient colors.orange colors.purple pwd_short)}}

# Multi-gradient prompt
{{gradient "#1abc9c" "#7aa2f7" user}}@{{gradient "#ff9e64" "#9d7cd8" host}}
```

---

**Note:** All helpers automatically handle UTF-8 characters, powerline fonts, and proper terminal width calculations for accurate prompt rendering.