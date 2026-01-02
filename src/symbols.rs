//! Built-in symbol definitions for powerline and nerd font glyphs
//!
//! This module provides a data-driven symbol registry instead of a large match statement,
//! following the Open/Closed Principle - new symbols can be added without modifying code.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Get the global built-in symbols registry
pub fn builtin_symbols() -> &'static HashMap<&'static str, &'static str> {
    static SYMBOLS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    SYMBOLS.get_or_init(|| {
        let mut m = HashMap::with_capacity(100);

        // Powerline triangles (solid arrows)
        m.insert("triangle_right", "\u{e0b0}");
        m.insert("tri_right", "\u{e0b0}");
        m.insert("arrow_right", "\u{e0b0}");
        m.insert("triangle_left", "\u{e0b2}");
        m.insert("tri_left", "\u{e0b2}");
        m.insert("arrow_left", "\u{e0b2}");

        // Inverted triangles
        m.insert("inverted_triangle_left", "\u{e0d7}");
        m.insert("inv_tri_right", "\u{e0d7}");
        m.insert("inv_arrow_right", "\u{e0d7}");
        m.insert("inverted_triangle_right", "\u{e0d6}");
        m.insert("inv_tri_left", "\u{e0d6}");
        m.insert("inv_arrow_left", "\u{e0d6}");

        // Powerline pills/rounded
        m.insert("pill_left", "\u{e0b6}");
        m.insert("round_left", "\u{e0b6}");
        m.insert("pill_right", "\u{e0b4}");
        m.insert("round_right", "\u{e0b4}");

        // Flame
        m.insert("flame_left", "\u{e0c0}");
        m.insert("flame_right", "\u{e0c2}");

        // Trapezoid shapes
        m.insert("trapezoid_right", "\u{e0d2}");
        m.insert("trapezoid_left", "\u{e0d4}");

        // Powerline angles (thin arrows)
        m.insert("angle_right", "\u{e0b1}");
        m.insert("thin_right", "\u{e0b1}");
        m.insert("angle_left", "\u{e0b3}");
        m.insert("thin_left", "\u{e0b3}");

        // Powerline thin pills/rounded
        m.insert("pill_right_thin", "\u{e0b5}");
        m.insert("round_right_thin", "\u{e0b5}");
        m.insert("pill_left_thin", "\u{e0b7}");
        m.insert("round_left_thin", "\u{e0b7}");

        // Powerline circles (semi-circles)
        m.insert("circle_right", "\u{e0b8}");
        m.insert("semicircle_right", "\u{e0b8}");
        m.insert("circle_left", "\u{e0ba}");
        m.insert("semicircle_left", "\u{e0ba}");

        // Powerline slants/diagonal
        m.insert("slant_right", "\u{e0bc}");
        m.insert("diagonal_right", "\u{e0bc}");
        m.insert("slant_left", "\u{e0be}");
        m.insert("diagonal_left", "\u{e0be}");

        // Misc shapes
        m.insert("ice_cream", "\u{f0efd}");
        m.insert("ice_cream_thick", "\u{ef888}");
        m.insert("ice_cream_outline", "\u{f082a}");

        // Slash
        m.insert("backslash", "\u{e216}");

        // Additional powerline shapes
        m.insert("lower_triangle_right", "\u{e0b8}");
        m.insert("lower_triangle_left", "\u{e0ba}");
        m.insert("upper_triangle_right", "\u{e0bc}");
        m.insert("upper_triangle_left", "\u{e0be}");

        // Common nerd font icons - Git
        m.insert("git_branch", "\u{e0a0}");
        m.insert("branch", "\u{e0a0}");
        m.insert("lock", "\u{e0a2}");

        // Common nerd font icons - UI
        m.insert("cog", "\u{e615}");
        m.insert("gear", "\u{e615}");
        m.insert("home", "\u{f015}");
        m.insert("folder", "\u{f07c}");
        m.insert("folder_open", "\u{f07b}");

        // Time & Status
        m.insert("timer", "\u{f0109}");
        m.insert("clock", "\u{f017}");
        m.insert("calendar", "\u{f133}");
        m.insert("check", "\u{f00c}");
        m.insert("cross", "\u{f00d}");
        m.insert("x", "\u{f00d}");
        m.insert("info", "\u{f129}");
        m.insert("warning", "\u{f071}");
        m.insert("question", "\u{f128}");

        // Communication
        m.insert("mail", "\u{f0e0}");
        m.insert("envelope", "\u{f0e0}");
        m.insert("phone", "\u{f095}");

        // Media
        m.insert("music", "\u{f001}");
        m.insert("camera", "\u{f030}");

        // Actions
        m.insert("search", "\u{f002}");
        m.insert("magnifying_glass", "\u{f002}");
        m.insert("trash", "\u{f1f8}");
        m.insert("trash_can", "\u{f1f8}");

        // Power & Connectivity
        m.insert("battery_full", "\u{f240}");
        m.insert("battery_half", "\u{f242}");
        m.insert("battery_low", "\u{f243}");
        m.insert("wifi", "\u{f1eb}");
        m.insert("plug", "\u{f1e6}");

        // Weather & Nature
        m.insert("cloud", "\u{f0c2}");
        m.insert("sun", "\u{f185}");
        m.insert("moon", "\u{f186}");
        m.insert("fire", "\u{f06d}");
        m.insert("leaf", "\u{f06c}");
        m.insert("paw", "\u{f1b0}");

        // Development
        m.insert("bug", "\u{f188}");
        m.insert("insect", "\u{f188}");
        m.insert("code", "\u{f121}");
        m.insert("terminal", "\u{f120}");
        m.insert("keyboard", "\u{f11c}");

        // Hardware
        m.insert("laptop", "\u{f109}");
        m.insert("desktop", "\u{f108}");
        m.insert("server", "\u{f233}");
        m.insert("computer", "\u{f4b3}");

        // Misc
        m.insert("heart", "\u{f004}");
        m.insert("star", "\u{f005}");
        m.insert("rocket", "\u{f135}");
        m.insert("shield", "\u{f3ed}");
        m.insert("lightning", "\u{f0e7}");
        m.insert("zap", "\u{f0e7}");

        // Terminal variants (all map to same glyph)
        m.insert("terminal_power", "\u{f489}");
        m.insert("terminal_fire", "\u{f489}");
        m.insert("terminal_bolt", "\u{f489}");
        m.insert("terminal_flame", "\u{f489}");

        m
    })
}

/// Resolve a symbol name to its Unicode character
/// Returns None if the symbol is not found
pub fn resolve_builtin(name: &str) -> Option<&'static str> {
    builtin_symbols().get(name.trim()).copied()
}

/// Get all available symbol names (for documentation/help)
pub fn available_symbols() -> Vec<&'static str> {
    let mut names: Vec<_> = builtin_symbols().keys().copied().collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_triangle() {
        assert_eq!(resolve_builtin("triangle_right"), Some("\u{e0b0}"));
        assert_eq!(resolve_builtin("tri_right"), Some("\u{e0b0}"));
        assert_eq!(resolve_builtin("arrow_right"), Some("\u{e0b0}"));
    }

    #[test]
    fn test_resolve_with_whitespace() {
        assert_eq!(resolve_builtin("  triangle_right  "), Some("\u{e0b0}"));
    }

    #[test]
    fn test_unknown_symbol() {
        assert_eq!(resolve_builtin("nonexistent_symbol"), None);
    }

    #[test]
    fn test_git_branch() {
        assert_eq!(resolve_builtin("git_branch"), Some("\u{e0a0}"));
        assert_eq!(resolve_builtin("branch"), Some("\u{e0a0}"));
    }

    #[test]
    fn test_available_symbols_not_empty() {
        let symbols = available_symbols();
        assert!(!symbols.is_empty());
        assert!(symbols.len() > 50); // We have ~100 symbols
    }

    #[test]
    fn test_aliases_resolve_same() {
        // Verify aliases point to the same symbol
        assert_eq!(resolve_builtin("mail"), resolve_builtin("envelope"));
        assert_eq!(resolve_builtin("cross"), resolve_builtin("x"));
        assert_eq!(resolve_builtin("cog"), resolve_builtin("gear"));
    }
}
