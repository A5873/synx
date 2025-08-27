//! Performance optimizations for Synx validation
//! 
//! This module provides high-performance validation capabilities including:
//! - Advanced caching with TTL and size limits
//! - Intelligent parallel processing with adaptive work distribution
//! - Memory management and resource optimization
//! - Performance metrics and monitoring

use anyhow::Result;
use serde::{Serialize, Deserialize};
use rayon::ThreadPoolBuilder;

pub mod cache;
pub mod parallel;
pub mod metrics;

pub use cache::{ValidationCache, CacheConfig, CacheEntry, CacheStats};
pub use parallel::{ParallelValidator, WorkloadDistributor, ValidationJob};
pub use metrics::{PerformanceMonitor, ValidationMetrics, ResourceUsage};

/// Performance configuration for validation operations
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Number of parallel worker threads (0 = auto-detect)
    pub worker_threads: usize,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    
    /// Cache configuration
    pub cache: CacheConfig,
    
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    
    /// Batch size for parallel processing
    pub batch_size: usize,
    
    /// Enable adaptive load balancing
    pub adaptive_balancing: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: 0, // Auto-detect
            max_memory_mb: 512,
            cache: CacheConfig::default(),
            enable_monitoring: true,
            batch_size: 32,
            adaptive_balancing: true,
        }
    }
}

/// High-performance validation engine
#[allow(dead_code)]
pub struct PerformanceEngine {
    config: PerformanceConfig,
    cache: ValidationCache,
    monitor: Option<PerformanceMonitor>,
    thread_pool: Option<rayon::ThreadPool>,
}

impl PerformanceEngine {
    /// Create a new performance engine with the given configuration
    pub fn new(config: PerformanceConfig) -> Result<Self> {
        let cache = ValidationCache::new(config.cache.clone())?;
        
        let monitor = if config.enable_monitoring {
            Some(PerformanceMonitor::new())
        } else {
            None
        };
        
        // Create custom thread pool if specific thread count is requested
        let thread_pool = if config.worker_threads > 0 {
            Some(ThreadPoolBuilder::new()
                .num_threads(config.worker_threads)
                .build()?)
        } else {
            None
        };
        
        Ok(Self {
            config,
            cache,
            monitor,
            thread_pool,
        })
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> Result<PerformanceStats> {
        Ok(PerformanceStats {
            cache_stats: self.cache.get_stats(),
            validation_metrics: self.monitor.as_ref()
                .map(|m| m.get_metrics())
                .unwrap_or_default(),
            thread_pool_size: self.get_thread_count(),
            memory_usage: self.get_memory_usage(),
        })
    }
    
    /// Clear all caches and reset statistics
    pub fn reset(&mut self) -> Result<()> {
        self.cache.clear()?;
        if let Some(monitor) = &mut self.monitor {
            monitor.reset();
        }
        Ok(())
    }
    
    /// Get the number of worker threads being used
    pub fn get_thread_count(&self) -> usize {
        self.thread_pool.as_ref()
            .map(|pool| pool.current_num_threads())
            .unwrap_or_else(|| rayon::current_num_threads())
    }
    
    /// Get current memory usage in MB
    pub fn get_memory_usage(&self) -> usize {
        // Basic memory usage estimation
        // In a real implementation, this would use platform-specific APIs
        self.cache.estimated_memory_usage()
    }
    
    /// Optimize cache based on usage patterns
    pub fn optimize_cache(&mut self) -> Result<()> {
        self.cache.optimize()
    }
}

/// Comprehensive performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub cache_stats: cache::CacheStats,
    pub validation_metrics: metrics::ValidationMetrics,
    pub thread_pool_size: usize,
    pub memory_usage: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_performance_engine_creation() {
        let config = PerformanceConfig::default();
        let engine = PerformanceEngine::new(config);
        assert!(engine.is_ok());
    }
    
    #[test]
    fn test_performance_stats() {
        let config = PerformanceConfig::default();
        let engine = PerformanceEngine::new(config).unwrap();
        let stats = engine.get_stats();
        assert!(stats.is_ok());
    }
    
    #[test]
    fn test_thread_count() {
        let mut config = PerformanceConfig::default();
        config.worker_threads = 4;
        let engine = PerformanceEngine::new(config).unwrap();
        assert_eq!(engine.get_thread_count(), 4);
    }
}
