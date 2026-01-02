#![allow(dead_code)]

//! Default colors and symbols - single source of truth
//!
//! This module centralizes all default values for colors and symbols
//! to eliminate duplication between main.rs and config/mod.rs

use crate::color::tokyo_night;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Get default colors as a HashMap<String, Value> for template context
pub fn default_colors_json() -> HashMap<String, Value> {
    let mut colors = HashMap::new();

    colors.insert("bg".to_string(), json!(tokyo_night::BG.to_hex()));
    colors.insert("fg".to_string(), json!(tokyo_night::FG.to_hex()));
    colors.insert("fg_dark".to_string(), json!(tokyo_night::FG_DARK.to_hex()));
    colors.insert("fg_dim".to_string(), json!(tokyo_night::FG_DIM.to_hex()));
    colors.insert("black".to_string(), json!(tokyo_night::BLACK.to_hex()));
    colors.insert("red".to_string(), json!(tokyo_night::RED.to_hex()));
    colors.insert("green".to_string(), json!(tokyo_night::GREEN.to_hex()));
    colors.insert("yellow".to_string(), json!(tokyo_night::YELLOW.to_hex()));
    colors.insert("blue".to_string(), json!(tokyo_night::BLUE.to_hex()));
    colors.insert("magenta".to_string(), json!(tokyo_night::MAGENTA.to_hex()));
    colors.insert("cyan".to_string(), json!(tokyo_night::CYAN.to_hex()));
    colors.insert("white".to_string(), json!(tokyo_night::WHITE.to_hex()));
    colors.insert("orange".to_string(), json!(tokyo_night::ORANGE.to_hex()));
    colors.insert("purple".to_string(), json!(tokyo_night::PURPLE.to_hex()));
    colors.insert("teal".to_string(), json!(tokyo_night::TEAL.to_hex()));

    colors
}

/// Get default colors as a HashMap<String, String> for preprocessing
pub fn default_colors_string() -> HashMap<String, String> {
    let mut colors = HashMap::new();

    colors.insert("bg".to_string(), tokyo_night::BG.to_hex());
    colors.insert("fg".to_string(), tokyo_night::FG.to_hex());
    colors.insert("fg_dark".to_string(), tokyo_night::FG_DARK.to_hex());
    colors.insert("fg_dim".to_string(), tokyo_night::FG_DIM.to_hex());
    colors.insert("black".to_string(), tokyo_night::BLACK.to_hex());
    colors.insert("red".to_string(), tokyo_night::RED.to_hex());
    colors.insert("green".to_string(), tokyo_night::GREEN.to_hex());
    colors.insert("yellow".to_string(), tokyo_night::YELLOW.to_hex());
    colors.insert("blue".to_string(), tokyo_night::BLUE.to_hex());
    colors.insert("magenta".to_string(), tokyo_night::MAGENTA.to_hex());
    colors.insert("cyan".to_string(), tokyo_night::CYAN.to_hex());
    colors.insert("white".to_string(), tokyo_night::WHITE.to_hex());
    colors.insert("orange".to_string(), tokyo_night::ORANGE.to_hex());
    colors.insert("purple".to_string(), tokyo_night::PURPLE.to_hex());
    colors.insert("teal".to_string(), tokyo_night::TEAL.to_hex());

    colors
}

/// Get default symbols as a HashMap<String, Value> for template context
pub fn default_symbols_json() -> HashMap<String, Value> {
    let mut symbols = HashMap::new();

    symbols.insert("prompt_arrow".to_string(), json!("❯"));
    symbols.insert("segment_separator".to_string(), json!(""));
    symbols.insert("segment_separator_thin".to_string(), json!(""));
    symbols.insert("git_branch".to_string(), json!(""));
    symbols.insert("git_dirty".to_string(), json!("✗"));
    symbols.insert("git_clean".to_string(), json!("✓"));
    symbols.insert("ssh".to_string(), json!(""));
    symbols.insert("root".to_string(), json!(""));
    symbols.insert("jobs".to_string(), json!(""));
    symbols.insert("error".to_string(), json!("✖"));
    symbols.insert("success".to_string(), json!("✓"));
    symbols.insert("folder".to_string(), json!(""));
    symbols.insert("home".to_string(), json!(""));
    symbols.insert("python".to_string(), json!("🐍"));
    symbols.insert("node".to_string(), json!(""));
    symbols.insert("rust".to_string(), json!("🦀"));
    symbols.insert("docker".to_string(), json!("🐳"));
    symbols.insert("k8s".to_string(), json!("☸"));
    symbols.insert("aws".to_string(), json!("☁"));

    symbols
}

/// Get default symbols as a HashMap<String, String> for preprocessing
pub fn default_symbols_string() -> HashMap<String, String> {
    let mut symbols = HashMap::new();

    symbols.insert("prompt_arrow".to_string(), "❯".to_string());
    symbols.insert("segment_separator".to_string(), "".to_string());
    symbols.insert("segment_separator_thin".to_string(), "".to_string());
    symbols.insert("git_branch".to_string(), "".to_string());
    symbols.insert("git_dirty".to_string(), "✗".to_string());
    symbols.insert("git_clean".to_string(), "✓".to_string());
    symbols.insert("ssh".to_string(), "".to_string());
    symbols.insert("root".to_string(), "".to_string());
    symbols.insert("jobs".to_string(), "".to_string());
    symbols.insert("error".to_string(), "✖".to_string());
    symbols.insert("success".to_string(), "✓".to_string());
    symbols.insert("folder".to_string(), "".to_string());
    symbols.insert("home".to_string(), "".to_string());
    symbols.insert("python".to_string(), "🐍".to_string());
    symbols.insert("node".to_string(), "".to_string());
    symbols.insert("rust".to_string(), "🦀".to_string());
    symbols.insert("docker".to_string(), "🐳".to_string());
    symbols.insert("k8s".to_string(), "☸".to_string());
    symbols.insert("aws".to_string(), "☁".to_string());

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_colors_json() {
        let colors = default_colors_json();
        assert!(colors.contains_key("bg"));
        assert!(colors.contains_key("red"));
        assert!(colors.contains_key("teal"));
    }

    #[test]
    fn test_default_symbols_json() {
        let symbols = default_symbols_json();
        assert!(symbols.contains_key("prompt_arrow"));
        assert!(symbols.contains_key("git_branch"));
    }

    #[test]
    fn test_colors_are_valid_hex() {
        let colors = default_colors_string();
        for (name, hex) in colors {
            assert!(
                hex.starts_with('#') && hex.len() == 7,
                "Color {} has invalid hex: {}",
                name,
                hex
            );
        }
    }
}
