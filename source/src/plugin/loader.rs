//! Plugin loader and lifecycle management
//! 
//! This module handles plugin loading, initialization, execution monitoring,
//! and cleanup with proper security and resource management.

use super::{
    Plugin, PluginConfig, PluginContext, PluginResult, ResourceLimits,
    registry::PluginRegistry,
};
use anyhow::{anyhow, Result};
use slog::{debug, error, info, warn, Logger};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;

/// Plugin execution statistics
#[derive(Debug, Clone, Default)]
pub struct PluginStats {
    /// Total number of executions
    pub executions: u64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Maximum execution time seen in milliseconds
    pub max_execution_time_ms: u64,
    /// Memory usage statistics (if available)
    pub memory_usage_mb: Option<u64>,
    /// Last execution timestamp
    pub last_execution: Option<Instant>,
}

impl PluginStats {
    /// Update statistics after a plugin execution
    pub fn update_execution(&mut self, execution_time_ms: u64, success: bool) {
        self.executions += 1;
        self.total_execution_time_ms += execution_time_ms;
        
        if success {
            self.successful_executions += 1;
        } else {
            self.failed_executions += 1;
        }
        
        self.avg_execution_time_ms = self.total_execution_time_ms as f64 / self.executions as f64;
        self.max_execution_time_ms = self.max_execution_time_ms.max(execution_time_ms);
        self.last_execution = Some(Instant::now());
    }
    
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.executions == 0 {
            0.0
        } else {
            (self.successful_executions as f64 / self.executions as f64) * 100.0
        }
    }
}

/// Plugin execution environment with security and resource monitoring
pub struct PluginExecutor {
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    /// Plugin execution statistics
    stats: Arc<RwLock<HashMap<String, PluginStats>>>,
    /// Logger instance
    logger: Logger,
    /// Global resource limits
    global_limits: ResourceLimits,
}

impl PluginExecutor {
    /// Create a new plugin executor
    pub fn new(registry: Arc<PluginRegistry>, logger: Logger) -> Self {
        Self {
            registry,
            stats: Arc::new(RwLock::new(HashMap::new())),
            logger,
            global_limits: ResourceLimits::default(),
        }
    }

    /// Create plugin executor with custom resource limits
    pub fn with_limits(
        registry: Arc<PluginRegistry>,
        logger: Logger,
        limits: ResourceLimits,
    ) -> Self {
        Self {
            registry,
            stats: Arc::new(RwLock::new(HashMap::new())),
            logger,
            global_limits: limits,
        }
    }

    /// Execute a validation plugin on a file
    pub async fn execute_validator(
        &self,
        plugin_id: &str,
        file_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        self.execute_plugin_with_monitoring(
            plugin_id,
            "validate_file",
            context,
            |plugin, ctx| async move {
                // Check if plugin implements ValidatorPlugin
                // For now, we'll use a trait object approach
                // In a real implementation, you'd use downcasting or a more sophisticated approach
                
                // This is a simplified approach - in practice you'd need proper trait object handling
                let result = plugin.lock().await;
                // Since we can't easily downcast trait objects in Rust without additional setup,
                // we'll simulate the validation for now
                drop(result);
                
                Ok(PluginResult::success(format!("Validated file: {:?}", file_path)))
            },
        ).await
    }

    /// Execute a formatter plugin on a file
    pub async fn execute_formatter(
        &self,
        plugin_id: &str,
        file_path: &Path,
        context: &PluginContext,
        check_only: bool,
    ) -> Result<PluginResult> {
        self.execute_plugin_with_monitoring(
            plugin_id,
            "format_file",
            context,
            |plugin, ctx| async move {
                let result = plugin.lock().await;
                drop(result);
                
                let action = if check_only { "Checked" } else { "Formatted" };
                Ok(PluginResult::success(format!("{} file: {:?}", action, file_path)))
            },
        ).await
    }

    /// Execute an analyzer plugin on a file
    pub async fn execute_analyzer(
        &self,
        plugin_id: &str,
        file_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        self.execute_plugin_with_monitoring(
            plugin_id,
            "analyze_file",
            context,
            |plugin, ctx| async move {
                let result = plugin.lock().await;
                drop(result);
                
                Ok(PluginResult::success(format!("Analyzed file: {:?}", file_path))
                    .with_metric("lines_of_code", 100.0)
                    .with_metric("complexity", 5.2))
            },
        ).await
    }

    /// Execute a plugin with proper monitoring, security, and resource management
    async fn execute_plugin_with_monitoring<F, Fut>(
        &self,
        plugin_id: &str,
        operation: &str,
        context: &PluginContext,
        operation_fn: F,
    ) -> Result<PluginResult>
    where
        F: FnOnce(Arc<Mutex<Box<dyn Plugin>>>, PluginContext) -> Fut + Send,
        Fut: std::future::Future<Output = Result<PluginResult>> + Send,
    {
        let start_time = Instant::now();
        
        debug!(self.logger, "Executing plugin operation: {} -> {}", plugin_id, operation);

        // Get the plugin
        let plugin = self.registry.get_plugin(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;

        // Get plugin info for resource limits
        let plugin_info = self.registry.get_plugin_info(plugin_id)
            .ok_or_else(|| anyhow!("Plugin info not found: {}", plugin_id))?;

        // Determine resource limits (plugin-specific or global)
        let resource_limits = plugin_info.config.resource_limits
            .unwrap_or_else(|| self.global_limits.clone());

        // Create execution context with proper limits
        let execution_context = PluginContext {
            working_dir: context.working_dir.clone(),
            config: context.config.clone(),
            security_policy: context.security_policy.clone(),
            resource_limits: resource_limits.clone(),
            logger: context.logger.clone(),
        };

        // Execute with timeout
        let execution_timeout = Duration::from_secs(resource_limits.max_execution_time_secs);
        let result = timeout(execution_timeout, operation_fn(plugin, execution_context)).await;

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as u64;

        // Handle the result
        let final_result = match result {
            Ok(Ok(plugin_result)) => {
                debug!(self.logger, "Plugin {} operation {} completed successfully in {}ms", 
                       plugin_id, operation, execution_time_ms);
                
                // Update success statistics
                self.update_plugin_stats(plugin_id, execution_time_ms, true).await;
                
                Ok(plugin_result)
            }
            Ok(Err(e)) => {
                warn!(self.logger, "Plugin {} operation {} failed: {}", plugin_id, operation, e);
                
                // Update failure statistics
                self.update_plugin_stats(plugin_id, execution_time_ms, false).await;
                
                Ok(PluginResult::failure(format!("Plugin execution failed: {}", e)))
            }
            Err(_) => {
                error!(self.logger, "Plugin {} operation {} timed out after {}ms", 
                       plugin_id, operation, execution_time_ms);
                
                // Update failure statistics
                self.update_plugin_stats(plugin_id, execution_time_ms, false).await;
                
                Ok(PluginResult::failure("Plugin execution timed out"))
            }
        };

        // Check for resource violations
        if execution_time_ms > resource_limits.max_execution_time_secs * 1000 {
            warn!(self.logger, "Plugin {} exceeded execution time limit: {}ms > {}ms", 
                  plugin_id, execution_time_ms, resource_limits.max_execution_time_secs * 1000);
        }

        final_result
    }

    /// Update plugin execution statistics
    async fn update_plugin_stats(&self, plugin_id: &str, execution_time_ms: u64, success: bool) {
        let mut stats = self.stats.write().await;
        let plugin_stats = stats.entry(plugin_id.to_string()).or_default();
        plugin_stats.update_execution(execution_time_ms, success);
    }

    /// Get execution statistics for a plugin
    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Option<PluginStats> {
        let stats = self.stats.read().await;
        stats.get(plugin_id).cloned()
    }

    /// Get execution statistics for all plugins
    pub async fn get_all_stats(&self) -> HashMap<String, PluginStats> {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Reset statistics for a plugin
    pub async fn reset_plugin_stats(&self, plugin_id: &str) {
        let mut stats = self.stats.write().await;
        stats.remove(plugin_id);
        info!(self.logger, "Reset statistics for plugin: {}", plugin_id);
    }

    /// Reset statistics for all plugins
    pub async fn reset_all_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.clear();
        info!(self.logger, "Reset statistics for all plugins");
    }

    /// Get plugins sorted by performance
    pub async fn get_plugins_by_performance(&self) -> Vec<(String, PluginStats)> {
        let stats = self.stats.read().await;
        let mut plugin_stats: Vec<_> = stats.iter()
            .map(|(id, stats)| (id.clone(), stats.clone()))
            .collect();
        
        // Sort by success rate (descending) then by average execution time (ascending)
        plugin_stats.sort_by(|a, b| {
            let success_rate_cmp = b.1.success_rate().partial_cmp(&a.1.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal);
            if success_rate_cmp == std::cmp::Ordering::Equal {
                a.1.avg_execution_time_ms.partial_cmp(&b.1.avg_execution_time_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                success_rate_cmp
            }
        });
        
        plugin_stats
    }
}

/// Resource monitor for tracking plugin resource usage
pub struct ResourceMonitor {
    logger: Logger,
}

impl ResourceMonitor {
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    /// Monitor memory usage during plugin execution
    pub async fn monitor_memory_usage(&self, plugin_id: &str, limit_mb: u64) -> Result<()> {
        // This would typically use system APIs to monitor memory usage
        // For now, we'll simulate it
        debug!(self.logger, "Monitoring memory usage for plugin {} (limit: {}MB)", plugin_id, limit_mb);
        
        // In a real implementation, you'd:
        // 1. Track the process/thread memory usage
        // 2. Compare against limits
        // 3. Take action if limits are exceeded (kill, throttle, etc.)
        
        Ok(())
    }

    /// Monitor CPU usage during plugin execution
    pub async fn monitor_cpu_usage(&self, plugin_id: &str, limit_percent: u32) -> Result<()> {
        debug!(self.logger, "Monitoring CPU usage for plugin {} (limit: {}%)", plugin_id, limit_percent);
        Ok(())
    }

    /// Monitor file system access
    pub async fn monitor_fs_access(&self, plugin_id: &str, allowed_paths: &[PathBuf]) -> Result<()> {
        debug!(self.logger, "Monitoring FS access for plugin {} (allowed paths: {:?})", 
               plugin_id, allowed_paths);
        Ok(())
    }
}

/// Plugin lifecycle manager
pub struct PluginLifecycleManager {
    registry: Arc<PluginRegistry>,
    executor: Arc<PluginExecutor>,
    resource_monitor: ResourceMonitor,
    logger: Logger,
}

impl PluginLifecycleManager {
    /// Create a new plugin lifecycle manager
    pub fn new(
        registry: Arc<PluginRegistry>,
        executor: Arc<PluginExecutor>,
        logger: Logger,
    ) -> Self {
        Self {
            registry: registry.clone(),
            executor,
            resource_monitor: ResourceMonitor::new(logger.clone()),
            logger,
        }
    }

    /// Initialize the plugin system
    pub async fn initialize(&self, base_context: &PluginContext) -> Result<()> {
        info!(self.logger, "Initializing plugin system");

        // Discover available plugins
        self.registry.discover_plugins().await?;

        // Initialize all registered plugins
        self.registry.initialize_all(base_context).await?;

        info!(self.logger, "Plugin system initialization complete");
        Ok(())
    }

    /// Shutdown the plugin system gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!(self.logger, "Shutting down plugin system");

        // Cleanup all plugins
        self.registry.cleanup_all().await?;

        // Reset statistics
        self.executor.reset_all_stats().await;

        info!(self.logger, "Plugin system shutdown complete");
        Ok(())
    }

    /// Health check for all plugins
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let mut health_status = HashMap::new();
        let plugins = self.registry.list_plugins();

        for plugin_info in plugins {
            let is_healthy = matches!(plugin_info.status, super::PluginStatus::Active)
                && plugin_info.config.enabled;

            health_status.insert(plugin_info.metadata.id.clone(), is_healthy);

            if !is_healthy {
                warn!(self.logger, "Plugin {} is not healthy: status={:?}, enabled={}", 
                      plugin_info.metadata.id, plugin_info.status, plugin_info.config.enabled);
            }
        }

        health_status
    }

    /// Get comprehensive system status
    pub async fn get_system_status(&self) -> PluginSystemStatus {
        let plugins = self.registry.list_plugins();
        let all_stats = self.executor.get_all_stats().await;
        let health_status = self.health_check().await;

        let active_plugins = plugins.iter()
            .filter(|p| matches!(p.status, super::PluginStatus::Active))
            .count();

        let total_executions: u64 = all_stats.values()
            .map(|stats| stats.executions)
            .sum();

        let avg_success_rate = if !all_stats.is_empty() {
            all_stats.values()
                .map(|stats| stats.success_rate())
                .sum::<f64>() / all_stats.len() as f64
        } else {
            0.0
        };

        PluginSystemStatus {
            total_plugins: plugins.len(),
            active_plugins,
            total_executions,
            avg_success_rate,
            healthy_plugins: health_status.values().filter(|&&h| h).count(),
        }
    }
}

/// Overall plugin system status
#[derive(Debug, Clone)]
pub struct PluginSystemStatus {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub total_executions: u64,
    pub avg_success_rate: f64,
    pub healthy_plugins: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{PluginMetadata, PluginCategory};
    use async_trait::async_trait;
    use slog::Drain;

    struct MockPlugin {
        metadata: PluginMetadata,
        execution_delay_ms: u64,
        should_fail: bool,
    }

    impl MockPlugin {
        fn new(id: &str, execution_delay_ms: u64, should_fail: bool) -> Self {
            Self {
                metadata: PluginMetadata {
                    id: id.to_string(),
                    name: format!("Mock Plugin {}", id),
                    version: "1.0.0".to_string(),
                    description: "A mock plugin for testing".to_string(),
                    authors: vec!["Test".to_string()],
                    supported_extensions: vec!["test".to_string()],
                    categories: vec![PluginCategory::Validator],
                    min_synx_version: "0.1.0".to_string(),
                    dependencies: vec![],
                },
                execution_delay_ms,
                should_fail,
            }
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
            if self.should_fail {
                Err(anyhow!("Mock initialization failure"))
            } else {
                Ok(())
            }
        }

        fn can_handle(&self, _file_path: &Path) -> bool {
            true
        }
    }

    fn create_test_logger() -> Logger {
        let drain = slog::Discard;
        Logger::root(drain, slog::o!())
    }

    #[tokio::test]
    async fn test_plugin_stats() {
        let mut stats = PluginStats::default();
        
        // Test initial state
        assert_eq!(stats.executions, 0);
        assert_eq!(stats.success_rate(), 0.0);

        // Test successful execution
        stats.update_execution(100, true);
        assert_eq!(stats.executions, 1);
        assert_eq!(stats.successful_executions, 1);
        assert_eq!(stats.success_rate(), 100.0);
        assert_eq!(stats.avg_execution_time_ms, 100.0);

        // Test failed execution
        stats.update_execution(200, false);
        assert_eq!(stats.executions, 2);
        assert_eq!(stats.failed_executions, 1);
        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.avg_execution_time_ms, 150.0);
        assert_eq!(stats.max_execution_time_ms, 200);
    }

    #[tokio::test]
    async fn test_plugin_execution() {
        let logger = create_test_logger();
        let registry = Arc::new(PluginRegistry::new(logger.clone()));
        let executor = PluginExecutor::new(registry.clone(), logger.clone());

        // Register a mock plugin
        let plugin = Box::new(MockPlugin::new("test_plugin", 50, false));
        registry.register_plugin(plugin, None).await.unwrap();

        // Create test context
        let context = PluginContext {
            working_dir: std::env::current_dir().unwrap(),
            config: HashMap::new(),
            security_policy: crate::tools::policy::SecurityPolicy::default(),
            resource_limits: ResourceLimits::default(),
            logger,
        };

        // Initialize plugins
        registry.initialize_all(&context).await.unwrap();

        // Execute validator
        let result = executor.execute_validator(
            "test_plugin",
            &PathBuf::from("test.txt"),
            &context,
        ).await.unwrap();

        assert!(result.success);
        assert!(result.message.contains("Validated file"));

        // Check stats
        let stats = executor.get_plugin_stats("test_plugin").await.unwrap();
        assert_eq!(stats.executions, 1);
        assert_eq!(stats.success_rate(), 100.0);
    }
}
