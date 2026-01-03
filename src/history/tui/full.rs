//! Full table view picker for history search

use crate::history::{search, HistoryEntry, SearchFilter};
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};

struct App {
    entries: Vec<HistoryEntry>,
    filtered: Vec<HistoryEntry>,
    query: String,
    cursor_pos: usize,
    table_state: TableState,
}

impl App {
    fn new(entries: Vec<HistoryEntry>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        // Reverse entries so most recent is first
        let entries: Vec<_> = entries.into_iter().rev().collect();
        let filtered = entries.clone();

        Self {
            entries,
            filtered,
            query: String::new(),
            cursor_pos: 0,
            table_state,
        }
    }

    fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = self.entries.clone();
        } else {
            let results =
                search::search(&self.entries, &self.query, &SearchFilter::default(), 1000);
            self.filtered = results.into_iter().map(|r| r.entry).collect();
        }

        // Reset selection
        self.table_state.select(Some(0));
    }

    fn move_up(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let new = selected.saturating_sub(1);
            self.table_state.select(Some(new));
        }
    }

    fn move_down(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let new = (selected + 1).min(self.filtered.len().saturating_sub(1));
            self.table_state.select(Some(new));
        }
    }

    fn page_up(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let new = selected.saturating_sub(10);
            self.table_state.select(Some(new));
        }
    }

    fn page_down(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let new = (selected + 10).min(self.filtered.len().saturating_sub(1));
            self.table_state.select(Some(new));
        }
    }

    fn selected_command(&self) -> Option<String> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered.get(i))
            .map(|e| e.cmd.clone())
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

    fn clear_query(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
        self.update_filter();
    }
}

pub fn run_full_picker(entries: Vec<HistoryEntry>) -> Result<Option<String>> {
    use std::os::unix::io::AsRawFd;

    // Open /dev/tty for both input and output (required when running in subshell)
    let tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?;

    // Save original stdin and redirect stdin from /dev/tty for event reading
    let original_stdin = unsafe { libc::dup(0) };
    unsafe { libc::dup2(tty.as_raw_fd(), 0) };

    // Setup terminal on /dev/tty
    terminal::enable_raw_mode()?;
    let mut tty_write = tty.try_clone()?;
    execute!(tty_write, terminal::EnterAlternateScreen, cursor::Hide)?;

    let backend = CrosstermBackend::new(tty_write);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(entries);
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;

    // Restore original stdin
    unsafe {
        libc::dup2(original_stdin, 0);
        libc::close(original_stdin);
    };

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::fs::File>>,
    app: &mut App,
) -> Result<Option<String>> {
    // Drain any pending input before starting
    while event::poll(std::time::Duration::from_millis(1))? {
        let _ = event::read()?;
    }

    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        // Wait for event with timeout to allow for clean exit
        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                // Exit without selection
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    return Ok(None);
                }

                // Confirm selection
                (KeyCode::Enter, _) => {
                    return Ok(app.selected_command());
                }

                // Navigation
                (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                    app.move_up();
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                    // Only use j if not in query mode
                    if app.query.is_empty() {
                        app.move_down();
                    } else {
                        app.insert_char('j');
                    }
                }
                (KeyCode::PageUp, _) => {
                    app.page_up();
                }
                (KeyCode::PageDown, _) => {
                    app.page_down();
                }

                // Editing
                (KeyCode::Backspace, _) => {
                    app.delete_char();
                }
                (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                    app.clear_query();
                }

                // Regular character input
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    if c != 'j' || !app.query.is_empty() {
                        app.insert_char(c);
                    }
                }

                _ => {}
            }
        }
    }
}

fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(1),    // Table
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    draw_search_input(f, app, chunks[0]);
    draw_table(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
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

fn draw_table(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let header = Row::new(vec![
        Cell::from("Time"),
        Cell::from("Directory"),
        Cell::from("Command"),
        Cell::from("Exit"),
        Cell::from("Duration"),
    ])
    .style(header_style)
    .height(1);

    let rows: Vec<Row> = app
        .filtered
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if Some(i) == app.table_state.selected() {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let exit_style = if entry.exit == 0 {
                style.fg(Color::Green)
            } else {
                style.fg(Color::Red)
            };

            Row::new(vec![
                Cell::from(entry.formatted_time()).style(style),
                Cell::from(entry.short_dir()).style(style),
                Cell::from(entry.cmd.clone()).style(style),
                Cell::from(entry.exit.to_string()).style(exit_style),
                Cell::from(entry.formatted_duration()).style(style),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(19), // Time
            Constraint::Length(25), // Directory
            Constraint::Min(30),    // Command
            Constraint::Length(5),  // Exit
            Constraint::Length(10), // Duration
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_status_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let total = app.entries.len();
    let filtered = app.filtered.len();
    let selected = app.table_state.selected().unwrap_or(0) + 1;

    let status = format!(
        " {}/{} ({} total) │ ↑↓/jk navigate │ PgUp/PgDn scroll │ Enter select │ Esc cancel ",
        selected, filtered, total
    );

    let status_bar =
        Paragraph::new(status).style(Style::default().fg(Color::DarkGray).bg(Color::Black));

    f.render_widget(status_bar, area);
}
