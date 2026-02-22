// Terraform module - shows current workspace

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct TerraformModule {
    symbol: String,
}

impl TerraformModule {
    pub fn new() -> Self {
        Self {
            symbol: "\u{f1b3}".to_string(),
        }
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = symbol;
        self
    }

    /// Check if we're in a Terraform project
    fn is_terraform_project(&self, context: &ModuleContext) -> bool {
        // Check for Terraform files
        context.fs.has_file("main.tf")
            || context.fs.has_file("terraform.tf")
            || context.fs.has_file("versions.tf")
            || context.fs.has_dir(".terraform")
    }

    /// Get current Terraform workspace
    fn get_workspace(&self, context: &ModuleContext) -> Option<String> {
        // First check TF_WORKSPACE env var
        if let Some(workspace) = context.get_env("TF_WORKSPACE") {
            return Some(workspace);
        }

        // Read from .terraform/environment file
        let env_file = context.pwd.join(".terraform").join("environment");
        if context.fs.exists(&env_file) {
            if let Ok(contents) = context.fs.read_to_string(&env_file) {
                let workspace = contents.trim();
                if !workspace.is_empty() {
                    return Some(workspace.to_string());
                }
            }
        }

        // Default workspace
        Some("default".to_string())
    }

    /// Get Terraform version from files
    fn get_version_constraint(&self, context: &ModuleContext) -> Option<String> {
        // Try to read version from versions.tf or main.tf
        for file in &["versions.tf", "terraform.tf", "main.tf"] {
            let file_path = context.pwd.join(file);
            if let Ok(contents) = context.fs.read_to_string(&file_path) {
                // Look for required_version
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("required_version") {
                        // Extract version constraint (simplified)
                        if let Some(version) = trimmed.split('"').nth(1) {
                            return Some(version.to_string());
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for TerraformModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for TerraformModule {
    fn id(&self) -> &str {
        "terraform"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        self.is_terraform_project(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Get workspace
        if let Some(workspace) = self.get_workspace(context) {
            // Highlight non-default workspaces (production, staging, etc.)
            if workspace != "default" {
                parts.push(workspace);
            } else {
                // Just show "tf" for default workspace
                parts.push("tf".to_string());
            }
        } else {
            parts.push("tf".to_string());
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("Terraform", "Terraform workspace detection")
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
    fn test_terraform_module_detection() {
        let temp_dir = TempDir::new().unwrap();
        let main_tf = temp_dir.path().join("main.tf");
        fs::write(&main_tf, "resource \"aws_instance\" \"example\" {}").unwrap();

        let module = TerraformModule::new();
        let context = ModuleContext {
            pwd: temp_dir.path().to_path_buf(),
            home: PathBuf::from("/home/user"),
            fs: SandboxedFs::new(vec![temp_dir.path().to_path_buf()]),
        };

        assert!(module.should_display(&context));
    }

    #[test]
    fn test_terraform_module_basic() {
        let module = TerraformModule::new();
        assert_eq!(module.id(), "terraform");
        assert!(module.enabled_by_default());
    }
}
