//! Plugin integration with existing validation system
//! 
//! This module provides integration between the plugin system and the
//! existing Validator struct and validation infrastructure.

use super::{
    PluginRegistry, PluginExecutor, PluginLifecycleManager, PluginContext,
    PluginResult, PluginCategory, BoxedPlugin,
    examples::{PythonValidatorPlugin, JsonFormatterPlugin, BasicAnalyzerPlugin},
};
use crate::{ValidationConfig, Result};
use anyhow::anyhow;
use slog::{info, warn, Logger};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Enhanced validator that uses the plugin system
pub struct PluginValidator {
    /// Original validation config
    config: ValidationConfig,
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    /// Plugin executor
    executor: Arc<PluginExecutor>,
    /// Plugin lifecycle manager
    lifecycle_manager: PluginLifecycleManager,
    /// Logger instance
    logger: Logger,
    /// Base plugin context
    base_context: PluginContext,
}

impl PluginValidator {
    /// Create a new plugin-based validator
    pub async fn new(config: ValidationConfig, logger: Logger) -> Result<Self> {
        // Create plugin registry
        let registry = Arc::new(PluginRegistry::new(logger.clone()));

        // Create plugin executor
        let executor = Arc::new(PluginExecutor::new(registry.clone(), logger.clone()));

        // Create lifecycle manager
        let lifecycle_manager = PluginLifecycleManager::new(
            registry.clone(),
            executor.clone(),
            logger.clone(),
        );

        // Create base plugin context
        let base_context = PluginContext {
            working_dir: std::env::current_dir()?,
            config: HashMap::new(),
            security_policy: crate::tools::policy::SecurityPolicy::default(),
            resource_limits: super::ResourceLimits::default(),
            logger: logger.clone(),
        };

        let mut validator = Self {
            config,
            registry,
            executor,
            lifecycle_manager,
            logger,
            base_context,
        };

        // Register built-in plugins
        validator.register_builtin_plugins().await?;

        // Initialize the plugin system
        validator.lifecycle_manager.initialize(&validator.base_context).await?;

        info!(validator.logger, "Plugin-based validator initialized successfully");
        Ok(validator)
    }

    /// Register built-in example plugins
    async fn register_builtin_plugins(&self) -> Result<()> {
        info!(self.logger, "Registering built-in plugins");

        // Register Python validator plugin
        let python_plugin = Box::new(PythonValidatorPlugin::new());
        self.registry.register_plugin(python_plugin, None).await?;

        // Register JSON formatter plugin
        let json_plugin = Box::new(JsonFormatterPlugin::new());
        self.registry.register_plugin(json_plugin, None).await?;

        // Register basic analyzer plugin
        let analyzer_plugin = Box::new(BasicAnalyzerPlugin::new());
        self.registry.register_plugin(analyzer_plugin, None).await?;

        info!(self.logger, "Built-in plugins registered successfully");
        Ok(())
    }

    /// Validate a file using appropriate plugins
    pub async fn validate_file(&self, path: &Path) -> Result<ValidationSummary> {
        info!(self.logger, "Validating file with plugins: {:?}", path);

        let mut summary = ValidationSummary::new(path.to_path_buf());

        // Find plugins that can handle this file
        let validator_plugins = self.registry
            .get_plugins_by_category(&PluginCategory::Validator);

        let compatible_plugins = self.registry
            .get_plugins_for_file(path).await;

        let applicable_validators: Vec<_> = validator_plugins
            .into_iter()
            .filter(|id| compatible_plugins.contains(id))
            .collect();

        if applicable_validators.is_empty() {
            warn!(self.logger, "No validator plugins available for file: {:?}", path);
            summary.add_warning("No validator plugins available for this file type".to_string());
            return Ok(summary);
        }

        // Run validation with each applicable plugin
        for plugin_id in applicable_validators {
            info!(self.logger, "Running validator plugin: {}", plugin_id);

            match self.executor.execute_validator(&plugin_id, path, &self.base_context).await {
                Ok(result) => {
                    summary.add_plugin_result(plugin_id.clone(), result);
                }
                Err(e) => {
                    warn!(self.logger, "Plugin {} failed: {}", plugin_id, e);
                    summary.add_error(format!("Plugin '{}' failed: {}", plugin_id, e));
                }
            }
        }

        // Determine overall success
        summary.success = summary.plugin_results.values().all(|r| r.success) && summary.errors.is_empty();

        Ok(summary)
    }

    /// Format a file using appropriate plugins
    pub async fn format_file(&self, path: &Path, check_only: bool) -> Result<ValidationSummary> {
        info!(self.logger, "Formatting file with plugins: {:?} (check_only: {})", path, check_only);

        let mut summary = ValidationSummary::new(path.to_path_buf());

        // Find formatter plugins that can handle this file
        let formatter_plugins = self.registry
            .get_plugins_by_category(&PluginCategory::Formatter);

        let compatible_plugins = self.registry
            .get_plugins_for_file(path).await;

        let applicable_formatters: Vec<_> = formatter_plugins
            .into_iter()
            .filter(|id| compatible_plugins.contains(id))
            .collect();

        if applicable_formatters.is_empty() {
            warn!(self.logger, "No formatter plugins available for file: {:?}", path);
            summary.add_warning("No formatter plugins available for this file type".to_string());
            return Ok(summary);
        }

        // Run formatting with each applicable plugin
        for plugin_id in applicable_formatters {
            info!(self.logger, "Running formatter plugin: {}", plugin_id);

            match self.executor.execute_formatter(&plugin_id, path, &self.base_context, check_only).await {
                Ok(result) => {
                    summary.add_plugin_result(plugin_id.clone(), result);
                }
                Err(e) => {
                    warn!(self.logger, "Plugin {} failed: {}", plugin_id, e);
                    summary.add_error(format!("Plugin '{}' failed: {}", plugin_id, e));
                }
            }
        }

        // Determine overall success
        summary.success = summary.plugin_results.values().all(|r| r.success) && summary.errors.is_empty();

        Ok(summary)
    }

    /// Analyze a file using appropriate plugins
    pub async fn analyze_file(&self, path: &Path) -> Result<ValidationSummary> {
        info!(self.logger, "Analyzing file with plugins: {:?}", path);

        let mut summary = ValidationSummary::new(path.to_path_buf());

        // Find analyzer plugins that can handle this file
        let analyzer_plugins = self.registry
            .get_plugins_by_category(&PluginCategory::Analyzer);

        let compatible_plugins = self.registry
            .get_plugins_for_file(path).await;

        let applicable_analyzers: Vec<_> = analyzer_plugins
            .into_iter()
            .filter(|id| compatible_plugins.contains(id))
            .collect();

        if applicable_analyzers.is_empty() {
            warn!(self.logger, "No analyzer plugins available for file: {:?}", path);
            summary.add_warning("No analyzer plugins available for this file type".to_string());
            return Ok(summary);
        }

        // Run analysis with each applicable plugin
        for plugin_id in applicable_analyzers {
            info!(self.logger, "Running analyzer plugin: {}", plugin_id);

            match self.executor.execute_analyzer(&plugin_id, path, &self.base_context).await {
                Ok(result) => {
                    summary.add_plugin_result(plugin_id.clone(), result);
                }
                Err(e) => {
                    warn!(self.logger, "Plugin {} failed: {}", plugin_id, e);
                    summary.add_error(format!("Plugin '{}' failed: {}", plugin_id, e));
                }
            }
        }

        // Always successful for analysis (unless no plugins available)
        summary.success = !summary.plugin_results.is_empty() || !summary.warnings.is_empty();

        Ok(summary)
    }

    /// Get system status
    pub async fn get_system_status(&self) -> super::loader::PluginSystemStatus {
        self.lifecycle_manager.get_system_status().await
    }

    /// Get plugin statistics
    pub async fn get_plugin_stats(&self) -> HashMap<String, super::loader::PluginStats> {
        self.executor.get_all_stats().await
    }

    /// List all available plugins
    pub fn list_plugins(&self) -> Vec<super::PluginInfo> {
        self.registry.list_plugins()
    }

    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        self.registry.enable_plugin(plugin_id, &self.base_context).await
    }

    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        self.registry.disable_plugin(plugin_id).await
    }

    /// Register a custom plugin
    pub async fn register_plugin(&self, plugin: BoxedPlugin) -> Result<()> {
        self.registry.register_plugin(plugin, None).await
    }

    /// Shutdown the plugin system
    pub async fn shutdown(self) -> Result<()> {
        self.lifecycle_manager.shutdown().await
    }
}

/// Summary of validation/formatting/analysis results
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    /// File path that was processed
    pub file_path: std::path::PathBuf,
    /// Whether the overall operation was successful
    pub success: bool,
    /// Results from individual plugins
    pub plugin_results: HashMap<String, PluginResult>,
    /// Overall warnings
    pub warnings: Vec<String>,
    /// Overall errors
    pub errors: Vec<String>,
    /// Combined metrics from all plugins
    pub metrics: HashMap<String, f64>,
}

impl ValidationSummary {
    /// Create a new validation summary
    pub fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            success: true,
            plugin_results: HashMap::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            metrics: HashMap::new(),
        }
    }

    /// Add a plugin result
    pub fn add_plugin_result(&mut self, plugin_id: String, result: PluginResult) {
        // Merge metrics
        for (key, value) in &result.metrics {
            self.metrics.insert(format!("{}_{}", plugin_id, key), *value);
        }

        // Merge warnings and errors
        self.warnings.extend(result.warnings.iter().map(|w| format!("{}: {}", plugin_id, w)));
        self.errors.extend(result.errors.iter().map(|e| format!("{}: {}", plugin_id, e)));

        self.plugin_results.insert(plugin_id, result);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.success = false;
    }

    /// Get a summary message
    pub fn summary_message(&self) -> String {
        let plugin_count = self.plugin_results.len();
        let successful_plugins = self.plugin_results.values().filter(|r| r.success).count();
        let failed_plugins = plugin_count - successful_plugins;

        if self.success {
            if failed_plugins > 0 {
                format!(
                    "Completed with warnings: {}/{} plugins successful",
                    successful_plugins, plugin_count
                )
            } else {
                format!("Completed successfully: {}/{} plugins passed", successful_plugins, plugin_count)
            }
        } else {
            format!(
                "Failed: {}/{} plugins successful, {} errors",
                successful_plugins, plugin_count, self.errors.len()
            )
        }
    }

    /// Check if any plugin reported the file needs formatting
    pub fn needs_formatting(&self) -> bool {
        self.plugin_results.values().any(|result| {
            result.metrics.get("needs_formatting").map(|v| *v > 0.0).unwrap_or(false)
        })
    }
}

/// Factory function to create a plugin validator
pub async fn create_plugin_validator(
    config: ValidationConfig,
    logger: Logger,
) -> Result<PluginValidator> {
    PluginValidator::new(config, logger).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use slog::Drain;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_logger() -> Logger {
        let drain = slog::Discard;
        Logger::root(drain, slog::o!())
    }

    #[tokio::test]
    async fn test_plugin_validator_creation() {
        let config = ValidationConfig::default();
        let logger = create_test_logger();

        let validator = PluginValidator::new(config, logger).await;
        assert!(validator.is_ok());

        let validator = validator.unwrap();
        let plugins = validator.list_plugins();
        assert!(!plugins.is_empty());

        // Clean shutdown
        validator.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_file_analysis() {
        let config = ValidationConfig::default();
        let logger = create_test_logger();
        let validator = PluginValidator::new(config, logger).await.unwrap();

        // Create a test file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!\nThis is a test file.\n\n").unwrap();

        // Analyze the file
        let summary = validator.analyze_file(&test_file).await.unwrap();
        assert!(summary.success);
        assert!(!summary.plugin_results.is_empty());

        // Check that basic analyzer was run
        assert!(summary.plugin_results.contains_key("basic_analyzer"));

        // Check metrics
        assert!(summary.metrics.contains_key("basic_analyzer_line_count"));
        assert!(summary.metrics.contains_key("basic_analyzer_character_count"));

        validator.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_json_formatting() {
        let config = ValidationConfig::default();
        let logger = create_test_logger();
        let validator = PluginValidator::new(config, logger).await.unwrap();

        // Create a JSON test file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, r#"{"b": 2, "a": 1}"#).unwrap();

        // Format check the file
        let summary = validator.format_file(&test_file, true).await.unwrap();
        assert!(!summary.plugin_results.is_empty());

        // Check that JSON formatter was run
        assert!(summary.plugin_results.contains_key("json_formatter"));

        validator.shutdown().await.unwrap();
    }

    #[test]
    fn test_validation_summary() {
        let mut summary = ValidationSummary::new("/test/file.txt".into());

        // Add a successful plugin result
        let result = PluginResult::success("Test passed")
            .with_metric("lines", 10.0);
        summary.add_plugin_result("test_plugin".to_string(), result);

        assert!(summary.success);
        assert_eq!(summary.plugin_results.len(), 1);
        assert_eq!(summary.metrics.get("test_plugin_lines"), Some(&10.0));

        // Add an error
        summary.add_error("Test error".to_string());
        assert!(!summary.success);
        assert_eq!(summary.errors.len(), 1);
    }
}
