//! Plugin system for synx
//! 
//! This module provides a flexible plugin architecture that allows extending
//! synx with custom validators, formatters, analyzers, and reporters.

pub mod registry;
pub mod loader;
pub mod examples;
pub mod integration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Re-export main types for easier access
pub use registry::{PluginRegistry, BoxedPlugin};
pub use loader::{PluginExecutor, PluginStats, PluginLifecycleManager, PluginSystemStatus};

/// Plugin metadata containing information about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author(s)
    pub authors: Vec<String>,
    /// Supported file extensions or patterns
    pub supported_extensions: Vec<String>,
    /// Plugin categories/tags
    pub categories: Vec<PluginCategory>,
    /// Minimum synx version required
    pub min_synx_version: String,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
}

/// Categories that plugins can belong to
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCategory {
    Validator,
    Formatter,
    Analyzer,
    Reporter,
    Security,
    Performance,
    Linter,
    Documentation,
    Integration,
    Custom(String),
}

/// Plugin execution context provided to plugins
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Current working directory
    pub working_dir: PathBuf,
    /// Plugin configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Security restrictions
    pub security_policy: crate::tools::policy::SecurityPolicy,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Logger instance
    pub logger: slog::Logger,
}

/// Resource limits for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum execution time in seconds
    pub max_execution_time_secs: u64,
    /// Maximum number of files that can be processed
    pub max_files: usize,
    /// Maximum file size that can be processed in MB
    pub max_file_size_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 256,
            max_execution_time_secs: 30,
            max_files: 1000,
            max_file_size_mb: 50,
        }
    }
}

/// Result of a plugin operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Output message
    pub message: String,
    /// Structured data output
    pub data: Option<serde_json::Value>,
    /// Metrics collected during execution
    pub metrics: HashMap<String, f64>,
    /// Warnings generated during execution
    pub warnings: Vec<String>,
    /// Errors encountered during execution
    pub errors: Vec<String>,
}

impl PluginResult {
    /// Create a successful result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            metrics: HashMap::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create a failure result
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            metrics: HashMap::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add structured data to the result
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Add a metric to the result
    pub fn with_metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    /// Add a warning to the result
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add an error to the result
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self
    }
}

/// Base trait that all plugins must implement
/// Note: We use synchronous methods to make this dyn-compatible
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin with the given context
    /// Returns a boxed future to maintain dyn compatibility
    fn initialize(&mut self, context: &PluginContext) -> Result<()>;

    /// Check if the plugin can handle the given file
    fn can_handle(&self, file_path: &Path) -> bool;

    /// Cleanup resources when the plugin is unloaded
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get plugin-specific configuration schema
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Validate plugin configuration
    fn validate_config(&self, config: &HashMap<String, serde_json::Value>) -> Result<()> {
        let _ = config;
        Ok(())
    }
}

/// Plugin trait for file validation
#[async_trait]
pub trait ValidatorPlugin: Plugin {
    /// Validate a file
    async fn validate_file(
        &self,
        file_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult>;

    /// Validate multiple files in batch
    async fn validate_files(
        &self,
        file_paths: &[PathBuf],
        context: &PluginContext,
    ) -> Result<Vec<PluginResult>> {
        let mut results = Vec::new();
        for path in file_paths {
            results.push(self.validate_file(path, context).await?);
        }
        Ok(results)
    }

    /// Get validation rules supported by this plugin
    fn supported_rules(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Plugin trait for file formatting
#[async_trait]
pub trait FormatterPlugin: Plugin {
    /// Format a file
    async fn format_file(
        &self,
        file_path: &Path,
        context: &PluginContext,
        check_only: bool,
    ) -> Result<PluginResult>;

    /// Format multiple files in batch
    async fn format_files(
        &self,
        file_paths: &[PathBuf],
        context: &PluginContext,
        check_only: bool,
    ) -> Result<Vec<PluginResult>> {
        let mut results = Vec::new();
        for path in file_paths {
            results.push(self.format_file(path, context, check_only).await?);
        }
        Ok(results)
    }

    /// Get formatting options supported by this plugin
    fn supported_options(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Plugin trait for code analysis
#[async_trait]
pub trait AnalyzerPlugin: Plugin {
    /// Analyze a file
    async fn analyze_file(
        &self,
        file_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult>;

    /// Analyze a project directory
    async fn analyze_project(
        &self,
        project_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult>;

    /// Get analysis metrics that this plugin provides
    fn supported_metrics(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Plugin trait for generating reports
#[async_trait]
pub trait ReporterPlugin: Plugin {
    /// Generate a report from analysis results
    async fn generate_report(
        &self,
        results: &[PluginResult],
        context: &PluginContext,
        output_format: &str,
    ) -> Result<PluginResult>;

    /// Get supported output formats
    fn supported_formats(&self) -> Vec<String> {
        vec!["text".to_string(), "json".to_string()]
    }

    /// Get report template if applicable
    fn report_template(&self, format: &str) -> Option<String> {
        let _ = format;
        None
    }
}

/// Plugin status tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    /// Plugin is loaded and ready to use
    Active,
    /// Plugin is loaded but disabled
    Disabled,
    /// Plugin failed to load
    Failed(String),
    /// Plugin is not loaded
    Unloaded,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin-specific configuration
    pub settings: HashMap<String, serde_json::Value>,
    /// Priority for plugin execution (higher = earlier)
    pub priority: i32,
    /// Resource limits for this plugin
    pub resource_limits: Option<ResourceLimits>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: HashMap::new(),
            priority: 0,
            resource_limits: None,
        }
    }
}

/// Plugin information combining metadata, status, and config
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub metadata: PluginMetadata,
    pub status: PluginStatus,
    pub config: PluginConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_result_creation() {
        let success = PluginResult::success("Test passed");
        assert!(success.success);
        assert_eq!(success.message, "Test passed");

        let failure = PluginResult::failure("Test failed");
        assert!(!failure.success);
        assert_eq!(failure.message, "Test failed");
    }

    #[test]
    fn test_plugin_result_chaining() {
        let result = PluginResult::success("Test")
            .with_metric("lines", 100.0)
            .with_warning("Minor issue")
            .with_data(serde_json::json!({"key": "value"}));

        assert!(result.success);
        assert_eq!(result.metrics.get("lines"), Some(&100.0));
        assert_eq!(result.warnings.len(), 1);
        assert!(result.data.is_some());
    }

    #[test]
    fn test_plugin_category_serialization() {
        let category = PluginCategory::Validator;
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: PluginCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_mb, 256);
        assert_eq!(limits.max_execution_time_secs, 30);
        assert_eq!(limits.max_files, 1000);
        assert_eq!(limits.max_file_size_mb, 50);
    }
}
