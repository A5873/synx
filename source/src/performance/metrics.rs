//! Performance metrics and monitoring
//!
//! This module provides comprehensive performance monitoring and metrics collection.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: usize,
    pub disk_io_mb: f64,
    pub network_kb: f64,
}

/// Validation performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationMetrics {
    pub total_files: usize,
    pub total_validation_time_ms: u64,
    pub average_validation_time_ms: f64,
    pub files_per_second: f64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub by_file_type: HashMap<String, FileTypeMetrics>,
}

/// Metrics for specific file types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileTypeMetrics {
    pub count: usize,
    pub total_time_ms: u64,
    pub average_time_ms: f64,
    pub success_rate: f64,
}

/// Performance monitor for tracking validation metrics
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<ValidationMetrics>>,
    resource_usage: Arc<RwLock<ResourceUsage>>,
    start_time: Instant,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ValidationMetrics::default())),
            resource_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            start_time: Instant::now(),
        }
    }
    
    /// Record a validation event
    pub fn record_validation(&self, file_type: &str, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write().unwrap();
        
        metrics.total_files += 1;
        metrics.total_validation_time_ms += duration.as_millis() as u64;
        
        let file_metrics = metrics.by_file_type.entry(file_type.to_string()).or_default();
        file_metrics.count += 1;
        file_metrics.total_time_ms += duration.as_millis() as u64;
        
        if success {
            file_metrics.success_rate = 
                (file_metrics.success_rate * (file_metrics.count - 1) as f64 + 1.0) 
                / file_metrics.count as f64;
        } else {
            file_metrics.success_rate = 
                (file_metrics.success_rate * (file_metrics.count - 1) as f64) 
                / file_metrics.count as f64;
        }
        
        file_metrics.average_time_ms = file_metrics.total_time_ms as f64 / file_metrics.count as f64;
        
        // Update overall metrics
        if metrics.total_files > 0 {
            metrics.average_validation_time_ms = 
                metrics.total_validation_time_ms as f64 / metrics.total_files as f64;
            
            let elapsed_seconds = self.start_time.elapsed().as_secs_f64();
            if elapsed_seconds > 0.0 {
                metrics.files_per_second = metrics.total_files as f64 / elapsed_seconds;
            }
        }
    }
    
    /// Update resource usage metrics
    pub fn update_resource_usage(&self, usage: ResourceUsage) {
        let mut resource_usage = self.resource_usage.write().unwrap();
        *resource_usage = usage;
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> ValidationMetrics {
        self.metrics.read().unwrap().clone()
    }
    
    /// Get current resource usage
    pub fn get_resource_usage(&self) -> ResourceUsage {
        self.resource_usage.read().unwrap().clone()
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = ValidationMetrics::default();
        
        let mut resource_usage = self.resource_usage.write().unwrap();
        *resource_usage = ResourceUsage::default();
        
        self.start_time = Instant::now();
    }
    
    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        let metrics = self.get_metrics();
        let resources = self.get_resource_usage();
        
        format!(
            "Performance Report\n\
             =================\n\
             Total Files Processed: {}\n\
             Average Validation Time: {:.2}ms\n\
             Files per Second: {:.2}\n\
             Cache Hit Rate: {:.1}%\n\
             Error Rate: {:.1}%\n\
             \n\
             Resource Usage:\n\
             - CPU: {:.1}%\n\
             - Memory: {}MB\n\
             - Disk I/O: {:.1}MB\n\
             \n\
             By File Type:\n{}",
            metrics.total_files,
            metrics.average_validation_time_ms,
            metrics.files_per_second,
            metrics.cache_hit_rate * 100.0,
            metrics.error_rate * 100.0,
            resources.cpu_percent,
            resources.memory_mb,
            resources.disk_io_mb,
            self.format_file_type_metrics(&metrics.by_file_type)
        )
    }
    
    fn format_file_type_metrics(&self, metrics: &HashMap<String, FileTypeMetrics>) -> String {
        metrics.iter()
            .map(|(file_type, metrics)| {
                format!(
                    "- {}: {} files, {:.2}ms avg, {:.1}% success",
                    file_type,
                    metrics.count,
                    metrics.average_time_ms,
                    metrics.success_rate * 100.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_files, 0);
    }
    
    #[test]
    fn test_record_validation() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_validation("rust", Duration::from_millis(100), true);
        monitor.record_validation("python", Duration::from_millis(200), false);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_files, 2);
        assert_eq!(metrics.by_file_type.len(), 2);
        
        let rust_metrics = &metrics.by_file_type["rust"];
        assert_eq!(rust_metrics.count, 1);
        assert_eq!(rust_metrics.average_time_ms, 100.0);
        assert_eq!(rust_metrics.success_rate, 1.0);
    }
    
    #[test]
    fn test_resource_usage_update() {
        let monitor = PerformanceMonitor::new();
        
        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_mb: 256,
            disk_io_mb: 10.0,
            network_kb: 0.0,
        };
        
        monitor.update_resource_usage(usage.clone());
        let retrieved_usage = monitor.get_resource_usage();
        
        assert_eq!(retrieved_usage.cpu_percent, 50.0);
        assert_eq!(retrieved_usage.memory_mb, 256);
    }
    
    #[test]
    fn test_performance_report() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_validation("rust", Duration::from_millis(100), true);
        monitor.record_validation("python", Duration::from_millis(200), true);
        
        let report = monitor.generate_report();
        assert!(report.contains("Performance Report"));
        assert!(report.contains("Total Files Processed: 2"));
    }
}
