// Module system for Zush prompt
// Provides a safe, sandboxed API for context-aware prompt modules

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod docker;
pub mod node;
pub mod python;
pub mod registry;
pub mod ruby;
pub mod rust_lang;

/// Module trait - implemented by all prompt modules
pub trait Module: Send + Sync {
    /// Unique module identifier
    fn id(&self) -> &str;

    /// Should this module display in the current context?
    fn should_display(&self, context: &ModuleContext) -> bool;

    /// Render the module output
    fn render(&self, context: &ModuleContext) -> Result<String>;

    /// Module metadata
    fn metadata(&self) -> ModuleMetadata;

    /// Whether the module is enabled by default
    fn enabled_by_default(&self) -> bool {
        true
    }
}

/// Context provided to modules - sandboxed and safe
pub struct ModuleContext {
    /// Current working directory
    pub pwd: PathBuf,

    /// Home directory
    pub home: PathBuf,

    /// Environment variables (read-only)
    pub env: HashMap<String, String>,

    /// Sandboxed filesystem access
    pub fs: SandboxedFs,
}

impl ModuleContext {
    /// Create a new module context
    pub fn new() -> Result<Self> {
        let pwd = std::env::current_dir()?;
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        // Collect environment variables
        let env: HashMap<String, String> = std::env::vars().collect();

        // Create sandboxed filesystem with allowed paths
        let fs = SandboxedFs::new(vec![pwd.clone(), home.clone()]);

        Ok(Self { pwd, home, env, fs })
    }

    /// Check if an environment variable exists
    pub fn has_env(&self, key: &str) -> bool {
        self.env.contains_key(key)
    }

    /// Get an environment variable
    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(|s| s.as_str())
    }
}

/// Sandboxed filesystem access - restricts what modules can access
pub struct SandboxedFs {
    allowed_paths: Vec<PathBuf>,
}

impl SandboxedFs {
    /// Create a new sandboxed filesystem
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    /// Check if a file/directory exists (only in allowed paths)
    pub fn exists(&self, path: &Path) -> bool {
        if !self.is_allowed(path) {
            return false;
        }
        path.exists()
    }

    /// Check if a file exists (only in allowed paths)
    pub fn has_file(&self, filename: &str) -> bool {
        for allowed in &self.allowed_paths {
            let path = allowed.join(filename);
            if self.is_allowed(&path) && path.is_file() {
                return true;
            }
        }
        false
    }

    /// Check if a directory exists (only in allowed paths)
    pub fn has_dir(&self, dirname: &str) -> bool {
        for allowed in &self.allowed_paths {
            let path = allowed.join(dirname);
            if self.is_allowed(&path) && path.is_dir() {
                return true;
            }
        }
        false
    }

    /// Read a file to string (with restrictions)
    pub fn read_to_string(&self, path: &Path) -> Result<String> {
        if !self.is_allowed(path) {
            anyhow::bail!("Access denied: path not in allowed list");
        }

        // Check file size (max 1MB)
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > 1024 * 1024 {
            anyhow::bail!("File too large (max 1MB)");
        }

        Ok(std::fs::read_to_string(path)?)
    }

    /// Check if a path is in the allowed list
    fn is_allowed(&self, path: &Path) -> bool {
        // Convert to absolute path
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            // Resolve relative to first allowed path (usually pwd)
            if let Some(base) = self.allowed_paths.first() {
                base.join(path)
            } else {
                return false;
            }
        };

        // Check if path starts with any allowed path
        self.allowed_paths
            .iter()
            .any(|allowed| abs_path.starts_with(allowed))
    }
}

/// Module metadata
#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
}

impl ModuleMetadata {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: "Zush".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_sandboxed_fs_allowed_path() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd.clone()]);

        // Should allow checking files in pwd
        assert!(fs.is_allowed(&pwd.join("Cargo.toml")));
    }

    #[test]
    fn test_sandboxed_fs_disallowed_path() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd]);

        // Should not allow /etc/passwd
        assert!(!fs.is_allowed(Path::new("/etc/passwd")));
    }

    #[test]
    fn test_module_context_creation() {
        let context = ModuleContext::new().unwrap();

        assert!(context.pwd.exists());
        assert!(context.home.exists());
        assert!(!context.env.is_empty());
    }
}
