// Kubernetes module - shows current context and namespace

use super::{Module, ModuleContext, ModuleMetadata};
use anyhow::Result;
use std::path::PathBuf;

pub struct KubernetesModule {
    symbol: String,
    show_namespace: bool,
}

impl KubernetesModule {
    pub fn new() -> Self {
        Self {
            symbol: "☸".to_string(),
            show_namespace: true,
        }
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn with_namespace(mut self, show_namespace: bool) -> Self {
        self.show_namespace = show_namespace;
        self
    }

    /// Get current kubeconfig path
    fn get_kubeconfig_path(&self, context: &ModuleContext) -> PathBuf {
        // Check KUBECONFIG env var first
        if let Some(kubeconfig) = context.get_env("KUBECONFIG") {
            PathBuf::from(kubeconfig)
        } else {
            // Default to ~/.kube/config
            context.home.join(".kube").join("config")
        }
    }

    /// Parse kubeconfig to get current context
    fn get_current_context(&self, context: &ModuleContext) -> Option<String> {
        let kubeconfig_path = self.get_kubeconfig_path(context);

        if !context.fs.exists(&kubeconfig_path) {
            return None;
        }

        // Read kubeconfig file
        let contents = context.fs.read_to_string(&kubeconfig_path).ok()?;

        // Simple YAML parsing - look for "current-context: value"
        for line in contents.lines() {
            let trimmed = line.trim();
            if let Some(ctx) = trimmed.strip_prefix("current-context:") {
                let ctx_value = ctx.trim().trim_matches('"').trim_matches('\'');
                if !ctx_value.is_empty() {
                    return Some(ctx_value.to_string());
                }
            }
        }

        None
    }

    /// Get current namespace from env or kubeconfig
    fn get_current_namespace(&self, context: &ModuleContext) -> Option<String> {
        // Check KUBECTL_CONTEXT_NAMESPACE or similar env vars first
        if let Some(ns) = context.get_env("KUBECTL_CONTEXT_NAMESPACE") {
            return Some(ns);
        }

        // Try reading from kubeconfig context
        let kubeconfig_path = self.get_kubeconfig_path(context);
        if !context.fs.exists(&kubeconfig_path) {
            return None;
        }

        let contents = context.fs.read_to_string(&kubeconfig_path).ok()?;
        let current_context = self.get_current_context(context)?;

        // Parse kubeconfig to find namespace for current context
        // This is a simplified parser - look for the context section
        let mut in_context_section = false;
        let mut in_current_context = false;

        for line in contents.lines() {
            let trimmed = line.trim();

            // Check if we're entering contexts section
            if trimmed.starts_with("contexts:") {
                in_context_section = true;
                continue;
            }

            // Check if we're in the current context
            if in_context_section && trimmed.contains(&format!("name: {}", current_context)) {
                in_current_context = true;
                continue;
            }

            // Look for namespace in current context
            if in_current_context {
                if let Some(ns) = trimmed.strip_prefix("namespace:") {
                    let ns_value = ns.trim().trim_matches('"').trim_matches('\'');
                    if !ns_value.is_empty() {
                        return Some(ns_value.to_string());
                    }
                }

                // Exit context if we hit another context or section
                if trimmed.starts_with("- name:") || trimmed.starts_with("users:") || trimmed.starts_with("clusters:") {
                    break;
                }
            }
        }

        // Default namespace
        Some("default".to_string())
    }
}

impl Default for KubernetesModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for KubernetesModule {
    fn id(&self) -> &str {
        "kubernetes"
    }

    fn should_display(&self, context: &ModuleContext) -> bool {
        // Show if KUBECONFIG is set or ~/.kube/config exists
        context.has_env("KUBECONFIG") || {
            let kubeconfig_path = context.home.join(".kube").join("config");
            context.fs.exists(&kubeconfig_path)
        }
    }

    fn render(&self, context: &ModuleContext) -> Result<String> {
        let mut parts = vec![self.symbol.clone()];

        // Get current context
        if let Some(ctx) = self.get_current_context(context) {
            // Shorten long context names (e.g., gke_project_region_cluster -> cluster)
            let short_ctx = if let Some(last_part) = ctx.split('_').last() {
                last_part
            } else {
                &ctx
            };

            parts.push(short_ctx.to_string());

            // Add namespace if requested
            if self.show_namespace {
                if let Some(ns) = self.get_current_namespace(context) {
                    if ns != "default" {
                        parts.push(format!("/{}", ns));
                    }
                }
            }
        } else {
            parts.push("no-context".to_string());
        }

        Ok(parts.join(" "))
    }

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata::new("Kubernetes", "Kubernetes context and namespace detection")
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
    fn test_kubernetes_module_basic() {
        let module = KubernetesModule::new();
        assert_eq!(module.id(), "kubernetes");
        assert!(module.enabled_by_default());
    }
}
