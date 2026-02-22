// Go module - detects Go projects and environments

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;
use std::path::Path;

pub struct GoModule {
    symbol: String,
    show_version: bool,
}

impl GoModule {
    pub fn new() -> Self {
        Self {
            symbol: "\u{e724}".to_string(),
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

    /// Check if we're in a Go project
    fn is_go_project(&self, context: &ModuleContext) -> bool {
        // Check for common Go files
        let has_go_files = context.fs.has_file("go.mod")
            || context.fs.has_file("go.sum")
            || context.fs.has_file("Gopkg.toml")
            || context.fs.has_file("Gopkg.lock")
            || context.fs.has_file(".go-version")
            || context.fs.has_file("glide.yaml");

        // Also check if we're inside GOPATH
        let in_gopath = self.is_in_gopath(context);

        has_go_files || in_gopath
    }

    /// Check if current directory is within GOPATH
    fn is_in_gopath(&self, context: &ModuleContext) -> bool {
        if let Some(ref gopath) = context.get_env("GOPATH") {
            // GOPATH can contain multiple paths separated by ':'
            for path in gopath.split(':') {
                let gopath_src = Path::new(path).join("src");
                if context.pwd.starts_with(&gopath_src) {
                    return true;
                }
            }
        }
        false
    }

    /// Get Go version
    fn get_go_version(&self) -> Option<String> {
        if !self.show_version {
            return None;
        }

        // Try to get Go version from go command
        std::process::Command::new("go")
            .arg("version")
            .output()
            .ok()
            .and_then(|output| {
                let version = String::from_utf8_lossy(&output.stdout);
                // Extract version number (e.g., "go version go1.21.5 darwin/arm64" -> "1.21.5")
                version
                    .split_whitespace()
                    .nth(2)
                    .and_then(|v| {
                        // Remove "go" prefix
                        v.strip_prefix("go")
                    })
                    .map(|v| v.to_string())
            })
    }

    /// Get module name from go.mod file (if available)
    fn get_module_name(&self, context: &ModuleContext) -> Option<String> {
        // Try to read go.mod for module name
        let gomod_path = context.pwd.join("go.mod");

        if let Ok(contents) = context.fs.read_to_string(&gomod_path) {
            // Look for module line
            for line in contents.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("module ") {
                    // Extract module name after "module "
                    if let Some(module_name) = trimmed.strip_prefix("module ") {
                        // Get just the last part of the module path
                        if let Some(last_part) = module_name.split('/').last() {
                            return Some(last_part.to_string());
                        }
                        return Some(module_name.to_string());
                    }
                }
            }
        }

        None
    }
}

impl Default for GoModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for GoModule {
    fn id(&self) -> &str {
        "go"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        self.is_go_project(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Add module name if available, otherwise just "go"
        if let Some(module_name) = self.get_module_name(context) {
            parts.push(module_name);
        } else {
            parts.push("go".to_string());
        }

        // Add version if requested
        if let Some(version) = self.get_go_version() {
            parts.push(format!("v{}", version));
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("Go", "Go project and module detection")
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
    fn test_go_module_basic() {
        let module = GoModule::new();

        let context = ModuleContext {
            pwd: PathBuf::from("/home/user/project"),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![PathBuf::from("/home/user/project")]),
        };

        // Test basic module properties
        assert_eq!(module.id(), "go");
        assert!(module.enabled_by_default());

        // In a real test with actual files, we'd check for go.mod
        let _ = module.should_display(&context);
    }

    #[test]
    fn test_go_module_render() {
        let module = GoModule::new().with_version(false);

        let context = ModuleContext {
            pwd: PathBuf::from("/home/user/my-app"),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![PathBuf::from("/home/user/my-app")]),
        };

        // Test basic rendering structure
        let result = module.render(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_go_module_gopath_detection() {
        let module = GoModule::new();

        // Note: is_in_gopath now uses std::env::var directly
        // This test will only work if GOPATH is actually set in the environment
        let context = ModuleContext {
            pwd: PathBuf::from("/home/user/go/src/myproject"),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![PathBuf::from("/home/user/go/src/myproject")]),
        };

        // Just verify it doesn't panic
        let _ = module.is_in_gopath(&context);
    }
}
