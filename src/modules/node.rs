// Node.js module - detects Node.js projects

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct NodeModule {
    symbol: String,
    show_version: bool,
}

impl NodeModule {
    pub fn new() -> Self {
        Self {
            symbol: "⬢".to_string(),
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

    /// Check if we're in a Node.js project
    fn is_node_project(&self, context: &ModuleContext) -> bool {
        // Check for common Node.js files
        context.fs.has_file("package.json")
            || context.fs.has_file(".nvmrc")
            || context.fs.has_file(".node-version")
            || context.fs.has_dir("node_modules")
    }

    /// Get Node.js version (if requested)
    fn get_node_version(&self) -> Option<String> {
        if !self.show_version {
            return None;
        }

        // Try to get Node version
        std::process::Command::new("node")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let version = String::from_utf8_lossy(&output.stdout);
                // Extract version (e.g., "v18.17.0" -> "18.17.0")
                version.trim().strip_prefix('v').map(|v| v.to_string())
            })
    }

    /// Get package name from package.json (if available)
    fn get_package_name(&self, context: &ModuleContext) -> Option<String> {
        // Try to read and parse package.json
        let package_json_path = context.pwd.join("package.json");

        if let Ok(contents) = context.fs.read_to_string(&package_json_path) {
            // Simple JSON parsing for name field
            // This is a simplified approach - could use serde_json for full parsing
            if let Some(name_line) = contents.lines().find(|line| line.contains("\"name\"")) {
                // Extract name value: "name": "my-package"
                if let Some(start) = name_line.find(':') {
                    let value_part = &name_line[start + 1..];
                    let name = value_part
                        .trim()
                        .trim_matches(',')
                        .trim_matches('"')
                        .to_string();
                    return Some(name);
                }
            }
        }

        None
    }
}

impl Default for NodeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for NodeModule {
    fn id(&self) -> &str {
        "node"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        self.is_node_project(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Add package name if available
        if let Some(package_name) = self.get_package_name(context) {
            parts.push(package_name);
        } else {
            parts.push("node".to_string());
        }

        // Add version if requested
        if let Some(version) = self.get_node_version() {
            parts.push(format!("v{}", version));
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("Node.js", "Node.js project detection")
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::SandboxedFs;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_node_module_detection() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");
        fs::write(&package_json, r#"{"name": "test-package"}"#).unwrap();

        let module = NodeModule::new();
        let context = ModuleContext {
            pwd: temp_dir.path().to_path_buf(),
            home: PathBuf::from("/home/user"),
            env: HashMap::new(),
            fs: SandboxedFs::new(vec![temp_dir.path().to_path_buf()]),
        };

        assert!(module.should_display(&context));
    }
}
