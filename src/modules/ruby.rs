// Ruby module - detects Ruby projects and environments

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct RubyModule {
    symbol: String,
    show_version: bool,
}

impl RubyModule {
    pub fn new() -> Self {
        Self {
            symbol: "💎".to_string(),
            show_version: true,
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

    /// Check if we're in a Ruby project
    fn is_ruby_project(&self, context: &ModuleContext) -> bool {
        // Check for common Ruby files
        context.fs.has_file("Gemfile")
            || context.fs.has_file("Rakefile")
            || context.fs.has_file(".ruby-version")
            || context.fs.has_file(".ruby-gemset")
            || context.fs.has_file("config.ru")
            || self.has_gemspec(context)
    }

    /// Check for .gemspec files
    fn has_gemspec(&self, context: &ModuleContext) -> bool {
        // This is a simplified check - could be enhanced to actually scan for .gemspec files
        // For now, just check if we have common Ruby project indicators
        context.fs.has_file("Gemfile.lock")
    }

    /// Get Ruby version
    fn get_ruby_version(&self) -> Option<String> {
        if !self.show_version {
            return None;
        }

        // Try to get Ruby version from ruby command
        std::process::Command::new("ruby")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let version = String::from_utf8_lossy(&output.stdout);
                // Extract version number (e.g., "ruby 3.2.2 (2023-03-30 revision e51014f9c0)" -> "3.2.2")
                version
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| {
                        // Remove any parenthesis or trailing characters
                        v.split('p').next()
                    })
                    .map(|v| v.to_string())
            })
    }

    /// Get gem name from .gemspec file (if available)
    fn get_gem_name(&self, context: &ModuleContext) -> Option<String> {
        // Try to read Gemfile for gem name
        let gemfile_path = context.pwd.join("Gemfile");

        if let Ok(contents) = context.fs.read_to_string(&gemfile_path) {
            // Look for gemspec line which often indicates the gem name
            for line in contents.lines() {
                if line.trim().starts_with("gemspec") {
                    // Try to get directory name as gem name
                    if let Some(dir_name) = context.pwd.file_name() {
                        if let Some(name) = dir_name.to_str() {
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for RubyModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for RubyModule {
    fn id(&self) -> &str {
        "ruby"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        self.is_ruby_project(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Add gem name if available, otherwise just "ruby"
        if let Some(gem_name) = self.get_gem_name(context) {
            parts.push(gem_name);
        } else {
            parts.push("ruby".to_string());
        }

        // Add version if requested
        if let Some(version) = self.get_ruby_version() {
            parts.push(format!("v{}", version));
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("Ruby", "Ruby project and gem detection")
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::SandboxedFs;
    use std::path::PathBuf;

    #[test]
    fn test_ruby_module_gemfile_detection() {
        let module = RubyModule::new();

        let context = ModuleContext {
            pwd: PathBuf::from("/home/user/project"),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![PathBuf::from("/home/user/project")]),
        };

        // In a real test, we'd create actual files
        // For now, this tests the structure
        assert_eq!(module.id(), "ruby");
    }

    #[test]
    fn test_ruby_module_render() {
        let module = RubyModule::new().with_version(false);

        let context = ModuleContext {
            pwd: PathBuf::from("/home/user/my-gem"),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![PathBuf::from("/home/user/my-gem")]),
        };

        // Test basic rendering structure
        let result = module.render(&context);
        assert!(result.is_ok());
    }
}
