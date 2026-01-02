#![allow(dead_code)]

// Module system for Zush prompt
// Provides a safe, sandboxed API for context-aware prompt modules

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod docker;
pub mod go;
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
    /// Canonicalized allowed paths for secure comparison
    allowed_paths: Vec<PathBuf>,
}

/// Maximum file size for read operations (1 MB)
const MAX_FILE_SIZE: u64 = 1024 * 1024;

impl SandboxedFs {
    /// Create a new sandboxed filesystem
    /// Canonicalizes allowed paths to prevent traversal attacks
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        // Canonicalize all allowed paths upfront
        let allowed_paths = allowed_paths
            .into_iter()
            .filter_map(|p| p.canonicalize().ok())
            .collect();

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
        // Reject filenames with path traversal attempts
        if filename.contains("..") {
            return false;
        }

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
        // Reject dirnames with path traversal attempts
        if dirname.contains("..") {
            return false;
        }

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

        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_FILE_SIZE {
            anyhow::bail!("File too large (max {} bytes)", MAX_FILE_SIZE);
        }

        Ok(std::fs::read_to_string(path)?)
    }

    /// Check if a path is in the allowed list
    /// Uses canonicalization to prevent path traversal attacks (e.g., "../../../etc/passwd")
    fn is_allowed(&self, path: &Path) -> bool {
        // Build absolute path
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            // Resolve relative to first allowed path (usually pwd)
            match self.allowed_paths.first() {
                Some(base) => base.join(path),
                None => return false,
            }
        };

        // Canonicalize to resolve any ".." or symlinks
        // This is the key security fix - prevents traversal attacks
        let canonical = match abs_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // File doesn't exist yet - use manual normalization
                // This handles cases like checking if we can create a file
                match Self::normalize_path(&abs_path) {
                    Some(p) => p,
                    None => return false,
                }
            }
        };

        // Check if canonicalized path starts with any allowed path
        self.allowed_paths
            .iter()
            .any(|allowed| canonical.starts_with(allowed))
    }

    /// Normalize a path without requiring it to exist
    /// Removes "." and ".." components safely
    fn normalize_path(path: &Path) -> Option<PathBuf> {
        use std::path::Component;

        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                Component::Prefix(p) => normalized.push(p.as_os_str()),
                Component::RootDir => normalized.push("/"),
                Component::CurDir => {} // Skip "."
                Component::ParentDir => {
                    // Go up one directory, but don't go above root
                    if !normalized.pop() {
                        // Can't go above root - this is a traversal attempt
                        return None;
                    }
                }
                Component::Normal(name) => normalized.push(name),
            }
        }

        Some(normalized)
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
    fn test_sandboxed_fs_path_traversal_absolute() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd.clone()]);

        // Path traversal with absolute path - should be blocked
        let traversal = pwd.join("../../../etc/passwd");
        assert!(!fs.is_allowed(&traversal));
    }

    #[test]
    fn test_sandboxed_fs_path_traversal_relative() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd]);

        // Path traversal with relative path - should be blocked
        assert!(!fs.is_allowed(Path::new("../../../etc/passwd")));
    }

    #[test]
    fn test_sandboxed_fs_has_file_traversal() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd]);

        // Should block traversal in has_file
        assert!(!fs.has_file("../../../etc/passwd"));
        assert!(!fs.has_file("foo/../../../etc/passwd"));
    }

    #[test]
    fn test_sandboxed_fs_has_dir_traversal() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd]);

        // Should block traversal in has_dir
        assert!(!fs.has_dir("../../../etc"));
        assert!(!fs.has_dir("foo/../../.."));
    }

    #[test]
    fn test_sandboxed_fs_dot_components() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd.clone()]);

        // Single dot should be fine (current dir)
        assert!(fs.is_allowed(&pwd.join("./Cargo.toml")));
    }

    #[test]
    fn test_sandboxed_fs_subdirectory_allowed() {
        let pwd = env::current_dir().unwrap();
        let fs = SandboxedFs::new(vec![pwd.clone()]);

        // Subdirectories should be allowed
        assert!(fs.is_allowed(&pwd.join("src/main.rs")));
    }

    #[test]
    fn test_normalize_path_removes_dotdot() {
        // Test the normalize_path function directly
        let path = Path::new("/home/user/project/../../../etc/passwd");
        let normalized = SandboxedFs::normalize_path(path);

        // Should normalize to /etc/passwd (not under /home/user/project)
        assert_eq!(normalized, Some(PathBuf::from("/etc/passwd")));
    }

    #[test]
    fn test_normalize_path_removes_dot() {
        let path = Path::new("/home/user/./project/./file.txt");
        let normalized = SandboxedFs::normalize_path(path);

        assert_eq!(normalized, Some(PathBuf::from("/home/user/project/file.txt")));
    }

    #[test]
    fn test_normalize_path_complex() {
        let path = Path::new("/a/b/c/../d/./e/../f");
        let normalized = SandboxedFs::normalize_path(path);

        assert_eq!(normalized, Some(PathBuf::from("/a/b/d/f")));
    }

    #[test]
    fn test_module_context_creation() {
        let context = ModuleContext::new().unwrap();

        assert!(context.pwd.exists());
        assert!(context.home.exists());
        assert!(!context.env.is_empty());
    }

    #[test]
    fn test_max_file_size_constant() {
        // Verify the constant is 1MB
        assert_eq!(MAX_FILE_SIZE, 1024 * 1024);
    }
}
