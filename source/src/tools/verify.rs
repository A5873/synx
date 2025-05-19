use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::process::Command;
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use log::{debug, warn, error};
use blake3;

use super::secure::SecureCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVerificationConfig {
    /// Known tool hashes
    pub known_hashes: HashMap<String, String>,
    /// Minimum required versions
    pub min_versions: HashMap<String, String>,
    /// Required capabilities
    pub required_capabilities: HashMap<String, Vec<String>>,
    /// Tool-specific security requirements
    pub security_requirements: HashMap<String, ToolSecurityRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSecurityRequirements {
    /// Whether the tool needs network access
    pub needs_network: bool,
    /// Whether the tool needs file write access
    pub needs_file_write: bool,
    /// Maximum allowed memory usage (MB)
    pub max_memory: u64,
    /// Maximum allowed CPU usage (%)
    pub max_cpu: u32,
    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,
}

impl Default for ToolSecurityRequirements {
    fn default() -> Self {
        Self {
            needs_network: false,
            needs_file_write: false,
            max_memory: 512,
            max_cpu: 50,
            allowed_extensions: vec![],
        }
    }
}

/// A verified tool that has passed security checks
#[derive(Debug)]
pub struct VerifiedTool {
    path: PathBuf,
    name: String,
    version: String,
    hash: String,
    security_requirements: ToolSecurityRequirements,
}

impl VerifiedTool {
    /// Verify and create a new tool instance
    pub fn new(name: &str, config: &ToolVerificationConfig) -> Result<Self> {
        let path = find_tool_path(name)?;
        
        // Verify tool hash
        let hash = compute_tool_hash(&path)?;
        verify_tool_hash(name, &hash, config)?;
        
        // Check tool version
        let version = get_tool_version(&path)?;
        verify_tool_version(name, &version, config)?;
        
        // Verify tool capabilities
        verify_tool_capabilities(name, &path, config)?;
        
        // Get security requirements
        let security_requirements = config.security_requirements
            .get(name)
            .cloned()
            .unwrap_or_default();
            
        Ok(Self {
            path,
            name: name.to_string(),
            version,
            hash,
            security_requirements,
        })
    }

    /// Get the path to the verified tool
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the tool's version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the tool's hash
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Get the tool's security requirements
    pub fn security_requirements(&self) -> &ToolSecurityRequirements {
        &self.security_requirements
    }

    /// Create a secure command for this tool
    pub fn create_command(&self) -> Result<SecureCommand> {
        let cmd = SecureCommand::new(&self.path)?;
        
        // Configure command based on security requirements
        // (Security configuration will be applied by SecureCommand)
        
        Ok(cmd)
    }
}

/// Find the full path to a tool
fn find_tool_path(name: &str) -> Result<PathBuf> {
    which::which(name)
        .context(format!("Failed to find tool: {}", name))
}

/// Compute the hash of a tool executable
fn compute_tool_hash(path: &Path) -> Result<String> {
    let contents = std::fs::read(path)
        .context("Failed to read tool executable")?;
        
    let hash = blake3::hash(&contents);
    Ok(hash.to_hex().to_string())
}

/// Verify a tool's hash against known good hashes
fn verify_tool_hash(name: &str, hash: &str, config: &ToolVerificationConfig) -> Result<()> {
    if let Some(expected_hash) = config.known_hashes.get(name) {
        if hash != expected_hash {
            return Err(anyhow!(
                "Tool hash verification failed for {}. Expected: {}, Got: {}",
                name, expected_hash, hash
            ));
        }
    }
    Ok(())
}

/// Get a tool's version string
fn get_tool_version(path: &Path) -> Result<String> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .context("Failed to get tool version")?;
        
    if !output.status.success() {
        return Err(anyhow!("Tool version check failed"));
    }
    
    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
        
    Ok(version)
}

/// Verify a tool's version meets minimum requirements
fn verify_tool_version(name: &str, version: &str, config: &ToolVerificationConfig) -> Result<()> {
    if let Some(min_version) = config.min_versions.get(name) {
        // Parse versions (assuming semantic versioning)
        let current = semver::Version::parse(version)
            .context("Failed to parse tool version")?;
        let minimum = semver::Version::parse(min_version)
            .context("Failed to parse minimum version requirement")?;
            
        if current < minimum {
            return Err(anyhow!(
                "Tool version {} is below minimum required version {} for {}",
                version, min_version, name
            ));
        }
    }
    Ok(())
}

/// Verify a tool has required capabilities
fn verify_tool_capabilities(name: &str, path: &Path, config: &ToolVerificationConfig) -> Result<()> {
    if let Some(required_caps) = config.required_capabilities.get(name) {
        for cap in required_caps {
            if !check_tool_capability(path, cap)? {
                return Err(anyhow!(
                    "Tool {} is missing required capability: {}",
                    name, cap
                ));
            }
        }
    }
    Ok(())
}

/// Check if a tool has a specific capability
fn check_tool_capability(path: &Path, capability: &str) -> Result<bool> {
    // Capability checking logic varies by tool
    match capability {
        "format" => check_formatting_capability(path),
        "lint" => check_linting_capability(path),
        "typecheck" => check_typechecking_capability(path),
        _ => Ok(false),
    }
}

/// Check if a tool has formatting capability
fn check_formatting_capability(path: &Path) -> Result<bool> {
    let output = Command::new(path)
        .arg("--help")
        .output()
        .context("Failed to check formatting capability")?;
        
    let help_text = String::from_utf8_lossy(&output.stdout);
    Ok(help_text.contains("format") || help_text.contains("fmt"))
}

/// Check if a tool has linting capability
fn check_linting_capability(path: &Path) -> Result<bool> {
    let output = Command::new(path)
        .arg("--help")
        .output()
        .context("Failed to check linting capability")?;
        
    let help_text = String::from_utf8_lossy(&output.stdout);
    Ok(help_text.contains("lint"))
}

/// Check if a tool has type checking capability
fn check_typechecking_capability(path: &Path) -> Result<bool> {
    let output = Command::new(path)
        .arg("--help")
        .output()
        .context("Failed to check type checking capability")?;
        
    let help_text = String::from_utf8_lossy(&output.stdout);
    Ok(help_text.contains("type") || help_text.contains("check"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> ToolVerificationConfig {
        ToolVerificationConfig {
            known_hashes: HashMap::new(),
            min_versions: {
                let mut map = HashMap::new();
                map.insert("rustc".to_string(), "1.70.0".to_string());
                map
            },
            required_capabilities: {
                let mut map = HashMap::new();
                map.insert("rustc".to_string(), vec!["format".to_string()]);
                map
            },
            security_requirements: HashMap::new(),
        }
    }

    #[test]
    fn test_find_tool_path() {
        // Test finding a common tool that should exist
        assert!(find_tool_path("ls").is_ok());
        
        // Test finding a nonexistent tool
        assert!(find_tool_path("nonexistent_tool_12345").is_err());
    }

    #[test]
    fn test_compute_tool_hash() {
        if let Ok(path) = find_tool_path("ls") {
            assert!(compute_tool_hash(&path).is_ok());
        }
    }

    #[test]
    fn test_verify_tool_version() {
        let config = create_test_config();
        
        // Test valid version
        assert!(verify_tool_version(
            "rustc",
            "1.70.0",
            &config
        ).is_ok());
        
        // Test invalid version
        assert!(verify_tool_version(
            "rustc",
            "1.69.0",
            &config
        ).is_err());
    }

    #[test]
    fn test_check_tool_capabilities() {
        if let Ok(path) = find_tool_path("rustc") {
            assert!(check_formatting_capability(&path).is_ok());
        }
    }
}
