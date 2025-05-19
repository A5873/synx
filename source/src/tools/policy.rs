use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use log::{debug, warn, error};

use super::audit;
use super::secure::SecurityConfig;
use super::paths::PathSecurityConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Global security settings
    pub global: GlobalSecuritySettings,
    /// Tool-specific policies
    pub tool_policies: HashMap<String, ToolPolicy>,
    /// File operation policies
    pub file_policies: FileOperationPolicy,
    /// User-specific restrictions
    pub user_restrictions: HashMap<String, UserRestrictions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSecuritySettings {
    /// Whether to enforce strict mode
    pub strict_mode: bool,
    /// Whether to allow network access
    pub allow_network: bool,
    /// Maximum concurrent processes
    pub max_processes: u32,
    /// Global resource limits
    pub resource_limits: ResourceLimits,
    /// Allowed working directories
    pub allowed_working_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory: u64,
    /// Maximum CPU usage percentage
    pub max_cpu: u32,
    /// Maximum disk I/O rate in MB/s
    pub max_io_rate: u32,
    /// Maximum execution time in seconds
    pub max_execution_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicy {
    /// Required permissions
    pub required_permissions: HashSet<Permission>,
    /// Resource limits override
    pub resource_limits: Option<ResourceLimits>,
    /// Allowed file operations
    pub allowed_operations: HashSet<FileOperation>,
    /// Required security checks
    pub security_checks: SecurityChecks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationPolicy {
    /// Default permissions for files
    pub default_permissions: HashSet<Permission>,
    /// Path-specific permissions
    pub path_permissions: HashMap<PathBuf, HashSet<Permission>>,
    /// Restricted paths
    pub restricted_paths: Vec<PathBuf>,
    /// Required file checks
    pub required_checks: FileSecurityChecks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRestrictions {
    /// Allowed tools
    pub allowed_tools: HashSet<String>,
    /// Allowed file operations
    pub allowed_operations: HashSet<FileOperation>,
    /// Resource limits override
    pub resource_limits: Option<ResourceLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Network,
    CreateProcess,
    ModifyEnv,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FileOperation {
    Read,
    Write,
    Delete,
    Create,
    Move,
    Copy,
    ChangePermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityChecks {
    /// Whether to verify tool hash
    pub verify_hash: bool,
    /// Whether to check tool version
    pub check_version: bool,
    /// Whether to validate capabilities
    pub validate_capabilities: bool,
    /// Whether to enforce resource limits
    pub enforce_resource_limits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSecurityChecks {
    /// Whether to verify file ownership
    pub verify_ownership: bool,
    /// Whether to check file permissions
    pub check_permissions: bool,
    /// Whether to validate content
    pub validate_content: bool,
    /// Whether to enforce size limits
    pub enforce_size_limits: bool,
}

/// Policy enforcer that manages security policies
pub struct PolicyEnforcer {
    policy: SecurityPolicy,
    current_user: String,
}

impl PolicyEnforcer {
    /// Create a new policy enforcer
    pub fn new(policy: SecurityPolicy) -> Result<Self> {
        let current_user = super::audit::get_current_user();
        
        Ok(Self {
            policy,
            current_user,
        })
    }

    /// Check if an operation is allowed for the current user
    pub fn check_operation_allowed(
        &self,
        tool: &str,
        operation: FileOperation,
        path: &Path,
    ) -> Result<()> {
        // Check user restrictions
        if let Some(restrictions) = self.policy.user_restrictions.get(&self.current_user) {
            if !restrictions.allowed_tools.contains(tool) {
                return Err(anyhow!("User is not allowed to use tool: {}", tool));
            }
            
            if !restrictions.allowed_operations.contains(&operation) {
                return Err(anyhow!("User is not allowed to perform operation: {:?}", operation));
            }
        }

        // Check tool policy
        if let Some(tool_policy) = self.policy.tool_policies.get(tool) {
            if !tool_policy.allowed_operations.contains(&operation) {
                return Err(anyhow!("Tool is not allowed to perform operation: {:?}", operation));
            }
        }

        // Check path restrictions
        if self.policy.file_policies.restricted_paths.iter().any(|restricted| {
            path.starts_with(restricted)
        }) {
            return Err(anyhow!("Path is restricted: {}", path.display()));
        }

        // Check path-specific permissions
        if let Some(path_permissions) = self.policy.file_policies.path_permissions.get(path) {
            match operation {
                FileOperation::Read => {
                    if !path_permissions.contains(&Permission::Read) {
                        return Err(anyhow!("Read permission denied for path: {}", path.display()));
                    }
                }
                FileOperation::Write | FileOperation::Create => {
                    if !path_permissions.contains(&Permission::Write) {
                        return Err(anyhow!("Write permission denied for path: {}", path.display()));
                    }
                }
                _ => {
                    if !path_permissions.contains(&Permission::Execute) {
                        return Err(anyhow!("Execute permission denied for path: {}", path.display()));
                    }
                }
            }
        }

        // Log the allowed operation
        audit::log_file_access(
            &path.to_path_buf(),
            &format!("{:?}", operation),
        )?;

        Ok(())
    }

    /// Get security configuration for a tool
    pub fn get_tool_security_config(&self, tool: &str) -> SecurityConfig {
        let mut config = SecurityConfig::default();
        
        // Apply global settings
        config.allow_network = self.policy.global.allow_network;
        config.timeout = self.policy.global.resource_limits.max_execution_time;
        config.memory_limit = self.policy.global.resource_limits.max_memory;
        config.cpu_limit = self.policy.global.resource_limits.max_cpu;
        
        // Apply tool-specific overrides
        if let Some(tool_policy) = self.policy.tool_policies.get(tool) {
            if let Some(limits) = &tool_policy.resource_limits {
                config.memory_limit = limits.max_memory;
                config.cpu_limit = limits.max_cpu;
            }
            
            config.restrictions.allow_file_writes = tool_policy.allowed_operations.contains(&FileOperation::Write);
            config.restrictions.allow_subprocesses = tool_policy.required_permissions.contains(&Permission::CreateProcess);
            config.restrictions.allow_env_modifications = tool_policy.required_permissions.contains(&Permission::ModifyEnv);
        }
        
        // Apply user-specific overrides
        if let Some(restrictions) = self.policy.user_restrictions.get(&self.current_user) {
            if let Some(limits) = &restrictions.resource_limits {
                config.memory_limit = limits.max_memory.min(config.memory_limit);
                config.cpu_limit = limits.max_cpu.min(config.cpu_limit);
            }
        }
        
        config
    }

    /// Get path security configuration
    pub fn get_path_security_config(&self, path: &Path) -> PathSecurityConfig {
        let mut config = PathSecurityConfig::default();
        
        // Set allowed directories from global policy
        config.allowed_dirs = self.policy.global.allowed_working_dirs.clone();
        
        // Set basic security checks
        config.check_ownership = self.policy.file_policies.required_checks.verify_ownership;
        
        // Get path-specific configuration if available
        if let Some(path_permissions) = self.policy.file_policies.path_permissions.get(path) {
            config.allow_symlinks = path_permissions.contains(&Permission::Execute);
        }
        
        config
    }

    /// Verify security checks for a tool
    pub fn verify_tool_security(&self, tool: &str) -> Result<()> {
        if let Some(tool_policy) = self.policy.tool_policies.get(tool) {
            let checks = &tool_policy.security_checks;
            
            if checks.verify_hash {
                // Implement tool hash verification
            }
            
            if checks.check_version {
                // Implement version checking
            }
            
            if checks.validate_capabilities {
                // Implement capability validation
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn create_test_policy() -> SecurityPolicy {
        SecurityPolicy {
            global: GlobalSecuritySettings {
                strict_mode: true,
                allow_network: false,
                max_processes: 10,
                resource_limits: ResourceLimits {
                    max_memory: 512,
                    max_cpu: 50,
                    max_io_rate: 10,
                    max_execution_time: 30,
                },
                allowed_working_dirs: vec![PathBuf::from("/tmp")],
            },
            tool_policies: {
                let mut map = HashMap::new();
                let mut tool_policy = ToolPolicy {
                    required_permissions: {
                        let mut set = HashSet::new();
                        set.insert(Permission::Read);
                        set.insert(Permission::Execute);
                        set
                    },
                    resource_limits: None,
                    allowed_operations: {
                        let mut set = HashSet::new();
                        set.insert(FileOperation::Read);
                        set
                    },
                    security_checks: SecurityChecks {
                        verify_hash: true,
                        check_version: true,
                        validate_capabilities: true,
                        enforce_resource_limits: true,
                    },
                };
                map.insert("test_tool".to_string(), tool_policy);
                map
            },
            file_policies: FileOperationPolicy {
                default_permissions: {
                    let mut set = HashSet::new();
                    set.insert(Permission::Read);
                    set
                },
                path_permissions: HashMap::new(),
                restricted_paths: vec![PathBuf::from("/etc")],
                required_checks: FileSecurityChecks {
                    verify_ownership: true,
                    check_permissions: true,
                    validate_content: true,
                    enforce_size_limits: true,
                },
            },
            user_restrictions: HashMap::new(),
        }
    }

    #[test]
    fn test_operation_allowed() {
        let policy = create_test_policy();
        let enforcer = PolicyEnforcer::new(policy).unwrap();
        
        // Test allowed operation
        assert!(enforcer.check_operation_allowed(
            "test_tool",
            FileOperation::Read,
            Path::new("/tmp/test.txt"),
        ).is_ok());
        
        // Test restricted path
        assert!(enforcer.check_operation_allowed(
            "test_tool",
            FileOperation::Read,
            Path::new("/etc/passwd"),
        ).is_err());
        
        // Test disallowed operation
        assert!(enforcer.check_operation_allowed(
            "test_tool",
            FileOperation::Write,
            Path::new("/tmp/test.txt"),
        ).is_err());
    }

    #[test]
    fn test_security_config() {
        let policy = create_test_policy();
        let enforcer = PolicyEnforcer::new(policy).unwrap();
        
        let config = enforcer.get_tool_security_config("test_tool");
        assert!(!config.allow_network);
        assert_eq!(config.memory_limit, 512);
        assert_eq!(config.cpu_limit, 50);
    }

    #[test]
    fn test_path_security_config() {
        let policy = create_test_policy();
        let enforcer = PolicyEnforcer::new(policy).unwrap();
        
        let config = enforcer.get_path_security_config(Path::new("/tmp/test.txt"));
        assert!(config.check_ownership);
        assert!(!config.allow_symlinks);
    }
}
