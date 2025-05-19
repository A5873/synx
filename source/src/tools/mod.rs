//! Tool management system with comprehensive security measures
//! 
//! This module provides a secure interface for managing and executing external tools,
//! with built-in security features including:
//! - Sandboxed command execution
//! - Path and file system security
//! - Tool verification and validation
//! - Security policy enforcement
//! - Audit logging

mod secure;
mod paths;
mod verify;
mod audit;
mod policy;

pub use secure::{SecureCommand, SecurityConfig};
pub use paths::{SecurePath, PathSecurityConfig};
pub use verify::{VerifiedTool, ToolVerificationConfig};
pub use policy::{SecurityPolicy, PolicyEnforcer};
pub use audit::AuditConfig;

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use log::{debug, warn, error};

/// Main tool manager that coordinates all security components
pub struct ToolManager {
    policy_enforcer: PolicyEnforcer,
    tool_cache: std::collections::HashMap<String, VerifiedTool>,
}

impl ToolManager {
    /// Create a new tool manager with the specified security policy
    pub fn new(policy: SecurityPolicy) -> Result<Self> {
        Ok(Self {
            policy_enforcer: PolicyEnforcer::new(policy)?,
            tool_cache: std::collections::HashMap::new(),
        })
    }

    /// Execute a tool securely
    pub fn execute_tool(
        &mut self,
        name: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> Result<std::process::Output> {
        // Verify and get tool (from cache or new)
        let tool = self.get_verified_tool(name)?;
        
        // Get security configurations
        let security_config = self.policy_enforcer.get_tool_security_config(name);
        
        // Create secure command
        let mut cmd = tool.create_command()?
            .with_config(security_config);
            
        // Add arguments safely
        cmd = cmd.args(args)?;
        
        // Set working directory if provided
        if let Some(dir) = working_dir {
            let path_config = self.policy_enforcer.get_path_security_config(dir);
            let secure_path = SecurePath::new(dir, path_config)?;
            cmd = cmd.current_dir(secure_path.as_path())?;
        }
        
        // Log the execution
        audit::log_tool_execution(
            name,
            &format!("{} {}", name, args.join(" ")),
            "started",
        )?;
        
        // Execute the command
        let output = cmd.output()?;
        
        // Log the result
        audit::log_tool_execution(
            name,
            &format!("{} {}", name, args.join(" ")),
            if output.status.success() { "success" } else { "failed" },
        )?;
        
        Ok(output)
    }

    /// Get a verified tool instance
    fn get_verified_tool(&mut self, name: &str) -> Result<&VerifiedTool> {
        // Check if tool is already verified and cached
        if !self.tool_cache.contains_key(name) {
            // Verify tool security
            self.policy_enforcer.verify_tool_security(name)?;
            
            // Create and verify tool
            let tool = VerifiedTool::new(
                name,
                &ToolVerificationConfig {
                    known_hashes: std::collections::HashMap::new(), // TODO: Load from config
                    min_versions: std::collections::HashMap::new(), // TODO: Load from config
                    required_capabilities: std::collections::HashMap::new(), // TODO: Load from config
                    security_requirements: std::collections::HashMap::new(), // TODO: Load from config
                },
            )?;
            
            // Cache the verified tool
            self.tool_cache.insert(name.to_string(), tool);
        }
        
        Ok(&self.tool_cache[name])
    }

    /// Read a file securely
    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        // Check if operation is allowed
        self.policy_enforcer.check_operation_allowed(
            "file_read",
            policy::FileOperation::Read,
            path,
        )?;
        
        // Create secure path
        let path_config = self.policy_enforcer.get_path_security_config(path);
        let secure_path = SecurePath::new(path, path_config)?;
        
        // Read file contents
        secure_path.read()
    }

    /// Write to a file securely
    pub fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        // Check if operation is allowed
        self.policy_enforcer.check_operation_allowed(
            "file_write",
            policy::FileOperation::Write,
            path,
        )?;
        
        // Create secure path
        let path_config = self.policy_enforcer.get_path_security_config(path);
        let secure_path = SecurePath::new(path, path_config)?;
        
        // Create temporary file
        let (temp_path, mut temp_file) = paths::create_secure_tempfile()?;
        
        // Write contents to temporary file
        std::io::Write::write_all(&mut temp_file, contents)?;
        temp_file.sync_all()?;
        
        // Rename temporary file to target path
        std::fs::rename(temp_path, path)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_policy() -> SecurityPolicy {
        // Create a basic security policy for testing
        SecurityPolicy {
            global: policy::GlobalSecuritySettings {
                strict_mode: true,
                allow_network: false,
                max_processes: 10,
                resource_limits: policy::ResourceLimits {
                    max_memory: 512,
                    max_cpu: 50,
                    max_io_rate: 10,
                    max_execution_time: 30,
                },
                allowed_working_dirs: vec![std::env::temp_dir()],
            },
            tool_policies: HashMap::new(),
            file_policies: policy::FileOperationPolicy {
                default_permissions: {
                    let mut set = std::collections::HashSet::new();
                    set.insert(policy::Permission::Read);
                    set.insert(policy::Permission::Write);
                    set
                },
                path_permissions: HashMap::new(),
                restricted_paths: vec![],
                required_checks: policy::FileSecurityChecks {
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
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        let manager = ToolManager::new(create_test_policy()).unwrap();
        
        // Test write
        let test_contents = b"test content";
        assert!(manager.write_file(&test_file, test_contents).is_ok());
        
        // Test read
        let read_contents = manager.read_file(&test_file).unwrap();
        assert_eq!(read_contents, test_contents);
    }

    #[test]
    fn test_tool_execution() {
        let manager = ToolManager::new(create_test_policy()).unwrap();
        
        // Test executing a basic command
        let output = manager.execute_tool(
            "echo",
            &["hello".to_string()],
            None,
        ).unwrap();
        
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "hello"
        );
    }

    #[test]
    fn test_restricted_operations() {
        let mut policy = create_test_policy();
        policy.file_policies.restricted_paths.push(PathBuf::from("/etc"));
        
        let manager = ToolManager::new(policy).unwrap();
        
        // Test accessing restricted path
        assert!(manager.read_file(Path::new("/etc/passwd")).is_err());
    }
}
