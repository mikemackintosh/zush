//! Shell initialization scripts
//!
//! This module provides shell integration scripts for various shells.
//! Currently only Zsh is supported.

pub mod zsh;

use anyhow::Result;

/// Print the initialization script for the specified shell
pub fn print_init_script(shell: &str) -> Result<()> {
    match shell {
        "zsh" => {
            println!("{}", zsh::INIT_SCRIPT);
            Ok(())
        }
        _ => Err(anyhow::anyhow!(
            "Shell '{}' is not supported. Currently only 'zsh' is available.",
            shell
        )),
    }
}

/// Print the default configuration template
pub fn print_default_config() -> Result<()> {
    println!("{}", DEFAULT_CONFIG);
    Ok(())
}

/// Default configuration template
pub const DEFAULT_CONFIG: &str = r##"# Zush Prompt Configuration

[colors]
# Define custom colors (hex format)
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

[symbols]
# Powerline and other symbols
prompt_arrow = "❯"
segment_separator = ""
segment_separator_thin = ""
git_branch = ""
git_dirty = "✗"
git_clean = "✓"
ssh = ""
root = ""
jobs = ""
error = "✖"
success = "✓"

[segments]
# Segment visibility and order
left = ["status", "user", "host", "directory"]
center = ["git"]
right = ["execution_time", "time"]

[templates.main]
# Main prompt template using Handlebars syntax
template = """
{{#if (eq exit_code 0)}}
  {{color colors.green symbols.success}}
{{else}}
  {{color colors.red symbols.error}}
{{/if}}
{{bold " "}}
{{#if ssh}}
  {{color colors.orange symbols.ssh}} {{color colors.cyan user}}@{{color colors.cyan host}}
{{else if (eq user "root")}}
  {{color colors.red symbols.root}} {{color colors.red "root"}}
{{else}}
  {{color colors.blue user}}
{{/if}}
{{color colors.white " in "}}
{{color colors.magenta pwd}}
{{#if git_branch}}
  {{color colors.white " on "}}
  {{color colors.green symbols.git_branch}} {{color colors.green git_branch}}
{{/if}}
{{#if (gt jobs "0")}}
  {{color colors.yellow " ["}}{{color colors.yellow symbols.jobs}}{{color colors.yellow jobs}}{{color colors.yellow "]"}}
{{/if}}
"""

[templates.transient]
# Transient prompt (simplified)
template = """
{{dim (color colors.fg_dim time)}}
{{color colors.blue symbols.prompt_arrow}}
"""

[templates.right]
# Right-side prompt
template = """
{{#if execution_time}}
  {{#if (gt execution_time 5)}}
    {{color colors.yellow execution_time}}s
  {{else if (gt execution_time 1)}}
    {{color colors.green execution_time}}s
  {{/if}}
{{/if}}
{{dim time}}
"""

[templates.continuation]
# Continuation prompt for multi-line commands
template = "{{color colors.blue \"...\"}} "
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_shell() {
        let result = print_init_script("bash");
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config_not_empty() {
        assert!(!DEFAULT_CONFIG.is_empty());
        assert!(DEFAULT_CONFIG.contains("[colors]"));
        assert!(DEFAULT_CONFIG.contains("[symbols]"));
        assert!(DEFAULT_CONFIG.contains("[templates.main]"));
    }
}
