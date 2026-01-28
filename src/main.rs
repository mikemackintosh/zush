mod buffer;
mod cli;
mod color;
mod config;
mod defaults;
mod git;
#[cfg(feature = "history")]
mod history;
mod init;
mod modules;
mod segments;
mod symbols;
mod template;
mod toml_helpers;

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use buffer::TerminalBuffer;
use cli::{Cli, Commands};
use template::TemplateEngine;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { shell }) => {
            init::print_init_script(shell)?;
        }
        Some(Commands::Config) => {
            init::print_default_config()?;
        }
        #[cfg(feature = "history")]
        Some(Commands::History { command }) => {
            handle_history_command(command)?;
        }
        Some(Commands::Prompt {
            context,
            exit_code,
            execution_time,
        }) => {
            render_prompt(&cli, context.as_deref(), *exit_code, *execution_time)?;
        }
        None => {
            // Default to rendering prompt
            render_prompt(&cli, None, None, None)?;
        }
    }

    Ok(())
}

fn load_theme(theme_name: &str) -> Result<String> {
    // Check if it's a path to a custom theme
    let theme_path = if theme_name.contains('/') || theme_name.contains('.') {
        PathBuf::from(theme_name)
    } else {
        // Look for theme in themes directory
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let theme_file = format!("{}.toml", theme_name);
        home.join(".config")
            .join("zush")
            .join("themes")
            .join(theme_file)
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

    // Parse config TOML once upfront (if it exists) to avoid double-parsing
    let config_parsed: Option<toml::Value> = config_str
        .as_ref()
        .and_then(|s| toml::from_str(s).ok());

    // Determine which theme to load
    // Priority: CLI flag > ZUSH_THEME env var > config file
    let theme_str = if let Some(theme_name) = &cli.theme {
        // CLI argument takes precedence
        load_theme(theme_name).ok()
    } else if let Ok(theme_name) = std::env::var("ZUSH_THEME") {
        // Environment variable is second priority
        load_theme(&theme_name).ok()
    } else if let Some(ref parsed) = config_parsed {
        // Use already-parsed config to get theme name
        if let Some(theme_name) = parsed.get("theme").and_then(|v| v.as_str()) {
            load_theme(theme_name).ok()
        } else {
            None
        }
    } else {
        None
    };

    // Create template engine
    let mut engine = TemplateEngine::new()?;

    // Create TOML parser - reuse already-parsed config when possible
    let toml_parser = if let Some(ref theme) = theme_str {
        // Theme needs to be parsed (it's a separate file)
        toml_helpers::TomlParser::new(Some(theme.as_str()))
    } else {
        // Reuse already-parsed config (avoids double-parsing)
        toml_helpers::TomlParser::from_parsed(config_parsed.clone())
    };

    // Extract colors for preprocessing (allows templates to use named colors)
    let colors_for_preprocessing = toml_parser.extract_colors();
    engine.set_colors(colors_for_preprocessing);

    // Extract symbols for preprocessing (@symbol_name shortcuts)
    let symbols_for_preprocessing = toml_parser.extract_symbols(parse_unicode_escapes);
    engine.set_symbols(symbols_for_preprocessing);

    // Extract segments for preprocessing (reusable segment definitions)
    let segments_for_preprocessing = toml_parser.extract_segments();
    if !segments_for_preprocessing.is_empty() {
        engine.add_segments(segments_for_preprocessing);
    }

    // Load templates from theme, config, or defaults
    let theme_or_config = theme_str.as_ref().or(config_str.as_ref());

    if let Some(toml_str) = theme_or_config {
        if let Err(e) = engine.load_templates_from_config(toml_str) {
            // If loading fails, print stylized error (unless quiet mode) and register defaults
            if !cli.quiet {
                eprintln!(
                    "\n\x1b[38;2;243;139;168m\x1b[1m✖ Template Loading Error\x1b[22m\x1b[39m"
                );
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
    context.insert(
        "execution_time_s".to_string(),
        json!(execution_time.unwrap_or(0.0)),
    );

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

    // Ensure pwd_short is derived from pwd if pwd was provided but pwd_short wasn't
    if !context.contains_key("pwd_short") {
        if let Some(pwd_value) = context.get("pwd").and_then(|v| v.as_str()) {
            // If pwd already starts with ~, use as-is; otherwise try to shorten
            let pwd_short = if pwd_value.starts_with('~') {
                pwd_value.to_string()
            } else if let Ok(home) = std::env::var("HOME") {
                pwd_value.replace(&home, "~")
            } else {
                pwd_value.to_string()
            };
            context.insert("pwd_short".to_string(), json!(pwd_short));
        }
    }

    // Detect if running over SSH
    let is_ssh = std::env::var("SSH_CONNECTION").is_ok() || std::env::var("SSH_TTY").is_ok();
    context.insert("is_ssh".to_string(), json!(is_ssh));

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
    context
        .entry("git_modified".to_string())
        .or_insert(json!(0));
    context.entry("git_added".to_string()).or_insert(json!(0));
    context.entry("git_deleted".to_string()).or_insert(json!(0));
    context.entry("git_renamed".to_string()).or_insert(json!(0));
    context
        .entry("git_untracked".to_string())
        .or_insert(json!(0));
    context
        .entry("git_conflicted".to_string())
        .or_insert(json!(0));

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

    // Load colors and symbols from theme/config or use defaults
    // Reuse the toml_parser we created earlier for preprocessing
    let mut colors = toml_parser.extract_colors_as_json();
    let mut symbols = toml_parser.extract_symbols_as_json(parse_unicode_escapes);

    // Apply overrides from main config if theme was loaded
    if theme_str.is_some() && config_str.is_some() {
        let config_parser = toml_helpers::TomlParser::new(config_str.as_deref());
        config_parser.apply_overrides(&mut colors, &mut symbols, parse_unicode_escapes);
    }

    // Use defaults if no colors/symbols were loaded from config
    if colors.is_empty() {
        colors = defaults::default_colors_json();
    }
    if symbols.is_empty() {
        symbols = defaults::default_symbols_json();
    }

    context.insert("colors".to_string(), json!(colors));
    context.insert("symbols".to_string(), json!(symbols));

    // Get terminal width directly from the terminal (not from shell)
    let terminal_width = if let Some((terminal_size::Width(w), _)) = terminal_size::terminal_size()
    {
        w as usize
    } else {
        // Fallback to context if terminal size detection fails
        context
            .get("terminal_width")
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
                    format!(
                        "{}{:width$}{}",
                        left_output,
                        "",
                        right_output,
                        width = spacing
                    )
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
    engine.register_template(
        "transient",
        r#"(dim){{time}}(/dim)
(fg #7aa2f7)❯(/fg) "#,
    )?;

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
                            let combined =
                                0x10000 + ((high - 0xD800) << 10) + (code_point - 0xDC00);
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

#[cfg(feature = "history")]
fn handle_history_command(command: &cli::HistoryCommands) -> Result<()> {
    use cli::HistoryCommands;

    match command {
        HistoryCommands::Add {
            session,
            exit_code,
            duration,
            directory,
            command,
        } => {
            let dir = directory.clone().unwrap_or_else(|| {
                std::env::var("PWD").unwrap_or_else(|_| {
                    std::env::current_dir()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| ".".to_string())
                })
            });

            let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());

            let entry = history::HistoryEntry::new(
                command.clone(),
                dir,
                session.clone(),
                *exit_code,
                (*duration * 1000.0) as u64, // Convert seconds to milliseconds
                hostname,
            );

            history::append_entry(&entry)?;
        }

        HistoryCommands::Search {
            tui,
            style,
            dir,
            session,
            successful,
            query,
            output,
            entries_file,
        } => {
            // Load entries from file if provided (internal respawn case), otherwise from history
            let entries = if let Some(file_path) = entries_file {
                // Read entries from temp file (used when respawning for TTY)
                use std::io::BufRead;
                let file = std::fs::File::open(file_path)?;
                let reader = std::io::BufReader::new(file);
                let mut entries = Vec::new();
                for line in reader.lines() {
                    let line = line?;
                    if let Ok(entry) = history::HistoryEntry::from_json(&line) {
                        entries.push(entry);
                    }
                }
                entries
            } else {
                history::read_all_entries()?
            };

            let filter = history::SearchFilter {
                directory: dir.clone(),
                session: session.clone(),
                successful_only: *successful,
                ..Default::default()
            };

            if *tui {
                // Run TUI picker
                let tui_style = history::tui::TuiStyle::from_str(style);
                let filtered: Vec<_> = entries.into_iter().filter(|e| filter.matches(e)).collect();

                if let Some(selected) = history::tui::run_picker(filtered, tui_style)? {
                    // Write to output file if specified (for ZLE widget), otherwise stdout
                    if let Some(output_path) = output {
                        std::fs::write(output_path, &selected)?;
                    } else {
                        print!("{}", selected);
                    }
                }
            } else {
                // Text-based search output
                let results =
                    history::search::search(&entries, query.as_deref().unwrap_or(""), &filter, 20);

                for result in results {
                    println!("{}", result.entry.cmd);
                }
            }
        }

        HistoryCommands::List { count, json } => {
            let entries = history::read_recent_entries(*count)?;

            if *json {
                let json_entries: Vec<_> = entries
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "ts": e.ts,
                            "dir": e.dir,
                            "cmd": e.cmd,
                            "exit": e.exit,
                            "dur": e.dur,
                            "sid": e.sid,
                            "host": e.host,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_entries)?);
            } else {
                for entry in entries.iter().rev() {
                    let time = entry.formatted_time();
                    let exit_indicator = if entry.exit == 0 {
                        "\x1b[32m✓\x1b[0m"
                    } else {
                        "\x1b[31m✗\x1b[0m"
                    };
                    println!(
                        "{} {} \x1b[90m{}\x1b[0m {}",
                        exit_indicator,
                        time,
                        entry.short_dir(),
                        entry.cmd
                    );
                }
            }
        }

        HistoryCommands::Clear {
            older_than,
            all,
            force,
        } => {
            if let Some(days) = older_than {
                let removed = history::clear_older_than(*days)?;
                println!("Removed {} entries older than {} days", removed, days);
            } else if *all {
                if *force {
                    history::clear_all()?;
                    println!("History cleared");
                } else {
                    eprintln!("Use --force to confirm clearing all history");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Specify --older-than <days> or --all --force");
                std::process::exit(1);
            }
        }

        HistoryCommands::Stats => {
            let stats = history::get_stats()?;
            let path = history::get_history_path()?;

            println!("History file: {}", path.display());
            println!("Total entries: {}", stats.entry_count);
            println!("File size: {:.2} KB", stats.file_size_bytes as f64 / 1024.0);

            if let Some(oldest) = stats.oldest_timestamp {
                use chrono::{Local, TimeZone};
                if let Some(dt) = Local.timestamp_opt(oldest, 0).single() {
                    println!("Oldest entry: {}", dt.format("%Y-%m-%d %H:%M:%S"));
                }
            }

            if let Some(newest) = stats.newest_timestamp {
                use chrono::{Local, TimeZone};
                if let Some(dt) = Local.timestamp_opt(newest, 0).single() {
                    println!("Newest entry: {}", dt.format("%Y-%m-%d %H:%M:%S"));
                }
            }
        }
    }

    Ok(())
}
