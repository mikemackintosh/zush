// Docker module - detects Docker projects and contexts

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct DockerModule {
    symbol: String,
    show_context: bool,
}

impl DockerModule {
    pub fn new() -> Self {
        Self {
            symbol: "🐳".to_string(),
            show_context: false,
        }
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn with_context(mut self, show_context: bool) -> Self {
        self.show_context = show_context;
        self
    }

    /// Check if we're in a Docker project
    fn is_docker_project(&self, context: &ModuleContext) -> bool {
        // Check for Docker files
        context.fs.has_file("Dockerfile")
            || context.fs.has_file("docker-compose.yml")
            || context.fs.has_file("docker-compose.yaml")
            || context.fs.has_file(".dockerignore")
            || context.fs.has_dir(".devcontainer")
    }

    /// Get Docker context (if requested and available)
    fn get_docker_context(&self) -> Option<String> {
        if !self.show_context {
            return None;
        }

        // Try to get current Docker context
        std::process::Command::new("docker")
            .arg("context")
            .arg("show")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let context = String::from_utf8_lossy(&output.stdout);
                    let context_name = context.trim().to_string();

                    // Only show if not default
                    if context_name != "default" && !context_name.is_empty() {
                        Some(context_name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    /// Detect which Docker files are present
    fn detect_docker_files(&self, context: &ModuleContext) -> Vec<&str> {
        let mut files = Vec::new();

        if context.fs.has_file("Dockerfile") {
            files.push("Dockerfile");
        }

        if context.fs.has_file("docker-compose.yml") || context.fs.has_file("docker-compose.yaml") {
            files.push("compose");
        }

        if context.fs.has_dir(".devcontainer") {
            files.push("devcontainer");
        }

        files
    }
}

impl Default for DockerModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for DockerModule {
    fn id(&self) -> &str {
        "docker"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        self.is_docker_project(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Detect which Docker files are present
        let docker_files = self.detect_docker_files(context);

        if !docker_files.is_empty() {
            parts.push(docker_files.join("+"));
        } else {
            parts.push("docker".to_string());
        }

        // Add Docker context if requested and available
        if let Some(ctx) = self.get_docker_context() {
            parts.push(format!("({})", ctx));
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("Docker", "Docker project and context detection")
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::SandboxedFs;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_docker_module_detection() {
        let temp_dir = TempDir::new().unwrap();
        let dockerfile = temp_dir.path().join("Dockerfile");
        fs::write(&dockerfile, "FROM alpine:latest").unwrap();

        let module = DockerModule::new();
        let context = ModuleContext {
            pwd: temp_dir.path().to_path_buf(),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![temp_dir.path().to_path_buf()]),
        };

        assert!(module.should_display(&context));
    }

    #[test]
    fn test_docker_module_detect_files() {
        let temp_dir = TempDir::new().unwrap();
        let dockerfile = temp_dir.path().join("Dockerfile");
        let compose = temp_dir.path().join("docker-compose.yml");

        fs::write(&dockerfile, "FROM alpine:latest").unwrap();
        fs::write(&compose, "version: '3'").unwrap();

        let module = DockerModule::new();
        let context = ModuleContext {
            pwd: temp_dir.path().to_path_buf(),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![temp_dir.path().to_path_buf()]),
        };

        let files = module.detect_docker_files(&context);
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"Dockerfile"));
        assert!(files.contains(&"compose"));
    }
}
