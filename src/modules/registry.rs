// Module registry - manages and executes modules

use super::{Module, ModuleContext};
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Registry that manages all available modules
pub struct ModuleRegistry {
    modules: HashMap<String, Box<dyn Module>>,
    enabled: Vec<String>,
    cache: ModuleCache,
}

impl ModuleRegistry {
    /// Create a new module registry with all built-in modules
    pub fn new() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
            enabled: Vec::new(),
            cache: ModuleCache::new(),
        };

        // Register all built-in modules
        registry.register(Box::new(super::python::PythonModule::new()));
        registry.register(Box::new(super::node::NodeModule::new()));
        registry.register(Box::new(super::ruby::RubyModule::new()));
        registry.register(Box::new(super::rust_lang::RustModule::new()));
        registry.register(Box::new(super::docker::DockerModule::new()));

        // Enable all modules by default
        registry.enabled = registry.modules.keys().cloned().collect();

        registry
    }

    /// Register a new module
    pub fn register(&mut self, module: Box<dyn Module>) {
        let id = module.id().to_string();
        self.modules.insert(id.clone(), module);

        // Add to enabled list if enabled by default
        if let Some(m) = self.modules.get(&id) {
            if m.enabled_by_default() && !self.enabled.contains(&id) {
                self.enabled.push(id);
            }
        }
    }

    /// Enable a module
    pub fn enable(&mut self, id: &str) {
        if self.modules.contains_key(id) && !self.enabled.contains(&id.to_string()) {
            self.enabled.push(id.to_string());
        }
    }

    /// Disable a module
    pub fn disable(&mut self, id: &str) {
        self.enabled.retain(|m| m != id);
    }

    /// Set which modules are enabled
    pub fn set_enabled(&mut self, enabled: Vec<String>) {
        // Filter to only valid module IDs
        self.enabled = enabled
            .into_iter()
            .filter(|id| self.modules.contains_key(id))
            .collect();
    }

    /// Get a module by ID
    pub fn get(&self, id: &str) -> Option<&dyn Module> {
        self.modules.get(id).map(|m| m.as_ref())
    }

    /// List all available module IDs
    pub fn available_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// List enabled module IDs
    pub fn enabled_modules(&self) -> Vec<String> {
        self.enabled.clone()
    }

    /// Render all enabled modules that should display
    pub fn render_all(&mut self, context: &ModuleContext) -> Vec<ModuleOutput> {
        let mut outputs = Vec::new();

        for module_id in &self.enabled {
            if let Some(module) = self.modules.get(module_id) {
                // Check cache first
                if let Some(cached) = self.cache.get(module_id) {
                    outputs.push(cached);
                    continue;
                }

                // Check if module should display
                if !module.should_display(context) {
                    continue;
                }

                // Render with timeout
                match Self::render_with_timeout(
                    module.as_ref(),
                    context,
                    Duration::from_millis(100),
                ) {
                    Ok(output) => {
                        let module_output = ModuleOutput {
                            id: module_id.clone(),
                            content: output,
                            timestamp: Instant::now(),
                        };

                        // Cache the output
                        self.cache.set(module_id.clone(), module_output.clone());

                        outputs.push(module_output);
                    }
                    Err(e) => {
                        eprintln!("Module '{}' error: {}", module_id, e);
                    }
                }
            }
        }

        outputs
    }

    /// Render a module with timeout
    fn render_with_timeout(
        module: &dyn Module,
        context: &ModuleContext,
        timeout: Duration,
    ) -> Result<String> {
        // For now, just render directly
        // In the future, could use tokio::time::timeout for async
        module.render(context)
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from a module render
#[derive(Clone, Debug)]
pub struct ModuleOutput {
    pub id: String,
    pub content: String,
    pub timestamp: Instant,
}

/// Cache for module outputs
struct ModuleCache {
    cache: HashMap<String, ModuleOutput>,
    cache_duration: Duration,
}

impl ModuleCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_duration: Duration::from_millis(200), // Cache for 200ms
        }
    }

    fn get(&self, id: &str) -> Option<ModuleOutput> {
        self.cache.get(id).and_then(|output| {
            if output.timestamp.elapsed() < self.cache_duration {
                Some(output.clone())
            } else {
                None
            }
        })
    }

    fn set(&mut self, id: String, output: ModuleOutput) {
        self.cache.insert(id, output);
    }

    fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ModuleRegistry::new();

        // Should have built-in modules
        assert!(registry.available_modules().len() > 0);
        assert!(registry.enabled_modules().len() > 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut registry = ModuleRegistry::new();

        // Disable a module
        registry.disable("python");
        assert!(!registry.enabled_modules().contains(&"python".to_string()));

        // Re-enable it
        registry.enable("python");
        assert!(registry.enabled_modules().contains(&"python".to_string()));
    }

    #[test]
    fn test_get_module() {
        let registry = ModuleRegistry::new();

        assert!(registry.get("python").is_some());
        assert!(registry.get("nonexistent").is_none());
    }
}
