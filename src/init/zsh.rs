//! Zsh shell integration script
//!
//! This module contains the Zsh initialization script that sets up:
//! - Theme switching functions
//! - Prompt hooks (preexec, precmd)
//! - Transient prompt support
//! - Command timing

/// The Zsh initialization script
pub const INIT_SCRIPT: &str = r#"#!/usr/bin/env zsh
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

# ZUSH_THEME - Which theme to use (optional)
# Default: Uses theme from ~/.config/zush/config.toml, or "minimal" if not set
# Options: dcs, minimal, powerline, split, or path to custom theme
# Example: export ZUSH_THEME="powerline"
# Runtime: Use `zush-theme <name>` to switch themes
# Note: Only set this if you want to override config.toml's theme setting
typeset -g ZUSH_THEME="${ZUSH_THEME:-}"

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

# ZUSH_GIT_MINIMAL - Minimal git mode (for huge repositories 1GB+)
# Default: 0 (disabled)
# Set to 1 to only show branch name, skip all git status checks
# This dramatically improves performance in very large repositories
# Example: export ZUSH_GIT_MINIMAL=1
# Note: This only affects git status display, not repository detection

# ZUSH_GIT_DISABLE_UNTRACKED - Disable untracked file scanning
# Default: 0 (disabled)
# Set to 1 to skip counting untracked files (improves performance with many untracked files)
# Example: export ZUSH_GIT_DISABLE_UNTRACKED=1

# ZUSH_DISABLE_MODULES - Disable all language/environment modules
# Default: 0 (disabled)
# Set to 1 to disable all modules (Python, Node, Go, Ruby, Rust, Docker detection)
# This can significantly improve performance if you don't use module indicators
# Example: export ZUSH_DISABLE_MODULES=1

# ZUSH_DISABLE_<MODULE> - Disable specific modules
# Set to 1 to disable a specific module (PYTHON, NODE, GO, RUBY, RUST, DOCKER)
# Example: export ZUSH_DISABLE_PYTHON=1
# Example: export ZUSH_DISABLE_NODE=1

# ============================================================================
# HISTORY CONFIGURATION - Ensure history is saved and shared across sessions
# These are set only if not already configured, to avoid overriding user prefs
# ============================================================================

# Set history file location if not set
[[ -z "$HISTFILE" ]] && export HISTFILE="${HOME}/.zsh_history"

# Set reasonable history sizes if not set or too small
[[ -z "$HISTSIZE" || "$HISTSIZE" -lt 1000 ]] && export HISTSIZE=50000
[[ -z "$SAVEHIST" || "$SAVEHIST" -lt 1000 ]] && export SAVEHIST=50000

# History options - append and share so sessions don't overwrite each other
# IMPORTANT: SHARE_HISTORY and INC_APPEND_HISTORY are mutually exclusive.
# SHARE_HISTORY already appends incrementally AND imports from other sessions.
# Having both set causes history entries to be lost (especially for failed commands).
# We explicitly disable INC_APPEND_HISTORY in case the user's .zshrc set it.
setopt SHARE_HISTORY          # Share history between all sessions (includes incremental append)
setopt NO_INC_APPEND_HISTORY  # Must be off when SHARE_HISTORY is on
setopt HIST_IGNORE_DUPS       # Ignore duplicated commands in history list
setopt HIST_IGNORE_SPACE      # Ignore commands that start with space
setopt HIST_EXPIRE_DUPS_FIRST # Delete duplicates first when HISTFILE exceeds HISTSIZE
setopt EXTENDED_HISTORY       # Record timestamp in history

# Function to switch themes dynamically
zush-theme() {
    local theme_name="$1"

    # If no argument, show current theme and available themes
    if [[ -z "$theme_name" ]]; then
        echo "Current theme: ${ZUSH_THEME}"
        echo ""
        echo "Available themes:"
        echo "  Built-in: dcs, minimal, powerline, split"
        echo "  Custom themes in ~/.config/zush/themes/"
        echo ""
        echo "Usage: zush-theme <theme-name>"
        echo "       zush-theme list [--preview]  # List all themes (with previews)"
        echo "       zush-theme preview [--compact]  # Preview all themes with scenarios"
        echo "       zush-theme reset             # Reset to config default"
        return 0
    fi

    # Special commands
    case "$theme_name" in
        list)
            shift
            _zush_theme_list "$@"
            return 0
            ;;
        preview)
            shift
            _zush_theme_preview_all "$@"
            return 0
            ;;
        reset)
            unset ZUSH_THEME
            ZUSH_THEME=""
            echo "✓ Reset to config default (reload shell to apply)"
            zle && zle reset-prompt
            return 0
            ;;
    esac

    # Switch to theme
    ZUSH_THEME="$theme_name"
    export ZUSH_THEME
    echo "✓ Switched to theme: ${ZUSH_THEME}"
    zle && zle reset-prompt
}

# Helper function to list themes with descriptions
_zush_theme_list() {
    local show_preview=false
    if [[ "$1" == "--preview" ]]; then
        show_preview=true
    fi

    local bold="\e[1m" dim="\e[2m" blue="\e[34m" cyan="\e[36m"
    local green="\e[32m" yellow="\e[33m" reset="\e[0m"

    echo ""
    echo -e "${bold}${cyan}╔══════════════════════════════════════════════════════╗${reset}"
    echo -e "${bold}${cyan}║${reset}           ${bold}Available Zush Themes${reset}                 ${bold}${cyan}║${reset}"
    echo -e "${bold}${cyan}╚══════════════════════════════════════════════════════╝${reset}"
    echo ""

    local themes_dir=~/.config/zush/themes
    local context='{"pwd":"~/projects/app","pwd_short":"~/projects/app","user":"'$USER'","git_branch":"main","time":"'$(date +%H:%M:%S)'","git_modified":2,"git_staged":1}'

    local theme_files=()
    for builtin in dcs minimal powerline split; do
        [[ -f "$themes_dir/${builtin}.toml" ]] && theme_files+=("$themes_dir/${builtin}.toml")
    done
    for theme_file in $themes_dir/*.toml(N); do
        local name="${theme_file:t:r}"
        [[ "$name" != "dcs" && "$name" != "minimal" && "$name" != "powerline" && "$name" != "split" ]] && theme_files+=("$theme_file")
    done

    for theme_file in "${theme_files[@]}"; do
        [[ ! -f "$theme_file" ]] && continue
        local name="${theme_file:t:r}"
        local description=$(grep '^description' "$theme_file" 2>/dev/null | sed 's/description = "\(.*\)"/\1/')
        local author=$(grep '^author' "$theme_file" 2>/dev/null | sed 's/author = "\(.*\)"/\1/')
        local version=$(grep '^version' "$theme_file" 2>/dev/null | sed 's/version = "\(.*\)"/\1/')

        local marker=""
        [[ "$name" == "$ZUSH_THEME" ]] && marker=" ${green}→ (active)${reset}"

        echo -e "${bold}${blue}${name}${reset}${marker}"
        [[ -n "$description" ]] && echo -e "  ${dim}${description}${reset}"
        [[ -n "$author" ]] && echo -e "  ${dim}Author: ${author}${reset}"
        [[ -n "$version" ]] && echo -e "  ${dim}Version: ${version}${reset}"
        echo -e "  ${yellow}Command:${reset} zush-theme ${name}"

        if [[ "$show_preview" == true ]]; then
            echo ""
            echo -e "  ${dim}Preview:${reset}"
            echo -n "  "
            zush-prompt --theme "$name" --format raw prompt --context "$context" --exit-code 0 2>/dev/null | head -1
        fi
        echo ""
    done

    echo -e "${dim}Tip: Use 'zush-theme list --preview' to see theme previews${reset}"
    echo -e "${dim}Tip: Use 'zush-theme preview' for detailed multi-scenario previews${reset}"
}

# Helper function to preview all themes
_zush_theme_preview_all() {
    local compact=false
    [[ "$1" == "--compact" ]] && compact=true

    local bold="\e[1m" dim="\e[2m" cyan="\e[36m" yellow="\e[33m" magenta="\e[35m" reset="\e[0m"

    echo ""
    echo -e "${bold}${cyan}╔══════════════════════════════════════════════════════╗${reset}"
    echo -e "${bold}${cyan}║${reset}              ${bold}Zush Theme Preview${reset}                  ${bold}${cyan}║${reset}"
    echo -e "${bold}${cyan}╚══════════════════════════════════════════════════════╝${reset}"
    echo ""

    local current_time=$(date +%H:%M:%S)
    local scenarios=(
        "Success|{\"pwd\":\"~/projects/app\",\"pwd_short\":\"~/projects/app\",\"user\":\"$USER\",\"git_branch\":\"main\",\"time\":\"$current_time\"}|0"
        "With Git Changes|{\"pwd\":\"~/code/zush\",\"pwd_short\":\"~/code/zush\",\"user\":\"$USER\",\"git_branch\":\"feature/preview\",\"git_modified\":3,\"git_staged\":1,\"git_untracked\":2,\"time\":\"$current_time\"}|0"
        "Error State|{\"pwd\":\"~/projects/app\",\"pwd_short\":\"~/projects/app\",\"user\":\"$USER\",\"git_branch\":\"main\",\"time\":\"$current_time\"}|1"
    )

    local themes_dir=~/.config/zush/themes
    local theme_files=()
    for builtin in dcs minimal powerline split; do
        [[ -f "$themes_dir/${builtin}.toml" ]] && theme_files+=("$themes_dir/${builtin}.toml")
    done
    for theme_file in $themes_dir/*.toml(N); do
        local name="${theme_file:t:r}"
        [[ "$name" != "dcs" && "$name" != "minimal" && "$name" != "powerline" && "$name" != "split" ]] && theme_files+=("$theme_file")
    done

    for theme_file in "${theme_files[@]}"; do
        [[ ! -f "$theme_file" ]] && continue
        local name="${theme_file:t:r}"

        echo -e "${bold}${magenta}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${reset}"
        echo -e "${bold}${yellow}Theme: ${name}${reset}"
        echo -e "${bold}${magenta}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${reset}"
        echo ""

        for scenario in "${scenarios[@]}"; do
            local label="${scenario%%|*}"
            local rest="${scenario#*|}"
            local context="${rest%%|*}"
            local exit_code="${rest##*|}"

            [[ "$compact" == false ]] && echo -e "  ${dim}${label}:${reset}"

            local prompt_output=$(zush-prompt --theme "$name" --format raw prompt --context "$context" --exit-code "$exit_code" 2>/dev/null)

            if [[ "$compact" == true ]]; then
                echo "$prompt_output" | head -1
            else
                echo "$prompt_output" | while IFS= read -r line; do echo "    $line"; done
                echo ""
            fi
        done
        echo ""
    done

    echo -e "${dim}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${reset}"
    echo -e "${dim}Tip: Use 'zush-theme <name>' to activate a theme${reset}"
    echo -e "${dim}Tip: Use 'zush-theme preview --compact' for single-line previews${reset}"
    echo ""
}

# Tab completion for zush-theme
_zush_theme_completion() {
    local -a commands themes
    commands=('list:List all available themes' 'preview:Preview all themes' 'reset:Reset to default')

    if [[ -d ~/.config/zush/themes ]]; then
        for theme_file in ~/.config/zush/themes/*.toml(N); do
            local name="${theme_file:t:r}"
            local desc=$(grep '^description' "$theme_file" 2>/dev/null | sed 's/description = "\(.*\)"/\1/')
            [[ -n "$desc" ]] && themes+=("${name}:${desc}") || themes+=("${name}")
        done
    fi

    if (( CURRENT == 2 )); then
        _describe -t commands 'commands' commands
        _describe -t themes 'themes' themes
    elif (( CURRENT == 3 )); then
        case "$words[2]" in
            list) _arguments '--preview[Show theme previews]' ;;
            preview) _arguments '--compact[Show compact previews]' ;;
        esac
    fi
}
compdef _zush_theme_completion zush-theme

# Aliases for quick theme switching
alias zt='zush-theme'

# State tracking
typeset -g ZUSH_LAST_EXIT_CODE=0
typeset -g ZUSH_CMD_START_TIME=0
typeset -g ZUSH_CMD_DURATION=0
typeset -g ZUSH_PROMPT_RENDERED=0
typeset -g ZUSH_PROMPT_LINES=3  # Number of lines in the current prompt (dynamically updated)
typeset -g ZUSH_LAST_COMMAND=""

# Generate unique session ID for history tracking (once per shell)
typeset -g ZUSH_SESSION_ID="${ZUSH_SESSION_ID:-$(head -c 8 /dev/urandom 2>/dev/null | xxd -p 2>/dev/null || echo $$)}"

# ============================================================================
# Helper functions to avoid code duplication
# ============================================================================

# Build theme arguments for zush-prompt command
_zush_theme_args() {
    [[ -n "$ZUSH_THEME" ]] && echo "--theme $ZUSH_THEME"
}

# Build minimal context JSON for transient prompts
_zush_transient_context() {
    echo "{\"time\": \"$(date +%H:%M:%S)\"}"
}

# Build full context JSON for main prompt
_zush_full_context() {
    cat <<EOF
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
}

# Render transient prompt and replace current prompt
# Args: $1 = exit_code, $2 = execution_time, $3 = optional command to append
_zush_render_transient() {
    local exit_code="$1"
    local exec_time="$2"
    local cmd="$3"

    local transient_prompt=$($ZUSH_PROMPT_BIN --template transient --format raw --quiet $(_zush_theme_args) prompt \
        --context "$(_zush_transient_context)" \
        --exit-code "$exit_code" \
        --execution-time "$exec_time")

    # Move cursor up, clear lines, print transient version (+ command if provided)
    # \e[<n>A moves cursor up n lines, \e[0G moves to line start, \e[0J clears to end
    if [[ -n "$cmd" ]]; then
        printf '\e[%dA\e[0G\e[0J%s%s\n' "$ZUSH_PROMPT_LINES" "$transient_prompt" "$cmd"
    else
        printf '\e[%dA\e[0G\e[0J%s\n' "$ZUSH_PROMPT_LINES" "$transient_prompt"
    fi
}

# ============================================================================
# Hook functions
# ============================================================================

# Preexec hook - called before command execution (only when a command is entered)
zush_preexec() {
    ZUSH_CMD_START_TIME=$EPOCHREALTIME
    ZUSH_PROMPT_RENDERED=0
    ZUSH_LAST_COMMAND="$1"  # Capture command for history

    # Render transient prompt with command appended
    _zush_render_transient "$ZUSH_LAST_EXIT_CODE" "$ZUSH_CMD_DURATION" "$1"

    # Add newline after prompt if configured (before command output)
    [[ $ZUSH_PROMPT_NEWLINE_AFTER -eq 1 ]] && print
}

# Precmd hook - called before prompt display
zush_precmd() {
    ZUSH_LAST_EXIT_CODE=$?

    # Calculate command duration
    if [[ $ZUSH_CMD_START_TIME -gt 0 ]]; then
        ZUSH_CMD_DURATION=$(( EPOCHREALTIME - ZUSH_CMD_START_TIME ))
        ZUSH_CMD_START_TIME=0
    else
        ZUSH_CMD_DURATION=0
    fi

    # Record command to history (background, non-blocking)
    if [[ -n "$ZUSH_LAST_COMMAND" ]]; then
        # Skip commands starting with space (private commands)
        if [[ "$ZUSH_LAST_COMMAND" != " "* ]]; then
            $ZUSH_PROMPT_BIN history add \
                --session "$ZUSH_SESSION_ID" \
                --exit-code $ZUSH_LAST_EXIT_CODE \
                --duration $ZUSH_CMD_DURATION \
                --directory "$PWD" \
                -- "$ZUSH_LAST_COMMAND" &!
        fi
        ZUSH_LAST_COMMAND=""
    fi

    # If preexec wasn't called (user just pressed Enter), convert to transient
    [[ $ZUSH_PROMPT_RENDERED -eq 1 ]] && _zush_render_transient "$ZUSH_LAST_EXIT_CODE" 0

    ZUSH_PROMPT_RENDERED=1

    # Add newline before prompt if configured
    [[ $ZUSH_PROMPT_NEWLINE_BEFORE -eq 1 ]] && print
}

# Generate main prompt
zush_prompt() {
    local output=$($ZUSH_PROMPT_BIN --template main --format zsh $(_zush_theme_args) prompt \
        --context "$(_zush_full_context)" \
        --exit-code $ZUSH_LAST_EXIT_CODE \
        --execution-time $ZUSH_CMD_DURATION)

    # Dynamically count prompt lines for accurate transient replacement
    # Count newlines in the raw output (strip zsh %{...%} wrappers for counting)
    local raw_output="${output//\%\{/}"
    raw_output="${raw_output//\%\}/}"
    local line_count=$(( $(echo -n "$raw_output" | wc -l) + 1 ))
    ZUSH_PROMPT_LINES=$line_count

    echo -n "$output"
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

# ============================================================================
# History search widget (Ctrl+R)
# ============================================================================

# History search using zush TUI
zush-history-widget() {
    local tmpfile="/tmp/zush-history-$$"
    # Run the TUI - it opens /dev/tty directly for input/output
    $ZUSH_PROMPT_BIN history search --tui --output "$tmpfile" 2>/dev/null
    if [[ -f "$tmpfile" ]]; then
        local selected="$(cat "$tmpfile")"
        rm -f "$tmpfile"
        if [[ -n "$selected" ]]; then
            LBUFFER="$selected"
            RBUFFER=""
        fi
    fi
    zle reset-prompt
}

# Register widget and bind to Ctrl+R
zle -N zush-history-widget
bindkey '^R' zush-history-widget

# Also provide a command alias
alias zh='$ZUSH_PROMPT_BIN history'
alias zhl='$ZUSH_PROMPT_BIN history list'
alias zhs='$ZUSH_PROMPT_BIN history search --tui'
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_script_not_empty() {
        assert!(!INIT_SCRIPT.is_empty());
    }

    #[test]
    fn test_init_script_has_shebang() {
        assert!(INIT_SCRIPT.starts_with("#!/usr/bin/env zsh"));
    }

    #[test]
    fn test_init_script_has_hooks() {
        assert!(INIT_SCRIPT.contains("add-zsh-hook preexec"));
        assert!(INIT_SCRIPT.contains("add-zsh-hook precmd"));
    }

    #[test]
    fn test_init_script_has_theme_function() {
        assert!(INIT_SCRIPT.contains("zush-theme()"));
    }

    #[test]
    fn test_init_script_has_prompt_function() {
        assert!(INIT_SCRIPT.contains("zush_prompt()"));
    }

    #[test]
    fn test_init_script_sets_prompt() {
        assert!(INIT_SCRIPT.contains("PROMPT='$(zush_prompt)'"));
    }
}
