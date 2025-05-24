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
