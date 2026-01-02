#![allow(dead_code)]

use anyhow::{Context, Result};
use std::io::{self, Write};
use terminal_size::{terminal_size, Height, Width};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Represents a position in the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

/// Alignment options for text
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// A buffer for building terminal output with perfect positioning
#[allow(dead_code)]
pub struct TerminalBuffer {
    width: u16,
    height: u16,
    content: Vec<Vec<char>>,
    styles: Vec<Vec<String>>,
    cursor: Position,
}

impl TerminalBuffer {
    /// Create a new terminal buffer
    pub fn new() -> Result<Self> {
        let (Width(width), Height(height)) =
            terminal_size().context("Failed to get terminal size")?;

        Ok(Self {
            width,
            height,
            content: vec![vec![' '; width as usize]; height as usize],
            styles: vec![vec![String::new(); width as usize]; height as usize],
            cursor: Position { row: 0, col: 0 },
        })
    }

    /// Create a buffer with specific dimensions
    pub fn with_dimensions(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            content: vec![vec![' '; width as usize]; height as usize],
            styles: vec![vec![String::new(); width as usize]; height as usize],
            cursor: Position { row: 0, col: 0 },
        }
    }

    /// Get current terminal width
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get current terminal height
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Write text at a specific position
    pub fn write_at(&mut self, pos: Position, text: &str, style: Option<&str>) -> Result<()> {
        if pos.row >= self.height || pos.col >= self.width {
            return Ok(()); // Silently ignore out-of-bounds writes
        }

        let style = style.unwrap_or("");
        let mut col = pos.col as usize;

        for grapheme in text.graphemes(true) {
            let width = UnicodeWidthStr::width(grapheme);

            if col + width > self.width as usize {
                break; // Stop at terminal edge
            }

            // Handle multi-column characters
            if width > 0 {
                for ch in grapheme.chars() {
                    if col < self.width as usize {
                        self.content[pos.row as usize][col] = ch;
                        self.styles[pos.row as usize][col] = style.to_string();
                        col += 1;
                        break; // Only use first char for display
                    }
                }

                // Fill remaining columns for wide characters
                for _ in 1..width {
                    if col < self.width as usize {
                        self.content[pos.row as usize][col] = '\0'; // Continuation marker
                        self.styles[pos.row as usize][col] = style.to_string();
                        col += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Write aligned text on a specific row
    pub fn write_aligned(
        &mut self,
        row: u16,
        text: &str,
        alignment: Alignment,
        style: Option<&str>,
    ) -> Result<()> {
        let text_width = UnicodeWidthStr::width(text);

        let col = match alignment {
            Alignment::Left => 0,
            Alignment::Center => self.width.saturating_sub(text_width as u16) / 2,
            Alignment::Right => self.width.saturating_sub(text_width as u16),
        };

        self.write_at(Position { row, col }, text, style)
    }

    /// Write text in three sections (left, center, right) on the same line
    pub fn write_three_sections(
        &mut self,
        row: u16,
        left: Option<&str>,
        center: Option<&str>,
        right: Option<&str>,
        left_style: Option<&str>,
        center_style: Option<&str>,
        right_style: Option<&str>,
    ) -> Result<()> {
        // Calculate available space
        let left_text = left.unwrap_or("");
        let center_text = center.unwrap_or("");
        let right_text = right.unwrap_or("");

        let left_width = UnicodeWidthStr::width(left_text);
        let center_width = UnicodeWidthStr::width(center_text);
        let right_width = UnicodeWidthStr::width(right_text);

        // Write left section
        if !left_text.is_empty() {
            self.write_at(Position { row, col: 0 }, left_text, left_style)?;
        }

        // Write center section
        if !center_text.is_empty() {
            let center_pos = (self.width as usize).saturating_sub(center_width) / 2;

            // Ensure center doesn't overlap with left
            let center_pos = center_pos.max(left_width + 1);

            self.write_at(
                Position {
                    row,
                    col: center_pos as u16,
                },
                center_text,
                center_style,
            )?;
        }

        // Write right section
        if !right_text.is_empty() {
            let right_pos = (self.width as usize).saturating_sub(right_width);

            // Ensure right doesn't overlap with center
            let min_right_pos = if !center_text.is_empty() {
                let center_pos = (self.width as usize).saturating_sub(center_width) / 2;
                (center_pos + center_width + 1).min(right_pos)
            } else {
                right_pos
            };

            self.write_at(
                Position {
                    row,
                    col: min_right_pos.max(left_width + 1) as u16,
                },
                right_text,
                right_style,
            )?;
        }

        Ok(())
    }

    /// Clear a line
    pub fn clear_line(&mut self, row: u16) {
        if row < self.height {
            self.content[row as usize] = vec![' '; self.width as usize];
            self.styles[row as usize] = vec![String::new(); self.width as usize];
        }
    }

    /// Clear the entire buffer
    pub fn clear(&mut self) {
        self.content = vec![vec![' '; self.width as usize]; self.height as usize];
        self.styles = vec![vec![String::new(); self.width as usize]; self.height as usize];
        self.cursor = Position { row: 0, col: 0 };
    }

    /// Render the buffer to a string
    pub fn render(&self) -> String {
        let mut output = String::new();

        for row in 0..self.height {
            let row_idx = row as usize;
            let mut last_style = String::new();

            for col in 0..self.width {
                let col_idx = col as usize;
                let ch = self.content[row_idx][col_idx];

                // Skip continuation markers for wide characters
                if ch == '\0' {
                    continue;
                }

                // Apply style if different from last
                if self.styles[row_idx][col_idx] != last_style {
                    if !last_style.is_empty() {
                        output.push_str("\x1b[0m"); // Reset previous style
                    }
                    output.push_str(&self.styles[row_idx][col_idx]);
                    last_style = self.styles[row_idx][col_idx].clone();
                }

                output.push(ch);
            }

            // Reset style at end of line
            if !last_style.is_empty() {
                output.push_str("\x1b[0m");
            }

            // Add newline except for last line
            if row < self.height - 1 {
                output.push('\n');
            }
        }

        output
    }

    /// Render only a specific line
    pub fn render_line(&self, row: u16) -> String {
        if row >= self.height {
            return String::new();
        }

        let row_idx = row as usize;
        let mut output = String::new();
        let mut last_style = String::new();
        let mut last_non_space = 0;

        // Find last non-space character to avoid trailing spaces
        for col in (0..self.width).rev() {
            if self.content[row_idx][col as usize] != ' ' {
                last_non_space = col;
                break;
            }
        }

        for col in 0..=last_non_space {
            let col_idx = col as usize;
            let ch = self.content[row_idx][col_idx];

            // Skip continuation markers
            if ch == '\0' {
                continue;
            }

            // Apply style if different
            if self.styles[row_idx][col_idx] != last_style {
                if !last_style.is_empty() {
                    output.push_str("\x1b[0m");
                }
                output.push_str(&self.styles[row_idx][col_idx]);
                last_style = self.styles[row_idx][col_idx].clone();
            }

            output.push(ch);
        }

        // Reset style at end
        if !last_style.is_empty() {
            output.push_str("\x1b[0m");
        }

        output
    }

    /// Write the buffer to stdout
    pub fn flush_to_stdout(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        for row in 0..self.height {
            let line = self.render_line(row);
            if !line.is_empty() || row == 0 {
                write!(stdout, "{}", line)?;
                if row < self.height - 1 {
                    writeln!(stdout)?;
                }
            }
        }

        stdout.flush()?;
        Ok(())
    }

    /// Calculate visible width of a string (accounting for ANSI escapes)
    pub fn visible_width(text: &str) -> usize {
        let mut width = 0;
        let mut in_escape = false;

        for ch in text.chars() {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if ch == 'm' {
                    in_escape = false;
                }
            } else {
                width += UnicodeWidthStr::width(ch.to_string().as_str());
            }
        }

        width
    }

    /// Strip ANSI escape codes from text
    pub fn strip_ansi(text: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;

        for ch in text.chars() {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if ch == 'm' {
                    in_escape = false;
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

/// Prompt line builder for structured prompt creation
#[allow(dead_code)]
pub struct PromptLine {
    left: String,
    center: String,
    right: String,
    left_style: String,
    center_style: String,
    right_style: String,
}

impl PromptLine {
    /// Create a new prompt line
    pub fn new() -> Self {
        Self {
            left: String::new(),
            center: String::new(),
            right: String::new(),
            left_style: String::new(),
            center_style: String::new(),
            right_style: String::new(),
        }
    }

    /// Set left section
    pub fn left(mut self, text: &str, style: Option<&str>) -> Self {
        self.left = text.to_string();
        self.left_style = style.unwrap_or("").to_string();
        self
    }

    /// Set center section
    pub fn center(mut self, text: &str, style: Option<&str>) -> Self {
        self.center = text.to_string();
        self.center_style = style.unwrap_or("").to_string();
        self
    }

    /// Set right section
    pub fn right(mut self, text: &str, style: Option<&str>) -> Self {
        self.right = text.to_string();
        self.right_style = style.unwrap_or("").to_string();
        self
    }

    /// Render to a terminal buffer
    pub fn render_to_buffer(&self, buffer: &mut TerminalBuffer, row: u16) -> Result<()> {
        buffer.write_three_sections(
            row,
            if self.left.is_empty() {
                None
            } else {
                Some(&self.left)
            },
            if self.center.is_empty() {
                None
            } else {
                Some(&self.center)
            },
            if self.right.is_empty() {
                None
            } else {
                Some(&self.right)
            },
            if self.left_style.is_empty() {
                None
            } else {
                Some(&self.left_style)
            },
            if self.center_style.is_empty() {
                None
            } else {
                Some(&self.center_style)
            },
            if self.right_style.is_empty() {
                None
            } else {
                Some(&self.right_style)
            },
        )
    }

    /// Render directly to a string with proper spacing
    pub fn render(&self, width: u16) -> String {
        let mut buffer = TerminalBuffer::with_dimensions(width, 1);
        self.render_to_buffer(&mut buffer, 0).unwrap_or_default();
        buffer.render_line(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = TerminalBuffer::with_dimensions(80, 24);
        assert_eq!(buffer.width(), 80);
        assert_eq!(buffer.height(), 24);
    }

    #[test]
    fn test_write_at() {
        let mut buffer = TerminalBuffer::with_dimensions(80, 24);
        buffer
            .write_at(Position { row: 0, col: 0 }, "Hello", None)
            .unwrap();
        let line = buffer.render_line(0);
        assert!(line.contains("Hello"));
    }

    #[test]
    fn test_three_sections() {
        let mut buffer = TerminalBuffer::with_dimensions(80, 1);
        buffer
            .write_three_sections(
                0,
                Some("Left"),
                Some("Center"),
                Some("Right"),
                None,
                None,
                None,
            )
            .unwrap();

        let line = buffer.render_line(0);
        assert!(line.contains("Left"));
        assert!(line.contains("Center"));
        assert!(line.contains("Right"));
    }

    #[test]
    fn test_visible_width() {
        assert_eq!(TerminalBuffer::visible_width("Hello"), 5);
        assert_eq!(TerminalBuffer::visible_width("\x1b[31mRed\x1b[0m"), 3);
        assert_eq!(TerminalBuffer::visible_width("日本語"), 6); // Wide characters
    }
}
