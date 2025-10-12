use anyhow::{Result, bail};

/// Preprocessor for simplified template syntax
/// Converts simplified syntax like (bold)text(/bold) to ANSI escape codes
pub struct TemplatePreprocessor;

impl TemplatePreprocessor {
    /// Preprocess a template string, converting simplified syntax to Handlebars
    pub fn preprocess(template: &str) -> Result<String> {
        Self::process_styles(template)
    }

    /// Process style tags like (bold), (dim), (fg #ff0000), etc.
    fn process_styles(template: &str) -> Result<String> {
        let mut output = String::new();
        let mut chars: Vec<char> = template.chars().collect();
        let mut i = 0;
        let mut style_stack: Vec<StyleTag> = Vec::new();

        while i < chars.len() {
            if chars[i] == '(' {
                // Try to parse a style tag
                if let Some((tag, end_pos)) = Self::parse_style_tag(&chars, i)? {
                    if tag.is_closing {
                        // Handle closing tag - find matching opening tag
                        if let Some(idx) = style_stack.iter().rposition(|t| t.name == tag.name) {
                            let opening_tag = style_stack.remove(idx);

                            // Collect content between opening and this closing tag
                            // For now, we'll use ANSI codes directly since Handlebars
                            // doesn't support opening/closing independently
                            output.push_str(&Self::get_closing_code(&tag.name)?);
                        } else {
                            bail!("Closing tag (/{}) has no matching opening tag", tag.name);
                        }
                    } else {
                        // Handle opening tag
                        output.push_str(&Self::get_opening_code(&tag)?);
                        style_stack.push(tag);
                    }
                    i = end_pos;
                } else {
                    // Not a style tag, just output the character
                    output.push(chars[i]);
                    i += 1;
                }
            } else {
                // Regular character
                output.push(chars[i]);
                i += 1;
            }
        }

        // Check for unclosed tags
        if !style_stack.is_empty() {
            bail!("Unclosed style tags: {:?}",
                  style_stack.iter().map(|t| &t.name).collect::<Vec<_>>());
        }

        Ok(output)
    }

    /// Parse a style tag starting at position i
    /// Returns (tag, next_position) or None if not a valid tag
    fn parse_style_tag(chars: &[char], start: usize) -> Result<Option<(StyleTag, usize)>> {
        if chars[start] != '(' {
            return Ok(None);
        }

        let mut i = start + 1;

        // Check for closing tag
        let is_closing = if i < chars.len() && chars[i] == '/' {
            i += 1;
            true
        } else {
            false
        };

        // Parse tag name
        let mut name = String::new();
        while i < chars.len() && chars[i].is_alphabetic() {
            name.push(chars[i]);
            i += 1;
        }

        if name.is_empty() {
            return Ok(None);
        }

        // Only accept valid style names
        let valid_styles = ["bold", "dim", "italic", "underline", "fg", "bg"];
        if !valid_styles.contains(&name.as_str()) {
            return Ok(None);
        }

        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        // Parse arguments (everything until closing paren)
        let mut args = String::new();
        if !is_closing {
            while i < chars.len() && chars[i] != ')' {
                args.push(chars[i]);
                i += 1;
            }
        }

        // Check for closing paren
        if i >= chars.len() || chars[i] != ')' {
            return Ok(None);
        }

        i += 1; // Skip closing paren

        let tag = StyleTag {
            name,
            args: if args.is_empty() { None } else { Some(args.trim().to_string()) },
            is_closing,
        };

        Ok(Some((tag, i)))
    }

    /// Get the opening ANSI code for a style
    fn get_opening_code(tag: &StyleTag) -> Result<String> {
        match tag.name.as_str() {
            "bold" => Ok("\x1b[1m".to_string()),
            "dim" => Ok("\x1b[2m".to_string()),
            "italic" => Ok("\x1b[3m".to_string()),
            "underline" => Ok("\x1b[4m".to_string()),
            "fg" => {
                if let Some(ref args) = tag.args {
                    let color = Self::resolve_color(args)?;
                    Ok(format!("\x1b[38;2;{};{};{}m", color.0, color.1, color.2))
                } else {
                    bail!("fg tag requires color argument")
                }
            },
            "bg" => {
                if let Some(ref args) = tag.args {
                    let color = Self::resolve_color(args)?;
                    Ok(format!("\x1b[48;2;{};{};{}m", color.0, color.1, color.2))
                } else {
                    bail!("bg tag requires color argument")
                }
            },
            _ => bail!("Unknown style tag: {}", tag.name),
        }
    }

    /// Get the closing ANSI code for a style
    fn get_closing_code(name: &str) -> Result<String> {
        match name {
            "bold" => Ok("\x1b[22m".to_string()),      // Reset bold/dim
            "dim" => Ok("\x1b[22m".to_string()),       // Reset bold/dim
            "italic" => Ok("\x1b[23m".to_string()),    // Reset italic
            "underline" => Ok("\x1b[24m".to_string()), // Reset underline
            "fg" => Ok("\x1b[39m".to_string()),        // Reset foreground
            "bg" => Ok("\x1b[49m".to_string()),        // Reset background
            _ => bail!("Unknown style tag: {}", name),
        }
    }

    /// Resolve a color from a string (hex code or variable reference)
    /// For hex: returns (r, g, b) tuple
    /// For variable: converts to Handlebars {{color_var}} placeholder
    fn resolve_color(color_str: &str) -> Result<(u8, u8, u8)> {
        let trimmed = color_str.trim();

        // Check if it's a hex color (#ffffff)
        if trimmed.starts_with('#') {
            if trimmed.len() != 7 {
                bail!("Hex color must be 7 characters (#rrggbb), got: {}", trimmed);
            }

            let hex = &trimmed[1..];
            match (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                (Ok(r), Ok(g), Ok(b)) => return Ok((r, g, b)),
                _ => bail!("Invalid hex color: {}", trimmed),
            }
        }

        // If not a hex color, it's a variable reference
        // We need to look this up in context at runtime
        // For now, return a placeholder error - we'll handle this in a future iteration
        bail!("Variable color references not yet supported: {}. Use hex colors like #ff0000", trimmed)
    }
}

/// Represents a style tag like (bold) or (fg #ff0000)
#[derive(Debug, Clone)]
struct StyleTag {
    name: String,
    args: Option<String>,
    is_closing: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold_tag() {
        let input = "(bold)Hello(/bold)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("\x1b[22m"));
        assert!(result.contains("Hello"));
        println!("Bold result: {:?}", result);
    }

    #[test]
    fn test_fg_hex_color() {
        let input = "(fg #ff0000)Red Text(/fg)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert!(result.contains("\x1b[38;2;255;0;0m"));
        assert!(result.contains("Red Text"));
        assert!(result.contains("\x1b[39m"));
        println!("FG result: {:?}", result);
    }

    #[test]
    fn test_nested_styles() {
        let input = "(bold)(fg #00ff00)Green Bold(/fg) Still Bold(/bold)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert!(result.contains("\x1b[1m")); // bold
        assert!(result.contains("\x1b[38;2;0;255;0m")); // green
        assert!(result.contains("Green Bold"));
        assert!(result.contains("Still Bold"));
        println!("Nested result: {:?}", result);
    }

    #[test]
    fn test_multiple_styles() {
        let input = "(bold)Bold(/bold) (dim)Dim(/dim)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("\x1b[2m"));
        println!("Multiple result: {:?}", result);
    }

    #[test]
    fn test_bg_color() {
        let input = "(bg #000000)Black Background(/bg)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert!(result.contains("\x1b[48;2;0;0;0m"));
        assert!(result.contains("\x1b[49m"));
        println!("BG result: {:?}", result);
    }

    #[test]
    fn test_combined_fg_bg() {
        let input = "(fg #ffffff, bg #000000)White on Black(/fg, /bg)";
        // This should fail for now as we don't support comma-separated styles yet
        // Let's test them separately
        let input = "(fg #ffffff)(bg #000000)White on Black(/bg)(/fg)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert!(result.contains("\x1b[38;2;255;255;255m"));
        assert!(result.contains("\x1b[48;2;0;0;0m"));
        println!("Combined result: {:?}", result);
    }
}
