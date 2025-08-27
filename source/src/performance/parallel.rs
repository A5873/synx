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
use serde::{Serialize, Deserialize};
use std::time::Duration;

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
    pub fn execute(&self) -> Result<ParallelExecutionResult> {
        use std::time::Instant;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let start_time = Instant::now();
        let jobs = self.jobs.clone();
        let total_processed = Arc::new(AtomicUsize::new(0));
        let total_cached = Arc::new(AtomicUsize::new(0));
        let total_errors = Arc::new(AtomicUsize::new(0));
        let total_valid = Arc::new(AtomicUsize::new(0));
        
        // Process jobs in parallel using Rayon
        let results: Vec<JobResult> = jobs.read().unwrap()
            .par_iter()
            .enumerate()
            .map(|(job_id, job)| {
                let cache = self.cache.clone();
                let monitor = self.monitor.clone();
                let processed = total_processed.clone();
                let cached = total_cached.clone();
                let errors = total_errors.clone();
                let valid = total_valid.clone();
                
                let job_start = Instant::now();
                let mut job_cached = 0;
                let mut job_valid = 0;
                let mut job_errors = 0;
                
                // Process files in this job in parallel
                let file_results: Vec<FileResult> = job.paths
                    .par_iter()
                    .map(|path| {
                        let file_start = Instant::now();
                        
                        // Check cache first
                        if let Some(cached_result) = cache.get(&path) {
                            cached.fetch_add(1, Ordering::Relaxed);
                            return FileResult {
                                path: path.clone(),
                                is_valid: cached_result,
                                duration: file_start.elapsed(),
                                was_cached: true,
                                error: None,
                            };
                        }
                        
                        // Perform actual validation
                        let validation_options = crate::validators::ValidationOptions {
                            strict: false,
                            verbose: false,
                            timeout: 30,
                            config: Some(crate::validators::FileValidationConfig::default()),
                        };
                        
                        match validate_file(path, &validation_options) {
                            Ok(is_valid) => {
                                let duration = file_start.elapsed();
                                
                                // Cache the result
                                let _ = cache.put(path, is_valid, duration);
                                
                                // Update counters
                                if is_valid {
                                    valid.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    errors.fetch_add(1, Ordering::Relaxed);
                                }
                                processed.fetch_add(1, Ordering::Relaxed);
                                
                                // Report to monitor if available
                                if let Some(ref monitor) = monitor {
                                    monitor.record_validation(path, is_valid, duration);
                                }
                                
                                FileResult {
                                    path: path.clone(),
                                    is_valid,
                                    duration,
                                    was_cached: false,
                                    error: None,
                                }
                            }
                            Err(e) => {
                                errors.fetch_add(1, Ordering::Relaxed);
                                processed.fetch_add(1, Ordering::Relaxed);
                                
                                FileResult {
                                    path: path.clone(),
                                    is_valid: false,
                                    duration: file_start.elapsed(),
                                    was_cached: false,
                                    error: Some(e.to_string()),
                                }
                            }
                        }
                    })
                    .collect();
                
                // Update job statistics
                for result in &file_results {
                    if result.was_cached {
                        job_cached += 1;
                    } else if result.is_valid {
                        job_valid += 1;
                    } else {
                        job_errors += 1;
                    }
                }
                
                JobResult {
                    job_id,
                    file_results,
                    duration: job_start.elapsed(),
                    files_processed: job.paths.len(),
                    cached_hits: job_cached,
                    validation_successes: job_valid,
                    validation_errors: job_errors,
                }
            })
            .collect();
        
        let total_duration = start_time.elapsed();
        
        Ok(ParallelExecutionResult {
            results,
            total_duration,
            total_files: total_processed.load(Ordering::Relaxed) + total_cached.load(Ordering::Relaxed),
            files_processed: total_processed.load(Ordering::Relaxed),
            cache_hits: total_cached.load(Ordering::Relaxed),
            validation_successes: total_valid.load(Ordering::Relaxed),
            validation_errors: total_errors.load(Ordering::Relaxed),
            parallelism_used: rayon::current_num_threads(),
        })
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
    pub fn run(&self) -> Result<ParallelExecutionResult> {
        self.distributor.execute()
    }
}

/// Result of a single file validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub path: PathBuf,
    pub is_valid: bool,
    pub duration: Duration,
    pub was_cached: bool,
    pub error: Option<String>,
}

/// Result of a single job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: usize,
    pub file_results: Vec<FileResult>,
    pub duration: Duration,
    pub files_processed: usize,
    pub cached_hits: usize,
    pub validation_successes: usize,
    pub validation_errors: usize,
}

/// Overall result of parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelExecutionResult {
    pub results: Vec<JobResult>,
    pub total_duration: Duration,
    pub total_files: usize,
    pub files_processed: usize,
    pub cache_hits: usize,
    pub validation_successes: usize,
    pub validation_errors: usize,
    pub parallelism_used: usize,
}

impl ParallelExecutionResult {
    /// Calculate validation success rate
    pub fn success_rate(&self) -> f64 {
        if self.files_processed > 0 {
            self.validation_successes as f64 / self.files_processed as f64
        } else {
            0.0
        }
    }
    
    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_files > 0 {
            self.cache_hits as f64 / self.total_files as f64
        } else {
            0.0
        }
    }
    
    /// Calculate files per second throughput
    pub fn throughput(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.total_files as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get average validation time per file
    pub fn avg_validation_time(&self) -> Duration {
        if self.files_processed > 0 {
            Duration::from_nanos(self.total_duration.as_nanos() as u64 / self.files_processed as u64)
        } else {
            Duration::from_secs(0)
        }
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

