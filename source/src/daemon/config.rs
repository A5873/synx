use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use log::{info, warn};

/// Configuration specific to daemon operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Directories and files to watch for changes
    pub watch_paths: Vec<PathBuf>,
    
    /// Debounce time in milliseconds to prevent rapid re-validations
    pub debounce_ms: u64,
    
    /// Enable verbose logging for daemon operations
    pub verbose_logging: bool,
    
    /// Validation timeout in seconds
    pub validation_timeout: u64,
    
    /// Health check interval in seconds
    pub health_check_interval: u64,
    
    /// PID file location
    pub pid_file: Option<PathBuf>,
    
    /// Log file location (None for stdout/stderr)
    pub log_file: Option<PathBuf>,
    
    /// Maximum log file size in MB before rotation
    pub max_log_size_mb: u64,
    
    /// Number of rotated log files to keep
    pub log_rotation_count: u32,
    
    /// Run as daemon (background process)
    pub daemonize: bool,
    
    /// User to run daemon as (Unix only)
    pub run_as_user: Option<String>,
    
    /// Group to run daemon as (Unix only) 
    pub run_as_group: Option<String>,
    
    /// File patterns to exclude from watching
    pub exclude_patterns: Vec<String>,
    
    /// Include patterns to explicitly watch (overrides exclusions)
    pub include_patterns: Vec<String>,
    
    /// Maximum number of concurrent validations
    pub max_concurrent_validations: usize,
    
    /// Enable system notifications for validation results
    pub enable_notifications: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![PathBuf::from(".")],
            debounce_ms: 500,
            verbose_logging: false,
            validation_timeout: 30,
            health_check_interval: 60,
            pid_file: Some(PathBuf::from("/var/run/synx-daemon.pid")),
            log_file: Some(PathBuf::from("/var/log/synx-daemon.log")),
            max_log_size_mb: 100,
            log_rotation_count: 5,
            daemonize: true,
            run_as_user: None,
            run_as_group: None,
            exclude_patterns: vec![
                "*.tmp".to_string(),
                "*.swp".to_string(),
                "*.bak".to_string(),
                "*~".to_string(),
                ".git/**".to_string(),
                "node_modules/**".to_string(),
                "target/**".to_string(),
                "build/**".to_string(),
                "dist/**".to_string(),
                "__pycache__/**".to_string(),
                ".pytest_cache/**".to_string(),
            ],
            include_patterns: vec![],
            max_concurrent_validations: 4,
            enable_notifications: false,
        }
    }
}

impl DaemonConfig {
    /// Load daemon configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(anyhow!("Configuration file does not exist: {}", path.display()));
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file {}: {}", path.display(), e))?;
            
        let config: DaemonConfig = toml::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse config file {}: {}", path.display(), e))?;
            
        info!("Loaded daemon configuration from {}", path.display());
        config.validate()?;
        
        Ok(config)
    }
    
    /// Save daemon configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create directory {}: {}", parent.display(), e))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize configuration: {}", e))?;
            
        fs::write(path, content)
            .map_err(|e| anyhow!("Failed to write config file {}: {}", path.display(), e))?;
            
        info!("Saved daemon configuration to {}", path.display());
        Ok(())
    }
    
    /// Create configuration with specified watch paths
    pub fn with_watch_paths(watch_paths: Vec<PathBuf>) -> Self {
        Self {
            watch_paths,
            ..Default::default()
        }
    }
    
    /// Add a watch path
    pub fn add_watch_path<P: Into<PathBuf>>(&mut self, path: P) {
        let path = path.into();
        if !self.watch_paths.contains(&path) {
            self.watch_paths.push(path);
        }
    }
    
    /// Remove a watch path
    pub fn remove_watch_path<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        self.watch_paths.retain(|p| p != path);
    }
    
    /// Add exclude pattern
    pub fn add_exclude_pattern<S: Into<String>>(&mut self, pattern: S) {
        let pattern = pattern.into();
        if !self.exclude_patterns.contains(&pattern) {
            self.exclude_patterns.push(pattern);
        }
    }
    
    /// Add include pattern  
    pub fn add_include_pattern<S: Into<String>>(&mut self, pattern: S) {
        let pattern = pattern.into();
        if !self.include_patterns.contains(&pattern) {
            self.include_patterns.push(pattern);
        }
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check watch paths exist
        for path in &self.watch_paths {
            if !path.exists() {
                warn!("Watch path does not exist: {}", path.display());
            }
        }
        
        // Validate numeric values
        if self.debounce_ms == 0 {
            return Err(anyhow!("Debounce time must be greater than 0"));
        }
        
        if self.validation_timeout == 0 {
            return Err(anyhow!("Validation timeout must be greater than 0"));
        }
        
        if self.health_check_interval == 0 {
            return Err(anyhow!("Health check interval must be greater than 0"));
        }
        
        if self.max_concurrent_validations == 0 {
            return Err(anyhow!("Max concurrent validations must be greater than 0"));
        }
        
        if self.max_log_size_mb == 0 {
            return Err(anyhow!("Max log size must be greater than 0"));
        }
        
        // Validate paths are accessible if specified
        if let Some(ref pid_file) = self.pid_file {
            if let Some(parent) = pid_file.parent() {
                if !parent.exists() {
                    return Err(anyhow!("PID file directory does not exist: {}", parent.display()));
                }
            }
        }
        
        if let Some(ref log_file) = self.log_file {
            if let Some(parent) = log_file.parent() {
                if !parent.exists() {
                    return Err(anyhow!("Log file directory does not exist: {}", parent.display()));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get default daemon configuration paths
    pub fn get_default_config_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/etc/synx/daemon.toml"),
            dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("synx").join("daemon.toml"),
            PathBuf::from(".synx-daemon.toml"),
        ]
    }
    
    /// Load configuration from default locations
    pub fn load_default() -> Result<Self> {
        let paths = Self::get_default_config_paths();
        
        for path in paths {
            if path.exists() {
                return Self::from_file(path);
            }
        }
        
        info!("No daemon configuration file found, using defaults");
        Ok(Self::default())
    }
    
    /// Generate a default configuration file
    pub fn generate_default_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = Self::default();
        config.save_to_file(path)?;
        Ok(())
    }
    
    /// Check if a file path matches exclude patterns
    pub fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Check exclude patterns
        for pattern in &self.exclude_patterns {
            if glob_match::glob_match(pattern, &path_str) {
                // Check if it's explicitly included
                for include_pattern in &self.include_patterns {
                    if glob_match::glob_match(include_pattern, &path_str) {
                        return false; // Include overrides exclude
                    }
                }
                return true;
            }
        }
        
        // If we have include patterns, check if file matches at least one
        if !self.include_patterns.is_empty() {
            for include_pattern in &self.include_patterns {
                if glob_match::glob_match(include_pattern, &path_str) {
                    return false;
                }
            }
            return true; // Not included when include patterns are specified
        }
        
        false // Not excluded
    }
}

/// TOML configuration file structure for the daemon
#[derive(Debug, Serialize, Deserialize)]
struct DaemonConfigFile {
    #[serde(default)]
    daemon: DaemonConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = DaemonConfig::default();
        assert!(!config.watch_paths.is_empty());
        assert!(config.debounce_ms > 0);
        assert!(config.validation_timeout > 0);
    }

    #[test]
    fn test_config_validation() {
        let mut config = DaemonConfig::default();
        assert!(config.validate().is_ok());
        
        config.debounce_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_exclude_patterns() {
        let config = DaemonConfig::default();
        
        assert!(config.is_excluded(Path::new("test.tmp")));
        assert!(config.is_excluded(Path::new(".git/config")));
        assert!(config.is_excluded(Path::new("node_modules/package.json")));
        assert!(!config.is_excluded(Path::new("src/main.rs")));
    }

    #[test]
    fn test_include_override() {
        let mut config = DaemonConfig::default();
        config.add_include_pattern("*.tmp");
        
        // .tmp files are excluded by default, but our include pattern overrides
        assert!(!config.is_excluded(Path::new("important.tmp")));
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("daemon.toml");
        
        let original_config = DaemonConfig::default();
        original_config.save_to_file(&config_path).unwrap();
        
        let loaded_config = DaemonConfig::from_file(&config_path).unwrap();
        assert_eq!(original_config.debounce_ms, loaded_config.debounce_ms);
        assert_eq!(original_config.validation_timeout, loaded_config.validation_timeout);
    }

    #[test]
    fn test_watch_path_management() {
        let mut config = DaemonConfig::default();
        let initial_count = config.watch_paths.len();
        
        config.add_watch_path("/tmp/test");
        assert_eq!(config.watch_paths.len(), initial_count + 1);
        
        // Adding same path again shouldn't increase count
        config.add_watch_path("/tmp/test");
        assert_eq!(config.watch_paths.len(), initial_count + 1);
        
        config.remove_watch_path("/tmp/test");
        assert_eq!(config.watch_paths.len(), initial_count);
    }
}
