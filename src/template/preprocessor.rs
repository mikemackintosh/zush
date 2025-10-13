use anyhow::{Result, bail};
use std::collections::HashMap;

/// Preprocessor for simplified template syntax
/// Converts simplified syntax like (bold)text(/bold) to ANSI escape codes
/// Also handles @symbol_name shorthand for symbols
/// And handles {{segment}}...{{endsegment}} blocks with {{seg:name}} references
pub struct TemplatePreprocessor {
    colors: HashMap<String, String>,
    symbols: HashMap<String, String>,
    segments: HashMap<String, SegmentDef>,
}

/// Parsed segment definition from {{segment}} blocks or TOML
#[derive(Debug, Clone)]
pub struct SegmentDef {
    pub name: String,
    pub bg: Option<String>,
    pub fg: Option<String>,
    pub content: String,
    pub sep: Option<String>,
    pub left_cap: Option<String>,
}

impl SegmentDef {
    /// Create a new segment definition
    pub fn new(name: String, content: String) -> Self {
        Self {
            name,
            bg: None,
            fg: None,
            content,
            sep: None,
            left_cap: None,
        }
    }

    /// Set background color
    pub fn with_bg(mut self, bg: String) -> Self {
        self.bg = Some(bg);
        self
    }

    /// Set foreground color
    pub fn with_fg(mut self, fg: String) -> Self {
        self.fg = Some(fg);
        self
    }

    /// Set separator shape
    pub fn with_sep(mut self, sep: String) -> Self {
        self.sep = Some(sep);
        self
    }

    /// Set left cap shape
    pub fn with_left_cap(mut self, left_cap: String) -> Self {
        self.left_cap = Some(left_cap);
        self
    }
}

impl TemplatePreprocessor {
    /// Create a new preprocessor with color and symbol maps
    pub fn new(colors: HashMap<String, String>) -> Self {
        Self {
            colors,
            symbols: HashMap::new(),
            segments: HashMap::new(),
        }
    }

    /// Create a new preprocessor with both colors and symbols
    pub fn with_symbols(colors: HashMap<String, String>, symbols: HashMap<String, String>) -> Self {
        Self {
            colors,
            symbols,
            segments: HashMap::new(),
        }
    }

    /// Add pre-defined segments from TOML configuration
    pub fn add_segments(&mut self, segments: HashMap<String, SegmentDef>) {
        self.segments.extend(segments);
    }

    /// Preprocess a template string, converting simplified syntax to Handlebars
    /// This includes:
    /// - Segment definitions: {{segment "name" ...}}...{{endsegment}}
    /// - Segment references: {{seg:name}}
    /// - Symbol shorthand: @symbol_name
    /// - Style tags: (b)text(/b), (fg color)text(/fg), etc.
    pub fn preprocess(&mut self, template: &str) -> Result<String> {
        // First, extract and remove segment definitions
        let (without_segments, segments) = self.extract_segments(template)?;
        // Extend segments instead of replacing - this preserves TOML-defined segments
        // Template-defined segments can override TOML ones
        self.segments.extend(segments);

        // Replace segment references with their content
        let with_segments = self.expand_segment_references(&without_segments)?;

        // Replace @symbol shortcuts
        let with_symbols = self.process_symbol_shortcuts(&with_segments)?;

        // Finally process style tags
        self.process_styles(&with_symbols)
    }

    /// Extract {{segment}}...{{endsegment}} blocks and remove them from template
    /// Returns (template_without_definitions, segment_map)
    fn extract_segments(&self, template: &str) -> Result<(String, HashMap<String, SegmentDef>)> {
        let mut result = String::new();
        let mut segments = HashMap::new();
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Look for {{segment
            if i + 9 < chars.len() &&
               chars[i..i+2] == ['{', '{'] &&
               chars[i+2..i+9].iter().collect::<String>() == "segment" {

                // Parse segment definition
                let seg_start = i;
                i += 9; // Skip "{{segment"

                // Find the end of opening tag }}
                let mut tag_end = i;
                while tag_end < chars.len() && !(chars[tag_end] == '}' && chars[tag_end+1] == '}') {
                    tag_end += 1;
                }

                if tag_end >= chars.len() {
                    bail!("Unclosed {{{{segment}}}} tag");
                }

                // Extract parameters from opening tag
                let params_str: String = chars[i..tag_end].iter().collect();
                let segment_def = self.parse_segment_params(&params_str)?;

                // Find {{endsegment}}
                i = tag_end + 2; // Skip }}
                let content_start = i;
                let mut content_end = i;
                let mut depth = 1;

                while content_end < chars.len() && depth > 0 {
                    if content_end + 13 < chars.len() &&
                       chars[content_end..content_end+2] == ['{', '{'] &&
                       chars[content_end+2..content_end+12].iter().collect::<String>() == "endsegment" {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    } else if content_end + 9 < chars.len() &&
                              chars[content_end..content_end+2] == ['{', '{'] &&
                              chars[content_end+2..content_end+9].iter().collect::<String>() == "segment" {
                        depth += 1;
                    }
                    content_end += 1;
                }

                if depth > 0 {
                    bail!("Unclosed {{{{segment}}}} block - missing {{{{endsegment}}}}");
                }

                // Extract content
                let content: String = chars[content_start..content_end].iter().collect();
                let mut seg_def = segment_def;
                seg_def.content = content.trim().to_string();

                segments.insert(seg_def.name.clone(), seg_def);

                // Skip past {{endsegment}}
                i = content_end + 14; // {{endsegment}} is 14 chars
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        Ok((result, segments))
    }

    /// Parse segment parameters from the opening tag
    /// Format: "name" bg="color" fg="color" sep="shape" left_cap="shape"
    fn parse_segment_params(&self, params: &str) -> Result<SegmentDef> {
        let params = params.trim();

        // Extract name (first quoted string)
        let name_start = params.find('"').ok_or_else(|| anyhow::anyhow!("Segment must have a name in quotes"))?;
        let name_end = params[name_start+1..].find('"').ok_or_else(|| anyhow::anyhow!("Unclosed segment name quote"))?;
        let name = params[name_start+1..name_start+1+name_end].to_string();

        // Extract other parameters (all optional)
        let bg = self.extract_param(params, "bg=").ok();
        let fg = self.extract_param(params, "fg=").ok();
        let sep = self.extract_param(params, "sep=").ok();
        let left_cap = self.extract_param(params, "left_cap=").ok();

        Ok(SegmentDef {
            name,
            bg,
            fg,
            content: String::new(), // Will be filled later
            sep,
            left_cap,
        })
    }

    /// Extract a parameter value from the params string
    fn extract_param(&self, params: &str, key: &str) -> Result<String> {
        let key_pos = params.find(key).ok_or_else(|| anyhow::anyhow!("Missing required parameter: {}", key))?;
        let value_start = key_pos + key.len();
        let start_quote = params[value_start..].find('"').ok_or_else(|| anyhow::anyhow!("Parameter {} must be quoted", key))?;
        let end_quote = params[value_start+start_quote+1..].find('"').ok_or_else(|| anyhow::anyhow!("Unclosed quote for parameter {}", key))?;

        Ok(params[value_start+start_quote+1..value_start+start_quote+1+end_quote].to_string())
    }

    /// Expand {{seg:name}} references with segment content
    fn expand_segment_references(&self, template: &str) -> Result<String> {
        let mut result = String::new();
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Look for {{seg:
            if i + 6 < chars.len() &&
               chars[i..i+2] == ['{', '{'] &&
               chars[i+2..i+6] == ['s', 'e', 'g', ':'] {

                i += 6; // Skip "{{seg:"

                // Extract segment name
                let name_start = i;
                while i < chars.len() && chars[i] != '}' {
                    i += 1;
                }

                if i >= chars.len() || i+1 >= chars.len() || chars[i] != '}' || chars[i+1] != '}' {
                    bail!("Unclosed {{{{seg:name}}}} reference");
                }

                let name: String = chars[name_start..i].iter().collect();
                let name = name.trim();

                // Look up segment and expand it
                if let Some(segment) = self.segments.get(name) {
                    let expanded = self.render_segment(segment)?;
                    result.push_str(&expanded);
                } else {
                    bail!("Unknown segment: '{}'. Define it with {{{{segment \"{}\" ...}}}}...{{{{endsegment}}}}", name, name);
                }

                i += 2; // Skip }}
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        Ok(result)
    }

    /// Render a segment with its styling
    fn render_segment(&self, segment: &SegmentDef) -> Result<String> {
        let mut output = String::new();

        // Add left cap if specified
        if let Some(ref cap_shape) = segment.left_cap {
            let cap_symbol = self.get_separator_symbol(cap_shape)?;
            if let Some(ref bg) = segment.bg {
                output.push_str(&format!("(fg {}){}(/fg)", bg, cap_symbol));
            } else {
                output.push_str(&cap_symbol);
            }
        }

        // Add background and foreground with content
        let has_bg = segment.bg.is_some();
        let has_fg = segment.fg.is_some();

        if has_bg {
            output.push_str(&format!("(bg {})", segment.bg.as_ref().unwrap()));
        }
        if has_fg {
            output.push_str(&format!("(fg {})", segment.fg.as_ref().unwrap()));
        }

        output.push_str(&format!(" {} ", segment.content));

        if has_fg {
            output.push_str("(/fg)");
        }
        if has_bg {
            output.push_str("(/bg)");
        }

        // Add right separator if specified
        if let Some(ref sep_shape) = segment.sep {
            let sep_symbol = self.get_separator_symbol(sep_shape)?;
            if let Some(ref bg) = segment.bg {
                output.push_str(&format!("(fg {}){}(/fg)", bg, sep_symbol));
            } else {
                output.push_str(&sep_symbol);
            }
        }

        Ok(output)
    }

    /// Get the separator symbol for a shape name
    fn get_separator_symbol(&self, shape: &str) -> Result<String> {
        let symbol = match shape {
            "sharp" | "triangle" => "@segment_separator",
            "pill" | "round" => "@pill_left",
            "slant" => "@slant_right",
            "flame" => "@flame_right",
            "none" => "",
            _ => bail!("Unknown separator shape: {}. Use: sharp, pill, slant, flame, none", shape),
        };
        Ok(symbol.to_string())
    }

    /// Process @symbol_name shortcuts, replacing them with the actual symbol characters
    /// Theme symbols take precedence over built-in symbols
    fn process_symbol_shortcuts(&self, template: &str) -> Result<String> {
        let mut output = String::new();
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Check for handlebars blocks - pass through without processing
            if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                output.push(chars[i]);
                i += 1;
                while i < chars.len() {
                    output.push(chars[i]);
                    if i + 1 < chars.len() && chars[i] == '}' && chars[i + 1] == '}' {
                        i += 1;
                        output.push(chars[i]);
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            // Check for @symbol_name syntax
            if chars[i] == '@' {
                i += 1;
                let mut symbol_name = String::new();

                // Parse symbol name (alphanumeric and underscore)
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    symbol_name.push(chars[i]);
                    i += 1;
                }

                if !symbol_name.is_empty() {
                    // Try theme symbols first, then built-in
                    if let Some(symbol_value) = self.symbols.get(&symbol_name) {
                        output.push_str(symbol_value);
                    } else if let Ok(builtin_symbol) = Self::resolve_symbol(&symbol_name) {
                        output.push_str(&builtin_symbol);
                    } else {
                        bail!("Unknown symbol '@{}'. Define it in [symbols] section or use a built-in symbol.", symbol_name);
                    }
                } else {
                    // Just a standalone @, output it
                    output.push('@');
                }
            } else {
                output.push(chars[i]);
                i += 1;
            }
        }

        Ok(output)
    }

    /// Process style tags like (bold), (dim), (fg #ff0000), etc.
    fn process_styles(&self, template: &str) -> Result<String> {
        let mut output = String::new();
        let mut chars: Vec<char> = template.chars().collect();
        let mut i = 0;
        let mut style_stack: Vec<StyleTag> = Vec::new();

        while i < chars.len() {
            // Check for handlebars block start {{ - pass through without processing
            if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                // Find the end of the handlebars block
                output.push(chars[i]);
                i += 1;
                while i < chars.len() {
                    output.push(chars[i]);
                    if i + 1 < chars.len() && chars[i] == '}' && chars[i + 1] == '}' {
                        i += 1;
                        output.push(chars[i]);
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            if chars[i] == '(' {
                // Try to parse a style tag
                if let Some((tag, end_pos)) = Self::parse_style_tag(&chars, i)? {
                    if tag.is_closing {
                        // Handle closing tag - find matching opening tag
                        if let Some(idx) = style_stack.iter().rposition(|t| t.name == tag.name) {
                            let _opening_tag = style_stack.remove(idx);

                            // Collect content between opening and this closing tag
                            // For now, we'll use ANSI codes directly since Handlebars
                            // doesn't support opening/closing independently
                            output.push_str(&Self::get_closing_code(&tag.name)?);
                        } else {
                            // Don't error on unmatched closing tags - they may be in handlebars branches
                            // Just output the closing code and continue
                            output.push_str(&Self::get_closing_code(&tag.name)?);
                        }
                    } else {
                        // Handle opening tag
                        output.push_str(&self.get_opening_code(&tag)?);

                        // Symbols don't need closing tags, so don't push to stack
                        if tag.name != "sym" {
                            style_stack.push(tag);
                        }
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

        // Check for unclosed tags - but be lenient for tags that might be closed in handlebars branches
        // We'll warn about unclosed tags at the end of the template
        if !style_stack.is_empty() {
            let unclosed: Vec<String> = style_stack.iter().map(|t| t.name.clone()).collect();
            bail!("Unclosed style tags at end of template: {:?}. Each opening tag like (bold) must have a matching closing tag like (/bold).", unclosed);
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

        // Only accept valid style names and symbol tag
        let valid_styles = ["bold", "b", "em", "i", "d", "u", "dim", "italic", "underline", "fg", "bg", "sym"];
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
    fn get_opening_code(&self, tag: &StyleTag) -> Result<String> {
        match tag.name.as_str() {
            "bold" | "b" => Ok("\x1b[1m".to_string()),
            "dim" | "d" => Ok("\x1b[2m".to_string()),
            "italic" | "em" | "i" => Ok("\x1b[3m".to_string()),
            "underline" | "u" => Ok("\x1b[4m".to_string()),
            "fg" => {
                if let Some(ref args) = tag.args {
                    let color = self.resolve_color(args)?;
                    Ok(format!("\x1b[38;2;{};{};{}m", color.0, color.1, color.2))
                } else {
                    bail!("fg tag requires color argument")
                }
            },
            "bg" => {
                if let Some(ref args) = tag.args {
                    let color = self.resolve_color(args)?;
                    Ok(format!("\x1b[48;2;{};{};{}m", color.0, color.1, color.2))
                } else {
                    bail!("bg tag requires color argument")
                }
            },
            "sym" => {
                if let Some(ref args) = tag.args {
                    Self::resolve_symbol(args)
                } else {
                    bail!("sym tag requires symbol name argument")
                }
            },
            _ => bail!("Unknown style tag: {}", tag.name),
        }
    }

    /// Get the closing ANSI code for a style
    fn get_closing_code(name: &str) -> Result<String> {
        match name {
            "bold" | "b" => Ok("\x1b[22m".to_string()),      // Reset bold/dim
            "dim" | "d" => Ok("\x1b[22m".to_string()),       // Reset bold/dim
            "italic" | "em" | "i" => Ok("\x1b[23m".to_string()),    // Reset italic
            "underline" | "u" => Ok("\x1b[24m".to_string()), // Reset underline
            "fg" => Ok("\x1b[39m".to_string()),        // Reset foreground
            "bg" => Ok("\x1b[49m".to_string()),        // Reset background
            "sym" => Ok("".to_string()),               // Symbols don't need closing
            _ => bail!("Unknown style tag: {}", name),
        }
    }

    /// Resolve a color from a string (hex code or named color reference)
    /// For hex: returns (r, g, b) tuple
    /// For named color: looks up in the colors HashMap and resolves to RGB
    fn resolve_color(&self, color_str: &str) -> Result<(u8, u8, u8)> {
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

        // If not a hex color, try to look it up as a named color
        if let Some(hex_value) = self.colors.get(trimmed) {
            // Recursively resolve the hex value
            return self.resolve_color(hex_value);
        }

        // Color not found
        bail!("Unknown color name '{}'. Define it in the [colors] section of your theme or use a hex color like #ff0000", trimmed)
    }

    /// Resolve a powerline symbol from a standardized name
    /// Returns the UTF-8 encoded symbol character
    fn resolve_symbol(symbol_name: &str) -> Result<String> {
        let symbol = match symbol_name.trim() {
            // Powerline triangles (solid arrows)
            "triangle_right" | "tri_right" | "arrow_right" => "\u{e0b0}",  //
            "triangle_left" | "tri_left" | "arrow_left" => "\u{e0b2}",     //

            "inverted_triangle_left" | "inv_tri_right" | "inv_arrow_right" => "\u{e0d7}", //
            "inverted_triangle_right" | "inv_tri_left" | "inv_arrow_left" => "\u{e0d6}",  //
            
            // Powerline pills/rounded (flame-like)
            "pill_left" | "round_left" => "\u{e0b6}",       //
            "pill_right" | "round_right" => "\u{e0b4}",    //

            // Flame
            "flame_left" => "\u{e0c0}",                                      //
            "flame_right" => "\u{e0c2}",                                     //

            // Trapezoid shapes
            "trapezoid_right" => "\u{e0d2}",                                //
            "trapezoid_left" => "\u{e0d4}",                                 //


            // Powerline angles (thin arrows)
            "angle_right" | "thin_right" => "\u{e0b1}",                    //
            "angle_left" | "thin_left" => "\u{e0b3}",                      //

            // Powerline thin pills/rounded
            "pill_right_thin" | "round_right_thin" => "\u{e0b5}",          //
            "pill_left_thin" | "round_left_thin" => "\u{e0b7}",            //

            // Powerline circles (semi-circles)
            "circle_right" | "semicircle_right" => "\u{e0b8}",             //
            "circle_left" | "semicircle_left" => "\u{e0ba}",               //

            // Powerline slants/diagonal
            "slant_right" | "diagonal_right" => "\u{e0bc}",                //
            "slant_left" | "diagonal_left" => "\u{e0be}",                  //

            // Misc shapes
            "ice_cream" => "\u{f0efd}",                                    // Ice Cream (fun)
            "ice_cream_thick" => "\u{ef888}",                              // Custom glyph
            "ice_cream_outline" => "\u{f082a}",                            // Ice Cream Outline (fun)

            // Slash
            "backslash" | "backslash" => "\u{e216}",                       //

            // Additional powerline shapes
            "lower_triangle_right" => "\u{e0b8}",                          //
            "lower_triangle_left" => "\u{e0ba}",                           //
            "upper_triangle_right" => "\u{e0bc}",                          //
            "upper_triangle_left" => "\u{e0be}",                           //

            // Common nerd font icons
            "git_branch" | "branch" => "\u{e0a0}",                         //
            "lock" => "\u{e0a2}",                                          //
            "cog" | "gear" => "\u{e615}",                                  //
            "home" => "\u{f015}",                                          //
            "folder" => "\u{f07c}",                                        //
            "folder_open" => "\u{f07b}",                                   //

            // Extras
            "timer" => "\u{f0109}",                                       //
            "heart" => "\u{f004}",                                        //
            "star" => "\u{f005}",                                         //
            "check" => "\u{f00c}",                                        //
            "cross" | "x" => "\u{f00d}",                                  //
            "info" => "\u{f129}",                                         //
            "warning" => "\u{f071}",                                      //
            "question" => "\u{f128}",                                     //
            "clock" => "\u{f017}",                                       //
            "calendar" => "\u{f133}",                                     //
            "mail" | "envelope" => "\u{f0e0}",                               //
            "phone" => "\u{f095}",                                        //
            "music" => "\u{f001}",                                        //
            "camera" => "\u{f030}",                                       //
            "search" | "magnifying_glass" => "\u{f002}",                     //
            "trash" | "trash_can" => "\u{f1f8}",                             //
            "battery_full" => "\u{f240}",                                 //
            "battery_half" => "\u{f242}",                                 //
            "battery_low" => "\u{f243}",                                  //
            "wifi" => "\u{f1eb}",                                        //
            "plug" => "\u{f1e6}",                                        //
            "cloud" => "\u{f0c2}",                                       //
            "sun" => "\u{f185}",                                         //
            "moon" => "\u{f186}",                                        //
            "fire" => "\u{f06d}",                                        //
            "bug" => "\u{f188}",                                         //
            "code" => "\u{f121}",                                        //
            "terminal" => "\u{f120}",                                    //
            "keyboard" => "\u{f11c}",                                    //
            "laptop" => "\u{f109}",                                      //
            "desktop" => "\u{f108}",                                     //
            "server" => "\u{f233}",                                      //
            "computer" => "\u{f4b3}",                                    //
            "rocket" => "\u{f135}",                                      //
            "shield" => "\u{f3ed}",                                      //
            "terminal_power" => "\u{f489}",                              //
            "terminal_fire" => "\u{f489}",                               //
            "terminal_bolt" => "\u{f489}",                               //
            "terminal_flame" => "\u{f489}",                              // "terminal_lightning" => "\u{f489}",                          //
            "lightning" => "\u{f0e7}",                                   //
            "zap" => "\u{f0e7}",                                        // "flash" => "\u{f0e7}",                                     //
            "insect" => "\u{f188}",                                     //
            "leaf" => "\u{f06c}",                                       //
            "paw" => "\u{f1b0}",                                       //

            _ => bail!("Unknown symbol name: {}. See documentation for available symbols.", symbol_name),
        };

        Ok(symbol.to_string())
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
        let colors = HashMap::new();
        let preprocessor = TemplatePreprocessor::new(colors);
        let input = "(bold)Hello(/bold)";
        let result = preprocessor.preprocess(input).unwrap();
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("\x1b[22m"));
        assert!(result.contains("Hello"));
        println!("Bold result: {:?}", result);
    }

    #[test]
    fn test_bold_short_tag() {
        let input = "(b)Hello(/b)";
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

    #[test]
    fn test_powerline_symbols() {
        // Test triangle right (arrow)
        let input = "(sym triangle_right)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert_eq!(result, "\u{e0b0}");
        println!("Triangle right: {:?}", result);

        // Test pill left
        let input = "(sym pill_left)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert_eq!(result, "\u{e0b6}");
        println!("Pill left: {:?}", result);

        // Test git branch
        let input = "(sym git_branch)";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert_eq!(result, "\u{e0a0}");
        println!("Git branch: {:?}", result);
    }

    #[test]
    fn test_symbols_with_styles() {
        // Symbols should work inline with other styles
        let input = "(fg #ff0000)(sym triangle_right)(/fg) Text";
        let result = TemplatePreprocessor::preprocess(input).unwrap();
        assert!(result.contains("\x1b[38;2;255;0;0m"));
        assert!(result.contains("\u{e0b0}"));
        assert!(result.contains("\x1b[39m"));
        println!("Symbols with styles: {:?}", result);
    }

    #[test]
    fn test_symbol_aliases() {
        // Test that aliases work
        let input1 = "(sym triangle_right)";
        let input2 = "(sym tri_right)";
        let input3 = "(sym arrow_right)";

        let result1 = TemplatePreprocessor::preprocess(input1).unwrap();
        let result2 = TemplatePreprocessor::preprocess(input2).unwrap();
        let result3 = TemplatePreprocessor::preprocess(input3).unwrap();

        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
        println!("All aliases produce same result: {:?}", result1);
    }
}

#[cfg(test)]
mod test_adjacent_tags {
    use super::*;

    #[test]
    fn test_adjacent_b_and_bg() {
        let input = "(b)(bg #ff0000)test";
        let result = TemplatePreprocessor::preprocess(input);
        println!("Input: {}", input);
        match &result {
            Ok(r) => println!("Success: {:?}", r),
            Err(e) => println!("Error: {}", e),
        }
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should have bold code AND background code
        assert!(output.contains("\x1b[1m"), "Should contain bold code");
        assert!(output.contains("\x1b[48;2;255;0;0m"), "Should contain bg code");
        println!("Test passed!");
    }
}
