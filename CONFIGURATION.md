# Zush Configuration Guide

This document explains the difference between environment variables and config.toml settings in Zush.

## Overview

Zush has two configuration layers:

1. **Environment Variables** - Shell-level settings (runtime behavior)
2. **Config/Theme TOML Files** - Visual appearance and templates

---

## Environment Variables (Shell-Level Settings)

These are set in your shell (`.zshrc` or directly in terminal) and control **runtime behavior**.

### Location
- Set in `~/.zshrc` BEFORE sourcing `~/.config/zush/zush.zsh`
- Or temporarily in your current shell session

### Available Environment Variables

#### `ZUSH_PROMPT_BIN`
**What it does:** Path to the zush-prompt binary
**Default:** `zush-prompt` (looks in $PATH)
**Example:**
```bash
export ZUSH_PROMPT_BIN="$HOME/.local/bin/zush-prompt"
```

#### `ZUSH_CURRENT_THEME`
**What it does:** Which theme to use
**Default:** `split`
**Example:**
```bash
export ZUSH_CURRENT_THEME="minimal"
# Available: dcs, minimal, powerline, split
# Or path to custom theme: ~/.config/zush/themes/mytheme.toml
```

#### `ZUSH_PROMPT_NEWLINE_BEFORE`
**What it does:** Add blank line before prompt (after command output)
**Default:** `1` (enabled)
**Example:**
```bash
export ZUSH_PROMPT_NEWLINE_BEFORE=0  # Disable
export ZUSH_PROMPT_NEWLINE_BEFORE=1  # Enable
```

#### `ZUSH_PROMPT_NEWLINE_AFTER`
**What it does:** Add blank line after prompt (before command output)
**Default:** `0` (disabled)
**Example:**
```bash
export ZUSH_PROMPT_NEWLINE_AFTER=1  # Enable spacing after prompt
```

### Example .zshrc Configuration

```bash
# Zush Environment Variables
export ZUSH_PROMPT_BIN="$HOME/.local/bin/zush-prompt"
export ZUSH_CURRENT_THEME="split"
export ZUSH_PROMPT_NEWLINE_BEFORE=1
export ZUSH_PROMPT_NEWLINE_AFTER=0

# Load Zush
source ~/.config/zush/zush.zsh
```

---

## Config/Theme TOML Files (Visual Appearance)

These files define **colors, symbols, and prompt templates**. They control how the prompt looks, not how it behaves.

### Locations

1. **Main config:** `~/.config/zush/config.toml` (optional)
2. **Themes:** `~/.config/zush/themes/*.toml`

### What Goes in TOML Files

#### Colors
Define RGB hex colors for your prompt elements:

```toml
[colors]
background = "#1a1b26"
foreground = "#c0caf5"
black = "#15161e"
red = "#f7768e"
green = "#9ece6a"
yellow = "#e0af68"
blue = "#7aa2f7"
magenta = "#bb9af7"
cyan = "#7dcfff"
white = "#c0caf5"
orange = "#ff9e64"
purple = "#9d7cd8"
teal = "#1abc9c"
```

#### Symbols
Define Unicode symbols and powerline characters:

```toml
[symbols]
prompt_arrow = "❯"
segment_separator = ""  # Powerline separator
segment_separator_thin = ""
git_branch = ""
git_dirty = "✗"
git_clean = "✓"
ssh = ""
root = ""
jobs = ""
error = "✖"
success = "✓"
```

#### Templates
Define prompt layout using Handlebars syntax:

```toml
[templates]

# Main prompt template (full version)
main = """{{first_line}}
{{color colors.green "❯"}} """

# Left side of first line
left = """{{color colors.blue user}}@{{color colors.blue host}}  {{color colors.magenta pwd_short}}  {{#if git_branch}}{{color colors.cyan git_branch}}{{/if}}"""

# Right side of first line
right = """{{#if execution_time}}{{#if (gt execution_time 1000)}}{{bg colors.red}}{{color colors.white (div execution_time 1000)}}s{{reset}} {{/if}}{{/if}}{{color colors.fg_dim time}}"""

# Transient prompt (simplified for scrollback)
transient = """{{color colors.cyan time}}
{{#if (eq exit_code 0)}}{{color colors.green "❯"}} {{else}}{{color colors.red "["}}{{exit_code}}{{color colors.red "] ❯"}} {{/if}}"""
```

### Template Variables Available

Your templates have access to these variables:

**System Info:**
- `user` - Username
- `host` - Hostname
- `pwd` - Full path
- `pwd_short` - Path with ~ for home
- `shell` - Shell name (zsh)
- `ssh` - "true" if in SSH session
- `virtual_env` - Python venv name if active

**Git Info:**
- `git_branch` - Current branch name
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
- `history_number` - Command history number
- `jobs` - Number of background jobs

**Terminal Info:**
- `terminal_width` - Current terminal width in columns

**Rendered Content:**
- `first_line` - Pre-rendered first line (for split theme)
- `colors.*` - Your color definitions
- `symbols.*` - Your symbol definitions

---

## Quick Reference

| Setting Type | What It Controls | Where to Set It |
|-------------|------------------|-----------------|
| **Environment Variable** | Runtime behavior (spacing, theme selection, binary path) | `.zshrc` or shell |
| **TOML Config** | Visual appearance (colors, symbols, templates) | `~/.config/zush/config.toml` or theme files |

### Common Questions

**Q: I want to change the prompt color. Where do I do that?**
A: In a TOML file (`config.toml` or your theme file). Edit the `[colors]` section.

**Q: I want to disable the blank line before my prompt. Where do I do that?**
A: Set environment variable: `export ZUSH_PROMPT_NEWLINE_BEFORE=0` in `.zshrc`

**Q: I want to change the git branch symbol. Where do I do that?**
A: In a TOML file. Edit `[symbols]` section: `git_branch = ""`

**Q: I want to switch themes. Where do I do that?**
A: Set environment variable: `export ZUSH_CURRENT_THEME="minimal"` in `.zshrc`
Or run at runtime: `zush-theme minimal`

**Q: I want to change what appears in the prompt. Where do I do that?**
A: In a TOML file. Edit the `[templates]` section using Handlebars syntax.

---

## Example: Complete Configuration

**~/.zshrc:**
```bash
# Runtime behavior
export ZUSH_CURRENT_THEME="split"
export ZUSH_PROMPT_NEWLINE_BEFORE=1
export ZUSH_PROMPT_NEWLINE_AFTER=0

source ~/.config/zush/zush.zsh
```

**~/.config/zush/themes/split.toml:**
```toml
# Visual appearance
[colors]
cyan = "#7dcfff"
green = "#9ece6a"
red = "#f7768e"

[symbols]
prompt_arrow = "❯"

[templates]
main = """{{first_line}}
{{color colors.green symbols.prompt_arrow}} """

left = """{{color colors.blue user}}@{{color colors.blue host}}"""

transient = """{{color colors.cyan time}}
{{color colors.green symbols.prompt_arrow}} """
```

---

## Summary

- **Environment Variables** = Behavior (spacing, theme choice, paths)
- **TOML Files** = Appearance (colors, symbols, layout)

If it's about **what to display** or **how it looks** → TOML
If it's about **runtime behavior** or **feature toggles** → Environment Variable
