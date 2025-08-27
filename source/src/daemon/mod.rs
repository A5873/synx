use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use log::{info, warn, error, debug};
use chrono::{DateTime, Utc};

use crate::config::Config as SynxConfig;
use crate::validators::{validate_file, ValidationOptions, FileValidationConfig};

pub mod config;
pub mod service;

pub use config::DaemonConfig;
pub use service::{install_service, uninstall_service, ServiceManager};

/// Events that the daemon can handle
#[derive(Debug, Clone)]
pub enum DaemonEvent {
    FileChanged(PathBuf),
    ConfigReloaded,
    Shutdown,
    HealthCheck,
}

/// Statistics about daemon operations
#[derive(Debug, Clone)]
pub struct DaemonStats {
    pub start_time: DateTime<Utc>,
    pub files_validated: u64,
    pub validation_errors: u64,
    pub validation_successes: u64,
    pub last_validation: Option<DateTime<Utc>>,
    pub watched_directories: Vec<PathBuf>,
    pub watched_files: u64,
}

impl Default for DaemonStats {
    fn default() -> Self {
        Self {
            start_time: Utc::now(),
            files_validated: 0,
            validation_errors: 0,
            validation_successes: 0,
            last_validation: None,
            watched_directories: Vec::new(),
            watched_files: 0,
        }
    }
}

/// The main daemon struct that manages file watching and validation
pub struct SynxDaemon {
    config: DaemonConfig,
    synx_config: SynxConfig,
    stats: DaemonStats,
    watcher: Option<RecommendedWatcher>,
    debounce_map: HashMap<PathBuf, Instant>,
}

impl SynxDaemon {
    /// Create a new daemon instance
    pub fn new(daemon_config: DaemonConfig, synx_config: SynxConfig) -> Result<Self> {
        let stats = DaemonStats {
            start_time: Utc::now(),
            watched_directories: daemon_config.watch_paths.clone(),
            ..Default::default()
        };

        Ok(Self {
            config: daemon_config,
            synx_config,
            stats,
            watcher: None,
            debounce_map: HashMap::new(),
        })
    }

    /// Start the daemon with async file watching
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Synx Daemon v{}", env!("CARGO_PKG_VERSION"));
        info!("Watching {} directories", self.config.watch_paths.len());
        
        // Create event channel
        let (tx, mut rx) = mpsc::channel::<DaemonEvent>(1000);
        
        // Setup file watcher
        self.setup_watcher(tx.clone()).await?;
        
        // Setup signal handlers for graceful shutdown
        self.setup_signal_handlers(tx.clone()).await?;
        
        // Main event loop
        loop {
            tokio::select! {
                // Handle file system events
                Some(event) = rx.recv() => {
                    match event {
                        DaemonEvent::FileChanged(path) => {
                            if let Err(e) = self.handle_file_change(&path).await {
                                error!("Error handling file change for {}: {}", path.display(), e);
                            }
                        }
                        DaemonEvent::ConfigReloaded => {
                            info!("Configuration reloaded");
                            // Restart watcher with new config
                            self.restart_watcher(tx.clone()).await?;
                        }
                        DaemonEvent::Shutdown => {
                            info!("Graceful shutdown requested");
                            break;
                        }
                        DaemonEvent::HealthCheck => {
                            self.perform_health_check().await;
                        }
                    }
                }
                
                // Periodic health checks and cleanup
                _ = tokio::time::sleep(Duration::from_secs(self.config.health_check_interval)) => {
                    self.perform_health_check().await;
                    self.cleanup_debounce_map();
                }
            }
        }
        
        info!("Synx Daemon shutting down");
        self.print_final_stats();
        Ok(())
    }

    /// Setup file system watcher
    async fn setup_watcher(&mut self, tx: mpsc::Sender<DaemonEvent>) -> Result<()> {
        let _debounce_duration = Duration::from_millis(self.config.debounce_ms);
        
        let watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                match res {
                    Ok(event) => {
                        // Filter for relevant file events
                        if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                            for path in event.paths {
                                if should_validate_file(&path) {
                                    if let Err(e) = tx_clone.send(DaemonEvent::FileChanged(path)).await {
                                        error!("Failed to send file change event: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("File watcher error: {}", e);
                    }
                }
            });
        })?;

        self.watcher = Some(watcher);
        
        // Start watching all configured paths
        for path in &self.config.watch_paths {
            info!("Watching directory: {}", path.display());
            if let Some(ref mut watcher) = self.watcher {
                watcher.watch(path, RecursiveMode::Recursive)?;
            }
        }

        // Count initial files
        self.stats.watched_files = self.count_watched_files();
        
        Ok(())
    }

    /// Restart the watcher (used when config is reloaded)
    async fn restart_watcher(&mut self, tx: mpsc::Sender<DaemonEvent>) -> Result<()> {
        info!("Restarting file watcher");
        self.watcher = None;
        self.setup_watcher(tx).await
    }

    /// Handle file change events with debouncing
    async fn handle_file_change(&mut self, path: &Path) -> Result<()> {
        let now = Instant::now();
        
        // Check debounce
        if let Some(&last_time) = self.debounce_map.get(path) {
            if now.duration_since(last_time) < Duration::from_millis(self.config.debounce_ms) {
                debug!("Debouncing file change for: {}", path.display());
                return Ok(());
            }
        }
        
        self.debounce_map.insert(path.to_path_buf(), now);
        
        // Validate the file
        self.validate_file_async(path).await
    }

    /// Async file validation
    async fn validate_file_async(&mut self, path: &Path) -> Result<()> {
        info!("Validating file: {}", path.display());
        
        let validation_options = ValidationOptions {
            strict: self.synx_config.strict,
            verbose: self.config.verbose_logging,
            timeout: self.config.validation_timeout,
            config: Some(FileValidationConfig::default()),
        };

        // Run validation in a blocking task to avoid blocking the async runtime
        let path_clone = path.to_path_buf();
        let validation_result = tokio::task::spawn_blocking(move || {
            validate_file(&path_clone, &validation_options)
        }).await?;

        // Update statistics
        self.stats.files_validated += 1;
        self.stats.last_validation = Some(Utc::now());

        match validation_result {
            Ok(true) => {
                self.stats.validation_successes += 1;
                if self.config.verbose_logging {
                    info!("✅ Validation passed: {}", path.display());
                }
            }
            Ok(false) => {
                self.stats.validation_errors += 1;
                warn!("❌ Validation failed: {}", path.display());
            }
            Err(e) => {
                self.stats.validation_errors += 1;
                error!("❌ Validation error for {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Setup signal handlers for graceful shutdown
    async fn setup_signal_handlers(&self, tx: mpsc::Sender<DaemonEvent>) -> Result<()> {
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                
                let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
                let mut sigint = signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");
                let mut sighup = signal(SignalKind::hangup()).expect("Failed to setup SIGHUP handler");
                
                loop {
                    tokio::select! {
                        _ = sigterm.recv() => {
                            info!("Received SIGTERM, initiating graceful shutdown");
                            let _ = tx_clone.send(DaemonEvent::Shutdown).await;
                            break;
                        }
                        _ = sigint.recv() => {
                            info!("Received SIGINT, initiating graceful shutdown");
                            let _ = tx_clone.send(DaemonEvent::Shutdown).await;
                            break;
                        }
                        _ = sighup.recv() => {
                            info!("Received SIGHUP, reloading configuration");
                            let _ = tx_clone.send(DaemonEvent::ConfigReloaded).await;
                        }
                    }
                }
            }
            
            #[cfg(windows)]
            {
                use tokio::signal::windows::{ctrl_c, ctrl_break};
                
                let mut ctrl_c_signal = ctrl_c().expect("Failed to setup Ctrl+C handler");
                let mut ctrl_break_signal = ctrl_break().expect("Failed to setup Ctrl+Break handler");
                
                tokio::select! {
                    _ = ctrl_c_signal.recv() => {
                        info!("Received Ctrl+C, initiating graceful shutdown");
                        let _ = tx_clone.send(DaemonEvent::Shutdown).await;
                    }
                    _ = ctrl_break_signal.recv() => {
                        info!("Received Ctrl+Break, initiating graceful shutdown");
                        let _ = tx_clone.send(DaemonEvent::Shutdown).await;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Perform health check
    async fn perform_health_check(&self) {
        debug!("Performing health check");
        
        // Check if watcher is still active
        if self.watcher.is_none() {
            warn!("File watcher is not active!");
        }
        
        // Log current statistics
        if self.config.verbose_logging {
            debug!("Stats: {} files validated, {} successes, {} errors", 
                   self.stats.files_validated, 
                   self.stats.validation_successes, 
                   self.stats.validation_errors);
        }
    }

    /// Clean up old entries from debounce map
    fn cleanup_debounce_map(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(300); // 5 minutes
        self.debounce_map.retain(|_, &mut last_time| last_time > cutoff);
    }

    /// Count total files being watched
    fn count_watched_files(&self) -> u64 {
        let mut count = 0;
        for path in &self.config.watch_paths {
            if path.is_dir() {
                count += count_files_in_directory(path);
            } else if path.is_file() {
                count += 1;
            }
        }
        count
    }

    /// Print final statistics on shutdown
    fn print_final_stats(&self) {
        let uptime = Utc::now().signed_duration_since(self.stats.start_time);
        
        info!("=== Synx Daemon Final Statistics ===");
        info!("Uptime: {} seconds", uptime.num_seconds());
        info!("Files validated: {}", self.stats.files_validated);
        info!("Validation successes: {}", self.stats.validation_successes);
        info!("Validation errors: {}", self.stats.validation_errors);
        info!("Watched directories: {}", self.stats.watched_directories.len());
        info!("Watched files: {}", self.stats.watched_files);
        
        if self.stats.files_validated > 0 {
            let success_rate = (self.stats.validation_successes as f64 / self.stats.files_validated as f64) * 100.0;
            info!("Success rate: {:.1}%", success_rate);
        }
    }

    /// Get current daemon statistics
    pub fn get_stats(&self) -> &DaemonStats {
        &self.stats
    }
}

/// Check if a file should be validated based on its extension
fn should_validate_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        matches!(extension.to_lowercase().as_str(),
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "java" | "go" | "c" | "cpp" | "cxx" | "cc" |
            "cs" | "html" | "htm" | "css" | "json" | "yaml" | "yml" | "sh" | "bash" | "dockerfile"
        )
    } else {
        // Check for files without extensions that might be relevant
        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
            matches!(file_name.to_lowercase().as_str(),
                "dockerfile" | "makefile" | "jenkinsfile"
            )
        } else {
            false
        }
    }
}

/// Count files in a directory recursively
fn count_files_in_directory(path: &Path) -> u64 {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_files_in_directory(&path);
            } else if should_validate_file(&path) {
                count += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_should_validate_file() {
        assert!(should_validate_file(Path::new("test.rs")));
        assert!(should_validate_file(Path::new("test.py")));
        assert!(should_validate_file(Path::new("test.js")));
        assert!(should_validate_file(Path::new("Dockerfile")));
        assert!(!should_validate_file(Path::new("test.txt")));
        assert!(!should_validate_file(Path::new("README.md")));
    }

    #[test]
    fn test_count_files_in_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("test.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("test.py"), "print('hello')").unwrap();
        fs::write(temp_path.join("README.md"), "# Test").unwrap(); // Should not be counted
        
        let count = count_files_in_directory(temp_path);
        assert_eq!(count, 2); // Only .rs and .py files should be counted
    }
}
