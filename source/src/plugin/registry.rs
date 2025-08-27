//! Plugin registry and discovery system
//! 
//! This module manages plugin registration, discovery, and lifecycle.

use super::{
    Plugin, PluginConfig, PluginContext, PluginInfo, PluginMetadata, PluginStatus,
    ValidatorPlugin, FormatterPlugin, AnalyzerPlugin, ReporterPlugin,
    ResourceLimits, PluginCategory,
};
use anyhow::{anyhow, Result};
use slog::{debug, error, info, warn, Logger};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

/// Type alias for a boxed plugin
pub type BoxedPlugin = Box<dyn Plugin>;

/// Registry for managing all plugins
pub struct PluginRegistry {
    /// All registered plugins
    plugins: RwLock<HashMap<String, Arc<Mutex<BoxedPlugin>>>>,
    /// Plugin configurations
    configs: RwLock<HashMap<String, PluginConfig>>,
    /// Plugin metadata cache
    metadata_cache: RwLock<HashMap<String, PluginMetadata>>,
    /// Plugin status tracking
    status_cache: RwLock<HashMap<String, PluginStatus>>,
    /// Logger instance
    logger: Logger,
    /// Default resource limits
    default_limits: ResourceLimits,
    /// Plugin discovery directories
    plugin_dirs: Vec<PathBuf>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(logger: Logger) -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
            metadata_cache: RwLock::new(HashMap::new()),
            status_cache: RwLock::new(HashMap::new()),
            logger,
            default_limits: ResourceLimits::default(),
            plugin_dirs: vec![
                PathBuf::from("plugins"),
                PathBuf::from("~/.synx/plugins"),
                PathBuf::from("/usr/local/lib/synx/plugins"),
            ],
        }
    }

    /// Register a plugin statically
    pub async fn register_plugin(
        &self,
        plugin: BoxedPlugin,
        config: Option<PluginConfig>,
    ) -> Result<()> {
        let metadata = plugin.metadata().clone();
        let plugin_id = metadata.id.clone();
        
        info!(self.logger, "Registering plugin: {}", plugin_id);
        
        // Validate plugin metadata
        self.validate_plugin_metadata(&metadata)?;
        
        // Get or create config
        let plugin_config = config.unwrap_or_default();
        
        // Store plugin and metadata
        {
            let mut plugins = self.plugins.write().unwrap();
            let mut configs = self.configs.write().unwrap();
            let mut metadata_cache = self.metadata_cache.write().unwrap();
            let mut status_cache = self.status_cache.write().unwrap();
            
            plugins.insert(plugin_id.clone(), Arc::new(Mutex::new(plugin)));
            configs.insert(plugin_id.clone(), plugin_config);
            metadata_cache.insert(plugin_id.clone(), metadata);
            status_cache.insert(plugin_id.clone(), PluginStatus::Unloaded);
        }
        
        info!(self.logger, "Successfully registered plugin: {}", plugin_id);
        Ok(())
    }

    /// Initialize all registered plugins
    pub async fn initialize_all(&self, base_context: &PluginContext) -> Result<()> {
        let plugin_ids: Vec<String> = {
            let plugins = self.plugins.read().unwrap();
            plugins.keys().cloned().collect()
        };

        for plugin_id in plugin_ids {
            if let Err(e) = self.initialize_plugin(&plugin_id, base_context).await {
                error!(self.logger, "Failed to initialize plugin {}: {}", plugin_id, e);
                // Mark plugin as failed but continue with others
                let mut status_cache = self.status_cache.write().unwrap();
                status_cache.insert(plugin_id, PluginStatus::Failed(e.to_string()));
            }
        }

        Ok(())
    }

    /// Initialize a specific plugin
    pub async fn initialize_plugin(
        &self,
        plugin_id: &str,
        base_context: &PluginContext,
    ) -> Result<()> {
        info!(self.logger, "Initializing plugin: {}", plugin_id);

        let (plugin_arc, config) = {
            let plugins = self.plugins.read().unwrap();
            let configs = self.configs.read().unwrap();

            let plugin_arc = plugins.get(plugin_id)
                .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?
                .clone();

            let config = configs.get(plugin_id)
                .ok_or_else(|| anyhow!("Config not found for plugin: {}", plugin_id))?
                .clone();

            (plugin_arc, config)
        };

        // Check if plugin is enabled
        if !config.enabled {
            warn!(self.logger, "Plugin {} is disabled, skipping initialization", plugin_id);
            let mut status_cache = self.status_cache.write().unwrap();
            status_cache.insert(plugin_id.to_string(), PluginStatus::Disabled);
            return Ok(());
        }

        // Create plugin-specific context
        let resource_limits = config.resource_limits
            .unwrap_or_else(|| self.default_limits.clone());

        let plugin_context = PluginContext {
            working_dir: base_context.working_dir.clone(),
            config: config.settings.clone(),
            security_policy: base_context.security_policy.clone(),
            resource_limits,
            logger: base_context.logger.clone(),
        };

        // Initialize the plugin
        {
            let mut plugin = plugin_arc.lock().await;
            plugin.initialize(&plugin_context)?;
        }

        // Mark as active
        {
            let mut status_cache = self.status_cache.write().unwrap();
            status_cache.insert(plugin_id.to_string(), PluginStatus::Active);
        }

        info!(self.logger, "Successfully initialized plugin: {}", plugin_id);
        Ok(())
    }

    /// Get all plugins of a specific type
    pub fn get_plugins_by_category(&self, category: &PluginCategory) -> Vec<String> {
        let metadata_cache = self.metadata_cache.read().unwrap();
        let status_cache = self.status_cache.read().unwrap();

        metadata_cache
            .iter()
            .filter(|(plugin_id, metadata)| {
                // Only include active plugins
                let is_active = if let Some(status) = status_cache.get(*plugin_id) {
                    matches!(status, PluginStatus::Active)
                } else {
                    false
                };
                is_active && metadata.categories.contains(category)
            })
            .map(|(plugin_id, _)| plugin_id.clone())
            .collect()
    }

    /// Get plugins that can handle a specific file
    pub async fn get_plugins_for_file(&self, file_path: &Path) -> Vec<String> {
        let mut matching_plugins = Vec::new();

        let plugin_ids: Vec<String> = {
            let status_cache = self.status_cache.read().unwrap();
            status_cache
                .iter()
                .filter(|(_, status)| matches!(status, PluginStatus::Active))
                .map(|(id, _)| id.clone())
                .collect()
        };

        for plugin_id in plugin_ids {
            if let Some(plugin_arc) = self.plugins.read().unwrap().get(&plugin_id) {
                let plugin = plugin_arc.lock().await;
                if plugin.can_handle(file_path) {
                    matching_plugins.push(plugin_id);
                }
            }
        }

        // Sort by priority (higher priority first)
        matching_plugins.sort_by(|a, b| {
            let configs = self.configs.read().unwrap();
            let priority_a = configs.get(a).map(|c| c.priority).unwrap_or(0);
            let priority_b = configs.get(b).map(|c| c.priority).unwrap_or(0);
            priority_b.cmp(&priority_a)
        });

        matching_plugins
    }

    /// Get a specific plugin
    pub fn get_plugin(&self, plugin_id: &str) -> Option<Arc<Mutex<BoxedPlugin>>> {
        self.plugins.read().unwrap().get(plugin_id).cloned()
    }

    /// Get plugin information
    pub fn get_plugin_info(&self, plugin_id: &str) -> Option<PluginInfo> {
        let metadata_cache = self.metadata_cache.read().unwrap();
        let status_cache = self.status_cache.read().unwrap();
        let configs = self.configs.read().unwrap();

        let metadata = metadata_cache.get(plugin_id)?.clone();
        let status = status_cache.get(plugin_id)?.clone();
        let config = configs.get(plugin_id)?.clone();

        Some(PluginInfo {
            metadata,
            status,
            config,
        })
    }

    /// Get information about all plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let metadata_cache = self.metadata_cache.read().unwrap();
        
        metadata_cache
            .keys()
            .filter_map(|plugin_id| self.get_plugin_info(plugin_id))
            .collect()
    }

    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: &str, base_context: &PluginContext) -> Result<()> {
        {
            let mut configs = self.configs.write().unwrap();
            if let Some(config) = configs.get_mut(plugin_id) {
                config.enabled = true;
            } else {
                return Err(anyhow!("Plugin not found: {}", plugin_id));
            }
        }

        // Initialize the plugin if it's not already active
        let status = {
            let status_cache = self.status_cache.read().unwrap();
            status_cache.get(plugin_id).cloned()
        };

        if !matches!(status, Some(PluginStatus::Active)) {
            self.initialize_plugin(plugin_id, base_context).await?;
        }

        info!(self.logger, "Enabled plugin: {}", plugin_id);
        Ok(())
    }

    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        {
            let mut configs = self.configs.write().unwrap();
            if let Some(config) = configs.get_mut(plugin_id) {
                config.enabled = false;
            } else {
                return Err(anyhow!("Plugin not found: {}", plugin_id));
            }
        }

        // Clean up the plugin
        if let Some(plugin_arc) = self.plugins.read().unwrap().get(plugin_id) {
            let mut plugin = plugin_arc.lock().await;
            if let Err(e) = plugin.cleanup() {
                warn!(self.logger, "Error cleaning up plugin {}: {}", plugin_id, e);
            }
        }

        // Mark as disabled
        {
            let mut status_cache = self.status_cache.write().unwrap();
            status_cache.insert(plugin_id.to_string(), PluginStatus::Disabled);
        }

        info!(self.logger, "Disabled plugin: {}", plugin_id);
        Ok(())
    }

    /// Update plugin configuration
    pub async fn update_plugin_config(
        &self,
        plugin_id: &str,
        new_config: PluginConfig,
        base_context: &PluginContext,
    ) -> Result<()> {
        // Validate configuration with the plugin
        if let Some(plugin_arc) = self.plugins.read().unwrap().get(plugin_id) {
            let plugin = plugin_arc.lock().await;
            plugin.validate_config(&new_config.settings)?;
        } else {
            return Err(anyhow!("Plugin not found: {}", plugin_id));
        }

        // Update configuration
        {
            let mut configs = self.configs.write().unwrap();
            configs.insert(plugin_id.to_string(), new_config.clone());
        }

        // Reinitialize plugin if it's active and enabled
        if new_config.enabled {
            let status = {
                let status_cache = self.status_cache.read().unwrap();
                status_cache.get(plugin_id).cloned()
            };

            if matches!(status, Some(PluginStatus::Active)) {
                self.initialize_plugin(plugin_id, base_context).await?;
            }
        } else {
            self.disable_plugin(plugin_id).await?;
        }

        info!(self.logger, "Updated configuration for plugin: {}", plugin_id);
        Ok(())
    }

    /// Cleanup all plugins
    pub async fn cleanup_all(&self) -> Result<()> {
        let plugin_ids: Vec<String> = {
            let plugins = self.plugins.read().unwrap();
            plugins.keys().cloned().collect()
        };

        for plugin_id in plugin_ids {
            if let Some(plugin_arc) = self.plugins.read().unwrap().get(&plugin_id) {
                let mut plugin = plugin_arc.lock().await;
                if let Err(e) = plugin.cleanup() {
                    error!(self.logger, "Error cleaning up plugin {}: {}", plugin_id, e);
                }
            }
        }

        info!(self.logger, "Cleaned up all plugins");
        Ok(())
    }

    /// Validate plugin metadata
    fn validate_plugin_metadata(&self, metadata: &PluginMetadata) -> Result<()> {
        if metadata.id.is_empty() {
            return Err(anyhow!("Plugin ID cannot be empty"));
        }

        if metadata.name.is_empty() {
            return Err(anyhow!("Plugin name cannot be empty"));
        }

        if metadata.version.is_empty() {
            return Err(anyhow!("Plugin version cannot be empty"));
        }

        // Check for duplicate plugin IDs
        {
            let metadata_cache = self.metadata_cache.read().unwrap();
            if metadata_cache.contains_key(&metadata.id) {
                return Err(anyhow!("Plugin with ID '{}' already registered", metadata.id));
            }
        }

        Ok(())
    }

    /// Discover plugins from configured directories
    pub async fn discover_plugins(&self) -> Result<()> {
        debug!(self.logger, "Starting plugin discovery");

        for plugin_dir in &self.plugin_dirs {
            if plugin_dir.exists() && plugin_dir.is_dir() {
                self.discover_plugins_in_directory(plugin_dir).await?;
            } else {
                debug!(self.logger, "Plugin directory not found: {:?}", plugin_dir);
            }
        }

        Ok(())
    }

    /// Discover plugins in a specific directory
    async fn discover_plugins_in_directory(&self, dir: &Path) -> Result<()> {
        debug!(self.logger, "Scanning plugin directory: {:?}", dir);

        // For now, we'll implement static discovery
        // In the future, this could scan for .so files, WASM modules, etc.
        
        // Look for plugin configuration files
        let entries = std::fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                debug!(self.logger, "Found potential plugin config: {:?}", path);
                // TODO: Load plugin configuration and attempt to load the plugin
                // This would involve dynamic loading which we'll implement later
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{PluginMetadata, PluginCategory};
    use async_trait::async_trait;
    use slog::Drain;

    struct TestPlugin {
        metadata: PluginMetadata,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: "test_plugin".to_string(),
                    name: "Test Plugin".to_string(),
                    version: "1.0.0".to_string(),
                    description: "A test plugin".to_string(),
                    authors: vec!["Test Author".to_string()],
                    supported_extensions: vec!["txt".to_string()],
                    categories: vec![PluginCategory::Validator],
                    min_synx_version: "0.1.0".to_string(),
                    dependencies: vec![],
                },
            }
        }
    }

    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
            Ok(())
        }

        fn can_handle(&self, file_path: &Path) -> bool {
            file_path.extension().and_then(|s| s.to_str()) == Some("txt")
        }
    }

    fn create_test_logger() -> Logger {
        let drain = slog::Discard;
        Logger::root(drain, slog::o!())
    }

    #[tokio::test]
    async fn test_plugin_registration() {
        let logger = create_test_logger();
        let registry = PluginRegistry::new(logger);
        let plugin = Box::new(TestPlugin::new());

        let result = registry.register_plugin(plugin, None).await;
        assert!(result.is_ok());

        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].metadata.id, "test_plugin");
    }

    #[tokio::test]
    async fn test_plugin_enable_disable() {
        let logger = create_test_logger();
        let registry = PluginRegistry::new(logger.clone());
        let plugin = Box::new(TestPlugin::new());

        registry.register_plugin(plugin, None).await.unwrap();

        // Create a test context
        let context = PluginContext {
            working_dir: std::env::current_dir().unwrap(),
            config: HashMap::new(),
            security_policy: crate::tools::policy::SecurityPolicy::default(),
            resource_limits: ResourceLimits::default(),
            logger,
        };

        // Test enabling
        registry.enable_plugin("test_plugin", &context).await.unwrap();
        let info = registry.get_plugin_info("test_plugin").unwrap();
        assert!(info.config.enabled);
        assert_eq!(info.status, PluginStatus::Active);

        // Test disabling
        registry.disable_plugin("test_plugin").await.unwrap();
        let info = registry.get_plugin_info("test_plugin").unwrap();
        assert!(!info.config.enabled);
        assert_eq!(info.status, PluginStatus::Disabled);
    }
}
