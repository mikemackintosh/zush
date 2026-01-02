#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a 24-bit RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color from RGB values
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse color from hex string (e.g., "#1a1b26" or "1a1b26")
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err(anyhow!("Invalid hex color: {}", hex));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;

        Ok(Self::new(r, g, b))
    }

    /// Convert to ANSI escape sequence for foreground
    pub fn to_ansi_fg(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Convert to ANSI escape sequence for background
    pub fn to_ansi_bg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Convert to Zsh-escaped foreground color
    pub fn to_zsh_fg(&self) -> String {
        format!("%{{\\e[38;2;{};{};{}m%}}", self.r, self.g, self.b)
    }

    /// Convert to Zsh-escaped background color
    pub fn to_zsh_bg(&self) -> String {
        format!("%{{\\e[48;2;{};{};{}m%}}", self.r, self.g, self.b)
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Lighten the color by a percentage (0.0 to 1.0)
    pub fn lighten(&self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 + (255.0 - self.r as f32) * amount) as u8,
            g: (self.g as f32 + (255.0 - self.g as f32) * amount) as u8,
            b: (self.b as f32 + (255.0 - self.b as f32) * amount) as u8,
        }
    }

    /// Darken the color by a percentage (0.0 to 1.0)
    pub fn darken(&self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * (1.0 - amount)) as u8,
            g: (self.g as f32 * (1.0 - amount)) as u8,
            b: (self.b as f32 * (1.0 - amount)) as u8,
        }
    }

    /// Mix with another color
    pub fn mix(&self, other: &Color, ratio: f32) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * (1.0 - ratio) + other.r as f32 * ratio) as u8,
            g: (self.g as f32 * (1.0 - ratio) + other.g as f32 * ratio) as u8,
            b: (self.b as f32 * (1.0 - ratio) + other.b as f32 * ratio) as u8,
        }
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Color::from_hex(s)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Tokyo Night color palette
#[allow(dead_code)]
pub mod tokyo_night {
    use super::Color;

    pub const BG: Color = Color {
        r: 0x1a,
        g: 0x1b,
        b: 0x26,
    };
    pub const FG: Color = Color {
        r: 0xc0,
        g: 0xca,
        b: 0xf5,
    };
    pub const FG_DARK: Color = Color {
        r: 0xa9,
        g: 0xb1,
        b: 0xd6,
    };
    pub const FG_DIM: Color = Color {
        r: 0x56,
        g: 0x5f,
        b: 0x89,
    };

    pub const BLACK: Color = Color {
        r: 0x15,
        g: 0x16,
        b: 0x1e,
    };
    pub const RED: Color = Color {
        r: 0xf7,
        g: 0x76,
        b: 0x8e,
    };
    pub const GREEN: Color = Color {
        r: 0x9e,
        g: 0xce,
        b: 0x6a,
    };
    pub const YELLOW: Color = Color {
        r: 0xe0,
        g: 0xaf,
        b: 0x68,
    };
    pub const BLUE: Color = Color {
        r: 0x7a,
        g: 0xa2,
        b: 0xf7,
    };
    pub const MAGENTA: Color = Color {
        r: 0xbb,
        g: 0x9a,
        b: 0xf7,
    };
    pub const CYAN: Color = Color {
        r: 0x7d,
        g: 0xcf,
        b: 0xff,
    };
    pub const WHITE: Color = Color {
        r: 0xc0,
        g: 0xca,
        b: 0xf5,
    };

    pub const ORANGE: Color = Color {
        r: 0xff,
        g: 0x9e,
        b: 0x64,
    };
    pub const PURPLE: Color = Color {
        r: 0x9d,
        g: 0x7c,
        b: 0xd8,
    };
    pub const TEAL: Color = Color {
        r: 0x1a,
        g: 0xbc,
        b: 0x9c,
    };

    pub const BRIGHT_BLACK: Color = Color {
        r: 0x41,
        g: 0x4a,
        b: 0x68,
    };
    pub const BRIGHT_RED: Color = Color {
        r: 0xf7,
        g: 0x76,
        b: 0x8e,
    };
    pub const BRIGHT_GREEN: Color = Color {
        r: 0x9e,
        g: 0xce,
        b: 0x6a,
    };
    pub const BRIGHT_YELLOW: Color = Color {
        r: 0xe0,
        g: 0xaf,
        b: 0x68,
    };
    pub const BRIGHT_BLUE: Color = Color {
        r: 0x7a,
        g: 0xa2,
        b: 0xf7,
    };
    pub const BRIGHT_MAGENTA: Color = Color {
        r: 0xbb,
        g: 0x9a,
        b: 0xf7,
    };
    pub const BRIGHT_CYAN: Color = Color {
        r: 0x7d,
        g: 0xcf,
        b: 0xff,
    };
    pub const BRIGHT_WHITE: Color = Color {
        r: 0xd5,
        g: 0xd6,
        b: 0xdb,
    };
}

/// Color scheme trait for custom themes
#[allow(dead_code)]
pub trait ColorScheme {
    fn background(&self) -> Color;
    fn foreground(&self) -> Color;
    fn black(&self) -> Color;
    fn red(&self) -> Color;
    fn green(&self) -> Color;
    fn yellow(&self) -> Color;
    fn blue(&self) -> Color;
    fn magenta(&self) -> Color;
    fn cyan(&self) -> Color;
    fn white(&self) -> Color;
}

/// ANSI escape code helpers
#[allow(dead_code)]
pub mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const BLINK: &str = "\x1b[5m";
    pub const REVERSE: &str = "\x1b[7m";
    pub const HIDDEN: &str = "\x1b[8m";
    pub const STRIKETHROUGH: &str = "\x1b[9m";

    /// Clear to end of line
    pub const CLEAR_EOL: &str = "\x1b[K";

    /// Save cursor position
    pub const SAVE_CURSOR: &str = "\x1b[s";

    /// Restore cursor position
    pub const RESTORE_CURSOR: &str = "\x1b[u";

    /// Move cursor up N lines
    pub fn cursor_up(n: u16) -> String {
        format!("\x1b[{}A", n)
    }

    /// Move cursor down N lines
    pub fn cursor_down(n: u16) -> String {
        format!("\x1b[{}B", n)
    }

    /// Move cursor forward N columns
    pub fn cursor_forward(n: u16) -> String {
        format!("\x1b[{}C", n)
    }

    /// Move cursor backward N columns
    pub fn cursor_backward(n: u16) -> String {
        format!("\x1b[{}D", n)
    }

    /// Move cursor to specific position (1-indexed)
    pub fn cursor_goto(row: u16, col: u16) -> String {
        format!("\x1b[{};{}H", row, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#1a1b26").unwrap();
        assert_eq!(color.r, 0x1a);
        assert_eq!(color.g, 0x1b);
        assert_eq!(color.b, 0x26);

        let color = Color::from_hex("ff9e64").unwrap();
        assert_eq!(color.r, 0xff);
        assert_eq!(color.g, 0x9e);
        assert_eq!(color.b, 0x64);
    }

    #[test]
    fn test_color_to_ansi() {
        let color = Color::new(255, 158, 100);
        assert_eq!(color.to_ansi_fg(), "\x1b[38;2;255;158;100m");
        assert_eq!(color.to_ansi_bg(), "\x1b[48;2;255;158;100m");
    }

    #[test]
    fn test_color_to_zsh() {
        let color = Color::new(255, 158, 100);
        assert_eq!(color.to_zsh_fg(), "%{\\e[38;2;255;158;100m%}");
        assert_eq!(color.to_zsh_bg(), "%{\\e[48;2;255;158;100m%}");
    }

    #[test]
    fn test_color_lighten_darken() {
        let color = Color::new(100, 100, 100);
        let lighter = color.lighten(0.5);
        assert!(lighter.r > color.r);

        let darker = color.darken(0.5);
        assert!(darker.r < color.r);
    }
}
