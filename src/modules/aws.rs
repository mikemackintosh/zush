// AWS module - shows current profile and region

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;

pub struct AwsModule {
    symbol: String,
    show_region: bool,
}

impl AwsModule {
    pub fn new() -> Self {
        Self {
            symbol: "☁️".to_string(),
            show_region: true,
        }
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn with_region(mut self, show_region: bool) -> Self {
        self.show_region = show_region;
        self
    }

    /// Get current AWS profile
    fn get_profile(&self, context: &ModuleContext) -> Option<String> {
        // Check AWS_PROFILE first (most common)
        if let Some(profile) = context.get_env("AWS_PROFILE") {
            return Some(profile);
        }

        // Check AWS_DEFAULT_PROFILE
        if let Some(profile) = context.get_env("AWS_DEFAULT_PROFILE") {
            return Some(profile);
        }

        // If credentials are set but no profile, show "credentials"
        if context.has_env("AWS_ACCESS_KEY_ID") {
            return Some("credentials".to_string());
        }

        // Default profile
        Some("default".to_string())
    }

    /// Get current AWS region
    fn get_region(&self, context: &ModuleContext) -> Option<String> {
        // Check AWS_REGION first
        if let Some(region) = context.get_env("AWS_REGION") {
            return Some(region);
        }

        // Check AWS_DEFAULT_REGION
        if let Some(region) = context.get_env("AWS_DEFAULT_REGION") {
            return Some(region);
        }

        None
    }

    /// Check if AWS credentials are configured
    fn has_credentials(&self, context: &ModuleContext) -> bool {
        // Check for environment variables
        context.has_env("AWS_PROFILE")
            || context.has_env("AWS_DEFAULT_PROFILE")
            || context.has_env("AWS_ACCESS_KEY_ID")
            || context.has_env("AWS_SESSION_TOKEN")
            // Check for config file
            || context.fs.exists(&context.home.join(".aws").join("config"))
            || context.fs.exists(&context.home.join(".aws").join("credentials"))
    }
}

impl Default for AwsModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for AwsModule {
    fn id(&self) -> &str {
        "aws"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        // Show if AWS credentials are configured
        self.has_credentials(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Get profile
        if let Some(profile) = self.get_profile(context) {
            // Don't show "default" to keep it clean
            if profile != "default" {
                parts.push(profile);
            }
        }

        // Add region if requested
        if self.show_region {
            if let Some(region) = self.get_region(context) {
                // Shorten region names (us-east-1 -> use1)
                let short_region = region
                    .replace("us-east-", "use")
                    .replace("us-west-", "usw")
                    .replace("eu-west-", "euw")
                    .replace("eu-central-", "euc")
                    .replace("ap-southeast-", "apse")
                    .replace("ap-northeast-", "apne");

                parts.push(format!("({})", short_region));
            }
        }

        // If only symbol, add "aws" label
        if parts.len() == 1 {
            parts.push("aws".to_string());
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("AWS", "AWS profile and region detection")
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
    fn test_aws_module_basic() {
        let module = AwsModule::new();
        assert_eq!(module.id(), "aws");
        assert!(module.enabled_by_default());
    }
}
