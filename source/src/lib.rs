//! Synx - A secure code validation and formatting system
//!
//! This library provides a comprehensive set of tools for securely validating
//! and formatting source code files, with built-in security measures including:
//! - Sandboxed command execution
//! - Safe file system operations
//! - Tool verification and validation
//! - Security policy enforcement
//! - Audit logging
//! - Resource limits

use std::path::PathBuf;

// Re-export main components
pub use crate::tools::{
    ToolManager,
    SecurityPolicy,
    PathSecurityConfig,
    AuditConfig,
    SecureCommand,
    SecurePath,
    VerifiedTool,
    PolicyEnforcer,
};

// Module declarations
pub mod tools;
pub mod validators;
pub mod config;
pub mod analysis;
pub mod detectors;
pub mod daemon;
pub mod performance;
pub mod tui;

// Private modules
mod banner;

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, anyhow::Error>;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Configuration for the validation system
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to use strict mode
    pub strict: bool,
    /// Whether to enable verbose output
    pub verbose: bool,
    /// Custom configuration file path
    pub config_path: Option<PathBuf>,
    /// Whether to watch for file changes
    pub watch: bool,
    /// Security policy configuration
    pub security: SecurityConfig,
}

/// Main entry point for running validation on files
pub fn run(files: &[String], config: &config::Config) -> Result<bool> {
    use std::path::Path;
    use std::time::Instant;
    use indicatif::{ProgressBar, ProgressStyle};
    
    if files.is_empty() {
        return Err(anyhow::anyhow!("No files specified for validation"));
    }
    
    let start_time = Instant::now();
    let mut overall_success = true;
    let total_files = files.len();
    
    // Calculate total file sizes for better progress indication
    let total_size: u64 = files.iter()
        .filter_map(|f| std::fs::metadata(f).ok())
        .map(|m| m.len())
        .sum();
    
    // Create validation options for built-in validators
    let validation_options = validators::ValidationOptions {
        strict: config.strict,
        verbose: config.verbose,
        timeout: 30, // 30 second timeout
        config: Some(validators::FileValidationConfig::default()),
    };
    
    // Create enhanced progress bar for multiple files
    let progress = if total_files > 1 {
        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("🔍 [{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} [{eta_precise}] {msg}")
                .unwrap()
                .progress_chars("█▇▆▅▄▃▂▁  ")
        );
        pb.set_message(format!("Validating {} files ({} total)", 
            total_files, 
            format_file_size(total_size)
        ));
        Some(pb)
    } else {
        None
    };
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for (_index, file_path) in files.iter().enumerate() {
        let path = Path::new(file_path);
        
        // Update progress bar message
        if let Some(ref pb) = progress {
            pb.set_message(format!("Validating: {}", 
                path.file_name().unwrap_or_default().to_string_lossy()));
        }
        
        if !path.exists() {
            eprintln!("❌ File not found: {}", file_path);
            overall_success = false;
            invalid_count += 1;
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
            continue;
        }
        
        if config.verbose || total_files == 1 {
            println!("🔍 Validating: {}", file_path);
        }
        
        // Use built-in validators instead of external tools
        match validators::validate_file(path, &validation_options) {
            Ok(success) => {
                if success {
                    valid_count += 1;
                    if config.verbose || total_files == 1 {
                        println!("✅ {}: Validation passed", file_path);
                    }
                } else {
                    invalid_count += 1;
                    println!("❌ {}: Validation failed", file_path);
                    overall_success = false;
                }
            }
            Err(e) => {
                invalid_count += 1;
                eprintln!("❌ {}: Error during validation: {}", file_path, e);
                overall_success = false;
            }
        }
        
        // Update progress bar
        if let Some(ref pb) = progress {
            pb.inc(1);
            if invalid_count > 0 {
                pb.set_message(format!("✅ {} passed, ❌ {} failed", valid_count, invalid_count));
            } else {
                pb.set_message(format!("✅ {} passed", valid_count));
            }
        }
    }
    
    // Finish progress bar with final summary
    let elapsed = start_time.elapsed();
    if let Some(pb) = progress {
        pb.finish_with_message(format!(
            "Completed: ✅ {} passed, {} {} total ({:.2}s)", 
            valid_count,
            if invalid_count > 0 { format!("❌ {} failed,", invalid_count) } else { "".to_string() },
            total_files,
            elapsed.as_secs_f64()
        ));
        
        // Print detailed performance summary
        let files_per_sec = total_files as f64 / elapsed.as_secs_f64();
        let bytes_per_sec = total_size as f64 / elapsed.as_secs_f64();
        
        println!("\n📊 Performance: {:.1} files/sec, {}/sec", 
            files_per_sec, 
            format_file_size(bytes_per_sec as u64)
        );
        
        // Print final summary
        if overall_success {
            println!("✅ All validations passed successfully!");
        } else {
            println!("❌ Some validations failed!");
        }
    }
    
    Ok(overall_success)
}

/// Security-specific configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Path to audit log file
    pub audit_log: Option<PathBuf>,
    /// Maximum allowed file size
    pub max_file_size: u64,
    /// Allowed working directories
    pub allowed_dirs: Vec<PathBuf>,
    /// Whether to enforce strict security measures
    pub strict_security: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict: false,
            verbose: false,
            config_path: None,
            watch: false,
            security: SecurityConfig {
                audit_log: None,
                max_file_size: 10 * 1024 * 1024, // 10MB
                allowed_dirs: vec![],
                strict_security: false,
            },
        }
    }
}

/// The main validator interface
pub struct Validator {
    config: ValidationConfig,
    tool_manager: ToolManager,
}

impl Validator {
    /// Create a new validator instance
    pub fn new(config: ValidationConfig) -> Result<Self> {
        // Create security policy from config
        let policy = create_security_policy(&config)?;
        
        // Initialize tool manager
        let tool_manager = ToolManager::new(policy)?;
        
        Ok(Self {
            config,
            tool_manager,
        })
    }

    /// Validate a file
    pub fn validate_file(&mut self, path: &std::path::Path) -> Result<bool> {
        // Create secure path
        let _path_config = PathSecurityConfig {
            allowed_dirs: self.config.security.allowed_dirs.clone(),
            max_file_size: self.config.security.max_file_size,
            allowed_extensions: std::collections::HashSet::new(),
            allow_symlinks: false,
            check_ownership: true,
        };
        
        // Read file contents securely
        let _contents = self.tool_manager.read_file(path)?;
        
        // Determine file type and get appropriate validator
        let file_type = get_file_type(path)?;
        
        // Execute validator
        let args = if self.config.strict {
            vec!["--strict".to_string()]
        } else {
            vec![]
        };
        
        let output = self.tool_manager.execute_tool(
            &format!("validate_{}", file_type),
            &args,
            Some(path.parent().unwrap_or(std::path::Path::new("."))),
        )?;
        
        Ok(output.status.success())
    }

    /// Format a file
    pub fn format_file(
        &mut self,
        path: &std::path::Path,
        check_only: bool,
    ) -> Result<bool> {
        // Create secure path
        let _path_config = PathSecurityConfig {
            allowed_dirs: self.config.security.allowed_dirs.clone(),
            max_file_size: self.config.security.max_file_size,
            allowed_extensions: std::collections::HashSet::new(),
            allow_symlinks: false,
            check_ownership: true,
        };
        
        // Determine file type and get appropriate formatter
        let file_type = get_file_type(path)?;
        
        // Build formatter arguments
        let mut args = vec![];
        if check_only {
            args.push("--check".to_string());
        }
        if self.config.strict {
            args.push("--strict".to_string());
        }
        args.push(path.to_string_lossy().to_string());
        
        // Execute formatter
        let output = self.tool_manager.execute_tool(
            &format!("format_{}", file_type),
            &args,
            Some(path.parent().unwrap_or(std::path::Path::new("."))),
        )?;
        
        Ok(output.status.success())
    }
}

/// Create a security policy from configuration
fn create_security_policy(config: &ValidationConfig) -> Result<SecurityPolicy> {
    use std::collections::{HashMap, HashSet};
    
    // Create base policy
    let policy = SecurityPolicy {
        global: tools::policy::GlobalSecuritySettings {
            strict_mode: config.security.strict_security,
            allow_network: false,
            max_processes: 5,
            resource_limits: tools::policy::ResourceLimits {
                max_memory: 512,
                max_cpu: 50,
                max_io_rate: 10,
                max_execution_time: 30,
            },
            allowed_working_dirs: config.security.allowed_dirs.clone(),
        },
        tool_policies: HashMap::new(),
        file_policies: tools::policy::FileOperationPolicy {
            default_permissions: {
                let mut perms = HashSet::new();
                perms.insert(tools::policy::Permission::Read);
                perms
            },
            path_permissions: HashMap::new(),
            restricted_paths: vec![],
            required_checks: tools::policy::FileSecurityChecks {
                verify_ownership: true,
                check_permissions: true,
                validate_content: true,
                enforce_size_limits: true,
            },
        },
        user_restrictions: HashMap::new(),
    };
    
    Ok(policy)
}

/// Determine file type from extension
fn get_file_type(path: &std::path::Path) -> Result<String> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| anyhow::anyhow!("File has no extension"))?;
        
    Ok(extension.to_lowercase())
}

/// Format file size in human-readable format
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if size == 0 {
        return "0 B".to_string();
    }
    
    let size_f = size as f64;
    let unit_index = (size_f.log10() / THRESHOLD.log10()).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);
    
    let size_in_unit = size_f / THRESHOLD.powi(unit_index as i32);
    
    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_in_unit, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_validator_creation() {
        let config = ValidationConfig::default();
        assert!(Validator::new(config).is_ok());
    }

    #[test]
    fn test_file_validation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");
        
        fs::write(&test_file, "def test(): pass\n").unwrap();
        
        let mut config = ValidationConfig::default();
        config.security.allowed_dirs.push(temp_dir.path().to_path_buf());
        
        let mut validator = Validator::new(config).unwrap();
        assert!(validator.validate_file(&test_file).is_ok());
    }

    #[test]
    fn test_file_formatting() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");
        
        fs::write(&test_file, "def test(): pass\n").unwrap();
        
        let mut config = ValidationConfig::default();
        config.security.allowed_dirs.push(temp_dir.path().to_path_buf());
        
        let mut validator = Validator::new(config).unwrap();
        assert!(validator.format_file(&test_file, true).is_ok());
    }

    #[test]
    fn test_security_policy_creation() {
        let config = ValidationConfig::default();
        assert!(create_security_policy(&config).is_ok());
    }
}
