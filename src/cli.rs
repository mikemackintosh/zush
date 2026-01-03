//! Command-line interface definitions for zush-prompt
//!
//! This module contains the CLI argument parsing structures using clap.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Zush - A high-performance Zsh prompt with perfect buffering and 24-bit colors
#[derive(Parser, Debug)]
#[command(name = "zush-prompt")]
#[command(about = "A Rust-powered Zsh prompt engine", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output format: zsh, raw, or debug
    #[arg(short, long, default_value = "zsh")]
    pub format: String,

    /// Template to render (defaults to main)
    #[arg(short, long, default_value = "main")]
    pub template: String,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Theme to use (overrides config)
    #[arg(long)]
    pub theme: Option<String>,

    /// Show transient prompt
    #[arg(long)]
    pub transient: bool,

    /// Suppress error messages (useful for transient prompts to avoid duplication)
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize shell integration
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

    /// History management
    #[cfg(feature = "history")]
    History {
        #[command(subcommand)]
        command: HistoryCommands,
    },
}

/// History subcommands
#[cfg(feature = "history")]
#[derive(Subcommand, Debug)]
pub enum HistoryCommands {
    /// Add a command to history
    Add {
        /// Session ID (unique per shell instance)
        #[arg(long)]
        session: String,

        /// Exit code of the command
        #[arg(long)]
        exit_code: i32,

        /// Duration in seconds
        #[arg(long)]
        duration: f64,

        /// Working directory (defaults to current)
        #[arg(long)]
        directory: Option<String>,

        /// The command that was executed
        command: String,
    },

    /// Search history
    Search {
        /// Use TUI interface
        #[arg(long)]
        tui: bool,

        /// TUI style: fzf (minimal) or full (table)
        #[arg(long, default_value = "fzf")]
        style: String,

        /// Filter by directory prefix
        #[arg(long)]
        dir: Option<String>,

        /// Filter by session ID
        #[arg(long)]
        session: Option<String>,

        /// Only show successful commands (exit code 0)
        #[arg(long)]
        successful: bool,

        /// Search query (fuzzy match)
        query: Option<String>,

        /// Write selected command to file instead of stdout (for ZLE widget)
        #[arg(long)]
        output: Option<String>,

        /// Internal: read entries from file (used for TTY respawn)
        #[arg(long, hide = true)]
        entries_file: Option<String>,
    },

    /// List recent history entries
    List {
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        count: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Clear history
    Clear {
        /// Clear entries older than N days
        #[arg(long)]
        older_than: Option<u32>,

        /// Clear all entries (requires confirmation or --force)
        #[arg(long)]
        all: bool,

        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// Show history statistics
    Stats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parses() {
        // Verify CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_default_format() {
        let cli = Cli::parse_from(["zush-prompt"]);
        assert_eq!(cli.format, "zsh");
    }

    #[test]
    fn test_default_template() {
        let cli = Cli::parse_from(["zush-prompt"]);
        assert_eq!(cli.template, "main");
    }

    #[test]
    fn test_custom_format() {
        let cli = Cli::parse_from(["zush-prompt", "--format", "raw"]);
        assert_eq!(cli.format, "raw");
    }

    #[test]
    fn test_theme_flag() {
        let cli = Cli::parse_from(["zush-prompt", "--theme", "minimal"]);
        assert_eq!(cli.theme, Some("minimal".to_string()));
    }

    #[test]
    fn test_quiet_flag() {
        let cli = Cli::parse_from(["zush-prompt", "--quiet"]);
        assert!(cli.quiet);
    }

    #[test]
    fn test_init_command() {
        let cli = Cli::parse_from(["zush-prompt", "init", "zsh"]);
        match cli.command {
            Some(Commands::Init { shell }) => assert_eq!(shell, "zsh"),
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_prompt_command_with_args() {
        let cli = Cli::parse_from([
            "zush-prompt",
            "prompt",
            "--exit-code",
            "0",
            "--execution-time",
            "1.5",
        ]);
        match cli.command {
            Some(Commands::Prompt {
                exit_code,
                execution_time,
                ..
            }) => {
                assert_eq!(exit_code, Some(0));
                assert_eq!(execution_time, Some(1.5));
            }
            _ => panic!("Expected Prompt command"),
        }
    }
}
