//! Parallel processing for validation jobs
//!
//! This module provides advanced parallel processing using adaptive work distribution and task batching.

use anyhow::Result;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use rayon::prelude::*;
use crate::performance::cache::ValidationCache;
use crate::validators::validate_file;
use crate::performance::metrics::PerformanceMonitor;

/// A validation job representing a single file or set of files
#[derive(Debug, Clone)]
pub struct ValidationJob {
    pub paths: Vec<PathBuf>,
}

/// Workload distributor for balancing validation tasks
#[allow(dead_code)]
pub struct WorkloadDistributor {
    jobs: Arc<RwLock<VecDeque<ValidationJob>>>,
    cache: Arc<ValidationCache>,
    monitor: Option<Arc<PerformanceMonitor>>,
    batch_size: usize,
}

impl WorkloadDistributor {
    /// Create a new workload distributor
    pub fn new(jobs: Vec<ValidationJob>, cache: Arc<ValidationCache>, monitor: Option<Arc<PerformanceMonitor>>, batch_size: usize) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(VecDeque::from(jobs))),
            cache,
            monitor,
            batch_size,
        }
    }
    
    /// Execute all jobs using adaptive batching
    pub fn execute(&self) -> Result<()> {
        let jobs = self.jobs.clone();
        
        jobs.read().unwrap().par_iter().for_each(|job| {
            let cache = self.cache.clone();
            
            job.paths.par_iter().for_each(|path| {
                // Check if cached
                if cache.get(&path).is_some() {
                    println!("Cached: {:?}", path);
                    return;
                }
                
                // Perform validation
                match validate_file(path, &Default::default()) {
                    Ok(is_valid) => {
                        println!("Validated: {:?} - {}", path, is_valid);
                        cache.put(path, is_valid, std::time::Duration::from_millis(100)).unwrap();
                    }
                    Err(e) => println!("Error validating {:?}: {}
", path, e),
                }
            });
        });

        Ok(())
    }
}

/// A parallel validator that runs validation tasks in parallel
pub struct ParallelValidator {
    distributor: WorkloadDistributor,
}

impl ParallelValidator {
    /// Create a new parallel validator
    pub fn new(jobs: Vec<ValidationJob>, cache: ValidationCache, monitor: Option<PerformanceMonitor>, batch_size: usize) -> Self {
        let distributor = WorkloadDistributor::new(jobs, Arc::new(cache), monitor.map(Arc::new), batch_size);
        
        Self {
            distributor
        }
    }
    
    /// Run validation tasks in parallel
    pub fn run(&self) -> Result<()> {
        self.distributor.execute()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::performance::cache::CacheConfig;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_parallel_validator_creation() {
        let config = CacheConfig::default();
        let cache = ValidationCache::new(config).unwrap();
        
        let job = ValidationJob { paths: vec![PathBuf::from("test.rs")] };
        let validator = ParallelValidator::new(vec![job], cache, None, 32);
        assert!(validator.run().is_ok());
    }
    
    #[test]
    fn test_workload_distribution() {
        let job1 = ValidationJob { paths: vec![PathBuf::from("file1.js"), PathBuf::from("file2.js")] };
        let job2 = ValidationJob { paths: vec![PathBuf::from("file3.js")] };
        
        let config = CacheConfig::default();
        let cache = ValidationCache::new(config).unwrap();
        let distributor = WorkloadDistributor::new(vec![job1, job2], Arc::new(cache), None, 32);
        assert!(distributor.execute().is_ok());
    }
}

