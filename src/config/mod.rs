use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[allow(dead_code)]
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
