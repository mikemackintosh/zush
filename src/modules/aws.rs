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
            symbol: "\u{f0c2}".to_string(),
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

        // Read from AWS config file for the active profile
        let config_file = context.home.join(".aws").join("config");
        if context.fs.exists(&config_file) {
            if let Ok(contents) = context.fs.read_to_string(&config_file) {
                let profile = self.get_profile(context).unwrap_or_default();
                let section = if profile == "default" {
                    "[default]".to_string()
                } else {
                    format!("[profile {}]", profile)
                };

                let mut in_section = false;
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if trimmed == section {
                        in_section = true;
                        continue;
                    }
                    if in_section && trimmed.starts_with('[') {
                        break;
                    }
                    if in_section {
                        if let Some(region) = trimmed.strip_prefix("region") {
                            let region = region.trim().strip_prefix('=').map(|r| r.trim().to_string());
                            if let Some(r) = region {
                                if !r.is_empty() {
                                    return Some(r);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if AWS is actively being used in the current context
    fn is_active(&self, context: &ModuleContext) -> bool {
        // Show if an explicit profile or credentials are set via env vars
        // (this means the user intentionally activated AWS for this session)
        context.has_env("AWS_PROFILE")
            || context.has_env("AWS_DEFAULT_PROFILE")
            || context.has_env("AWS_ACCESS_KEY_ID")
            || context.has_env("AWS_SESSION_TOKEN")
            // Show if current directory has AWS-related project files
            || context.fs.has_dir(".aws-sam")
            || context.fs.has_dir("cdk.out")
            || context.fs.has_file("samconfig.toml")
            || context.fs.has_file("cdk.json")
            || context.fs.has_file("serverless.yml")
            || context.fs.has_file("serverless.yaml")
            || context.fs.has_file("template.yaml")  // SAM/CloudFormation
            || self.has_aws_terraform(context)
    }

    /// Check if Terraform files reference AWS provider
    fn has_aws_terraform(&self, context: &ModuleContext) -> bool {
        // Only check if we're in a Terraform project
        if !context.fs.has_dir(".terraform") && !context.fs.has_file("main.tf") {
            return false;
        }
        // Check for AWS provider in common Terraform files
        for file in &["main.tf", "providers.tf", "versions.tf", "terraform.tf"] {
            let path = context.pwd.join(file);
            if let Ok(contents) = context.fs.read_to_string(&path) {
                if contents.contains("provider \"aws\"") || contents.contains("aws_") {
                    return true;
                }
            }
        }
        false
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
        self.is_active(context)
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        // Label in orange (AWS brand color)
        let label = format!(
            "\x1b[38;2;255;153;0m{}aws\x1b[39m",
            self.symbol
        );

        // Detail: profile + region, dimmed
        let mut detail_parts = Vec::new();
        if let Some(profile) = self.get_profile(context) {
            detail_parts.push(profile);
        }
        if self.show_region {
            if let Some(region) = self.get_region(context) {
                let short_region = region
                    .replace("us-east-", "use")
                    .replace("us-west-", "usw")
                    .replace("eu-west-", "euw")
                    .replace("eu-central-", "euc")
                    .replace("ap-southeast-", "apse")
                    .replace("ap-northeast-", "apne");
                detail_parts.push(format!("({})", short_region));
            }
        }

        if detail_parts.is_empty() {
            Ok(label)
        } else {
            Ok(format!(
                "{} \x1b[2m{}\x1b[22m",
                label,
                detail_parts.join(" ")
            ))
        }
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
