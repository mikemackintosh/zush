// GCloud module - shows current GCP project and config

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct GCloudModule {
    symbol: String,
    show_config: bool,
}

impl GCloudModule {
    pub fn new() -> Self {
        Self {
            symbol: "☁️".to_string(),
            show_config: false,
        }
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn with_config(mut self, show_config: bool) -> Self {
        self.show_config = show_config;
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

        // Read from gcloud config
        let config_dir = context.home.join(".config").join("gcloud");
        let properties_file = config_dir.join("configurations").join("config_default");

        if context.fs.exists(&properties_file) {
            if let Ok(contents) = context.fs.read_to_string(&properties_file) {
                // Parse INI-style config
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if let Some(project) = trimmed.strip_prefix("project = ") {
                        return Some(project.trim().to_string());
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

    /// Check if gcloud is configured
    fn has_gcloud_config(&self, context: &ModuleContext) -> bool {
        // Check for environment variables
        context.has_env("CLOUDSDK_CORE_PROJECT")
            || context.has_env("GCP_PROJECT")
            || context.has_env("GOOGLE_CLOUD_PROJECT")
            || context.has_env("CLOUDSDK_ACTIVE_CONFIG_NAME")
            // Check for config directory
            || context.fs.exists(&context.home.join(".config").join("gcloud").join("configurations"))
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
        // Show if gcloud is configured
        self.has_gcloud_config(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone(), "gcp".to_string()];

        // Get project
        if let Some(project) = self.get_project(context) {
            // Shorten long project IDs (e.g., my-company-production-12345 -> prod-12345)
            let short_project = if project.len() > 20 {
                // Take last part after last hyphen
                project.split('-').last().unwrap_or(&project).to_string()
            } else {
                project
            };

            parts.push(short_project);
        }

        // Add config name if requested and not default
        if self.show_config {
            if let Some(config) = self.get_config_name(context) {
                parts.push(format!("({})", config));
            }
        }

        Ok(parts.join(" "))
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
