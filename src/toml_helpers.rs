#![allow(dead_code)]

//! TOML parsing helper functions to eliminate code duplication
//!
//! This module provides utility functions for extracting colors, symbols,
//! and segments from TOML configuration strings.

use crate::template::SegmentDef;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Parse a TOML string and cache the parsed result
pub struct TomlParser {
    parsed: Option<toml::Value>,
}

impl TomlParser {
    /// Create a new parser from an optional TOML string
    pub fn new(toml_str: Option<&str>) -> Self {
        let parsed = toml_str.and_then(|s| toml::from_str::<toml::Value>(s).ok());
        Self { parsed }
    }

    /// Get a reference to the parsed TOML value
    pub fn get(&self) -> Option<&toml::Value> {
        self.parsed.as_ref()
    }

    /// Extract a string table section (e.g., colors, symbols) as a HashMap
    pub fn extract_string_section(&self, section: &str) -> HashMap<String, String> {
        let mut result = HashMap::new();

        if let Some(parsed) = &self.parsed {
            if let Some(table) = parsed.get(section).and_then(|v| v.as_table()) {
                for (key, value) in table {
                    if let Some(str_value) = value.as_str() {
                        result.insert(key.clone(), str_value.to_string());
                    }
                }
            }
        }

        result
    }

    /// Extract colors section as a HashMap<String, String>
    pub fn extract_colors(&self) -> HashMap<String, String> {
        self.extract_string_section("colors")
    }

    /// Extract symbols section as a HashMap<String, String>, with Unicode escape parsing
    pub fn extract_symbols<F>(&self, unicode_parser: F) -> HashMap<String, String>
    where
        F: Fn(&str) -> String,
    {
        let mut result = HashMap::new();

        if let Some(parsed) = &self.parsed {
            if let Some(table) = parsed.get("symbols").and_then(|v| v.as_table()) {
                for (key, value) in table {
                    if let Some(str_value) = value.as_str() {
                        result.insert(key.clone(), unicode_parser(str_value));
                    }
                }
            }
        }

        result
    }

    /// Extract colors as JSON values for template context
    pub fn extract_colors_as_json(&self) -> HashMap<String, Value> {
        self.extract_colors()
            .into_iter()
            .map(|(k, v)| (k, json!(v)))
            .collect()
    }

    /// Extract symbols as JSON values for template context, with Unicode escape parsing
    pub fn extract_symbols_as_json<F>(&self, unicode_parser: F) -> HashMap<String, Value>
    where
        F: Fn(&str) -> String,
    {
        self.extract_symbols(unicode_parser)
            .into_iter()
            .map(|(k, v)| (k, json!(v)))
            .collect()
    }

    /// Extract segment definitions from the [segments] section
    pub fn extract_segments(&self) -> HashMap<String, SegmentDef> {
        let mut result = HashMap::new();

        if let Some(parsed) = &self.parsed {
            if let Some(table) = parsed.get("segments").and_then(|v| v.as_table()) {
                for (name, data) in table {
                    if let Some(segment) = Self::parse_segment(name, data) {
                        result.insert(name.clone(), segment);
                    }
                }
            }
        }

        result
    }

    /// Parse a single segment definition from TOML
    fn parse_segment(name: &str, data: &toml::Value) -> Option<SegmentDef> {
        let props = data.as_table()?;
        let content = props.get("content").and_then(|v| v.as_str())?;

        // Normalize multiline content
        let normalized_content = normalize_multiline_content(content);

        let mut segment = SegmentDef::new(name.to_string(), normalized_content);

        // Add optional properties
        if let Some(bg) = props.get("bg").and_then(|v| v.as_str()) {
            segment = segment.with_bg(bg.to_string());
        }
        if let Some(fg) = props.get("fg").and_then(|v| v.as_str()) {
            segment = segment.with_fg(fg.to_string());
        }
        if let Some(sep) = props.get("sep").and_then(|v| v.as_str()) {
            segment = segment.with_sep(sep.to_string());
        }
        if let Some(left_cap) = props.get("left_cap").and_then(|v| v.as_str()) {
            segment = segment.with_left_cap(left_cap.to_string());
        }

        Some(segment)
    }

    /// Apply overrides from a config file to existing maps
    /// Looks for [overrides] section with keys like "colors.bg" or "symbols.arrow"
    pub fn apply_overrides<F>(
        &self,
        colors: &mut HashMap<String, Value>,
        symbols: &mut HashMap<String, Value>,
        unicode_parser: F,
    ) where
        F: Fn(&str) -> String,
    {
        if let Some(parsed) = &self.parsed {
            if let Some(overrides) = parsed.get("overrides").and_then(|v| v.as_table()) {
                for (key, value) in overrides {
                    if let Some(str_value) = value.as_str() {
                        if let Some(color_key) = key.strip_prefix("colors.") {
                            colors.insert(color_key.to_string(), json!(str_value));
                        } else if let Some(symbol_key) = key.strip_prefix("symbols.") {
                            symbols.insert(symbol_key.to_string(), json!(unicode_parser(str_value)));
                        }
                    }
                }
            }
        }
    }
}

/// Normalize multiline TOML content into a single line
/// Preserves single-line content as-is to keep intentional trailing spaces
pub fn normalize_multiline_content(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() == 1 {
        // Single line - keep as-is (preserves trailing spaces for background fill)
        content.to_string()
    } else {
        // Multiple lines - trim and join
        lines
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TOML: &str = r##"
[colors]
bg = "#1a1b26"
fg = "#c0caf5"
red = "#f7768e"

[symbols]
arrow = ">"
branch = ""

[segments.status]
bg = "green"
fg = "black"
content = "ok"
sep = "sharp"

[overrides]
"colors.bg" = "#000000"
"symbols.arrow" = ">"
"##;

    #[test]
    fn test_extract_colors() {
        let parser = TomlParser::new(Some(TEST_TOML));
        let colors = parser.extract_colors();

        assert_eq!(colors.get("bg"), Some(&"#1a1b26".to_string()));
        assert_eq!(colors.get("fg"), Some(&"#c0caf5".to_string()));
        assert_eq!(colors.get("red"), Some(&"#f7768e".to_string()));
    }

    #[test]
    fn test_extract_symbols() {
        let parser = TomlParser::new(Some(TEST_TOML));
        let symbols = parser.extract_symbols(|s| s.to_string());

        assert_eq!(symbols.get("arrow"), Some(&">".to_string()));
        assert_eq!(symbols.get("branch"), Some(&"".to_string()));
    }

    #[test]
    fn test_extract_segments() {
        let parser = TomlParser::new(Some(TEST_TOML));
        let segments = parser.extract_segments();

        let status = segments.get("status").unwrap();
        assert_eq!(status.bg, Some("green".to_string()));
        assert_eq!(status.fg, Some("black".to_string()));
        assert_eq!(status.content, "ok");
    }

    #[test]
    fn test_apply_overrides() {
        let parser = TomlParser::new(Some(TEST_TOML));

        let mut colors = parser.extract_colors_as_json();
        let mut symbols = parser.extract_symbols_as_json(|s| s.to_string());

        parser.apply_overrides(&mut colors, &mut symbols, |s| s.to_string());

        assert_eq!(colors.get("bg"), Some(&json!("#000000")));
        assert_eq!(symbols.get("arrow"), Some(&json!(">")));
    }

    #[test]
    fn test_normalize_multiline() {
        // Single line preserved
        assert_eq!(normalize_multiline_content("hello "), "hello ");

        // Multiline trimmed and joined
        let multiline = "  first  \n  second  \n  third  ";
        assert_eq!(normalize_multiline_content(multiline), "firstsecondthird");
    }

    #[test]
    fn test_empty_toml() {
        let parser = TomlParser::new(None);

        assert!(parser.extract_colors().is_empty());
        assert!(parser.extract_symbols(|s| s.to_string()).is_empty());
        assert!(parser.extract_segments().is_empty());
    }
}
