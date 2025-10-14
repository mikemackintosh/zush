use std::collections::HashMap;
use handlebars::{Handlebars, RenderContext, Helper, Output, HelperResult, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, Context as AnyhowContext};
use crate::color::Color;

mod preprocessor;
pub use preprocessor::{TemplatePreprocessor, SegmentDef};

/// Template engine for prompt rendering
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    context_data: HashMap<String, Value>,
    colors: HashMap<String, String>,
    symbols: HashMap<String, String>,
    segments: HashMap<String, SegmentDef>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register custom helpers
        handlebars.register_helper("color", Box::new(color_helper));
        handlebars.register_helper("bg", Box::new(bg_helper));
        handlebars.register_helper("fg", Box::new(fg_helper));
        handlebars.register_helper("segment", Box::new(segment_helper));
        handlebars.register_helper("bold", Box::new(bold_helper));
        handlebars.register_helper("dim", Box::new(dim_helper));
        handlebars.register_helper("italic", Box::new(italic_helper));
        handlebars.register_helper("underline", Box::new(underline_helper));
        handlebars.register_helper("reset", Box::new(reset_helper));
        handlebars.register_helper("truncate", Box::new(truncate_helper));
        handlebars.register_helper("pad_left", Box::new(pad_left_helper));
        handlebars.register_helper("pad_right", Box::new(pad_right_helper));
        handlebars.register_helper("center", Box::new(center_helper));
        handlebars.register_helper("line", Box::new(line_helper));
        handlebars.register_helper("format_path", Box::new(format_path_helper));
        handlebars.register_helper("format_time", Box::new(format_time_helper));
        handlebars.register_helper("fill_space", Box::new(fill_space_helper));

        // Disable HTML escaping for terminal output
        handlebars.register_escape_fn(handlebars::no_escape);

        Ok(Self {
            handlebars,
            context_data: HashMap::new(),
            colors: HashMap::new(),
            symbols: HashMap::new(),
            segments: HashMap::new(),
        })
    }

    /// Set colors for template preprocessing
    pub fn set_colors(&mut self, colors: HashMap<String, String>) {
        self.colors = colors;
    }

    /// Set symbols for template preprocessing (used for @symbol_name shortcuts)
    pub fn set_symbols(&mut self, symbols: HashMap<String, String>) {
        self.symbols = symbols;
    }

    /// Add pre-defined segments from TOML configuration
    pub fn add_segments(&mut self, segments: HashMap<String, SegmentDef>) {
        self.segments.extend(segments);
    }

    /// Register a template (with preprocessing for simplified syntax)
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        // Preprocess the template to convert simplified syntax
        let mut preprocessor = TemplatePreprocessor::with_symbols(
            self.colors.clone(),
            self.symbols.clone()
        );
        // Add pre-defined segments from TOML
        preprocessor.add_segments(self.segments.clone());
        let processed = preprocessor.preprocess(template)?;

        self.handlebars
            .register_template_string(name, &processed)
            .with_context(|| format!("Failed to register template: {}", name))?;
        Ok(())
    }

    /// Load templates from a TOML configuration
    pub fn load_templates_from_config(&mut self, config_str: &str) -> Result<()> {
        let config: TemplateConfig = toml::from_str(config_str)?;

        for (name, template) in config.templates {
            self.register_template(&name, &template)?;
        }

        Ok(())
    }

    /// Set context data
    pub fn set_context(&mut self, data: HashMap<String, Value>) {
        self.context_data = data;
    }

    /// Add or update a context value
    pub fn set_value(&mut self, key: &str, value: Value) {
        self.context_data.insert(key.to_string(), value);
    }

    /// Render a template
    pub fn render(&self, template_name: &str) -> Result<String> {
        let result = self.handlebars
            .render(template_name, &self.context_data)
            .with_context(|| format!("Failed to render template: {}", template_name))?;
        Ok(result)
    }

    /// Render a template string directly (with preprocessing for simplified syntax)
    pub fn render_string(&self, template: &str) -> Result<String> {
        // Preprocess the template to convert simplified syntax
        let mut preprocessor = TemplatePreprocessor::with_symbols(
            self.colors.clone(),
            self.symbols.clone()
        );
        // Add pre-defined segments from TOML
        preprocessor.add_segments(self.segments.clone());
        let processed = preprocessor.preprocess(template)?;

        let result = self.handlebars
            .render_template(&processed, &self.context_data)
            .with_context(|| "Failed to render template string")?;
        Ok(result)
    }
}

/// Template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    #[serde(default)]
    pub templates: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colors: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbols: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<HashMap<String, SegmentDefinition>>,
}

/// Segment definition for reusable prompt components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDefinition {
    pub bg: String,
    pub fg: String,
    pub content: String,

    #[serde(default = "default_sep")]
    pub sep: String,  // "sharp", "pill", "slant", "flame", "none"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sep_fg: Option<String>,  // Separator color (defaults to bg)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_cap: Option<String>,  // "pill", "sharp", "slant", "flame", "none"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_cap_fg: Option<String>,  // Left cap color

    #[serde(default)]
    pub fill: bool,  // Fill rest of line with background
}

fn default_sep() -> String {
    "sharp".to_string()
}

// Helper functions for Handlebars

/// Color helper: {{color "hex" "text"}} or {{color r g b "text"}}
fn color_helper(h: &Helper, _: &Handlebars, _ctx: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let params = h.params();

    if params.len() == 2 {
        // Get the color value - it could be a direct hex string or a reference to context
        let color_value = params[0].value();
        let text = params[1].value().as_str().unwrap_or("");

        let hex = if let Some(hex_str) = color_value.as_str() {
            // Direct hex string
            hex_str
        } else {
            // Default fallback
            "#ffffff"
        };

        if let Ok(color) = Color::from_hex(hex) {
            write!(out, "{}{}\x1b[0m", color.to_ansi_fg(), text)?;
        }
    } else if params.len() == 4 {
        // RGB format
        let r = params[0].value().as_u64().unwrap_or(255) as u8;
        let g = params[1].value().as_u64().unwrap_or(255) as u8;
        let b = params[2].value().as_u64().unwrap_or(255) as u8;
        let text = params[3].value().as_str().unwrap_or("");

        let color = Color::new(r, g, b);
        write!(out, "{}{}\x1b[0m", color.to_ansi_fg(), text)?;
    }

    Ok(())
}

/// Background color helper: {{bg "hex"}} or {{bg "hex" "text"}}
fn bg_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let params = h.params();

    if params.len() == 1 {
        // Just set background color (NO RESET)
        let hex = params[0].value().as_str().unwrap_or("#000000");
        if let Ok(color) = Color::from_hex(hex) {
            write!(out, "{}", color.to_ansi_bg())?;
        }
    } else if params.len() == 2 {
        // Background color with text
        let hex = params[0].value().as_str().unwrap_or("#000000");
        let text = params[1].value().as_str().unwrap_or("");
        if let Ok(color) = Color::from_hex(hex) {
            write!(out, "{}{}\x1b[0m", color.to_ansi_bg(), text)?;
        }
    }

    Ok(())
}

/// Foreground color helper (no auto-reset): {{fg "hex"}}
fn fg_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let hex = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("#ffffff");
    if let Ok(color) = Color::from_hex(hex) {
        write!(out, "{}", color.to_ansi_fg())?;
    }
    Ok(())
}

/// Powerline segment helper: {{segment "bg_color" "fg_color" "text"}}
/// Sets both background and foreground, no auto-reset
fn segment_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let params = h.params();

    if params.len() == 3 {
        let bg_hex = params[0].value().as_str().unwrap_or("#000000");
        let fg_hex = params[1].value().as_str().unwrap_or("#ffffff");
        let text = params[2].value().as_str().unwrap_or("");

        if let (Ok(bg_color), Ok(fg_color)) = (Color::from_hex(bg_hex), Color::from_hex(fg_hex)) {
            write!(out, "{}{}{}", bg_color.to_ansi_bg(), fg_color.to_ansi_fg(), text)?;
        }
    }

    Ok(())
}

/// Bold helper: {{bold "text"}}
fn bold_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    write!(out, "\x1b[1m{}\x1b[0m", text)?;
    Ok(())
}

/// Dim helper: {{dim "text"}}
fn dim_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    write!(out, "\x1b[2m{}\x1b[0m", text)?;
    Ok(())
}

/// Italic helper: {{italic "text"}}
fn italic_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    write!(out, "\x1b[3m{}\x1b[0m", text)?;
    Ok(())
}

/// Underline helper: {{underline "text"}}
fn underline_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    write!(out, "\x1b[4m{}\x1b[0m", text)?;
    Ok(())
}

/// Reset helper: {{reset}}
fn reset_helper(_: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    write!(out, "\x1b[0m")?;
    Ok(())
}


/// Truncate helper: {{truncate text max_length}}
fn truncate_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let max_len = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(30) as usize;

    if text.len() > max_len {
        write!(out, "{}...", &text[..max_len.saturating_sub(3)])?;
    } else {
        write!(out, "{}", text)?;
    }

    Ok(())
}

/// Pad left helper: {{pad_left text width}}
fn pad_left_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let width = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(0) as usize;

    write!(out, "{:>width$}", text, width = width)?;
    Ok(())
}

/// Pad right helper: {{pad_right text width}}
fn pad_right_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let width = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(0) as usize;

    write!(out, "{:<width$}", text, width = width)?;
    Ok(())
}

/// Center helper: {{center text width}}
fn center_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let width = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(0) as usize;

    let text_len = unicode_width::UnicodeWidthStr::width(text);
    if text_len < width {
        let padding = (width - text_len) / 2;
        write!(out, "{:padding$}{}", "", text, padding = padding)?;
    } else {
        write!(out, "{}", text)?;
    }

    Ok(())
}

/// Line helper: {{line terminal_width "left_content" "right_content"}}
/// Renders a line with left and right content, filling the space between
fn line_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    use crate::buffer::TerminalBuffer;

    let params = h.params();
    if params.len() < 3 {
        return Ok(());
    }

    let terminal_width = params[0].value().as_u64().unwrap_or(80) as usize;
    let left = params[1].value().as_str().unwrap_or("");
    let right = params[2].value().as_str().unwrap_or("");

    // Calculate visible width (stripping ANSI codes)
    let left_visible = TerminalBuffer::visible_width(left);
    let right_visible = TerminalBuffer::visible_width(right);

    // Calculate spacing needed
    let total_content = left_visible + right_visible;

    if total_content >= terminal_width {
        // No space for padding, just output left and right
        write!(out, "{}{}", left, right)?;
    } else {
        // Add spacing between left and right
        let spacing = terminal_width - total_content;
        write!(out, "{}{:width$}{}", left, "", right, width = spacing)?;
    }

    Ok(())
}

/// Format path helper: {{format_path path "mode"}}
/// Modes: "last", "first:N", "depth:N", "ellipsis", "full"
/// Examples:
///   {{format_path pwd "last"}} -> …/current-dir
///   {{format_path pwd "first:1"}} -> ~/p/z/z-p-r
///   {{format_path pwd "depth:2"}} -> ~/zuper-shell-prompt/zush-prompt-rust
///   {{format_path pwd "ellipsis"}} -> ~/…/zush-prompt-rust
fn format_path_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let path = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let mode = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("full");

    let result = match mode {
        "last" => {
            // Only last segment with ellipsis
            if let Some(last) = path.split('/').last() {
                if path.contains('/') {
                    format!("…/{}", last)
                } else {
                    last.to_string()
                }
            } else {
                path.to_string()
            }
        },
        mode if mode.starts_with("first:") => {
            // First N characters of each segment
            let n = mode.strip_prefix("first:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);

            let segments: Vec<&str> = path.split('/').collect();
            let formatted: Vec<String> = segments.iter().enumerate().map(|(i, seg)| {
                // Don't abbreviate ~ or the last segment
                if seg == &"~" || i == segments.len() - 1 || seg.is_empty() {
                    seg.to_string()
                } else {
                    seg.chars().take(n).collect()
                }
            }).collect();
            formatted.join("/")
        },
        mode if mode.starts_with("depth:") => {
            // Only deepest N directories
            let n = mode.strip_prefix("depth:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(2);

            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if segments.len() <= n {
                // Showing all segments, keep original path
                path.to_string()
            } else {
                // Truncating: use ellipsis to show we're hiding parent directories
                let start_idx = segments.len() - n;
                format!("…/{}", segments[start_idx..].join("/"))
            }
        },
        "ellipsis" => {
            // Base + ellipsis + current (show first and last, hide middle)
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if segments.len() <= 2 {
                path.to_string()
            } else {
                let first = segments[0];
                let last = segments[segments.len() - 1];
                format!("{}/…/{}", first, last)
            }
        },
        _ => path.to_string(), // "full" or unknown mode
    };

    write!(out, "{}", result)?;
    Ok(())
}

/// Format time helper: {{format_time time "format_string"}}
/// Format string uses % placeholders for time parts and style tags for formatting:
///   %H - hours (00-23)
///   %M - minutes (00-59)
///   %S - seconds (00-59)
///   %I - hours 12-hour format (01-12)
///   %p - AM/PM
///
/// Style tags (can wrap any text including time parts):
///   (bold)...(/bold) - bold text
///   (dim)...(/dim) - dim/faint text
///   (italic)...(/italic) - italic text
///   (underline)...(/underline) - underlined text
///
/// Examples:
///   {{format_time time "(bold)%H:%M:%S(/bold)"}} -> bold timestamp
///   {{format_time time "(dim)%H(/dim):%M:%S"}} -> dim hour, normal minutes/seconds
///   {{format_time time "(dim)%H(/dim):(bold)%M:%S(/bold)"}} -> dim hour, bold min:sec
///   {{format_time time "%I:%M %p"}} -> 12-hour format with AM/PM
fn format_time_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    use chrono::{Local, Timelike};

    // Get the time string from context (if provided) or use current time
    let time_str = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let format_str = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("%H:%M:%S");

    // Get current time for formatting
    let now = Local::now();

    // Replace time placeholders
    let mut result = format_str.to_string();
    result = result.replace("%H", &format!("{:02}", now.hour()));
    result = result.replace("%M", &format!("{:02}", now.minute()));
    result = result.replace("%S", &format!("{:02}", now.second()));
    result = result.replace("%I", &format!("{:02}", now.hour12().1));
    result = result.replace("%p", if now.hour() < 12 { "AM" } else { "PM" });

    // Process style tags (both long and short forms)
    result = result.replace("(bold)", "\x1b[1m");
    result = result.replace("(/bold)", "\x1b[22m");
    result = result.replace("(b)", "\x1b[1m");
    result = result.replace("(/b)", "\x1b[22m");

    result = result.replace("(dim)", "\x1b[2m");
    result = result.replace("(/dim)", "\x1b[22m");
    result = result.replace("(d)", "\x1b[2m");
    result = result.replace("(/d)", "\x1b[22m");

    result = result.replace("(italic)", "\x1b[3m");
    result = result.replace("(/italic)", "\x1b[23m");
    result = result.replace("(i)", "\x1b[3m");
    result = result.replace("(/i)", "\x1b[23m");

    result = result.replace("(underline)", "\x1b[4m");
    result = result.replace("(/underline)", "\x1b[24m");
    result = result.replace("(u)", "\x1b[4m");
    result = result.replace("(/u)", "\x1b[24m");

    write!(out, "{}", result)?;
    Ok(())
}

/// Fill space helper: {{fill_space terminal_width left_content right_content offset}}
/// Calculates how much space is needed between left and right content to fill the terminal width.
/// This is useful for creating full-width backgrounds with content on both sides.
///
/// Parameters:
///   - terminal_width: The width of the terminal
///   - left_content: The content on the left side (plain text or with ANSI codes)
///   - right_content: The content on the right side (plain text or with ANSI codes)
///   - offset (optional): Additional characters to subtract (for content not in left/right, like status icons)
///
/// Example:
///   {{fill_space terminal_width pwd_short " / " 4}}
///   This accounts for pwd_short (left), " / " (right), and 4 extra characters (status icon segment)
fn fill_space_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    use crate::buffer::TerminalBuffer;

    let terminal_width = h.param(0)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(80) as usize;

    let left_content = h.param(1)
        .and_then(|v| v.value().as_str())
        .unwrap_or("");

    let right_content = h.param(2)
        .and_then(|v| v.value().as_str())
        .unwrap_or("");

    let offset = h.param(3)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(0) as usize;

    // Calculate visible widths (stripping ANSI codes)
    let left_visible = TerminalBuffer::visible_width(left_content);
    let right_visible = TerminalBuffer::visible_width(right_content);

    // Calculate how many spaces we need to fill the gap
    // Add the offset to account for other content on the line (like status icons)
    let total_content = left_visible + right_visible + offset;
    if total_content < terminal_width {
        let spaces_needed = terminal_width - total_content;
        write!(out, "{:width$}", "", width = spaces_needed)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_rendering() {
        let mut engine = TemplateEngine::new().unwrap();

        engine.register_template("test", "Hello {{name}}!").unwrap();
        engine.set_value("name", json!("World"));

        let result = engine.render("test").unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_color_helper() {
        let mut engine = TemplateEngine::new().unwrap();

        engine.register_template("test", r##"{{color "#ff0000" "Red Text"}}"##).unwrap();

        let result = engine.render("test").unwrap();
        assert!(result.contains("\x1b[38;2;255;0;0m"));
        assert!(result.contains("Red Text"));
    }

    #[test]
    fn test_line_helper() {
        let mut engine = TemplateEngine::new().unwrap();

        engine.register_template("test", r##"{{line 80 "LEFT" "RIGHT"}}"##).unwrap();

        let result = engine.render("test").unwrap();
        assert!(result.contains("LEFT"));
        assert!(result.contains("RIGHT"));
        println!("Line helper output: {}", result);
    }

    #[test]
    fn test_simplified_syntax() {
        let mut engine = TemplateEngine::new().unwrap();

        // Test bold with simplified syntax
        engine.register_template("test_bold", "(bold)Hello World(/bold)").unwrap();
        let result = engine.render("test_bold").unwrap();
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("Hello World"));
        assert!(result.contains("\x1b[22m"));
        println!("Bold test: {:?}", result);

        // Test color with simplified syntax
        engine.register_template("test_color", "(fg #ff0000)Red Text(/fg)").unwrap();
        let result = engine.render("test_color").unwrap();
        assert!(result.contains("\x1b[38;2;255;0;0m"));
        assert!(result.contains("Red Text"));
        println!("Color test: {:?}", result);

        // Test mixed syntax (simplified + handlebars variables)
        engine.register_template("test_mixed", "(bold)User: {{user}}(/bold)").unwrap();
        engine.set_value("user", json!("testuser"));
        let result = engine.render("test_mixed").unwrap();
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("testuser"));
        assert!(result.contains("\x1b[22m"));
        println!("Mixed test: {:?}", result);

        // Test nested styles
        engine.register_template("test_nested", "(bold)(fg #00ff00)Green Bold(/fg) Still Bold(/bold)").unwrap();
        let result = engine.render("test_nested").unwrap();
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("\x1b[38;2;0;255;0m"));
        assert!(result.contains("Green Bold"));
        assert!(result.contains("Still Bold"));
        println!("Nested test: {:?}", result);
    }

    #[test]
    fn test_format_path() {
        let mut engine = TemplateEngine::new().unwrap();
        let test_path = "~/projects/zush/zuper-shell-prompt/zush-prompt-rust";

        // Test "last" mode
        engine.register_template("path_last", r##"{{format_path pwd "last"}}"##).unwrap();
        engine.set_value("pwd", json!(test_path));
        let result = engine.render("path_last").unwrap();
        assert_eq!(result, "…/zush-prompt-rust");
        println!("Path last: {}", result);

        // Test "first:1" mode
        engine.register_template("path_first1", r##"{{format_path pwd "first:1"}}"##).unwrap();
        let result = engine.render("path_first1").unwrap();
        assert_eq!(result, "~/p/z/z/zush-prompt-rust");
        println!("Path first:1: {}", result);

        // Test "first:3" mode
        engine.register_template("path_first3", r##"{{format_path pwd "first:3"}}"##).unwrap();
        let result = engine.render("path_first3").unwrap();
        assert_eq!(result, "~/pro/zus/zup/zush-prompt-rust");
        println!("Path first:3: {}", result);

        // Test "depth:2" mode
        engine.register_template("path_depth2", r##"{{format_path pwd "depth:2"}}"##).unwrap();
        let result = engine.render("path_depth2").unwrap();
        assert_eq!(result, "~/zuper-shell-prompt/zush-prompt-rust");
        println!("Path depth:2: {}", result);

        // Test "ellipsis" mode
        engine.register_template("path_ellipsis", r##"{{format_path pwd "ellipsis"}}"##).unwrap();
        let result = engine.render("path_ellipsis").unwrap();
        assert_eq!(result, "~/…/zush-prompt-rust");
        println!("Path ellipsis: {}", result);

        // Test "full" mode (default)
        engine.register_template("path_full", r##"{{format_path pwd "full"}}"##).unwrap();
        let result = engine.render("path_full").unwrap();
        assert_eq!(result, test_path);
        println!("Path full: {}", result);

        // Test short path
        let short_path = "~/documents";
        engine.set_value("pwd", json!(short_path));
        engine.register_template("path_short_last", r##"{{format_path pwd "last"}}"##).unwrap();
        let result = engine.render("path_short_last").unwrap();
        assert_eq!(result, "…/documents");
        println!("Short path last: {}", result);
    }
}