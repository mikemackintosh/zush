// Rust module - detects Rust/Cargo projects

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct RustModule {
    symbol: String,
    show_version: bool,
}

impl RustModule {
    pub fn new() -> Self {
        Self {
            symbol: "🦀".to_string(),
            show_version: false,
        }
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn with_version(mut self, show_version: bool) -> Self {
        self.show_version = show_version;
        self
    }

    /// Check if we're in a Rust project
    fn is_rust_project(&self, context: &ModuleContext) -> bool {
        // Check for Rust project files
        context.fs.has_file("Cargo.toml") ||
        context.fs.has_file("Cargo.lock") ||
        context.fs.has_file("rust-toolchain") ||
        context.fs.has_file("rust-toolchain.toml")
    }

    /// Get Rust version (if requested)
    fn get_rust_version(&self) -> Option<String> {
        if !self.show_version {
            return None;
        }

        // Try to get rustc version
        std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let version = String::from_utf8_lossy(&output.stdout);
                // Extract version (e.g., "rustc 1.73.0 (abc123 2023-10-01)" -> "1.73.0")
                version
                    .split_whitespace()
                    .nth(1)
                    .map(|v| v.to_string())
            })
    }

    /// Get package name from Cargo.toml (if available)
    fn get_package_name(&self, context: &ModuleContext) -> Option<String> {
        let cargo_toml_path = context.pwd.join("Cargo.toml");

        if let Ok(contents) = context.fs.read_to_string(&cargo_toml_path) {
            // Simple TOML parsing for name field in [package] section
            let mut in_package_section = false;

            for line in contents.lines() {
                let trimmed = line.trim();

                // Check if we entered [package] section
                if trimmed == "[package]" {
                    in_package_section = true;
                    continue;
                }

                // Check if we left [package] section
                if in_package_section && trimmed.starts_with('[') {
                    break;
                }

                // Look for name field in [package] section
                if in_package_section && trimmed.starts_with("name") {
                    if let Some(eq_pos) = trimmed.find('=') {
                        let name = trimmed[eq_pos + 1..]
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        return Some(name);
                    }
                }
            }
        }

        None
    }
}

impl Default for RustModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for RustModule {
    fn id(&self) -> &str {
        "rust"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        self.is_rust_project(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Add package name if available
        if let Some(package_name) = self.get_package_name(context) {
            parts.push(package_name);
        } else {
            parts.push("rust".to_string());
        }

        // Add version if requested
        if let Some(version) = self.get_rust_version() {
            parts.push(format!("v{}", version));
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new(
            "Rust",
            "Rust/Cargo project detection"
        )
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use crate::modules::SandboxedFs;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_rust_module_detection() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, r#"[package]
name = "test-crate"
version = "0.1.0"
"#).unwrap();

        let module = RustModule::new();
        let context = ModuleContext {
            pwd: temp_dir.path().to_path_buf(),
            home: PathBuf::from("/home/user"),
            env: HashMap::new(),
            fs: SandboxedFs::new(vec![temp_dir.path().to_path_buf()]),
        };

        assert!(module.should_display(&context));
    }

    #[test]
    fn test_rust_module_package_name() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, r#"[package]
name = "my-awesome-crate"
version = "1.0.0"
"#).unwrap();

        let module = RustModule::new();
        let context = ModuleContext {
            pwd: temp_dir.path().to_path_buf(),
            home: PathBuf::from("/home/user"),
            env: HashMap::new(),
            fs: SandboxedFs::new(vec![temp_dir.path().to_path_buf()]),
        };

        let package_name = module.get_package_name(&context);
        assert_eq!(package_name, Some("my-awesome-crate".to_string()));
    }
}
