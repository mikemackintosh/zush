// Python module - detects Python virtual environments and projects

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;
use std::path::Path;

pub struct PythonModule {
    symbol: String,
    show_version: bool,
}

impl PythonModule {
    pub fn new() -> Self {
        Self {
            symbol: "🐍".to_string(),
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

    /// Check if we're in a Python project
    fn is_python_project(&self, context: &ModuleContext) -> bool {
        // Check for common Python files
        context.fs.has_file("pyproject.toml")
            || context.fs.has_file("requirements.txt")
            || context.fs.has_file("setup.py")
            || context.fs.has_file("Pipfile")
            || context.fs.has_file(".python-version")
            || context.fs.has_file("poetry.lock")
            || context.fs.has_dir(".venv")
            || context.fs.has_dir("venv")
    }

    /// Get virtual environment name
    fn get_venv_name(&self, context: &ModuleContext) -> Option<String> {
        context.get_env("VIRTUAL_ENV").and_then(|venv_path| {
            // Extract just the venv name from the path
            Path::new(&venv_path)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
    }

    /// Get Python version (if requested)
    fn get_python_version(&self) -> Option<String> {
        if !self.show_version {
            return None;
        }

        // Try to get Python version from python command
        std::process::Command::new("python")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let version = String::from_utf8_lossy(&output.stdout);
                // Extract version number (e.g., "Python 3.11.5" -> "3.11.5")
                version.split_whitespace().nth(1).map(|v| v.to_string())
            })
    }
}

impl Default for PythonModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for PythonModule {
    fn id(&self) -> &str {
        "python"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        // Show if virtual environment is active OR we're in a Python project
        context.has_env("VIRTUAL_ENV") || self.is_python_project(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Add venv name if active
        if let Some(venv_name) = self.get_venv_name(context) {
            parts.push(venv_name);
        } else {
            // Just show "python" if not in venv but in project
            parts.push("python".to_string());
        }

        // Add version if requested
        if let Some(version) = self.get_python_version() {
            parts.push(format!("v{}", version));
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("Python", "Python virtual environment and project detection")
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
    fn test_python_module_basic() {
        let module = PythonModule::new();

        let context = ModuleContext {
            pwd: PathBuf::from("/home/user/project"),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![PathBuf::from("/home/user/project")]),
        };

        // Test basic module properties
        assert_eq!(module.id(), "python");
        assert!(module.enabled_by_default());

        // Note: should_display depends on actual env vars or filesystem
        // In real tests, we'd check for pyproject.toml etc.
        let _ = module.should_display(&context);
    }

    #[test]
    fn test_python_module_render() {
        let module = PythonModule::new();

        let context = ModuleContext {
            pwd: PathBuf::from("/home/user/project"),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![PathBuf::from("/home/user/project")]),
        };

        // Test render doesn't panic
        let result = module.render(&context);
        assert!(result.is_ok());
    }
}
