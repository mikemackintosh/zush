mod buffer;
mod color;
mod template;
mod segments;
mod config;

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

# Path to the zush-prompt binary
ZUSH_PROMPT_BIN="${ZUSH_PROMPT_BIN:-zush-prompt}"

# Theme management
typeset -g ZUSH_CURRENT_THEME="${ZUSH_CURRENT_THEME:-split}"

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

# Preexec hook - called before command execution (only when a command is entered)
# Arguments: $1 = command line, $2 = command string, $3 = expanded command
zush_preexec() {
    ZUSH_CMD_START_TIME=$EPOCHREALTIME

    # Mark that preexec was called (a command was entered)
    ZUSH_PROMPT_RENDERED=0

    # Capture the command that's about to be executed
    local cmd="$1"

    # Rewrite the current prompt to transient version before command executes
    # Count how many lines the prompt takes (for split theme it's 2 lines)
    local prompt_lines=2

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
    local transient_prompt=$($ZUSH_PROMPT_BIN --template transient --format raw $theme_args prompt \
        --context "$context_json" \
        --exit-code $ZUSH_LAST_EXIT_CODE \
        --execution-time $ZUSH_CMD_DURATION)

    # Move cursor up to beginning of prompt, clear lines, and print transient version + command
    # \e[<n>A moves cursor up n lines
    # \e[0G moves cursor to beginning of line
    # \e[0J clears from cursor to end of screen
    printf '\e[%dA\e[0G\e[0J%s%s\n' "$prompt_lines" "$transient_prompt" "$cmd"
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
        local transient_prompt=$($ZUSH_PROMPT_BIN --template transient --format raw $theme_args prompt \
            --context "$context_json" \
            --exit-code $ZUSH_LAST_EXIT_CODE \
            --execution-time 0)

        # Move cursor up to beginning of prompt, clear lines, and print transient version
        local prompt_lines=2
        printf '\e[%dA\e[0G\e[0J%s\n' "$prompt_lines" "$transient_prompt"
    fi

    # Mark that we're about to render a new prompt
    # preexec will set this to 0 if a command is executed
    ZUSH_PROMPT_RENDERED=1
}

# Generate prompt
zush_prompt() {
    # Collect git status if in a git repo
    local git_branch=$(git branch --show-current 2>/dev/null || echo '')
    local git_staged=0
    local git_modified=0
    local git_added=0
    local git_deleted=0
    local git_renamed=0
    local git_untracked=0
    local git_conflicted=0

    if [[ -n "$git_branch" ]]; then
        # Parse git status --porcelain output
        while IFS= read -r line; do
            local index_status="${line:0:1}"
            local work_status="${line:1:1}"

            # Index/staged changes
            case "$index_status" in
                "M") ((git_staged++)) ;;
                "A") ((git_added++)); ((git_staged++)) ;;
                "D") ((git_deleted++)); ((git_staged++)) ;;
                "R") ((git_renamed++)); ((git_staged++)) ;;
            esac

            # Working tree changes
            case "$work_status" in
                "M") ((git_modified++)) ;;
                "D") ((git_deleted++)) ;;
            esac

            # Untracked and conflicted
            if [[ "$index_status" == "?" ]]; then
                ((git_untracked++))
            elif [[ "$index_status" == "U" || "$work_status" == "U" ]]; then
                ((git_conflicted++))
            fi
        done < <(git status --porcelain 2>/dev/null)
    fi

    local context_json=$(cat <<EOF
{
    "pwd": "$PWD",
    "pwd_short": "${PWD/#$HOME/~}",
    "user": "$USER",
    "host": "$HOST",
    "shell": "zsh",
    "git_branch": "$git_branch",
    "git_staged": $git_staged,
    "git_modified": $git_modified,
    "git_added": $git_added,
    "git_deleted": $git_deleted,
    "git_renamed": $git_renamed,
    "git_untracked": $git_untracked,
    "git_conflicted": $git_conflicted,
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

# Right prompt
zush_rprompt() {
    local context_json=$(cat <<EOF
{
    "pwd": "$PWD",
    "time": "$(date +%H:%M:%S)"
}
EOF
    )

    # Use ZUSH_CURRENT_THEME if set
    local theme_args=""
    if [[ -n "$ZUSH_CURRENT_THEME" ]]; then
        theme_args="--theme $ZUSH_CURRENT_THEME"
    fi

    $ZUSH_PROMPT_BIN --template "right" --format zsh $theme_args prompt \
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

# Only set RPROMPT for themes that use it (not split, which handles right content inline)
case "$ZUSH_CURRENT_THEME" in
    split)
        # Split theme handles right content inline, don't use RPROMPT
        RPROMPT=''
        ;;
    *)
        # Other themes use RPROMPT
        RPROMPT='$(zush_rprompt)'
        ;;
esac
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

    // Load templates from theme, config, or defaults
    let theme_or_config = theme_str.as_ref().or(config_str.as_ref());

    if let Some(toml_str) = theme_or_config {
        if engine.load_templates_from_config(toml_str).is_err() {
            // If loading fails, register defaults
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

    // Ensure git status variables exist with defaults
    context.entry("git_staged".to_string()).or_insert(json!(0));
    context.entry("git_modified".to_string()).or_insert(json!(0));
    context.entry("git_added".to_string()).or_insert(json!(0));
    context.entry("git_deleted".to_string()).or_insert(json!(0));
    context.entry("git_renamed".to_string()).or_insert(json!(0));
    context.entry("git_untracked".to_string()).or_insert(json!(0));
    context.entry("git_conflicted".to_string()).or_insert(json!(0));

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

    // Pre-render left and right templates if they exist, and build complete first line in Rust
    // This bypasses the need for a line helper in templates (which had registration issues)
    let left_result = engine.render("left");
    let right_result = engine.render("right");

    if let (Ok(left_output), Ok(right_output)) = (left_result, right_result) {
        // Use the terminal width we detected above

        // Calculate visible widths (stripping ANSI codes)
        let left_visible = TerminalBuffer::visible_width(&left_output);
        let right_visible = TerminalBuffer::visible_width(&right_output);
        let total_content = left_visible + right_visible;

        // Build the complete first line with proper spacing
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

    // Update context with rendered templates
    engine.set_context(context);

    // Render template
    let output = engine.render(&cli.template)?;

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
    // Main prompt
    engine.register_template("main", r#"{{color colors.green symbols.success}} {{bold (color colors.blue user)}} {{color colors.white "in"}} {{color colors.magenta pwd}}
{{color colors.blue symbols.prompt_arrow}} "#)?;

    // Transient prompt
    engine.register_template("transient", r#"{{dim time}}
{{color colors.blue symbols.prompt_arrow}} "#)?;

    // Right prompt
    engine.register_template("right", r#"{{#if execution_time}}{{color colors.yellow execution_time}}s {{/if}}{{dim time}}"#)?;

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
