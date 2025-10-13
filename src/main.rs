mod buffer;
mod color;
mod template;
mod segments;
mod config;
mod git;
mod modules;

use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};

use buffer::TerminalBuffer;
use color::tokyo_night;
use template::TemplateEngine;

/// Zush - A high-performance Zsh prompt with perfect buffering and 24-bit colors
#[derive(Parser, Debug)]
#[command(name = "zush-prompt")]
#[command(about = "A Rust-powered Zsh prompt engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output format: zsh, raw, or debug
    #[arg(short, long, default_value = "zsh")]
    format: String,

    /// Template to render (defaults to main)
    #[arg(short, long, default_value = "main")]
    template: String,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Theme to use (overrides config)
    #[arg(long)]
    theme: Option<String>,

    /// Show transient prompt
    #[arg(long)]
    transient: bool,

    /// Suppress error messages (useful for transient prompts to avoid duplication)
    #[arg(long)]
    quiet: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize Zsh integration
    Init {
        /// Shell type (currently only zsh supported)
        #[arg(default_value = "zsh")]
        shell: String,
    },

    /// Render the prompt
    Prompt {
        /// JSON context data from environment
        #[arg(short, long)]
        context: Option<String>,

        /// Exit code of last command
        #[arg(short = 'e', long)]
        exit_code: Option<i32>,

        /// Command execution time in seconds
        #[arg(short = 't', long)]
        execution_time: Option<f64>,
    },

    /// Print configuration template
    Config,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { shell }) => {
            print_init_script(shell)?;
        }
        Some(Commands::Config) => {
            print_default_config()?;
        }
        Some(Commands::Prompt { context, exit_code, execution_time }) => {
            render_prompt(&cli, context.as_deref(), *exit_code, *execution_time)?;
        }
        None => {
            // Default to rendering prompt
            render_prompt(&cli, None, None, None)?;
        }
    }

    Ok(())
}

fn print_init_script(shell: &str) -> Result<()> {
    if shell != "zsh" {
        return Err(anyhow::anyhow!("Only zsh is currently supported"));
    }

    let script = r#"#!/usr/bin/env zsh
# Zush Prompt Integration for Zsh

# Load datetime module for EPOCHREALTIME support
zmodload zsh/datetime

# ============================================================================
# ENVIRONMENT VARIABLES - Runtime behavior configuration
# Set these in ~/.zshrc BEFORE sourcing this file to customize behavior
# For visual appearance (colors, symbols, templates), edit theme TOML files
# See CONFIGURATION.md for details
# ============================================================================

# ZUSH_PROMPT_BIN - Path to the zush-prompt binary
# Default: "zush-prompt" (searches $PATH)
# Example: export ZUSH_PROMPT_BIN="$HOME/.local/bin/zush-prompt"
ZUSH_PROMPT_BIN="${ZUSH_PROMPT_BIN:-zush-prompt}"

# ZUSH_CURRENT_THEME - Which theme to use
# Default: "split"
# Options: dcs, minimal, powerline, split, or path to custom theme
# Example: export ZUSH_CURRENT_THEME="minimal"
# Runtime: Use `zush-theme <name>` to switch themes
typeset -g ZUSH_CURRENT_THEME="${ZUSH_CURRENT_THEME:-split}"

# ZUSH_PROMPT_NEWLINE_BEFORE - Add blank line before prompt (after command output)
# Default: 1 (enabled)
# Set to 0 to disable spacing before prompt
# Example: export ZUSH_PROMPT_NEWLINE_BEFORE=0
typeset -g ZUSH_PROMPT_NEWLINE_BEFORE="${ZUSH_PROMPT_NEWLINE_BEFORE:-1}"

# ZUSH_PROMPT_NEWLINE_AFTER - Add blank line after prompt (before command output)
# Default: 0 (disabled)
# Set to 1 to add spacing between prompt and command output
# Example: export ZUSH_PROMPT_NEWLINE_AFTER=1
typeset -g ZUSH_PROMPT_NEWLINE_AFTER="${ZUSH_PROMPT_NEWLINE_AFTER:-0}"

# Function to switch themes dynamically
zush-theme() {
    local theme_name="$1"

    if [[ -z "$theme_name" ]]; then
        echo "Current theme: ${ZUSH_CURRENT_THEME}"
        echo ""
        echo "Available themes: dcs, minimal, powerline, split"
        echo "Custom themes in ~/.config/zush/themes/"
        echo ""
        echo "Usage: zush-theme <theme-name>"
        return 0
    fi

    ZUSH_CURRENT_THEME="$theme_name"
    export ZUSH_CURRENT_THEME
    echo "✓ Switched to theme: ${ZUSH_CURRENT_THEME}"
    zle && zle reset-prompt
}

# State tracking
typeset -g ZUSH_LAST_EXIT_CODE=0
typeset -g ZUSH_CMD_START_TIME=0
typeset -g ZUSH_CMD_DURATION=0
typeset -g ZUSH_PROMPT_RENDERED=0
typeset -g ZUSH_PROMPT_LINES=2  # Number of lines in the current prompt (dynamically calculated)

# Preexec hook - called before command execution (only when a command is entered)
# Arguments: $1 = command line, $2 = command string, $3 = expanded command
zush_preexec() {
    ZUSH_CMD_START_TIME=$EPOCHREALTIME

    # Mark that preexec was called (a command was entered)
    ZUSH_PROMPT_RENDERED=0

    # Capture the command that's about to be executed
    local cmd="$1"

    # Build minimal context for transient prompt
    local theme_args=""
    if [[ -n "$ZUSH_CURRENT_THEME" ]]; then
        theme_args="--theme $ZUSH_CURRENT_THEME"
    fi

    local context_json=$(cat <<EOF
{
    "time": "$(date +%H:%M:%S)"
}
EOF
    )

    # Render transient prompt (without Zsh escaping for raw terminal output)
    # Use --quiet to suppress error messages (already shown in main prompt)
    local transient_prompt=$($ZUSH_PROMPT_BIN --template transient --format raw --quiet $theme_args prompt \
        --context "$context_json" \
        --exit-code $ZUSH_LAST_EXIT_CODE \
        --execution-time $ZUSH_CMD_DURATION)

    # TODO: Make this dynamic based on theme
    # For now, hardcoded to 3 for splug theme
    local prompt_lines=3

    # Move cursor up to beginning of prompt, clear lines, and print transient version + command
    # \e[<n>A moves cursor up n lines
    # \e[0G moves cursor to beginning of line
    # \e[0J clears from cursor to end of screen
    printf '\e[%dA\e[0G\e[0J%s%s\n' "$prompt_lines" "$transient_prompt" "$cmd"

    # Add newline after prompt if configured (before command output)
    if [[ $ZUSH_PROMPT_NEWLINE_AFTER -eq 1 ]]; then
        print
    fi
}

# Precmd hook - called before prompt display
zush_precmd() {
    ZUSH_LAST_EXIT_CODE=$?

    # Calculate command duration
    if [[ $ZUSH_CMD_START_TIME -gt 0 ]]; then
        local end_time=$EPOCHREALTIME
        ZUSH_CMD_DURATION=$(( end_time - ZUSH_CMD_START_TIME ))
        ZUSH_CMD_START_TIME=0
    else
        ZUSH_CMD_DURATION=0
    fi

    # If preexec wasn't called (user just pressed Enter without typing a command),
    # convert the previous prompt to transient
    if [[ $ZUSH_PROMPT_RENDERED -eq 1 ]]; then
        # Build minimal context for transient prompt
        local theme_args=""
        if [[ -n "$ZUSH_CURRENT_THEME" ]]; then
            theme_args="--theme $ZUSH_CURRENT_THEME"
        fi

        local context_json=$(cat <<EOF
{
    "time": "$(date +%H:%M:%S)"
}
EOF
        )

        # Render transient prompt (empty command line)
        # Use --quiet to suppress error messages (already shown in main prompt)
        local transient_prompt=$($ZUSH_PROMPT_BIN --template transient --format raw --quiet $theme_args prompt \
            --context "$context_json" \
            --exit-code $ZUSH_LAST_EXIT_CODE \
            --execution-time 0)

        # Move cursor up to beginning of prompt, clear lines, and print transient version
        # TODO: Make this dynamic based on theme
        local prompt_lines=3
        printf '\e[%dA\e[0G\e[0J%s\n' "$prompt_lines" "$transient_prompt"
    fi

    # Mark that we're about to render a new prompt
    # preexec will set this to 0 if a command is executed
    ZUSH_PROMPT_RENDERED=1

    # Add newline before prompt if configured
    if [[ $ZUSH_PROMPT_NEWLINE_BEFORE -eq 1 ]]; then
        print
    fi
}

# Generate prompt
zush_prompt() {
    # Build context JSON
    # NOTE: Git status is now handled natively in Rust for much better performance
    # The Rust binary reads .git directory directly instead of spawning git commands
    local context_json=$(cat <<EOF
{
    "pwd": "$PWD",
    "pwd_short": "${PWD/#$HOME/~}",
    "user": "$USER",
    "host": "$HOST",
    "shell": "zsh",
    "ssh": "${SSH_CONNECTION:+true}",
    "virtual_env": "${VIRTUAL_ENV:+$(basename $VIRTUAL_ENV)}",
    "jobs": "$(jobs | wc -l | tr -d ' ')",
    "history_number": "$HISTCMD",
    "time": "$(date +%H:%M:%S)"
}
EOF
    )

    # Use ZUSH_CURRENT_THEME if set
    local theme_args=""
    if [[ -n "$ZUSH_CURRENT_THEME" ]]; then
        theme_args="--theme $ZUSH_CURRENT_THEME"
    fi

    # Always use main template for the new prompt
    # The transient prompt is rendered in preexec for the previous prompt
    $ZUSH_PROMPT_BIN --template main --format zsh $theme_args prompt \
        --context "$context_json" \
        --exit-code $ZUSH_LAST_EXIT_CODE \
        --execution-time $ZUSH_CMD_DURATION
}

# Setup hooks
add-zsh-hook preexec zush_preexec
add-zsh-hook precmd zush_precmd

# Handle terminal resize - force prompt redraw
TRAPWINCH() {
    zle && zle reset-prompt
}

# Set prompts
setopt PROMPT_SUBST
PROMPT='$(zush_prompt)'

# Never use RPROMPT - all right-aligned content is handled inline on the first line
RPROMPT=''
"#;

    println!("{}", script);
    Ok(())
}

fn print_default_config() -> Result<()> {
    let config = r##"# Zush Prompt Configuration

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

    println!("{}", config);
    Ok(())
}

fn load_theme(theme_name: &str) -> Result<String> {
    // Check if it's a path to a custom theme
    let theme_path = if theme_name.contains('/') || theme_name.contains('.') {
        PathBuf::from(theme_name)
    } else {
        // Look for theme in themes directory
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let theme_file = format!("{}.toml", theme_name);
        home.join(".config").join("zush").join("themes").join(theme_file)
    };

    if theme_path.exists() {
        fs::read_to_string(&theme_path)
            .with_context(|| format!("Failed to read theme file: {:?}", theme_path))
    } else {
        Err(anyhow::anyhow!("Theme file not found: {:?}", theme_path))
    }
}

fn render_prompt(
    cli: &Cli,
    context_json: Option<&str>,
    exit_code: Option<i32>,
    execution_time: Option<f64>,
) -> Result<()> {
    // Load main configuration
    let config_path = cli.config.clone().or_else(|| {
        // Try .config first, then fall back to platform default
        let home = dirs::home_dir()?;
        let config_file = home.join(".config").join("zush").join("config.toml");
        if config_file.exists() {
            Some(config_file)
        } else {
            dirs::config_dir().map(|d| d.join("zush").join("config.toml"))
        }
    });

    let config_str = if let Some(path) = &config_path {
        if path.exists() {
            fs::read_to_string(path).ok()
        } else {
            None
        }
    } else {
        None
    };

    // Determine which theme to load
    let theme_str = if let Some(theme_name) = &cli.theme {
        // CLI argument takes precedence
        load_theme(theme_name).ok()
    } else if let Some(config) = &config_str {
        // Parse config to get theme name
        if let Ok(parsed) = toml::from_str::<toml::Value>(config) {
            if let Some(theme_name) = parsed.get("theme").and_then(|v| v.as_str()) {
                load_theme(theme_name).ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Create template engine
    let mut engine = TemplateEngine::new()?;

    // Parse colors from theme/config FIRST, before loading templates
    // This allows templates to use named colors in (fg color) and (bg color) tags
    let mut colors_for_preprocessing = HashMap::new();
    let source = theme_str.as_ref().or(config_str.as_ref());
    if let Some(toml_str) = source {
        if let Ok(parsed) = toml::from_str::<toml::Value>(toml_str) {
            if let Some(colors_table) = parsed.get("colors").and_then(|v| v.as_table()) {
                for (key, value) in colors_table {
                    if let Some(color_str) = value.as_str() {
                        colors_for_preprocessing.insert(key.clone(), color_str.to_string());
                    }
                }
            }
        }
    }

    // Set colors in engine for preprocessing
    engine.set_colors(colors_for_preprocessing);

    // Parse symbols from theme/config for @symbol_name shortcuts
    // This allows templates to use @symbol_name instead of {{symbols.symbol_name}}
    let mut symbols_for_preprocessing = HashMap::new();
    if let Some(toml_str) = source {
        if let Ok(parsed) = toml::from_str::<toml::Value>(toml_str) {
            if let Some(symbols_table) = parsed.get("symbols").and_then(|v| v.as_table()) {
                for (key, value) in symbols_table {
                    if let Some(symbol_str) = value.as_str() {
                        // Parse Unicode escape sequences
                        let parsed_symbol = parse_unicode_escapes(symbol_str);
                        symbols_for_preprocessing.insert(key.clone(), parsed_symbol);
                    }
                }
            }
        }
    }

    // Set symbols in engine for preprocessing
    engine.set_symbols(symbols_for_preprocessing);

    // Parse segments from theme/config for reusable segment definitions
    // This allows templates to define segments in a [segments] block
    let mut segments_for_preprocessing = HashMap::new();
    if let Some(toml_str) = source {
        if let Ok(parsed) = toml::from_str::<toml::Value>(toml_str) {
            if let Some(segments_table) = parsed.get("segments").and_then(|v| v.as_table()) {
                for (segment_name, segment_data) in segments_table {
                    if let Some(segment_props) = segment_data.as_table() {
                        // Extract content (required)
                        if let Some(content) = segment_props.get("content").and_then(|v| v.as_str()) {
                            // Normalize multiline content: strip leading/trailing whitespace from each line
                            // and join into a single line. This allows readable multiline TOML without
                            // inserting actual newlines into the template.
                            let normalized_content = content
                                .lines()
                                .map(|line| line.trim())
                                .filter(|line| !line.is_empty())
                                .collect::<Vec<_>>()
                                .join("");

                            let mut segment = template::SegmentDef::new(
                                segment_name.clone(),
                                normalized_content
                            );

                            // Add optional properties using builder methods
                            if let Some(bg) = segment_props.get("bg").and_then(|v| v.as_str()) {
                                segment = segment.with_bg(bg.to_string());
                            }
                            if let Some(fg) = segment_props.get("fg").and_then(|v| v.as_str()) {
                                segment = segment.with_fg(fg.to_string());
                            }
                            if let Some(sep) = segment_props.get("sep").and_then(|v| v.as_str()) {
                                segment = segment.with_sep(sep.to_string());
                            }
                            if let Some(left_cap) = segment_props.get("left_cap").and_then(|v| v.as_str()) {
                                segment = segment.with_left_cap(left_cap.to_string());
                            }

                            segments_for_preprocessing.insert(segment_name.clone(), segment);
                        }
                    }
                }
            }
        }
    }

    // Add pre-defined segments to engine for preprocessing
    if !segments_for_preprocessing.is_empty() {
        engine.add_segments(segments_for_preprocessing);
    }

    // Load templates from theme, config, or defaults
    let theme_or_config = theme_str.as_ref().or(config_str.as_ref());

    if let Some(toml_str) = theme_or_config {
        if let Err(e) = engine.load_templates_from_config(toml_str) {
            // If loading fails, print stylized error (unless quiet mode) and register defaults
            if !cli.quiet {
                eprintln!("\n\x1b[38;2;243;139;168m\x1b[1m✖ Template Loading Error\x1b[22m\x1b[39m");
                eprintln!("\x1b[38;2;249;226;175m{}\x1b[39m\n", e);
            }
            register_default_templates(&mut engine)?;
        }
    } else {
        // No theme or config, use defaults
        register_default_templates(&mut engine)?;
    }

    // Build context
    let mut context = HashMap::new();

    // Add environment context
    if let Some(json_str) = context_json {
        if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
            if let Value::Object(map) = parsed {
                for (key, value) in map {
                    context.insert(key, value);
                }
            }
        }
    }

    // Add command status
    context.insert("exit_code".to_string(), json!(exit_code.unwrap_or(0)));

    // Convert execution time from seconds to milliseconds for display
    let exec_time_ms = execution_time.unwrap_or(0.0) * 1000.0;
    context.insert("execution_time".to_string(), json!(exec_time_ms));
    context.insert("execution_time_ms".to_string(), json!(exec_time_ms as i64));
    context.insert("execution_time_s".to_string(), json!(execution_time.unwrap_or(0.0)));

    // Collect environment information natively (avoids shell overhead)
    // Get current time (replaces date +%H:%M:%S)
    if !context.contains_key("time") {
        use chrono::Local;
        let time = Local::now().format("%H:%M:%S").to_string();
        context.insert("time".to_string(), json!(time));
    }

    // Get user and hostname from environment (much faster than shell variables)
    if !context.contains_key("user") {
        if let Ok(user) = std::env::var("USER") {
            context.insert("user".to_string(), json!(user));
        }
    }
    if !context.contains_key("host") {
        if let Ok(host) = std::env::var("HOST") {
            context.insert("host".to_string(), json!(host));
        } else if let Ok(hostname) = std::env::var("HOSTNAME") {
            context.insert("host".to_string(), json!(hostname));
        } else {
            // Fallback to whoami crate for hostname
            let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "localhost".to_string());
            context.insert("host".to_string(), json!(hostname));
        }
    }

    // Get PWD from environment if not provided
    if !context.contains_key("pwd") {
        if let Ok(pwd) = std::env::var("PWD") {
            context.insert("pwd".to_string(), json!(pwd.clone()));
            // Also create pwd_short
            if let Ok(home) = std::env::var("HOME") {
                let pwd_short = pwd.replace(&home, "~");
                context.insert("pwd_short".to_string(), json!(pwd_short));
            } else {
                context.insert("pwd_short".to_string(), json!(pwd));
            }
        } else {
            // Fallback to current_dir
            if let Ok(pwd) = std::env::current_dir() {
                let pwd_str = pwd.display().to_string();
                context.insert("pwd".to_string(), json!(pwd_str.clone()));
                if let Ok(home) = std::env::var("HOME") {
                    let pwd_short = pwd_str.replace(&home, "~");
                    context.insert("pwd_short".to_string(), json!(pwd_short));
                } else {
                    context.insert("pwd_short".to_string(), json!(pwd_str));
                }
            }
        }
    }

    // Count background jobs from environment (replaces jobs | wc -l)
    // This is tricky - we need to count from parent shell's job table
    // For now, allow shell to pass it, but provide a default
    if !context.contains_key("jobs") {
        // Try to count from /proc (Linux) or fallback to 0
        #[cfg(target_os = "linux")]
        {
            // Count child processes with state 'T' (stopped) or 'S' (sleeping in background)
            // This is an approximation
            context.entry("jobs".to_string()).or_insert(json!(0));
        }
        #[cfg(not(target_os = "linux"))]
        {
            context.entry("jobs".to_string()).or_insert(json!(0));
        }
    }

    // Get git status natively (much faster than shell git commands)
    // This reads .git directory directly instead of spawning git processes
    if let Some(pwd) = context.get("pwd").and_then(|v| v.as_str()) {
        if let Some(git_status) = git::get_git_status(std::path::Path::new(pwd)) {
            let git_json = git::git_status_to_json(&git_status);
            if let Value::Object(git_map) = git_json {
                for (key, value) in git_map {
                    context.insert(key, value);
                }
            }
        }
    }

    // Ensure git status variables exist with defaults (if not in git repo)
    context.entry("git_branch".to_string()).or_insert(json!(""));
    context.entry("git_staged".to_string()).or_insert(json!(0));
    context.entry("git_modified".to_string()).or_insert(json!(0));
    context.entry("git_added".to_string()).or_insert(json!(0));
    context.entry("git_deleted".to_string()).or_insert(json!(0));
    context.entry("git_renamed".to_string()).or_insert(json!(0));
    context.entry("git_untracked".to_string()).or_insert(json!(0));
    context.entry("git_conflicted".to_string()).or_insert(json!(0));

    // Collect module information (Python, Node, Rust, Docker, etc.)
    // This is done natively for performance - context-aware detection
    if let Ok(module_context) = modules::ModuleContext::new() {
        let mut registry = modules::registry::ModuleRegistry::new();

        // Render all enabled modules that should display in current context
        let module_outputs = registry.render_all(&module_context);

        // Add module outputs to context
        let mut modules_data = Vec::new();
        for output in module_outputs {
            modules_data.push(json!({
                "id": output.id,
                "content": output.content,
            }));
        }

        if !modules_data.is_empty() {
            context.insert("modules".to_string(), json!(modules_data));
        }
    }

    // Load colors from theme/config or use defaults
    let mut colors = HashMap::new();

    // Try to parse colors from theme first, then config
    let source = theme_str.as_ref().or(config_str.as_ref());
    if let Some(toml_str) = source {
        if let Ok(parsed) = toml::from_str::<toml::Value>(toml_str) {
            if let Some(colors_table) = parsed.get("colors").and_then(|v| v.as_table()) {
                for (key, value) in colors_table {
                    if let Some(color_str) = value.as_str() {
                        colors.insert(key.clone(), json!(color_str));
                    }
                }
            }
        }
    }

    // Apply overrides from main config if theme was loaded
    if theme_str.is_some() && config_str.is_some() {
        if let Ok(parsed) = toml::from_str::<toml::Value>(config_str.as_ref().unwrap()) {
            if let Some(overrides) = parsed.get("overrides").and_then(|v| v.as_table()) {
                for (key, value) in overrides {
                    if key.starts_with("colors.") {
                        let color_key = key.strip_prefix("colors.").unwrap();
                        if let Some(color_str) = value.as_str() {
                            colors.insert(color_key.to_string(), json!(color_str));
                        }
                    }
                }
            }
        }
    }

    // Add default colors if not in config
    if colors.is_empty() {
        colors.insert("bg".to_string(), json!(tokyo_night::BG.to_hex()));
        colors.insert("fg".to_string(), json!(tokyo_night::FG.to_hex()));
        colors.insert("fg_dark".to_string(), json!(tokyo_night::FG_DARK.to_hex()));
        colors.insert("fg_dim".to_string(), json!(tokyo_night::FG_DIM.to_hex()));
        colors.insert("black".to_string(), json!(tokyo_night::BLACK.to_hex()));
        colors.insert("red".to_string(), json!(tokyo_night::RED.to_hex()));
        colors.insert("green".to_string(), json!(tokyo_night::GREEN.to_hex()));
        colors.insert("yellow".to_string(), json!(tokyo_night::YELLOW.to_hex()));
        colors.insert("blue".to_string(), json!(tokyo_night::BLUE.to_hex()));
        colors.insert("magenta".to_string(), json!(tokyo_night::MAGENTA.to_hex()));
        colors.insert("cyan".to_string(), json!(tokyo_night::CYAN.to_hex()));
        colors.insert("white".to_string(), json!(tokyo_night::WHITE.to_hex()));
        colors.insert("orange".to_string(), json!(tokyo_night::ORANGE.to_hex()));
        colors.insert("purple".to_string(), json!(tokyo_night::PURPLE.to_hex()));
        colors.insert("teal".to_string(), json!(tokyo_night::TEAL.to_hex()));
    }

    context.insert("colors".to_string(), json!(colors));

    // Load symbols from theme/config or use defaults
    let mut symbols = HashMap::new();

    // Try to parse symbols from theme first, then config
    let source = theme_str.as_ref().or(config_str.as_ref());
    if let Some(toml_str) = source {
        if let Ok(parsed) = toml::from_str::<toml::Value>(toml_str) {
            if let Some(symbols_table) = parsed.get("symbols").and_then(|v| v.as_table()) {
                for (key, value) in symbols_table {
                    if let Some(symbol_str) = value.as_str() {
                        // Parse Unicode escape sequences
                        let parsed_symbol = parse_unicode_escapes(symbol_str);
                        symbols.insert(key.clone(), json!(parsed_symbol));
                    }
                }
            }
        }
    }

    // Apply overrides from main config if theme was loaded
    if theme_str.is_some() && config_str.is_some() {
        if let Ok(parsed) = toml::from_str::<toml::Value>(config_str.as_ref().unwrap()) {
            if let Some(overrides) = parsed.get("overrides").and_then(|v| v.as_table()) {
                for (key, value) in overrides {
                    if key.starts_with("symbols.") {
                        let symbol_key = key.strip_prefix("symbols.").unwrap();
                        if let Some(symbol_str) = value.as_str() {
                            let parsed_symbol = parse_unicode_escapes(symbol_str);
                            symbols.insert(symbol_key.to_string(), json!(parsed_symbol));
                        }
                    }
                }
            }
        }
    }

    // Add default symbols if not in config
    if symbols.is_empty() {
        symbols.insert("prompt_arrow".to_string(), json!("❯"));
        symbols.insert("segment_separator".to_string(), json!(""));
        symbols.insert("git_branch".to_string(), json!(""));
        symbols.insert("git_dirty".to_string(), json!("✗"));
        symbols.insert("git_clean".to_string(), json!("✓"));
        symbols.insert("ssh".to_string(), json!(""));
        symbols.insert("root".to_string(), json!(""));
        symbols.insert("jobs".to_string(), json!(""));
        symbols.insert("error".to_string(), json!("✖"));
        symbols.insert("success".to_string(), json!("✓"));
    }

    context.insert("symbols".to_string(), json!(symbols));

    // Get terminal width directly from the terminal (not from shell)
    let terminal_width = if let Some((terminal_size::Width(w), _)) = terminal_size::terminal_size() {
        w as usize
    } else {
        // Fallback to context if terminal size detection fails
        context.get("terminal_width")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as usize
    };

    // Always set terminal_width in context for templates that might use it
    context.insert("terminal_width".to_string(), json!(terminal_width));

    // Set context in engine
    engine.set_context(context.clone());

    // Only build first_line for the main template (not for transient or other templates)
    if cli.template == "main" {
        // Pre-render left and right templates if they exist, and build complete first line in Rust
        // This bypasses the need for a line helper in templates (which had registration issues)
        let left_result = engine.render("left");
        let right_result = engine.render("right");

        // Build first_line only if both left and right templates render successfully
        // If right template is empty or fails, just use left content
        match (left_result, right_result) {
            (Ok(left_output), Ok(right_output)) if !right_output.trim().is_empty() => {
                // Both templates exist and right is not empty - build complete line with spacing
                let left_visible = TerminalBuffer::visible_width(&left_output);
                let right_visible = TerminalBuffer::visible_width(&right_output);
                let total_content = left_visible + right_visible;

                let first_line = if total_content >= terminal_width {
                    // No space for padding, just concatenate
                    format!("{}{}", left_output, right_output)
                } else {
                    // Add spacing between left and right
                    let spacing = terminal_width - total_content;
                    format!("{}{:width$}{}", left_output, "", right_output, width = spacing)
                };

                context.insert("first_line".to_string(), json!(first_line));
            }
            (Ok(left_output), _) => {
                // Only left template exists or right is empty - use left only
                context.insert("first_line".to_string(), json!(left_output));
            }
            _ => {
                // Neither template rendered successfully - leave first_line empty
                context.insert("first_line".to_string(), json!(""));
            }
        }

        // Update context with rendered templates
        engine.set_context(context);
    } else {
        // For non-main templates (like transient), explicitly set first_line to empty
        // to ensure it doesn't render anything if the template accidentally references it
        context.insert("first_line".to_string(), json!(""));
        engine.set_context(context);
    }

    // Render template with error handling
    let output = match engine.render(&cli.template) {
        Ok(result) => result,
        Err(e) => {
            // Display rendering error above the prompt
            eprintln!("\n\x1b[38;2;243;139;168m\x1b[1m✖ Template Rendering Error\x1b[22m\x1b[39m");
            eprintln!("\x1b[38;2;249;226;175m{}\x1b[39m\n", e);

            // Fall back to a minimal safe prompt with user@host and directory
            // Get these from env variables since context was already moved
            let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
            let pwd_short = if let Ok(pwd) = std::env::var("PWD") {
                if let Ok(home) = std::env::var("HOME") {
                    pwd.replace(&home, "~")
                } else {
                    pwd
                }
            } else {
                "~".to_string()
            };
            format!("\x1b[38;2;137;180;250m{}\x1b[39m in \x1b[38;2;189;147;249m{}\x1b[39m\n\x1b[38;2;243;139;168m❯\x1b[39m ", user, pwd_short)
        }
    };

    // Format output based on requested format
    match cli.format.as_str() {
        "zsh" => {
            // Convert to Zsh format with proper escaping
            print!("{}", convert_to_zsh_format(&output));
        }
        "raw" => {
            // Raw ANSI output
            print!("{}", output);
        }
        "debug" => {
            // Debug output showing escape codes
            println!("Template: {}", cli.template);
            println!("Output: {:?}", output);
            println!("Visible width: {}", TerminalBuffer::visible_width(&output));
        }
        _ => {
            print!("{}", output);
        }
    }

    Ok(())
}

fn register_default_templates(engine: &mut TemplateEngine) -> Result<()> {
    // Main prompt - two line format with status, user, directory on first line and arrow on second
    engine.register_template("main", r#"(fg #9ece6a)✓(/fg) (bold)(fg #7aa2f7){{user}}(/fg)(/bold) (fg #c0caf5)in(/fg) (fg #bb9af7){{pwd_short}}(/fg)
(fg #7aa2f7)❯(/fg) "#)?;

    // Left template (empty for default)
    engine.register_template("left", "")?;

    // Right template (empty for default)
    engine.register_template("right", "")?;

    // Transient prompt
    engine.register_template("transient", r#"(dim){{time}}(/dim)
(fg #7aa2f7)❯(/fg) "#)?;

    Ok(())
}

/// Parse Unicode escape sequences in a string (e.g., "\ue0b0" -> actual character)
fn parse_unicode_escapes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut pending_surrogate: Option<u32> = None;

    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'u') {
            chars.next(); // consume 'u'

            // Try to parse the next 4 hex digits
            let mut hex = String::new();
            for _ in 0..4 {
                if let Some(hex_char) = chars.next() {
                    if hex_char.is_ascii_hexdigit() {
                        hex.push(hex_char);
                    } else {
                        // Not a valid hex sequence, add what we have
                        result.push('\\');
                        result.push('u');
                        result.push_str(&hex);
                        result.push(hex_char);
                        break;
                    }
                }
            }

            if hex.len() == 4 {
                // Parse the hex value
                if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                    // Check if it's a surrogate pair
                    if (0xD800..=0xDBFF).contains(&code_point) {
                        // High surrogate
                        pending_surrogate = Some(code_point);
                        continue;
                    } else if (0xDC00..=0xDFFF).contains(&code_point) {
                        // Low surrogate
                        if let Some(high) = pending_surrogate {
                            // Combine surrogates to get actual code point
                            let combined = 0x10000 + ((high - 0xD800) << 10) + (code_point - 0xDC00);
                            if let Some(unicode_char) = char::from_u32(combined) {
                                result.push(unicode_char);
                                pending_surrogate = None;
                                continue;
                            }
                        }
                    } else if let Some(unicode_char) = char::from_u32(code_point) {
                        result.push(unicode_char);
                        continue;
                    }
                }
                // If parsing failed, add the original sequence
                result.push('\\');
                result.push('u');
                result.push_str(&hex);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn convert_to_zsh_format(ansi_str: &str) -> String {
    // Wrap ANSI escape sequences in %{...%} for Zsh
    let mut result = String::new();
    let mut in_escape = false;
    let mut escape_seq = String::new();

    for ch in ansi_str.chars() {
        if ch == '\x1b' {
            in_escape = true;
            escape_seq.clear();
            escape_seq.push(ch);
        } else if in_escape {
            escape_seq.push(ch);
            if ch == 'm' {
                // End of color escape sequence
                result.push_str("%{");
                result.push_str(&escape_seq);
                result.push_str("%}");
                in_escape = false;
            }
        } else {
            result.push(ch);
        }
    }

    result
}
