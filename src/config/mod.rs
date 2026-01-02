#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorConfig,

    #[serde(default)]
    pub symbols: SymbolConfig,

    #[serde(default)]
    pub segments: SegmentConfig,

    #[serde(default)]
    pub templates: HashMap<String, TemplateDefinition>,

    #[serde(default)]
    pub behavior: BehaviorConfig,
}

/// Color configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub background: String,
    pub foreground: String,
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub orange: String,
    pub purple: String,
    pub teal: String,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            background: "#1a1b26".to_string(),
            foreground: "#c0caf5".to_string(),
            black: "#15161e".to_string(),
            red: "#f7768e".to_string(),
            green: "#9ece6a".to_string(),
            yellow: "#e0af68".to_string(),
            blue: "#7aa2f7".to_string(),
            magenta: "#bb9af7".to_string(),
            cyan: "#7dcfff".to_string(),
            white: "#c0caf5".to_string(),
            orange: "#ff9e64".to_string(),
            purple: "#9d7cd8".to_string(),
            teal: "#1abc9c".to_string(),
        }
    }
}

/// Symbol configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolConfig {
    pub prompt_arrow: String,
    pub segment_separator: String,
    pub segment_separator_thin: String,
    pub git_branch: String,
    pub git_dirty: String,
    pub git_clean: String,
    pub ssh: String,
    pub root: String,
    pub jobs: String,
    pub error: String,
    pub success: String,
    pub folder: String,
    pub home: String,
    pub python: String,
    pub node: String,
    pub rust: String,
    pub docker: String,
    pub k8s: String,
    pub aws: String,
}

impl Default for SymbolConfig {
    fn default() -> Self {
        Self {
            prompt_arrow: "❯".to_string(),
            segment_separator: "".to_string(),
            segment_separator_thin: "".to_string(),
            git_branch: "".to_string(),
            git_dirty: "✗".to_string(),
            git_clean: "✓".to_string(),
            ssh: "".to_string(),
            root: "".to_string(),
            jobs: "".to_string(),
            error: "✖".to_string(),
            success: "✓".to_string(),
            folder: "".to_string(),
            home: "".to_string(),
            python: "🐍".to_string(),
            node: "".to_string(),
            rust: "🦀".to_string(),
            docker: "🐳".to_string(),
            k8s: "☸".to_string(),
            aws: "☁".to_string(),
        }
    }
}

/// Segment configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentConfig {
    pub left: Vec<String>,
    pub center: Vec<String>,
    pub right: Vec<String>,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            left: vec![
                "status".to_string(),
                "user".to_string(),
                "directory".to_string(),
            ],
            center: vec!["git".to_string()],
            right: vec!["execution_time".to_string(), "time".to_string()],
        }
    }
}

/// Template definition
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDefinition {
    pub template: String,
    #[serde(default)]
    pub description: String,
}

/// Behavior configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub transient_prompt: bool,
    pub auto_newline: bool,
    pub show_execution_time_threshold: f64,
    pub truncate_dir_depth: Option<usize>,
    pub enable_icons: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            transient_prompt: true,
            auto_newline: true,
            show_execution_time_threshold: 2.0,
            truncate_dir_depth: Some(3),
            enable_icons: true,
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Self =
            toml::from_str(&content).with_context(|| "Failed to parse TOML configuration")?;

        Ok(config)
    }

    /// Load configuration from default locations
    pub fn load_default() -> Result<Self> {
        let config_paths = vec![
            // User config
            dirs::config_dir().map(|d| d.join("zush").join("config.toml")),
            // Home directory
            dirs::home_dir().map(|d| d.join(".zushrc.toml")),
            // Current directory
            Some(PathBuf::from(".zushrc.toml")),
        ];

        for path_opt in config_paths {
            if let Some(path) = path_opt {
                if path.exists() {
                    return Self::load(path);
                }
            }
        }

        // Return default config if no file found
        Ok(Self::default())
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge(mut self, other: Config) -> Self {
        // Merge colors
        self.colors = other.colors;

        // Merge symbols
        self.symbols = other.symbols;

        // Merge segments
        self.segments = other.segments;

        // Merge templates
        for (key, value) in other.templates {
            self.templates.insert(key, value);
        }

        // Merge behavior
        self.behavior = other.behavior;

        self
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut templates = HashMap::new();

        templates.insert(
            "main".to_string(),
            TemplateDefinition {
                template: r#"{{color colors.green symbols.success}} {{bold user}} {{color colors.white "in"}} {{color colors.magenta pwd}}
{{color colors.blue symbols.prompt_arrow}} "#.to_string(),
                description: "Main prompt template".to_string(),
            },
        );

        templates.insert(
            "transient".to_string(),
            TemplateDefinition {
                template: r#"{{dim time}}
{{color colors.blue symbols.prompt_arrow}} "#
                    .to_string(),
                description: "Transient prompt template".to_string(),
            },
        );

        templates.insert(
            "right".to_string(),
            TemplateDefinition {
                template: r#"{{#if execution_time}}{{color colors.yellow execution_time}}s {{/if}}{{dim time}}"#.to_string(),
                description: "Right-side prompt template".to_string(),
            },
        );

        Self {
            colors: ColorConfig::default(),
            symbols: SymbolConfig::default(),
            segments: SegmentConfig::default(),
            templates,
            behavior: BehaviorConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_color_config_defaults() {
        let colors = ColorConfig::default();
        assert_eq!(colors.background, "#1a1b26");
        assert_eq!(colors.foreground, "#c0caf5");
        assert_eq!(colors.red, "#f7768e");
        assert_eq!(colors.green, "#9ece6a");
        assert_eq!(colors.teal, "#1abc9c");
    }

    #[test]
    fn test_symbol_config_defaults() {
        let symbols = SymbolConfig::default();
        assert_eq!(symbols.prompt_arrow, "❯");
        assert_eq!(symbols.git_branch, "");
        assert_eq!(symbols.git_dirty, "✗");
        assert_eq!(symbols.git_clean, "✓");
    }

    #[test]
    fn test_segment_config_defaults() {
        let segments = SegmentConfig::default();
        assert!(segments.left.contains(&"status".to_string()));
        assert!(segments.left.contains(&"directory".to_string()));
        assert!(segments.center.contains(&"git".to_string()));
        assert!(segments.right.contains(&"time".to_string()));
    }

    #[test]
    fn test_behavior_config_defaults() {
        let behavior = BehaviorConfig::default();
        assert!(behavior.transient_prompt);
        assert!(behavior.auto_newline);
        assert_eq!(behavior.show_execution_time_threshold, 2.0);
        assert_eq!(behavior.truncate_dir_depth, Some(3));
        assert!(behavior.enable_icons);
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert!(config.templates.contains_key("main"));
        assert!(config.templates.contains_key("transient"));
        assert!(config.templates.contains_key("right"));
    }

    #[test]
    fn test_config_load_from_toml() {
        let toml_content = r##"
[colors]
background = "#000000"
foreground = "#ffffff"
black = "#111111"
red = "#ff0000"
green = "#00ff00"
yellow = "#ffff00"
blue = "#0000ff"
magenta = "#ff00ff"
cyan = "#00ffff"
white = "#ffffff"
orange = "#ffa500"
purple = "#800080"
teal = "#008080"

[symbols]
prompt_arrow = ">"
segment_separator = "|"
segment_separator_thin = "|"
git_branch = "B"
git_dirty = "D"
git_clean = "C"
ssh = "S"
root = "R"
jobs = "J"
error = "E"
success = "S"
folder = "F"
home = "H"
python = "P"
node = "N"
rust = "R"
docker = "D"
k8s = "K"
aws = "A"

[behavior]
transient_prompt = false
auto_newline = false
show_execution_time_threshold = 5.0
truncate_dir_depth = 2
enable_icons = false
"##;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load(temp_file.path()).unwrap();

        assert_eq!(config.colors.background, "#000000");
        assert_eq!(config.colors.red, "#ff0000");
        assert_eq!(config.symbols.prompt_arrow, ">");
        assert!(!config.behavior.transient_prompt);
        assert_eq!(config.behavior.show_execution_time_threshold, 5.0);
        assert_eq!(config.behavior.truncate_dir_depth, Some(2));
    }

    #[test]
    fn test_config_save_and_load_roundtrip() {
        let original = Config::default();

        let temp_file = NamedTempFile::new().unwrap();
        original.save(temp_file.path()).unwrap();

        let loaded = Config::load(temp_file.path()).unwrap();

        assert_eq!(original.colors.background, loaded.colors.background);
        assert_eq!(original.symbols.prompt_arrow, loaded.symbols.prompt_arrow);
        assert_eq!(
            original.behavior.transient_prompt,
            loaded.behavior.transient_prompt
        );
    }

    #[test]
    fn test_config_merge() {
        let base = Config::default();

        let mut override_config = Config::default();
        override_config.colors.background = "#ff0000".to_string();
        override_config.symbols.prompt_arrow = ">>".to_string();
        override_config.behavior.transient_prompt = false;

        let merged = base.merge(override_config);

        assert_eq!(merged.colors.background, "#ff0000");
        assert_eq!(merged.symbols.prompt_arrow, ">>");
        assert!(!merged.behavior.transient_prompt);
    }

    #[test]
    fn test_config_merge_templates() {
        let base = Config::default();

        let mut override_config = Config::default();
        override_config.templates.insert(
            "custom".to_string(),
            TemplateDefinition {
                template: "custom template".to_string(),
                description: "A custom template".to_string(),
            },
        );

        let merged = base.merge(override_config);

        assert!(merged.templates.contains_key("main")); // from override (same as base)
        assert!(merged.templates.contains_key("custom")); // new template added
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let result = Config::load("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"this is not valid toml {{{").unwrap();

        let result = Config::load(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        // Only specify some fields, rest should use defaults
        let toml_content = r##"
[colors]
background = "#000000"
foreground = "#ffffff"
black = "#111111"
red = "#ff0000"
green = "#00ff00"
yellow = "#ffff00"
blue = "#0000ff"
magenta = "#ff00ff"
cyan = "#00ffff"
white = "#ffffff"
orange = "#ffa500"
purple = "#800080"
teal = "#008080"
"##;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load(temp_file.path()).unwrap();

        // Specified value
        assert_eq!(config.colors.background, "#000000");
        // Default values for unspecified sections
        assert_eq!(config.symbols.prompt_arrow, "❯");
        assert!(config.behavior.transient_prompt);
    }
}
