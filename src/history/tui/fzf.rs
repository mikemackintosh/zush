//! FZF-style minimal picker for history search

use crate::history::{search, HistoryEntry, SearchFilter};
use anyhow::Result;
use crossterm::{cursor, execute, terminal};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Terminal,
};
use std::os::unix::io::AsRawFd;

const MAX_VISIBLE_ITEMS: usize = 15;

#[derive(Clone, Copy, PartialEq)]
enum HistoryTab {
    Session,
    ZshHistory,
}

impl HistoryTab {
    fn titles() -> Vec<&'static str> {
        vec!["Session", "Zsh History"]
    }

    fn index(&self) -> usize {
        match self {
            HistoryTab::Session => 0,
            HistoryTab::ZshHistory => 1,
        }
    }

    fn next(&self) -> Self {
        match self {
            HistoryTab::Session => HistoryTab::ZshHistory,
            HistoryTab::ZshHistory => HistoryTab::Session,
        }
    }
}

/// Read and parse ~/.zsh_history
fn read_zsh_history() -> Vec<String> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };

    let history_path = home.join(".zsh_history");
    let content = match std::fs::read(&history_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    // zsh history can have multi-byte encoding issues, be lenient
    let text = String::from_utf8_lossy(&content);

    let mut commands = Vec::new();
    let mut current_cmd = String::new();

    for line in text.lines() {
        // Extended history format: ": timestamp:duration;command"
        if line.starts_with(": ") {
            if let Some(semi_pos) = line.find(';') {
                let cmd = &line[semi_pos + 1..];
                if !cmd.is_empty() {
                    // Handle line continuations
                    if cmd.ends_with('\\') {
                        current_cmd = cmd[..cmd.len() - 1].to_string();
                    } else {
                        commands.push(cmd.to_string());
                    }
                }
            }
        } else if !current_cmd.is_empty() {
            // Continuation of previous command
            if line.ends_with('\\') {
                current_cmd.push_str(&line[..line.len() - 1]);
            } else {
                current_cmd.push_str(line);
                commands.push(std::mem::take(&mut current_cmd));
            }
        } else if !line.is_empty() {
            // Simple history format (no timestamps)
            commands.push(line.to_string());
        }
    }

    // Deduplicate keeping most recent (last occurrence)
    let mut seen = std::collections::HashSet::new();
    let mut unique: Vec<String> = commands
        .into_iter()
        .rev()
        .filter(|cmd| seen.insert(cmd.clone()))
        .collect();
    unique.reverse();

    unique
}

struct App {
    // Session history (zush)
    session_entries: Vec<HistoryEntry>,
    session_filtered: Vec<HistoryEntry>,
    // Zsh history
    zsh_entries: Vec<String>,
    zsh_filtered: Vec<String>,
    // Current tab
    current_tab: HistoryTab,
    // Search
    query: String,
    cursor_pos: usize,
    selected: usize,
    list_state: ListState,
}

impl App {
    fn new(session_entries: Vec<HistoryEntry>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        // Reverse entries so most recent is first
        let session_entries: Vec<_> = session_entries.into_iter().rev().collect();
        let session_filtered = session_entries.clone();

        // Load zsh history
        let zsh_entries = read_zsh_history();
        let zsh_filtered: Vec<_> = zsh_entries.iter().rev().cloned().collect();

        Self {
            session_entries,
            session_filtered,
            zsh_entries,
            zsh_filtered,
            current_tab: HistoryTab::Session,
            query: String::new(),
            cursor_pos: 0,
            selected: 0,
            list_state,
        }
    }

    fn switch_tab(&mut self) {
        self.current_tab = self.current_tab.next();
        self.selected = 0;
        self.list_state.select(Some(0));
        self.update_filter();
    }

    fn current_list_len(&self) -> usize {
        match self.current_tab {
            HistoryTab::Session => self.session_filtered.len(),
            HistoryTab::ZshHistory => self.zsh_filtered.len(),
        }
    }

    fn update_filter(&mut self) {
        match self.current_tab {
            HistoryTab::Session => {
                if self.query.is_empty() {
                    self.session_filtered = self.session_entries.clone();
                } else {
                    let results = search::search(
                        &self.session_entries,
                        &self.query,
                        &SearchFilter::default(),
                        MAX_VISIBLE_ITEMS * 10,
                    );
                    self.session_filtered = results.into_iter().map(|r| r.entry).collect();
                }
            }
            HistoryTab::ZshHistory => {
                if self.query.is_empty() {
                    self.zsh_filtered = self.zsh_entries.iter().rev().cloned().collect();
                } else {
                    let query_lower = self.query.to_lowercase();
                    self.zsh_filtered = self.zsh_entries
                        .iter()
                        .rev()
                        .filter(|cmd| cmd.to_lowercase().contains(&query_lower))
                        .take(MAX_VISIBLE_ITEMS * 10)
                        .cloned()
                        .collect();
                }
            }
        }

        // Reset selection
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    fn move_up(&mut self) {
        if self.current_list_len() > 0 {
            self.selected = self.selected.saturating_sub(1);
            self.list_state.select(Some(self.selected));
        }
    }

    fn move_down(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = (self.selected + 1).min(len - 1);
            self.list_state.select(Some(self.selected));
        }
    }

    fn page_up(&mut self) {
        if self.current_list_len() > 0 {
            self.selected = self.selected.saturating_sub(10);
            self.list_state.select(Some(self.selected));
        }
    }

    fn page_down(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = (self.selected + 10).min(len - 1);
            self.list_state.select(Some(self.selected));
        }
    }

    fn selected_command(&self) -> Option<String> {
        match self.current_tab {
            HistoryTab::Session => {
                self.session_filtered.get(self.selected).map(|e| e.cmd.clone())
            }
            HistoryTab::ZshHistory => {
                self.zsh_filtered.get(self.selected).cloned()
            }
        }
    }

    fn insert_char(&mut self, c: char) {
        self.query.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.update_filter();
    }

    fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.query.remove(self.cursor_pos);
            self.update_filter();
        }
    }

    fn delete_word(&mut self) {
        // Delete backwards to the previous word boundary
        while self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let c = self.query.remove(self.cursor_pos);
            if c == ' ' || c == '/' || c == '-' {
                break;
            }
        }
        self.update_filter();
    }

    fn clear_query(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
        self.update_filter();
    }
}

pub fn run_fzf_picker(entries: Vec<HistoryEntry>) -> Result<Option<String>> {
    // Always use /dev/tty directly - this works regardless of stdin/stdout state
    // This is essential for ZLE widgets where stdin/stdout may be redirected
    run_with_tty(entries)
}

fn run_with_tty(entries: Vec<HistoryEntry>) -> Result<Option<String>> {
    // Open /dev/tty for reading input
    let mut tty_input = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map_err(|e| anyhow::anyhow!("Cannot open /dev/tty: {}. Are you running in a terminal?", e))?;

    // Open a separate handle for output
    let tty_output = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?;

    // Enable raw mode on the TTY
    // We need to do this on the correct fd - use the input fd
    let tty_fd = tty_input.as_raw_fd();

    // Get current terminal attributes
    let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();
    if unsafe { libc::tcgetattr(tty_fd, termios.as_mut_ptr()) } != 0 {
        anyhow::bail!("Failed to get terminal attributes");
    }
    let original_termios = unsafe { termios.assume_init() };

    // Set raw mode
    let mut raw_termios = original_termios;
    unsafe {
        libc::cfmakeraw(&mut raw_termios);
    }
    if unsafe { libc::tcsetattr(tty_fd, libc::TCSANOW, &raw_termios) } != 0 {
        anyhow::bail!("Failed to set raw mode");
    }

    // Setup terminal using /dev/tty for output
    let mut tty_out = tty_output;
    execute!(tty_out, terminal::EnterAlternateScreen, cursor::Hide)?;

    let backend = CrosstermBackend::new(tty_out);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(entries);

    // Run the app with direct TTY input reading (bypasses crossterm's event system)
    let result = run_app_tty(&mut terminal, &mut app, &mut tty_input);

    // Restore terminal state
    let _ = execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        cursor::Show
    );

    // Restore original terminal attributes
    unsafe { libc::tcsetattr(tty_fd, libc::TCSANOW, &original_termios) };

    result
}

fn run_app_tty(
    terminal: &mut Terminal<CrosstermBackend<std::fs::File>>,
    app: &mut App,
    tty_input: &mut std::fs::File,
) -> Result<Option<String>> {
    use std::io::Read;

    // Drain any pending input (like the Ctrl+R keypress)
    // Set non-blocking temporarily to drain
    let fd = tty_input.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    let mut drain_buf = [0u8; 64];
    while tty_input.read(&mut drain_buf).unwrap_or(0) > 0 {}
    // Restore blocking mode
    unsafe { libc::fcntl(fd, libc::F_SETFL, flags) };

    // Buffer for reading input
    let mut buf = [0u8; 16];

    loop {
        // Draw the UI
        terminal.draw(|f| draw_ui(f, app))
            .map_err(|e| anyhow::anyhow!("Draw error: {}", e))?;

        // Read input directly from TTY (blocking)
        // Use select/poll to add a timeout so we can redraw periodically
        let ready = unsafe {
            let mut fds: libc::fd_set = std::mem::zeroed();
            libc::FD_ZERO(&mut fds);
            libc::FD_SET(fd, &mut fds);
            let mut timeout = libc::timeval {
                tv_sec: 0,
                tv_usec: 100_000, // 100ms
            };
            libc::select(fd + 1, &mut fds, std::ptr::null_mut(), std::ptr::null_mut(), &mut timeout)
        };

        if ready <= 0 {
            continue; // Timeout or error, just redraw
        }

        // Read available bytes
        let n = match tty_input.read(&mut buf) {
            Ok(0) => continue,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(anyhow::anyhow!("Read error: {}", e)),
        };

        // Parse the input bytes
        match &buf[..n] {
            // Escape or Ctrl+C
            [0x1b] | [0x03] => return Ok(None),

            // Enter
            [0x0d] | [0x0a] => return Ok(app.selected_command()),

            // Tab - switch between history sources
            [0x09] => app.switch_tab(),

            // Ctrl+P (up)
            [0x10] => app.move_up(),

            // Ctrl+N (down)
            [0x0e] => app.move_down(),

            // Backspace (DEL or Ctrl+H)
            [0x7f] | [0x08] => app.delete_char(),

            // Ctrl+W (delete word)
            [0x17] => app.delete_word(),

            // Ctrl+U (clear line)
            [0x15] => app.clear_query(),

            // CSI sequences: ESC [ ...
            [0x1b, 0x5b, rest @ ..] => {
                handle_csi_sequence(app, rest);
            }

            // Regular printable character (but not [ which could be CSI start)
            [c] if *c >= 0x20 && *c < 0x7f && *c != 0x5b => {
                app.insert_char(*c as char);
            }

            // Single [ - could be start of CSI sequence, ignore it
            [0x5b] => {}

            // Multi-byte input - filter out escape sequence fragments
            bytes => {
                // Check if this looks like a CSI sequence fragment
                // Patterns: starts with [, or contains ; with surrounding digits (like 1;2B)
                let dominated_by_csi_chars = bytes.iter().all(|&b| {
                    b == 0x5b  // [
                        || b == 0x3b  // ;
                        || (b >= 0x30 && b <= 0x39)  // 0-9
                        || (b >= 0x41 && b <= 0x44)  // A-D (arrow finals)
                        || b == 0x5a  // Z (shift+tab)
                        || b == 0x7e  // ~ (page up/down)
                });

                if bytes.first() == Some(&0x1b)
                    || bytes.first() == Some(&0x5b)
                    || dominated_by_csi_chars
                {
                    // Ignore escape sequences
                } else if let Ok(s) = std::str::from_utf8(bytes) {
                    for c in s.chars() {
                        if c.is_ascii_graphic() || c == ' ' {
                            app.insert_char(c);
                        }
                    }
                }
            }
        }
    }
}

/// Handle CSI (Control Sequence Introducer) sequences: ESC [ ...
/// The `rest` parameter contains bytes after "ESC ["
fn handle_csi_sequence(app: &mut App, rest: &[u8]) {
    match rest {
        // Arrow keys
        [0x41] => app.move_up(),         // A = Up
        [0x42] => app.move_down(),       // B = Down
        [0x43] => app.switch_tab(),      // C = Right (switch tab)
        [0x44] => app.switch_tab(),      // D = Left (switch tab)

        // Shift+Tab: Z
        [0x5a] => app.switch_tab(),

        // Page Up/Down: 5~ and 6~
        [0x35, 0x7e] => app.page_up(),   // 5~ = Page Up
        [0x36, 0x7e] => app.page_down(), // 6~ = Page Down

        // Modified arrow keys: 1;N[ABCD] where N is modifier
        // N=2: Shift, N=5: Ctrl, N=3: Alt, N=6: Ctrl+Shift
        [0x31, 0x3b, modifier, direction] => {
            match (*modifier, *direction) {
                // Shift+Arrow or Ctrl+Arrow = page
                (0x32, 0x41) | (0x35, 0x41) => app.page_up(),   // Shift/Ctrl + Up
                (0x32, 0x42) | (0x35, 0x42) => app.page_down(), // Shift/Ctrl + Down
                (0x32, 0x43) | (0x35, 0x43) => app.switch_tab(), // Shift/Ctrl + Right
                (0x32, 0x44) | (0x35, 0x44) => app.switch_tab(), // Shift/Ctrl + Left
                _ => {} // Ignore other modified arrows
            }
        }

        // Ignore any other CSI sequences (don't insert as text)
        _ => {}
    }
}

fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Tabs
            Constraint::Length(3), // Search input
            Constraint::Min(1),    // Results list
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);
    draw_search_input(f, app, chunks[1]);
    draw_results_list(f, app, chunks[2]);
    draw_status_bar(f, app, chunks[3]);
}

fn draw_tabs(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = HistoryTab::titles()
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let count = match i {
                0 => app.session_entries.len(),
                1 => app.zsh_entries.len(),
                _ => 0,
            };
            Line::from(format!(" {} ({}) ", t, count))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.current_tab.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");

    f.render_widget(tabs, area);
}

fn draw_search_input(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let prompt_style = Style::default().fg(Color::Cyan);
    let input_style = Style::default().fg(Color::White);

    let input = Paragraph::new(Line::from(vec![
        Span::styled("> ", prompt_style),
        Span::styled(&app.query, input_style),
        Span::styled("█", Style::default().fg(Color::Gray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Search History "),
    );

    f.render_widget(input, area);
}

fn draw_results_list(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    // Calculate visible window based on area height (minus borders)
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.current_list_len();

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = match app.current_tab {
        HistoryTab::Session => {
            app.session_filtered
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(visible_height)
                .map(|(i, entry)| {
                    let is_selected = i == app.selected;
                    let cmd_style = if is_selected {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let prefix_style = if is_selected {
                        Style::default().bg(Color::DarkGray).fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    let dir_style = if is_selected {
                        Style::default().bg(Color::DarkGray).fg(Color::Blue)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    // Format: "HH:MM ~/path  command"
                    use chrono::{Local, TimeZone};
                    let time_str = Local
                        .timestamp_opt(entry.ts, 0)
                        .single()
                        .map(|dt| dt.format("%H:%M").to_string())
                        .unwrap_or_else(|| "??:??".to_string());

                    let short_dir = entry.short_dir();
                    // Truncate directory if too long
                    let max_dir_len = 20;
                    let dir_display = if short_dir.len() > max_dir_len {
                        format!("…{}", &short_dir[short_dir.len() - max_dir_len + 1..])
                    } else {
                        short_dir
                    };

                    // Calculate remaining space for command
                    // Format: "▸ HH:MM dir  cmd" = 2 + 5 + 1 + dir_len + 2 + cmd
                    let prefix_len = 2 + 5 + 1 + dir_display.len() + 2;
                    let max_cmd_len = (area.width as usize).saturating_sub(prefix_len + 2);
                    let display_cmd = if entry.cmd.len() > max_cmd_len && max_cmd_len > 1 {
                        format!("{}…", &entry.cmd[..max_cmd_len - 1])
                    } else {
                        entry.cmd.clone()
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(if is_selected { "▸ " } else { "  " }, cmd_style),
                        Span::styled(time_str, prefix_style),
                        Span::styled(" ", cmd_style),
                        Span::styled(dir_display, dir_style),
                        Span::styled("  ", cmd_style),
                        Span::styled(display_cmd, cmd_style),
                    ]))
                })
                .collect()
        }
        HistoryTab::ZshHistory => {
            app.zsh_filtered
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(visible_height)
                .map(|(i, cmd)| {
                    let style = if i == app.selected {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    // Truncate command if too long
                    let max_len = area.width.saturating_sub(4) as usize;
                    let display_cmd = if cmd.len() > max_len && max_len > 1 {
                        format!("{}…", &cmd[..max_len - 1])
                    } else {
                        cmd.clone()
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(if i == app.selected { "▸ " } else { "  " }, style),
                        Span::styled(display_cmd, style),
                    ]))
                })
                .collect()
        }
    };

    // Show scroll indicator in title if there are more items
    let title = if total > visible_height {
        format!(" {}-{} of {} ", scroll_offset + 1, (scroll_offset + visible_height).min(total), total)
    } else {
        String::new()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray))
            .title_bottom(Line::from(title).right_aligned()),
    );

    f.render_widget(list, area);
}

fn draw_status_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let (filtered, total) = match app.current_tab {
        HistoryTab::Session => (app.session_filtered.len(), app.session_entries.len()),
        HistoryTab::ZshHistory => (app.zsh_filtered.len(), app.zsh_entries.len()),
    };

    let status = format!(
        " {}/{} │ Tab switch │ ↑↓ navigate │ S/C-↑↓ page │ Enter select │ Esc cancel ",
        filtered, total
    );

    let status_bar =
        Paragraph::new(status).style(Style::default().fg(Color::DarkGray).bg(Color::Black));

    f.render_widget(status_bar, area);
}
