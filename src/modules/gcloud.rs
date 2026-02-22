// GCloud module - shows current GCP project and config

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct GCloudModule {
    symbol: String,
}

impl GCloudModule {
    pub fn new() -> Self {
        Self {
            symbol: "\u{f0c2}".to_string(),
        }
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = symbol;
        self
    }

    /// Get current GCP project
    fn get_project(&self, context: &ModuleContext) -> Option<String> {
        // Check CLOUDSDK_CORE_PROJECT first
        if let Some(project) = context.get_env("CLOUDSDK_CORE_PROJECT") {
            return Some(project);
        }

        // Check GCP_PROJECT or GOOGLE_CLOUD_PROJECT
        if let Some(project) = context.get_env("GCP_PROJECT") {
            return Some(project);
        }

        if let Some(project) = context.get_env("GOOGLE_CLOUD_PROJECT") {
            return Some(project);
        }

        // Read from gcloud config - check active config first, then default
        let config_dir = context.home.join(".config").join("gcloud");
        let active_config_name = self.get_config_name(context).unwrap_or_else(|| "default".to_string());
        let properties_file = config_dir.join("configurations").join(format!("config_{}", active_config_name));

        if context.fs.exists(&properties_file) {
            if let Ok(contents) = context.fs.read_to_string(&properties_file) {
                // Parse INI-style config
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if let Some(project) = trimmed.strip_prefix("project") {
                        let project = project.trim().strip_prefix('=').map(|p| p.trim().to_string());
                        if let Some(p) = project {
                            if !p.is_empty() {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Get current gcloud configuration name
    fn get_config_name(&self, context: &ModuleContext) -> Option<String> {
        // Check CLOUDSDK_ACTIVE_CONFIG_NAME env var
        if let Some(config) = context.get_env("CLOUDSDK_ACTIVE_CONFIG_NAME") {
            return Some(config);
        }

        // Read from gcloud config
        let config_dir = context.home.join(".config").join("gcloud");
        let active_config = config_dir.join("active_config");

        if context.fs.exists(&active_config) {
            if let Ok(contents) = context.fs.read_to_string(&active_config) {
                let config_name = contents.trim();
                if !config_name.is_empty() && config_name != "default" {
                    return Some(config_name.to_string());
                }
            }
        }

        None
    }

    /// Check if GCP is actively being used in the current context
    fn is_active(&self, context: &ModuleContext) -> bool {
        // Show if explicit GCP env vars are set
        // (this means the user intentionally activated GCP for this session)
        context.has_env("CLOUDSDK_CORE_PROJECT")
            || context.has_env("GCP_PROJECT")
            || context.has_env("GOOGLE_CLOUD_PROJECT")
            || context.has_env("CLOUDSDK_ACTIVE_CONFIG_NAME")
            // Show if current directory has GCP-related project files
            || context.fs.has_file("app.yaml")         // App Engine
            || context.fs.has_file("cloudbuild.yaml")
            || context.fs.has_file("cloudbuild.json")
            || context.fs.has_file(".gcloudignore")
            || self.has_gcp_terraform(context)
    }

    /// Check if Terraform files reference Google provider
    fn has_gcp_terraform(&self, context: &ModuleContext) -> bool {
        if !context.fs.has_dir(".terraform") && !context.fs.has_file("main.tf") {
            return false;
        }
        for file in &["main.tf", "providers.tf", "versions.tf", "terraform.tf"] {
            let path = context.pwd.join(file);
            if let Ok(contents) = context.fs.read_to_string(&path) {
                if contents.contains("provider \"google\"") || contents.contains("google_") {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for GCloudModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for GCloudModule {
    fn id(&self) -> &str {
        "gcloud"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        self.is_active(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        // Label in blue (Google brand color)
        let label = format!(
            "\x1b[38;2;66;133;244m{}gcp\x1b[39m",
            self.symbol
        );

        // Detail: project + config name, dimmed in square brackets
        let mut detail_parts = Vec::new();
        if let Some(project) = self.get_project(context) {
            detail_parts.push(project);
        }
        if let Some(config) = self.get_config_name(context) {
            detail_parts.push(format!("({})", config));
        }

        if detail_parts.is_empty() {
            Ok(label)
        } else {
            Ok(format!(
                "{} \x1b[2m[{}]\x1b[22m",
                label,
                detail_parts.join(" ")
            ))
        }
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("GCloud", "Google Cloud project and configuration detection")
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
    fn test_gcloud_module_basic() {
        let module = GCloudModule::new();
        assert_eq!(module.id(), "gcloud");
        assert!(module.enabled_by_default());
    }
}
